//! Aviso de versión nueva: lógica pura, sin red ni reloj.
//!
//! Al arrancar, la interfaz pide una vez la última release publicada del repositorio
//! (https://docs.github.com/en/rest/releases/releases#get-the-latest-release) y pasa aquí la
//! respuesta. Este módulo construye la dirección de la consulta, lee la respuesta, valida el
//! enlace de descarga y decide si hay que avisar. La petición la hace la interfaz, que es la única
//! que toca la red.

use std::cmp::Ordering;
use std::fmt;

use serde::Deserialize;

/// Principio de la dirección de la API de releases de GitHub. La dirección completa es
/// `<esto><propietario>/<nombre>/releases/latest`, según la documentación citada arriba.
pub const API_DE_RELEASES: &str = "https://api.github.com/repos/";

/// Final de la dirección de la última release publicada. GitHub no devuelve aquí ni borradores
/// ni versiones previas («The latest release is the most recent non-prerelease, non-draft
/// release»).
pub const ULTIMA_RELEASE: &str = "/releases/latest";

/// Cabecera Accept que recomienda la API de GitHub (https://docs.github.com/en/rest).
pub const ACEPTA_LA_API: &str = "application/vnd.github+json";

/// Nombre del programa en la cabecera User-Agent, que la API de GitHub exige («All API requests
/// must include a valid User-Agent header»,
/// https://docs.github.com/en/rest/using-the-rest-api/getting-started-with-the-rest-api). Va
/// seguido de la versión y no lleva ningún dato del usuario.
pub const AGENTE_DE_USUARIO: &str = "gestor-agentes";

/// Principio de todo enlace de descarga aceptable: la web de GitHub.
const WEB_DE_GITHUB: &str = "https://github.com/";

/// Caracteres que GitHub admite en el nombre de un propietario o de un repositorio, además de las
/// letras y las cifras ASCII.
const SIGNOS_DE_UN_NOMBRE: [char; 3] = ['-', '_', '.'];

/// Versión SemVer sin versión previa ni metadatos: mayor, menor y corrección. Se ordena como
/// manda SemVer 2.0.0 (https://semver.org/lang/es/), campo a campo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    mayor: u64,
    menor: u64,
    correccion: u64,
}

impl Version {
    /// Lee «3.1.0» o «v3.1.0». Rechaza lo que no sean exactamente tres números, incluidas las
    /// versiones previas («3.2.0-rc.1»), porque solo se avisa de versiones estables.
    #[must_use]
    pub fn leer(texto: &str) -> Option<Self> {
        let texto = texto.trim();
        let texto = texto.strip_prefix('v').unwrap_or(texto);
        let mut partes = texto.split('.');
        let mut numero = || -> Option<u64> {
            let parte = partes.next()?;
            if parte.is_empty() || !parte.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            parte.parse().ok()
        };
        let version = Self {
            mayor: numero()?,
            menor: numero()?,
            correccion: numero()?,
        };
        partes.next().is_none().then_some(version)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.mayor, self.menor, self.correccion)
    }
}

/// Release publicada: su versión y la página donde se descarga.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publicacion {
    /// Versión de la etiqueta.
    pub version: Version,
    /// Página de la release en GitHub, ya validada.
    pub enlace: String,
}

/// Lo que se lee de la respuesta de la API.
#[derive(Deserialize)]
struct Respuesta {
    tag_name: String,
    html_url: String,
}

/// Indica si `repositorio` tiene la forma `propietario/nombre` que admite GitHub.
#[must_use]
pub fn repositorio_valido(repositorio: &str) -> bool {
    let mut partes = repositorio.split('/');
    let valido = |parte: Option<&str>| {
        parte.is_some_and(|p| {
            !p.is_empty()
                && p.chars()
                    .all(|c| c.is_ascii_alphanumeric() || SIGNOS_DE_UN_NOMBRE.contains(&c))
        })
    };
    valido(partes.next()) && valido(partes.next()) && partes.next().is_none()
}

/// Dirección de la API con la última release de `repositorio`, o nada si el repositorio no es
/// válido.
#[must_use]
pub fn direccion_de_la_consulta(repositorio: &str) -> Option<String> {
    repositorio_valido(repositorio)
        .then(|| format!("{API_DE_RELEASES}{repositorio}{ULTIMA_RELEASE}"))
}

/// Valor de la cabecera User-Agent para la versión instalada.
#[must_use]
pub fn agente_de_usuario(version_instalada: &str) -> String {
    format!("{AGENTE_DE_USUARIO}/{version_instalada}")
}

/// Lee la respuesta de la API. Devuelve nada si no es el JSON esperado, si la etiqueta no es una
/// versión estable o si el enlace no es una página de `repositorio` en la web de GitHub, para no
/// mostrar nunca una dirección que no sea la del proyecto.
#[must_use]
pub fn leer_publicacion(json: &str, repositorio: &str) -> Option<Publicacion> {
    let respuesta: Respuesta = serde_json::from_str(json).ok()?;
    let version = Version::leer(&respuesta.tag_name)?;
    let esperado = format!("{WEB_DE_GITHUB}{repositorio}/").to_lowercase();
    respuesta
        .html_url
        .to_lowercase()
        .starts_with(&esperado)
        .then_some(Publicacion {
            version,
            enlace: respuesta.html_url,
        })
}

/// La publicación de la que hay que avisar: solo si es más nueva que la instalada. Con una
/// versión instalada ilegible no se avisa, porque no se puede comparar.
#[must_use]
pub fn novedad(version_instalada: &str, publicada: Option<Publicacion>) -> Option<Publicacion> {
    let instalada = Version::leer(version_instalada)?;
    publicada.filter(|p| p.version.cmp(&instalada) == Ordering::Greater)
}
