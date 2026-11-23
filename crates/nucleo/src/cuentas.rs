//! Cuentas de agente instaladas en el equipo y perfiles que les corresponden.
//!
//! Cada cuenta de Claude Code o de Codex vive en su propia carpeta dentro de la carpeta personal:
//! la que la herramienta usa por defecto (~/.claude, ~/.codex) o una con número para las demás
//! (~/.claude2, ~/.codex2), que el lanzador pasa con CLAUDE_CONFIG_DIR o CODEX_HOME. Una carpeta
//! cuenta como cuenta si contiene el archivo de sesión de su herramienta.
//!
//! La decisión es pura ([`perfiles_de_cuentas`] e [`incorporar`]) y recibe lo que haya en la
//! carpeta personal ya listado; [`explorar`] es la única función que toca el disco. Gemini (agy no
//! tiene variable documentada para separar cuentas) y Programator no tienen cuentas: sus perfiles
//! de base los crea la configuración.

use std::fs;
use std::path::{Path, PathBuf};

use crate::configuracion::{HerramientaDelPerfil, PerfilDeAgente};

/// Lo que se sabe de una carpeta de la carpeta personal: su nombre y los nombres de los archivos
/// que contiene, sin su contenido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarpetaPersonal {
    /// Nombre de la carpeta, como «.claude2».
    pub nombre: String,
    /// Nombres de los archivos que contiene, sin subcarpetas.
    pub archivos: Vec<String>,
}

/// Cómo se reconoce la carpeta de una cuenta de una herramienta.
struct ReglaDeCuenta {
    herramienta: HerramientaDelPerfil,
    /// Nombre de la carpeta por defecto; las demás cuentas añaden su número.
    carpeta: &'static str,
    /// Archivo de sesión que hace de una carpeta una cuenta.
    archivo_de_sesion: &'static str,
    /// Si la cuenta por defecto fija su carpeta en el perfil. Claude 1 la lleva («~/.claude»)
    /// desde la 2.x; Codex 1 no, y el lanzador deja que codex use su carpeta por defecto.
    fija_la_carpeta_por_defecto: bool,
}

/// Reglas de las herramientas con cuentas. Los archivos de sesión se comprobaron en un equipo con
/// dos cuentas de Claude Code y una de Codex: ~/.claude y ~/.claude2 tienen cada una .credentials.json, y ~/.codex
/// tiene auth.json. No se ha encontrado documentación oficial que fije esos nombres; si una
/// versión futura los cambia, la detección deja de ver la cuenta, pero no rompe nada.
const REGLAS: [ReglaDeCuenta; 2] = [
    ReglaDeCuenta {
        herramienta: HerramientaDelPerfil::Claude,
        carpeta: ".claude",
        archivo_de_sesion: ".credentials.json",
        fija_la_carpeta_por_defecto: true,
    },
    ReglaDeCuenta {
        herramienta: HerramientaDelPerfil::Codex,
        carpeta: ".codex",
        archivo_de_sesion: "auth.json",
        fija_la_carpeta_por_defecto: false,
    },
];

/// Primera instancia que lleva su número en el nombre de la carpeta: la 1 es la carpeta sin
/// número, así que «.claude1» no es una cuenta.
const PRIMERA_INSTANCIA_CON_NUMERO: u32 = 2;

/// Prefijo con que se escribe la carpeta de una cuenta en el perfil, relativa a la carpeta
/// personal, como en los perfiles de la 2.x («~/.claude2»).
const PREFIJO_CARPETA_PERSONAL: &str = "~/";

impl ReglaDeCuenta {
    /// Instancia de la cuenta a que corresponde la carpeta, si lo es: 1 para la carpeta por
    /// defecto y el número para las demás, desde el 2. Solo vale si tiene su archivo de sesión.
    fn instancia(&self, carpeta: &CarpetaPersonal) -> Option<u32> {
        let resto = carpeta.nombre.strip_prefix(self.carpeta)?;
        let instancia = if resto.is_empty() {
            1
        } else if resto.bytes().all(|byte| byte.is_ascii_digit()) && !resto.starts_with('0') {
            resto
                .parse()
                .ok()
                .filter(|&numero| numero >= PRIMERA_INSTANCIA_CON_NUMERO)?
        } else {
            return None;
        };
        carpeta
            .archivos
            .iter()
            .any(|archivo| archivo == self.archivo_de_sesion)
            .then_some(instancia)
    }

    /// Perfil de la cuenta de esa carpeta.
    fn perfil(&self, instancia: u32, carpeta: &str) -> PerfilDeAgente {
        let fija = instancia > 1 || self.fija_la_carpeta_por_defecto;
        let perfil = PerfilDeAgente::new(self.herramienta, instancia);
        if fija {
            perfil.con_directorio(&format!("{PREFIJO_CARPETA_PERSONAL}{carpeta}"))
        } else {
            perfil
        }
    }
}

/// Perfiles de las cuentas que hay en la carpeta personal, ordenados por herramienta (Claude,
/// después Codex) y por instancia. Las carpetas sin su archivo de sesión, las que no siguen el
/// patrón (como «.claude-old» o «.claude1») y los archivos sueltos como .claude.json no cuentan.
#[must_use]
pub fn perfiles_de_cuentas(carpetas: &[CarpetaPersonal]) -> Vec<PerfilDeAgente> {
    REGLAS
        .iter()
        .flat_map(|regla| {
            let mut cuentas: Vec<(u32, &str)> = carpetas
                .iter()
                .filter_map(|carpeta| {
                    regla
                        .instancia(carpeta)
                        .map(|instancia| (instancia, carpeta.nombre.as_str()))
                })
                .collect();
            cuentas.sort_unstable();
            cuentas
                .into_iter()
                .map(|(instancia, carpeta)| regla.perfil(instancia, carpeta))
        })
        .collect()
}

/// Añade a los perfiles guardados los detectados que falten, por su identidad («Claude 2»). Los
/// guardados no se quitan ni se cambian nunca: su carpeta y su ejecutable son del usuario. Cada
/// perfil nuevo va detrás del último de su herramienta, o al final si no hay ninguno. Devuelve
/// cuántos se han añadido, para saber si hay que guardar.
pub fn incorporar(perfiles: &mut Vec<PerfilDeAgente>, detectados: Vec<PerfilDeAgente>) -> usize {
    let mut anadidos = 0;
    for detectado in detectados {
        if perfiles
            .iter()
            .any(|perfil| perfil.nombre() == detectado.nombre())
        {
            continue;
        }
        let posicion = perfiles
            .iter()
            .rposition(|perfil| perfil.tipo() == detectado.tipo())
            .map_or(perfiles.len(), |ultimo| ultimo + 1);
        perfiles.insert(posicion, detectado);
        anadidos += 1;
    }
    anadidos
}

/// Por qué no se puede dar de alta una cuenta a mano.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AltaRechazada {
    /// La herramienta no tiene cuentas separadas por carpeta (Gemini, Programator).
    SinCuentas,
    /// Otro perfil de la misma herramienta ya usa esa carpeta; lleva su identidad («Claude 2»).
    CarpetaEnUso(String),
}

/// Da de alta a mano la cuenta de `herramienta` que vive en `carpeta`, para las que no están en
/// la carpeta personal con el nombre de siempre (CLAUDE_CONFIG_DIR o CODEX_HOME en otra ruta).
/// El perfil nuevo lleva la instancia siguiente a la mayor de su herramienta y va detrás del
/// último de ella. Devuelve su identidad.
///
/// # Errors
///
/// [`AltaRechazada`] si la herramienta no tiene cuentas o si otro perfil suyo ya usa la carpeta,
/// comparada ya expandida y sin distinguir barras, barra final ni, en Windows, mayúsculas.
pub fn dar_de_alta(
    perfiles: &mut Vec<PerfilDeAgente>,
    herramienta: HerramientaDelPerfil,
    carpeta: &Path,
    carpeta_personal: &Path,
) -> Result<String, AltaRechazada> {
    let regla = REGLAS
        .iter()
        .find(|regla| regla.herramienta == herramienta)
        .ok_or(AltaRechazada::SinCuentas)?;
    let buscada = comparable(carpeta);
    if let Some(ocupado) = perfiles
        .iter()
        .filter(|p| p.tipo() == herramienta)
        .find(|perfil| {
            carpeta_de_la_cuenta(perfil, regla, carpeta_personal)
                .is_some_and(|suya| comparable(&suya) == buscada)
        })
    {
        return Err(AltaRechazada::CarpetaEnUso(ocupado.nombre()));
    }
    let instancia = perfiles
        .iter()
        .filter(|perfil| perfil.tipo() == herramienta)
        .map(PerfilDeAgente::instancia)
        .max()
        .map_or(1, |mayor| mayor + 1);
    let nuevo =
        PerfilDeAgente::new(herramienta, instancia).con_directorio(&carpeta.to_string_lossy());
    let nombre = nuevo.nombre();
    incorporar(perfiles, vec![nuevo]);
    Ok(nombre)
}

/// Carpeta donde vive la cuenta de un perfil: la fijada o, para la instancia 1 sin carpeta, la de
/// la herramienta por defecto en la carpeta personal.
fn carpeta_de_la_cuenta(
    perfil: &PerfilDeAgente,
    regla: &ReglaDeCuenta,
    carpeta_personal: &Path,
) -> Option<PathBuf> {
    perfil
        .directorio_configuracion_expandido(carpeta_personal)
        .or_else(|| (perfil.instancia() == 1).then(|| carpeta_personal.join(regla.carpeta)))
}

/// Forma de una ruta para compararla: barras normales, sin barra final y, en Windows, en
/// minúsculas, porque allí las rutas no distinguen mayúsculas.
fn comparable(ruta: &Path) -> String {
    let texto = ruta.to_string_lossy().replace('\\', "/");
    let texto = texto.trim_end_matches('/');
    if cfg!(windows) {
        texto.to_lowercase()
    } else {
        texto.to_owned()
    }
}

/// Lista las carpetas de la carpeta personal que pueden ser de una cuenta (las que empiezan por
/// el nombre de la carpeta de alguna herramienta), con los nombres de sus archivos. Lo que no se
/// puede leer se omite: una carpeta ilegible no es una cuenta que se pueda usar.
#[must_use]
pub fn explorar(carpeta_personal: &Path) -> Vec<CarpetaPersonal> {
    let Ok(entradas) = fs::read_dir(carpeta_personal) else {
        return Vec::new();
    };
    entradas
        .filter_map(Result::ok)
        .filter(|entrada| entrada.file_type().is_ok_and(|tipo| tipo.is_dir()))
        .filter_map(|entrada| {
            let nombre = entrada.file_name().into_string().ok()?;
            REGLAS
                .iter()
                .any(|regla| nombre.starts_with(regla.carpeta))
                .then(|| CarpetaPersonal {
                    archivos: archivos_de(&entrada.path()),
                    nombre,
                })
        })
        .collect()
}

/// Nombres de los archivos de una carpeta, sin entrar en sus subcarpetas.
fn archivos_de(carpeta: &Path) -> Vec<String> {
    fs::read_dir(carpeta)
        .map(|entradas| {
            entradas
                .filter_map(Result::ok)
                .filter(|entrada| entrada.file_type().is_ok_and(|tipo| tipo.is_file()))
                .filter_map(|entrada| entrada.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}
