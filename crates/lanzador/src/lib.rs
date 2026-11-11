//! Lanzador de agentes de Gestor Agentes.
//!
//! Decide cómo se abre cada agente en una terminal (Claude Code, Codex CLI, Gemini con agy y
//! Programator) en cada sistema operativo, y después lo ejecuta. Las dos cosas están separadas:
//!
//! - [`planificar`] es una función pura. Recibe la petición, el sistema y un
//!   [`EntornoDelSistema`] inyectado (búsquedas en el PATH y existencia de archivos), y devuelve
//!   un [`PlanDeLanzamiento`] con el programa, la lista de argumentos, las variables y la
//!   carpeta. Se prueba sin abrir nada.
//! - [`detectar_ejecutable`] dice, también sin efectos, qué programa abriría cada herramienta, o
//!   que no hay ninguno. La interfaz la usa al arrancar para mostrar solo los botones de los
//!   agentes instalados, y [`planificar`] la usa para lanzar, así que las dos nunca discrepan.
//! - [`ejecutar`] lanza un plan ya decidido, sin esperar a que termine.
//!
//! Desde la 3.0.0 no se abren aplicaciones de escritorio: ni
//! la de Codex de la Microsoft Store ni Google Antigravity. Quien las quiera las abre desde su
//! acceso directo.
//!
//! Los argumentos van siempre como lista, nunca concatenados en una cadena de órdenes, salvo
//! donde la plataforma lo obliga (el guion de PowerShell o de bash de la terminal), y ahí se
//! escapan con las reglas del módulo de escapes.
//!
//! Los tipos y firmas públicas de este archivo son un contrato: la interfaz depende de ellos.
//! El comportamiento de referencia es el del lanzador de la 2.3.2, más Programator, que es
//! nuevo. Diferencia declarada frente a la 2.3.2: el escape de PowerShell también duplica las
//! comillas tipográficas.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

mod base64;
mod ejecucion;
mod entorno_real;
mod escapes;
mod planificacion;
mod terminal;
mod textos;

pub use entorno_real::EntornoReal;

/// Variable de entorno con la identidad del agente, que lee la cabecera común de AGENTS.md (el
/// único archivo de reglas desde la 3.0.0) para que cada agente sepa qué bandeja y qué diario
/// son suyos.
pub const VARIABLE_IDENTIDAD: &str = "GESTOR_AGENTE";

/// Herramienta que se abre, siempre en una terminal y en la carpeta del proyecto. Programator es
/// nuevo en la 3.0.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Herramienta {
    /// Claude Code (el programa claude).
    Claude,
    /// Codex CLI (el programa codex).
    Codex,
    /// Gemini con agy, la CLI de Antigravity.
    Gemini,
    /// El arnés Programator, siempre con --ruta apuntando al proyecto abierto. Es un programa de
    /// consola: en Windows se abre en una consola nueva y en Linux dentro de un emulador de
    /// terminal. No se busca: su ejecutable lo fija el perfil.
    Programator,
}

/// Sistema operativo para el que se planifica. Se pasa explícito para poder probar los dos
/// sistemas desde cualquiera de ellos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sistema {
    /// Windows 10 u 11.
    Windows,
    /// Linux con escritorio X11 o Wayland.
    Linux,
}

impl Sistema {
    /// Sistema en el que se está ejecutando el programa.
    #[must_use]
    pub const fn actual() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

/// Lo que la interfaz pide abrir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeticionDeLanzamiento {
    /// Herramienta que se abre.
    pub herramienta: Herramienta,
    /// Identidad del agente tal como se escribe en GESTOR_AGENTE, por ejemplo «Claude 2».
    pub identidad: String,
    /// Carpeta del proyecto abierto.
    pub carpeta_proyecto: PathBuf,
    /// Nombre del proyecto para el título de la ventana.
    pub nombre_proyecto: String,
    /// Carpeta de configuración del perfil (CLAUDE_CONFIG_DIR o CODEX_HOME), si la tiene.
    pub directorio_configuracion: Option<PathBuf>,
    /// Ejecutable fijado en el perfil. Obligatorio para Programator; opcional para el resto. Si
    /// está fijado tiene que existir: no se cae a la búsqueda automática (véase
    /// [`detectar_ejecutable`]).
    pub ejecutable: Option<PathBuf>,
    /// Argumentos del perfil que se pasan al programa después de su nombre, cada uno escapado,
    /// por ejemplo `--model <id> --effort high` para agy o `-c project_doc_max_bytes=65536` para
    /// codex. Solo los usan [`Herramienta::Codex`] y [`Herramienta::Gemini`]; Claude y
    /// Programator los ignoran.
    pub argumentos_extra: Vec<String>,
}

/// Decisión de lanzamiento, completa y sin efectos. En Windows el proceso se crea siempre con
/// una consola nueva: todo lo que se abre es un programa de consola que maneja el usuario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDeLanzamiento {
    /// Programa que se ejecuta.
    pub programa: PathBuf,
    /// Argumentos, uno por elemento.
    pub argumentos: Vec<OsString>,
    /// Variables de entorno que se añaden al proceso hijo.
    pub variables: Vec<(String, OsString)>,
    /// Carpeta de trabajo del proceso hijo.
    pub carpeta: PathBuf,
    /// Carpetas que [`ejecutar`] crea antes de abrir el proceso, como la de configuración del
    /// perfil si todavía no existe.
    pub carpetas_a_crear: Vec<PathBuf>,
}

/// Consultas al sistema que necesita la planificación. Se inyecta para que [`planificar`] sea
/// pura y se pueda probar con un doble.
pub trait EntornoDelSistema {
    /// Busca un programa por su nombre en las carpetas del PATH, por orden. En Windows prueba en
    /// cada carpeta las extensiones dadas, por orden; en Linux, `extensiones` va vacía y se busca
    /// el nombre tal cual.
    fn buscar_en_path(&self, nombre: &str, extensiones: &[&str]) -> Option<PathBuf>;
    /// Carpeta personal del usuario (%USERPROFILE% o $HOME), si se conoce.
    fn carpeta_personal(&self) -> Option<PathBuf>;
    /// Carpeta de datos locales de aplicación (%LOCALAPPDATA% en Windows), donde el instalador
    /// oficial de agy deja su programa. En Linux no existe y devuelve nada.
    fn carpeta_de_datos_locales(&self) -> Option<PathBuf>;
    /// Indica si existe un archivo.
    fn existe(&self, ruta: &Path) -> bool;
}

/// Errores de lanzamiento, con lo necesario para explicárselos al usuario.
#[derive(Debug, thiserror::Error)]
pub enum ErrorDeLanzamiento {
    /// La herramienta no está instalada o no se encuentra.
    #[error("{herramienta} no está instalado o no se encuentra: {detalle}")]
    NoEncontrada {
        /// Nombre de la herramienta.
        herramienta: String,
        /// Qué se buscó y dónde.
        detalle: String,
    },
    /// El perfil no tiene un dato obligatorio, como el ejecutable de Programator.
    #[error("el perfil de {identidad} no está completo: {detalle}")]
    PerfilIncompleto {
        /// Identidad del perfil.
        identidad: String,
        /// Qué falta.
        detalle: String,
    },
    /// Una ruta que hay que escribir dentro del guion de la terminal no es Unicode válido (puede
    /// pasar en Linux). Se rechaza en lugar de escribirla cambiada.
    #[error("la ruta {} no es Unicode válido y no se puede pasar a la terminal", ruta.display())]
    RutaNoUnicode {
        /// Ruta rechazada.
        ruta: PathBuf,
    },
    /// No se pudo crear una carpeta del plan.
    #[error("no se pudo crear la carpeta {carpeta}: {origen}")]
    Carpeta {
        /// Carpeta que se intentó crear.
        carpeta: PathBuf,
        /// Error del sistema.
        #[source]
        origen: std::io::Error,
    },
    /// El sistema no pudo crear el proceso.
    #[error("no se pudo abrir {programa}: {origen}")]
    Proceso {
        /// Programa que se intentó abrir.
        programa: PathBuf,
        /// Error del sistema.
        #[source]
        origen: std::io::Error,
    },
}

/// Decide cómo se abre la herramienta pedida en el sistema indicado.
///
/// No tiene efectos: todo lo que consulta del sistema pasa por `entorno`.
///
/// # Errors
///
/// Devuelve [`ErrorDeLanzamiento::NoEncontrada`] si la herramienta no se localiza o si el
/// ejecutable fijado en el perfil no existe, [`ErrorDeLanzamiento::PerfilIncompleto`] si falta un
/// dato obligatorio del perfil y [`ErrorDeLanzamiento::RutaNoUnicode`] si una ruta que va en el
/// guion de la terminal no es Unicode válido.
pub fn planificar(
    peticion: &PeticionDeLanzamiento,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
    planificacion::planificar(peticion, sistema, entorno)
}

/// Programa que abriría `herramienta`, o `None` si no hay ninguno.
///
/// Si hay `ruta_fijada` (la del perfil), solo vale ella: se devuelve si existe y, si no existe, la
/// respuesta es `None`, sin probar las ubicaciones por defecto, para que una ruta mal puesta se
/// vea como «no encontrado» y no quede tapada por otro programa. Sin ruta fijada se busca donde
/// lo instala cada herramienta: claude y codex en ~/.local/bin y en el PATH (y en Linux también en
/// /usr/local/bin y /usr/bin), y agy además, en Windows, en %LOCALAPPDATA%\agy\bin. Programator no
/// se busca: sin ruta fijada es `None`.
///
/// No tiene efectos: todo lo que consulta del sistema pasa por `entorno`. Es la misma búsqueda que
/// usa [`planificar`].
#[must_use]
pub fn detectar_ejecutable(
    herramienta: Herramienta,
    ruta_fijada: Option<&Path>,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Option<PathBuf> {
    planificacion::detectar_ejecutable(herramienta, ruta_fijada, sistema, entorno)
}

/// Ejecuta un plan sin esperar a que el proceso termine: crea las carpetas del plan y abre el
/// proceso con su carpeta, sus variables y su tipo de ventana. Un hilo aparte recoge el proceso
/// cuando termina, para que no quede como zombi en Linux.
///
/// # Errors
///
/// Devuelve [`ErrorDeLanzamiento::Carpeta`] si no puede crear una carpeta del plan y
/// [`ErrorDeLanzamiento::Proceso`] si el sistema no puede crear el proceso.
pub fn ejecutar(plan: &PlanDeLanzamiento) -> Result<(), ErrorDeLanzamiento> {
    ejecucion::ejecutar(plan)
}
