//! Traducción entre las cuentas que encuentra la biblioteca deteccion-agentes y los perfiles de
//! Gestor Agentes.
//!
//! Qué es una cuenta, dónde se busca y cómo se abre su consola lo decide la biblioteca (a través
//! de [`crate::cuentas_de_agentes`], el único módulo que la llama). Aquí solo se decide lo que es
//! de Gestor: la identidad de cada cuenta («Claude 2»), qué perfiles se añaden, el alta a mano
//! sobre los perfiles y el ajuste de entorno con que se abre cada perfil. Todo es lógica sobre
//! cuentas ya encontradas; el disco solo se consulta al comparar carpetas, que se resuelven a su
//! forma canónica cuando existen, con la misma regla que la biblioteca (ver [`misma_carpeta`]).
//!
//! Correspondencia de herramientas: Claude es Claude Code, Codex es Codex CLI y Gemini es agy
//! (Antigravity CLI). Programator no tiene cuentas. agy no tiene cuentas separadas por carpeta:
//! su perfil de base lo crea la configuración y aquí no se añade ninguno.

use std::path::{Path, PathBuf};

use crate::configuracion::{HerramientaDelPerfil, PerfilDeAgente};
use crate::cuentas_de_agentes::{self, CuentaManual};
pub use deteccion_agentes::{
    AjusteDeEntorno, Cuenta, Herramienta, OrigenDeLaCuenta, RechazoDeAlta,
};

/// Instancia de la cuenta por defecto de cada herramienta: «Claude 1», «Codex 1».
const INSTANCIA_DE_LA_CUENTA_POR_DEFECTO: u32 = 1;

/// Primera instancia que una carpeta puede pedir por el número de su nombre («.claude2» pide la
/// 2): la 1 es la cuenta por defecto, sin número.
const PRIMERA_INSTANCIA_CON_NUMERO: u32 = 2;

/// Prefijo con que se escribe en el perfil una carpeta de la carpeta personal, como en los
/// perfiles de la 2.x («~/.claude2»).
const PREFIJO_CARPETA_PERSONAL: &str = "~/";

/// Prefijo de ruta extendida que pone `canonicalize` en Windows, ya con barras normales. Es el
/// mismo que quita la biblioteca al comparar carpetas.
const PREFIJO_DE_RUTA_EXTENDIDA: &str = "//?/";

/// Herramienta de la biblioteca que corresponde a la de un perfil; nada para Programator.
#[must_use]
pub const fn herramienta_de_la_biblioteca(
    herramienta: HerramientaDelPerfil,
) -> Option<Herramienta> {
    match herramienta {
        HerramientaDelPerfil::Claude => Some(Herramienta::ClaudeCode),
        HerramientaDelPerfil::Codex => Some(Herramienta::CodexCli),
        HerramientaDelPerfil::Gemini => Some(Herramienta::Agy),
        HerramientaDelPerfil::Programator => None,
    }
}

/// Herramienta de perfil que corresponde a una de la biblioteca.
const fn herramienta_del_perfil(herramienta: Herramienta) -> HerramientaDelPerfil {
    match herramienta {
        Herramienta::ClaudeCode => HerramientaDelPerfil::Claude,
        Herramienta::CodexCli => HerramientaDelPerfil::Codex,
        Herramienta::Agy => HerramientaDelPerfil::Gemini,
    }
}

/// Carpeta de la cuenta de un perfil, con «~» sustituida por `personal`: la que fija el perfil o,
/// si no fija ninguna, la que la herramienta usa por defecto (la de la biblioteca). agy usa
/// siempre la suya por defecto, porque no tiene cuentas separadas. Nada para Programator.
#[must_use]
pub fn carpeta_de_la_cuenta(perfil: &PerfilDeAgente, personal: &Path) -> Option<PathBuf> {
    let herramienta = herramienta_de_la_biblioteca(perfil.tipo())?;
    let por_defecto = herramienta.carpeta_por_defecto(personal);
    if !herramienta.admite_alta_manual() {
        return Some(por_defecto);
    }
    Some(
        perfil
            .directorio_configuracion_expandido(personal)
            .unwrap_or(por_defecto),
    )
}

/// La cuenta de un perfil, tal como la ve la biblioteca. Es la cuenta por defecto si el perfil no
/// fija carpeta o fija la carpeta por defecto de su herramienta; si fija otra, es una cuenta
/// guardada en las preferencias de Gestor, es decir, manual. Nada para Programator.
#[must_use]
pub fn cuenta_del_perfil(perfil: &PerfilDeAgente, personal: &Path) -> Option<Cuenta> {
    let herramienta = herramienta_de_la_biblioteca(perfil.tipo())?;
    let carpeta = carpeta_de_la_cuenta(perfil, personal)?;
    let origen = if misma_carpeta(&carpeta, &herramienta.carpeta_por_defecto(personal)) {
        OrigenDeLaCuenta::PorDefecto
    } else {
        OrigenDeLaCuenta::Manual
    };
    Some(Cuenta {
        herramienta,
        carpeta,
        origen,
    })
}

/// Cómo se ajusta el entorno de la consola del perfil (ver `Cuenta::ajuste_de_entorno`): poner
/// la variable de la cuenta con su carpeta o, en la cuenta por defecto, quitarla, para que la
/// consola no herede la de Gestor. Nada para agy y Programator, que no tocan el entorno.
#[must_use]
pub fn ajuste_de_lanzamiento(perfil: &PerfilDeAgente, personal: &Path) -> Option<AjusteDeEntorno> {
    cuenta_del_perfil(perfil, personal)?.ajuste_de_entorno()
}

/// Cuentas que los perfiles guardados dan de alta a mano: las de los perfiles de Claude y Codex
/// que fijan una carpeta que no es la por defecto. Se pasan a la biblioteca con
/// [`cuentas_de_agentes::cuentas_del_equipo`], que las conserva aunque su carpeta ya no exista.
#[must_use]
pub fn manuales_de(perfiles: &[PerfilDeAgente], personal: &Path) -> Vec<CuentaManual> {
    perfiles
        .iter()
        .filter_map(|perfil| cuenta_del_perfil(perfil, personal))
        .filter(|cuenta| cuenta.origen == OrigenDeLaCuenta::Manual)
        .map(|cuenta| CuentaManual {
            herramienta: cuenta.herramienta,
            carpeta: cuenta.carpeta,
        })
        .collect()
}

/// Cambios que hace [`incorporar_cuentas`] en los perfiles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Incorporacion {
    /// Perfiles que han dejado de fijar la carpeta por defecto de su herramienta.
    pub normalizados: usize,
    /// Perfiles nuevos, uno por cuenta que no tenía perfil.
    pub anadidos: usize,
}

impl Incorporacion {
    /// Si ha cambiado algo y hay que guardar los perfiles.
    #[must_use]
    pub const fn hay_cambios(self) -> bool {
        self.normalizados > 0 || self.anadidos > 0
    }
}

/// Pone al día los perfiles con las cuentas encontradas y devuelve qué ha cambiado, para saber si
/// hay que guardar y qué decirle al usuario.
///
/// 1. Un perfil de Claude o Codex que fija la carpeta por defecto de su herramienta (como el
///    «~/.claude» de Claude 1 hasta la 3.8) deja de fijarla: es la cuenta por defecto, que se abre
///    quitando la variable de cuenta.
/// 2. Cada cuenta de Claude Code o Codex CLI que no es la de ningún perfil se añade con una
///    identidad nueva, detrás del último perfil de su herramienta. La identidad es estable porque
///    el perfil se guarda: una cuenta ya guardada conserva la suya aunque aparezcan otras.
///
/// Las cuentas de agy no añaden perfiles, ni las de una carpeta cuya ruta no es texto válido, que
/// el perfil no podría guardar tal cual. Ningún perfil se quita ni cambia de identidad.
pub fn incorporar_cuentas(
    perfiles: &mut Vec<PerfilDeAgente>,
    cuentas: &[Cuenta],
    personal: &Path,
) -> Incorporacion {
    let normalizados = quitar_carpetas_por_defecto(perfiles, personal);
    let mut nuevas: Vec<&Cuenta> = cuentas
        .iter()
        .filter(|cuenta| cuenta.herramienta.admite_alta_manual())
        .filter(|cuenta| perfil_de_la_cuenta(perfiles, cuenta, personal).is_none())
        .collect();
    // Primero las que piden su número, para que «.claude2» sea Claude 2 aunque otra carpeta sin
    // número vaya antes en el orden de la biblioteca.
    nuevas.sort_by_key(|cuenta| instancia_pedida(cuenta, personal).is_none());
    let mut anadidos = 0;
    for cuenta in nuevas {
        if let Some(nuevo) = perfil_nuevo(perfiles, cuenta, personal) {
            anadir_perfil(perfiles, nuevo);
            anadidos += 1;
        }
    }
    Incorporacion {
        normalizados,
        anadidos,
    }
}

/// Quita la carpeta a los perfiles que fijan la por defecto de su herramienta. Devuelve cuántos.
fn quitar_carpetas_por_defecto(perfiles: &mut [PerfilDeAgente], personal: &Path) -> usize {
    let mut quitadas = 0;
    for perfil in perfiles.iter_mut() {
        let fija_la_por_defecto = perfil.directorio_configuracion().is_some()
            && cuenta_del_perfil(perfil, personal)
                .is_some_and(|cuenta| cuenta.origen == OrigenDeLaCuenta::PorDefecto);
        if fija_la_por_defecto {
            perfil.quitar_directorio();
            quitadas += 1;
        }
    }
    quitadas
}

/// El perfil cuya cuenta es `cuenta`: misma herramienta y misma carpeta.
fn perfil_de_la_cuenta<'a>(
    perfiles: &'a [PerfilDeAgente],
    cuenta: &Cuenta,
    personal: &Path,
) -> Option<&'a PerfilDeAgente> {
    perfiles.iter().find(|perfil| {
        cuenta_del_perfil(perfil, personal).is_some_and(|suya| {
            suya.herramienta == cuenta.herramienta && misma_carpeta(&suya.carpeta, &cuenta.carpeta)
        })
    })
}

/// Instancia que pide una cuenta por su carpeta: la 1 la por defecto y N una carpeta de la
/// carpeta personal que se llama como la por defecto seguida de N (sin ceros a la izquierda,
/// desde la 2). Las demás no piden ninguna.
fn instancia_pedida(cuenta: &Cuenta, personal: &Path) -> Option<u32> {
    if cuenta.es_la_por_defecto() {
        return Some(INSTANCIA_DE_LA_CUENTA_POR_DEFECTO);
    }
    let por_defecto = cuenta.herramienta.carpeta_por_defecto(personal);
    let prefijo = por_defecto.file_name()?.to_str()?;
    if !cuenta
        .carpeta
        .parent()
        .is_some_and(|padre| misma_carpeta(padre, personal))
    {
        return None;
    }
    let numero = cuenta
        .carpeta
        .file_name()?
        .to_str()?
        .strip_prefix(prefijo)?;
    if numero.is_empty() || numero.starts_with('0') || !numero.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    numero
        .parse()
        .ok()
        .filter(|&instancia| instancia >= PRIMERA_INSTANCIA_CON_NUMERO)
}

/// Perfil de una cuenta sin perfil: la instancia que pide si está libre o, si no, la siguiente a
/// la mayor de su herramienta. La cuenta por defecto no fija carpeta; las de la carpeta personal
/// la fijan con «~/» y las demás, con su ruta completa. Nada si la ruta no es texto válido.
fn perfil_nuevo(
    perfiles: &[PerfilDeAgente],
    cuenta: &Cuenta,
    personal: &Path,
) -> Option<PerfilDeAgente> {
    let tipo = herramienta_del_perfil(cuenta.herramienta);
    let libre = |instancia: &u32| {
        !perfiles
            .iter()
            .any(|perfil| perfil.tipo() == tipo && perfil.instancia() == *instancia)
    };
    let instancia = instancia_pedida(cuenta, personal)
        .filter(libre)
        .unwrap_or_else(|| siguiente_instancia(perfiles, tipo));
    let perfil = PerfilDeAgente::new(tipo, instancia);
    if cuenta.es_la_por_defecto() {
        return Some(perfil);
    }
    Some(perfil.con_directorio(&carpeta_para_el_perfil(&cuenta.carpeta, personal)?))
}

/// La instancia siguiente a la mayor de la herramienta, o la 1 si no tiene ninguna.
fn siguiente_instancia(perfiles: &[PerfilDeAgente], tipo: HerramientaDelPerfil) -> u32 {
    perfiles
        .iter()
        .filter(|perfil| perfil.tipo() == tipo)
        .map(PerfilDeAgente::instancia)
        .max()
        .map_or(INSTANCIA_DE_LA_CUENTA_POR_DEFECTO, |mayor| mayor + 1)
}

/// Cómo se escribe una carpeta en el perfil: «~/<nombre>» si cuelga de la carpeta personal y la
/// ruta completa si no. Nada si no es texto válido: guardarla con caracteres sustituidos daría otra
/// carpeta, que ya no casaría con su cuenta.
fn carpeta_para_el_perfil(carpeta: &Path, personal: &Path) -> Option<String> {
    let en_la_personal = carpeta
        .parent()
        .is_some_and(|padre| misma_carpeta(padre, personal));
    match carpeta.file_name() {
        Some(nombre) if en_la_personal => {
            Some(format!("{PREFIJO_CARPETA_PERSONAL}{}", nombre.to_str()?))
        }
        _ => carpeta.to_str().map(str::to_owned),
    }
}

/// Añade un perfil detrás del último de su herramienta, o al final si no hay ninguno.
fn anadir_perfil(perfiles: &mut Vec<PerfilDeAgente>, nuevo: PerfilDeAgente) {
    let posicion = perfiles
        .iter()
        .rposition(|perfil| perfil.tipo() == nuevo.tipo())
        .map_or(perfiles.len(), |ultimo| ultimo + 1);
    perfiles.insert(posicion, nuevo);
}

/// Por qué no se puede dar de alta una cuenta a mano.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AltaRechazada {
    /// La herramienta no tiene cuentas separadas por carpeta (Gemini, Programator).
    SinCuentas,
    /// Otra cuenta de la misma herramienta ya usa esa carpeta; lleva la identidad de su perfil
    /// («Claude 2») o, si no tiene perfil, la carpeta.
    CarpetaEnUso(String),
    /// No se sabe cuál es la carpeta personal del usuario, y sin ella no se puede saber si la
    /// carpeta es la de otra cuenta.
    SinCarpetaPersonal,
    /// La ruta de la carpeta no es texto válido y el perfil no la podría guardar tal cual.
    RutaNoValida,
}

/// Da de alta a mano la cuenta de `herramienta` que vive en `carpeta`, para las que la búsqueda no
/// encuentra (CLAUDE_CONFIG_DIR o CODEX_HOME en otra ruta). La valida la biblioteca contra las
/// cuentas de los perfiles y las `encontradas`; el perfil nuevo se compone como el de una cuenta
/// encontrada (instancia pedida por el nombre si está libre, carpeta con «~/» si cuelga de la
/// personal) y va detrás del último de su herramienta. Devuelve su identidad.
///
/// # Errors
///
/// [`AltaRechazada`] si la herramienta no admite alta manual, si otra cuenta suya ya usa la
/// carpeta o si su ruta no es texto válido.
pub fn dar_de_alta(
    perfiles: &mut Vec<PerfilDeAgente>,
    herramienta: HerramientaDelPerfil,
    carpeta: &Path,
    personal: &Path,
    encontradas: &[Cuenta],
) -> Result<String, AltaRechazada> {
    let biblioteca = herramienta_de_la_biblioteca(herramienta).ok_or(AltaRechazada::SinCuentas)?;
    let mut cuentas: Vec<Cuenta> = perfiles
        .iter()
        .filter_map(|perfil| cuenta_del_perfil(perfil, personal))
        .collect();
    cuentas.extend_from_slice(encontradas);
    let nueva = CuentaManual {
        herramienta: biblioteca,
        carpeta: carpeta.to_path_buf(),
    };
    if let Err(rechazo) = cuentas_de_agentes::validar_alta(&nueva, &cuentas) {
        return Err(match rechazo {
            RechazoDeAlta::SinVariableDeCuenta => AltaRechazada::SinCuentas,
            RechazoDeAlta::CarpetaYaUsada(usada) => AltaRechazada::CarpetaEnUso(
                perfil_de_la_cuenta(perfiles, &usada, personal).map_or_else(
                    || usada.carpeta.display().to_string(),
                    PerfilDeAgente::nombre,
                ),
            ),
        });
    }
    let cuenta = Cuenta {
        herramienta: biblioteca,
        carpeta: nueva.carpeta,
        origen: OrigenDeLaCuenta::Manual,
    };
    let nuevo = perfil_nuevo(perfiles, &cuenta, personal).ok_or(AltaRechazada::RutaNoValida)?;
    let nombre = nuevo.nombre();
    anadir_perfil(perfiles, nuevo);
    Ok(nombre)
}

/// Si dos rutas son la misma carpeta, con la regla de la biblioteca: por su forma canónica cuando
/// existen (enlaces, unidades sustituidas) y en todo caso sin distinguir la barra final ni, en
/// Windows, el tipo de barra ni mayúsculas. Tiene que ser la misma regla, porque la biblioteca
/// descarta con ella las cuentas manuales repetidas y aquí se casan con ella perfiles y cuentas;
/// la biblioteca no la exporta.
fn misma_carpeta(una: &Path, otra: &Path) -> bool {
    comparable(una) == comparable(otra)
}

/// Forma de una ruta para compararla (ver [`misma_carpeta`]).
fn comparable(ruta: &Path) -> String {
    let canonica = ruta.canonicalize().unwrap_or_else(|_| ruta.to_path_buf());
    // En Windows la barra invertida separa carpetas; en los demás sistemas es un carácter válido
    // de un nombre, y «a\b» no es la misma carpeta que «a/b».
    let texto = if cfg!(windows) {
        canonica.to_string_lossy().replace('\\', "/")
    } else {
        canonica.to_string_lossy().into_owned()
    };
    let texto = texto
        .strip_prefix(PREFIJO_DE_RUTA_EXTENDIDA)
        .unwrap_or(&texto)
        .trim_end_matches('/');
    if cfg!(windows) {
        texto.to_lowercase()
    } else {
        texto.to_owned()
    }
}
