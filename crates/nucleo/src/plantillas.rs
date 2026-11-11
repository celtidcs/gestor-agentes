//! Plantillas canónicas incluidas en el binario: el archivo único de reglas (cabecera común,
//! núcleo y encabezado del entorno), el puntero de Claude, los documentos de consulta, el archivo
//! propio de cada proyecto, los esqueletos de .gestor/ y los textos de versiones anteriores que
//! sirven para reconocer lo que escribió la aplicación en un proyecto heredado.
//!
//! Los textos se devuelven normalizados con \n y sin saltos finales. Los archivos están en
//! crates/nucleo/plantillas/ y se incluyen al compilar, así que el ejecutable portable no depende
//! de ninguna carpeta externa. Los de legado/ son copia byte a byte de los que escribieron
//! versiones anteriores y no se cambian nunca, porque sirven para reconocer sus textos.

use crate::agentes::IdentidadAgente;
use crate::instrucciones::formato::{self, FormatoDeTexto};

/// Nombre de la aplicación en los textos que genera.
pub const NOMBRE_DE_LA_APLICACION: &str = "Gestor Agentes";

/// Nombre que llevaban los mismos textos hasta la 2.3.2.
pub const NOMBRE_ANTERIOR: &str = "Gestor Claude";

/// Texto que la 2.x escribía en lugar de `vigente`: el mismo con el nombre anterior de la
/// aplicación. Sirve para reconocer los textos que dejó la 2.x en un proyecto.
#[must_use]
pub fn version_de_la_2x(vigente: &str) -> String {
    vigente.replace(NOMBRE_DE_LA_APLICACION, NOMBRE_ANTERIOR)
}

/// Marcador que se sustituye por el nombre del agente en los esqueletos de .gestor/.
pub const MARCADOR_AGENTE: &str = "{{AGENTE}}";

const NUCLEO_COMUN: &[u8] = include_bytes!("../plantillas/instrucciones/nucleo-comun.md");
const CABECERA: &[u8] = include_bytes!("../plantillas/instrucciones/cabecera-comun.md");
const ENCABEZADO_ENTORNO: &[u8] =
    include_bytes!("../plantillas/instrucciones/encabezado-entorno.md");
const ENCABEZADO_PENDIENTE: &[u8] =
    include_bytes!("../plantillas/instrucciones/encabezado-pendiente.md");
const PUNTERO_CLAUDE: &[u8] = include_bytes!("../plantillas/instrucciones/puntero-claude.md");
const AVISO_ARCHIVADOS: &[u8] = include_bytes!("../plantillas/instrucciones/aviso-archivados.md");
const CONSULTA_CODIGO: &[u8] = include_bytes!("../plantillas/consulta/codigo.md");
const CONSULTA_METODO: &[u8] = include_bytes!("../plantillas/consulta/metodo.md");
const INSTRUCCIONES_DEL_PROYECTO: &[u8] =
    include_bytes!("../plantillas/gestor/instrucciones-del-proyecto.md");
const ESTADO: &[u8] = include_bytes!("../plantillas/gestor/estado.md");
const BANDEJA: &[u8] = include_bytes!("../plantillas/gestor/bandeja.md");
const TAREA_EN_CURSO: &[u8] = include_bytes!("../plantillas/gestor/tarea-en-curso.md");
const VERIFICACIONES: &[u8] = include_bytes!("../plantillas/gestor/verificaciones.md");
const FUNCIONES: &[u8] = include_bytes!("../plantillas/gestor/funciones.md");
const ORDENES_DEL_DIRECTOR: &[u8] = include_bytes!("../plantillas/gestor/ordenes-del-director.md");
const LEGADO_CLAUDE_V1: &[u8] = include_bytes!("../plantillas/legado/claude-instructions-v1.txt");
const LEGADO_GEMINI_V1: &[u8] = include_bytes!("../plantillas/legado/gemini-md-v1.md");
const LEGADO_ZONA_DEL_PROYECTO: &[u8] = include_bytes!("../plantillas/legado/zona-del-proyecto.md");
const LEGADO_ENCABEZADO: &[u8] = include_bytes!("../plantillas/legado/encabezado-legado.md");
const LEGADO_PUNTERO_MEMORIA: &[u8] =
    include_bytes!("../plantillas/legado/puntero-memoria-compartida.md");

/// Esqueleto de un archivo de .gestor/ que se crea si no existe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EsqueletoGestor {
    /// .gestor/canal/estado.md, común.
    Estado,
    /// .gestor/ordenes-del-director.md, común.
    OrdenesDelDirector,
    /// Bandeja de un agente en .gestor/canal/.
    Bandeja,
    /// Diario de tarea de un agente.
    TareaEnCurso,
    /// Acta de verificaciones de un agente.
    Verificaciones,
    /// Registro de funciones de un agente.
    Funciones,
}

impl EsqueletoGestor {
    /// Todos los esqueletos.
    pub const TODOS: [Self; 6] = [
        Self::Estado,
        Self::OrdenesDelDirector,
        Self::Bandeja,
        Self::TareaEnCurso,
        Self::Verificaciones,
        Self::Funciones,
    ];

    /// Esqueletos de la carpeta propia de cada agente, en el orden en que los crea la 2.3.2.
    pub const DE_LA_CARPETA_PROPIA: [Self; 3] =
        [Self::TareaEnCurso, Self::Verificaciones, Self::Funciones];

    /// Nombre del archivo, que es también el de la plantilla.
    #[must_use]
    pub const fn nombre_archivo(self) -> &'static str {
        match self {
            Self::Estado => "estado.md",
            Self::OrdenesDelDirector => "ordenes-del-director.md",
            Self::Bandeja => "bandeja.md",
            Self::TareaEnCurso => "tarea-en-curso.md",
            Self::Verificaciones => "verificaciones.md",
            Self::Funciones => "funciones.md",
        }
    }

    const fn bytes(self) -> &'static [u8] {
        match self {
            Self::Estado => ESTADO,
            Self::OrdenesDelDirector => ORDENES_DEL_DIRECTOR,
            Self::Bandeja => BANDEJA,
            Self::TareaEnCurso => TAREA_EN_CURSO,
            Self::Verificaciones => VERIFICACIONES,
            Self::Funciones => FUNCIONES,
        }
    }
}

/// Documento de consulta que Gestor Agentes escribe en .gestor/consulta/ y al que remite el
/// núcleo. Se lee solo cuando llega el momento que dice cada uno.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentoDeConsulta {
    /// Estándares de código: se lee antes de escribir, cambiar o revisar código.
    Codigo,
    /// Control del proyecto, registro de funciones, documentación y publicación.
    Metodo,
}

impl DocumentoDeConsulta {
    /// Todos los documentos de consulta.
    pub const TODOS: [Self; 2] = [Self::Codigo, Self::Metodo];

    /// Ruta relativa al proyecto, con barras normales.
    #[must_use]
    pub const fn ruta_relativa(self) -> &'static str {
        match self {
            Self::Codigo => ".gestor/consulta/codigo.md",
            Self::Metodo => ".gestor/consulta/metodo.md",
        }
    }

    /// Texto vigente del documento.
    #[must_use]
    pub fn texto(self) -> String {
        leer(match self {
            Self::Codigo => CONSULTA_CODIGO,
            Self::Metodo => CONSULTA_METODO,
        })
    }
}

/// Texto del núcleo común del archivo único de reglas.
#[must_use]
pub fn nucleo_comun() -> String {
    leer(NUCLEO_COMUN)
}

/// Cabecera de identidad del archivo único de reglas, la misma para todas las herramientas.
#[must_use]
pub fn cabecera() -> String {
    leer(CABECERA)
}

/// Encabezado del bloque de entorno, con el marcador {{RUTA}} del archivo de origen.
#[must_use]
pub fn encabezado_entorno() -> String {
    leer(ENCABEZADO_ENTORNO)
}

/// Encabezado del bloque pendiente de consolidar, con el marcador {{FECHA}}.
#[must_use]
pub fn encabezado_pendiente() -> String {
    leer(ENCABEZADO_PENDIENTE)
}

/// Frase que presenta, dentro del bloque pendiente, la lista de archivos del sistema anterior
/// llevados al archivo.
#[must_use]
pub fn aviso_archivados() -> String {
    leer(AVISO_ARCHIVADOS)
}

/// Texto completo de CLAUDE.md: el puntero que hace que Claude Code cargue AGENTS.md y el
/// archivo propio del proyecto.
#[must_use]
pub fn puntero_claude() -> String {
    leer(PUNTERO_CLAUDE)
}

/// Contenido inicial del archivo propio del proyecto, .gestor/instrucciones-del-proyecto.md.
#[must_use]
pub fn instrucciones_del_proyecto() -> String {
    leer(INSTRUCCIONES_DEL_PROYECTO)
}

/// Textos que escribieron versiones anteriores dentro de los archivos de instrucciones, fuera de
/// sus regiones: la zona del proyecto, el encabezado del legado y el aviso de la memoria
/// compartida, cada uno con el nombre vigente de la aplicación y con el de la 2.x. Sirven para no
/// confundirlos con texto del proyecto al migrar. Pueden llevar marcadores {{...}}.
#[must_use]
pub fn textos_anteriores_de_la_aplicacion() -> Vec<String> {
    [
        LEGADO_ZONA_DEL_PROYECTO,
        LEGADO_ENCABEZADO,
        LEGADO_PUNTERO_MEMORIA,
    ]
    .iter()
    .map(|bytes| leer(bytes))
    .flat_map(|texto| [version_de_la_2x(&texto), texto])
    .collect()
}

/// Esqueleto de .gestor/ listo para escribir, con salto de línea final y, si se indica un agente,
/// con su nombre en lugar de [`MARCADOR_AGENTE`].
#[must_use]
pub fn esqueleto_gestor(esqueleto: EsqueletoGestor, agente: Option<&IdentidadAgente>) -> String {
    let texto = leer(esqueleto.bytes());
    let texto = match agente {
        Some(identidad) => texto.replace(MARCADOR_AGENTE, &identidad.nombre()),
        None => texto,
    };
    texto + "\n"
}

/// Plantillas completas que generaron versiones anteriores para un archivo de instrucciones
/// (CLAUDE.md y GEMINI.md de la 1.x), normalizadas para comparar. Un archivo igual a una de
/// ellas no tiene nada propio del proyecto.
#[must_use]
pub fn plantillas_antiguas() -> Vec<String> {
    [LEGADO_CLAUDE_V1, LEGADO_GEMINI_V1]
        .iter()
        .map(|bytes| formato::normalizar_para_comparar(&leer(bytes)))
        .collect()
}

/// Decodifica una plantilla como texto normalizado y sin saltos finales.
fn leer(bytes: &[u8]) -> String {
    let (texto, _) = FormatoDeTexto::leer(bytes);
    texto.trim_end_matches('\n').to_owned()
}
