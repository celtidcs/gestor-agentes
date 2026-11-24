//! Nombres que comparten la aplicación, el lanzador y el vigilante: carpetas y archivos de
//! .gestor/, variables de entorno, el rastro del vigilante y sus opciones. Están aquí, en una sola
//! fuente, para que un cambio en uno no deje desacordados a los demás.

use std::path::{Path, PathBuf};

/// Carpeta de trabajo de los agentes en cada proyecto.
pub const CARPETA_GESTOR: &str = ".gestor";

/// Carpeta del canal dentro de .gestor/.
pub const CARPETA_CANAL: &str = "canal";

/// Estado común del proyecto, dentro del canal.
pub const ARCHIVO_ESTADO: &str = "estado.md";

/// Variable de entorno con la identidad del agente, como «Claude 2».
pub const VARIABLE_DE_IDENTIDAD: &str = "GESTOR_AGENTE";

/// Variable de entorno con la carpeta del proyecto.
pub const VARIABLE_DEL_PROYECTO: &str = "GESTOR_PROYECTO";

/// Guía que Programator publica en el canal al arrancar: no es la bandeja de nadie.
pub const GUIA_DE_PROGRAMATOR: &str = "como-encargar-a-programator.md";

/// Archivos del canal que no son la bandeja de nadie, en minúsculas: el estado común y los que
/// Programator reserva en su propio canal (src/protocolo/canal.rs del proyecto Programator).
pub const ARCHIVOS_DEL_CANAL_QUE_NO_SON_BANDEJAS: [&str; 4] = [
    ARCHIVO_ESTADO,
    "readme.md",
    "indice.md",
    GUIA_DE_PROGRAMATOR,
];

/// Prefijos de los históricos que se guardan en el canal, con y sin tilde, como los reconoce
/// Programator. Un histórico no es una bandeja viva.
pub const PREFIJOS_DE_HISTORICO: [&str; 2] = ["historico-", "histórico-"];

/// Extensión de los archivos del canal.
pub const EXTENSION_DEL_CANAL: &str = ".md";

/// Carpeta del vigilante: en .gestor/ tiene su ejecutable y, en la carpeta de cada agente, su
/// recuerdo, su latido y su registro de avisos.
pub const CARPETA_DEL_VIGILANTE: &str = "vigilante";

/// Nombre del ejecutable del vigilante, sin la extensión de Windows.
pub const NOMBRE_DEL_VIGILANTE: &str = "gestor-vigilante";

/// Archivo de relevo, en la carpeta del agente: la marca del único vigilante vivo.
pub const ARCHIVO_DE_RELEVO: &str = "vigilante.pid";

/// Latido de la vigilancia, en la carpeta del recuerdo.
pub const ARCHIVO_VIVO: &str = "vivo.json";

/// Registro de los avisos, en la carpeta del recuerdo.
pub const ARCHIVO_DE_AVISOS: &str = "avisos.log";

/// Etiqueta de avisos.log para un encargo.
pub const ETIQUETA_DE_ENCARGO: &str = "encargo";

/// Etiqueta de avisos.log para la respuesta de Programator a un encargo.
pub const ETIQUETA_DE_RESPUESTA: &str = "respuesta";

/// Etiqueta de avisos.log para una mutación viva.
pub const ETIQUETA_DE_MUTACION: &str = "mutacion";

/// Separador de las etiquetas en una línea de avisos.log.
pub const SEPARADOR_DE_ETIQUETAS: char = ',';

/// Claves de vivo.json.
pub mod claves_de_vivo {
    /// Identidad del agente.
    pub const IDENTIDAD: &str = "identidad";
    /// Número de proceso del vigilante.
    pub const PID: &str = "pid";
    /// Hora de arranque del proceso, en segundos desde 1970.
    pub const ARRANQUE: &str = "arranque";
    /// Modo del vigilante.
    pub const MODO: &str = "modo";
    /// Desde cuándo vigila, en segundos desde 1970.
    pub const DESDE: &str = "desde";
}

/// Opciones de la línea de órdenes del vigilante.
pub mod opciones_del_vigilante {
    /// Una pasada y termina.
    pub const UNA_VEZ: &str = "--una-vez";
    /// Espera hasta un cambio con etiqueta.
    pub const HASTA_CAMBIO: &str = "--hasta-cambio";
    /// Vigila toda la sesión y entrega cada aviso.
    pub const PERSISTENTE: &str = "--persistente";
    /// Anota la sesión de Codex y termina.
    pub const ANOTAR_HILO: &str = "--anotar-hilo";
    /// Terminal que abrió Codex.
    pub const DESDE_TERMINAL: &str = "--desde-terminal";
    /// Identidad del agente.
    pub const AGENTE: &str = "--agente";
    /// Vida máxima en segundos.
    pub const VIDA_SEGUNDOS: &str = "--vida-segundos";
    /// Cómo se entrega el aviso.
    pub const ENTREGA: &str = "--entrega";
    /// Valor de --entrega para Codex.
    pub const ENTREGA_CODEX: &str = "codex";
    /// Proceso ya en segundo plano.
    pub const HIJO: &str = "--hijo";
    /// Proceso del agente al que sirve.
    pub const DUENO: &str = "--dueno";
    /// Ejecutable de Codex.
    pub const CODEX: &str = "--codex";
    /// Sesión de Codex.
    pub const HILO: &str = "--hilo";
    /// Carpeta .gestor/.
    pub const GESTOR: &str = "--gestor";
    /// Segundos entre pasadas.
    pub const INTERVALO_SEGUNDOS: &str = "--intervalo-segundos";
}

/// Ruta de estado.md, relativa a la raíz del proyecto, con barras normales.
#[must_use]
pub fn ruta_relativa_del_estado() -> String {
    format!("{CARPETA_GESTOR}/{CARPETA_CANAL}/{ARCHIVO_ESTADO}")
}

/// Ruta de estado.md en el proyecto de `raiz`.
#[must_use]
pub fn ruta_del_estado(raiz: &Path) -> PathBuf {
    raiz.join(CARPETA_GESTOR)
        .join(CARPETA_CANAL)
        .join(ARCHIVO_ESTADO)
}

/// Ruta relativa del ejecutable del vigilante dentro del proyecto, con el separador `separador` y
/// la extensión `extension` («.exe» o nada).
#[must_use]
pub fn ruta_relativa_del_vigilante(separador: char, extension: &str) -> String {
    format!("{CARPETA_GESTOR}{separador}{CARPETA_DEL_VIGILANTE}{separador}{NOMBRE_DEL_VIGILANTE}{extension}")
}

/// Si un archivo del canal es un histórico. No distingue mayúsculas.
#[must_use]
pub fn es_historico(nombre: &str) -> bool {
    let nombre = nombre.to_lowercase();
    PREFIJOS_DE_HISTORICO
        .iter()
        .any(|prefijo| nombre.starts_with(prefijo))
}

/// Si un archivo de .gestor/canal/ es un .md vivo: ni un histórico ni la guía de Programator. No
/// distingue mayúsculas.
#[must_use]
pub fn es_archivo_vivo_del_canal(nombre: &str) -> bool {
    let minusculas = nombre.to_lowercase();
    minusculas.ends_with(EXTENSION_DEL_CANAL)
        && minusculas != GUIA_DE_PROGRAMATOR
        && !es_historico(&minusculas)
}

/// Si un archivo de .gestor/canal/ es la bandeja de un agente: un .md vivo que no es uno de
/// [`ARCHIVOS_DEL_CANAL_QUE_NO_SON_BANDEJAS`]. No distingue mayúsculas.
#[must_use]
pub fn es_bandeja(nombre: &str) -> bool {
    let minusculas = nombre.to_lowercase();
    es_archivo_vivo_del_canal(&minusculas)
        && !ARCHIVOS_DEL_CANAL_QUE_NO_SON_BANDEJAS.contains(&minusculas.as_str())
}

/// Carpeta de agy en la carpeta personal, donde guarda sus proyectos (no la de una cuenta: las
/// carpetas de las cuentas las da la biblioteca deteccion-agentes).
pub const CARPETA_DE_GEMINI: &str = ".gemini";
