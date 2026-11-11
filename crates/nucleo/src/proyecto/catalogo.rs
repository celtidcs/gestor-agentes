//! Rutas y textos canónicos de los archivos de instrucciones de un proyecto.
//!
//! Desde la 3.0.0 hay un solo archivo de reglas para todos los agentes, AGENTS.md, que la
//! aplicación escribe entero con la cabecera de identidad común, el núcleo común y las normas de
//! la carpeta proyecto. CLAUDE.md es solo el puntero que hace que Claude Code lo cargue junto con
//! el archivo propio del proyecto, y los documentos de consulta de .gestor/consulta/ recogen lo que
//! solo hace falta en momentos concretos. Lo propio de cada proyecto vive en
//! .gestor/instrucciones-del-proyecto.md, que la aplicación crea una vez y no vuelve a tocar.

use super::localizador::InstruccionesDelEspacioDeTrabajo;
use crate::instrucciones::migracion::TextosDeLaAplicacion;
use crate::instrucciones::{formato, marcas};
use crate::plantillas::{self, DocumentoDeConsulta};

/// Archivo único de reglas de todos los agentes. Es el nombre que leen Codex y Antigravity por su
/// cuenta (https://learn.chatgpt.com/docs/agent-configuration/agents-md y
/// https://antigravity.google/docs/rules).
pub const RUTA_REGLAS: &str = "AGENTS.md";

/// Puntero de Claude Code, que no lee AGENTS.md si hay un CLAUDE.md
/// (https://code.claude.com/docs/en/memory).
pub const RUTA_PUNTERO_CLAUDE: &str = "CLAUDE.md";

/// Archivo de instrucciones de Gemini en las versiones anteriores. Desde la 3.0.0 no se genera:
/// Gemini lee AGENTS.md.
pub const RUTA_GEMINI_ANTERIOR: &str = "GEMINI.md";

/// Archivo propio del proyecto, que la aplicación solo crea si falta.
pub const RUTA_INSTRUCCIONES_DEL_PROYECTO: &str = ".gestor/instrucciones-del-proyecto.md";

/// Antigua memoria compartida del sistema de relevo anterior al canal.
pub const RUTA_MEMORIA_COMPARTIDA: &str = ".gestor/memoria-compartida.md";

/// Archivos de instrucciones de las versiones anteriores, en el orden en que se recoge su texto
/// propio al pasar al archivo único.
pub const ARCHIVOS_DE_INSTRUCCIONES_ANTERIORES: [&str; 3] =
    [RUTA_PUNTERO_CLAUDE, RUTA_REGLAS, RUTA_GEMINI_ANTERIOR];

/// Texto de la región de entorno cuando no se han encontrado instrucciones del espacio de trabajo.
pub const ENTORNO_SIN_CONFIGURAR: &str = "Gestor Agentes no ha encontrado instrucciones del espacio de trabajo (CLAUDE.md, AGENTS.md o GEMINI.md) en ninguna carpeta superior a este proyecto.";

/// Marcador del encabezado de entorno que se sustituye por la ruta del archivo de origen.
const MARCADOR_RUTA: &str = "{{RUTA}}";

/// Archivo que la aplicación escribe entero en cada lanzamiento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivoGestionado {
    /// Ruta relativa al proyecto, con barras normales.
    pub ruta_relativa: &'static str,
    /// Texto vigente, con saltos \n y salto final.
    pub texto: String,
}

/// Archivos que la aplicación escribe enteros: el de reglas, el puntero de Claude y los
/// documentos de consulta.
#[must_use]
pub fn archivos_gestionados(
    espacio_de_trabajo: Option<&InstruccionesDelEspacioDeTrabajo>,
) -> Vec<ArchivoGestionado> {
    let mut archivos = vec![
        ArchivoGestionado {
            ruta_relativa: RUTA_REGLAS,
            texto: reglas(espacio_de_trabajo),
        },
        ArchivoGestionado {
            ruta_relativa: RUTA_PUNTERO_CLAUDE,
            texto: puntero_claude(),
        },
    ];
    archivos.extend(
        DocumentoDeConsulta::TODOS
            .iter()
            .map(|documento| ArchivoGestionado {
                ruta_relativa: documento.ruta_relativa(),
                texto: documento.texto() + "\n",
            }),
    );
    archivos
}

/// Texto completo de AGENTS.md: cabecera, núcleo común y entorno, cada uno en su región.
#[must_use]
pub fn reglas(espacio_de_trabajo: Option<&InstruccionesDelEspacioDeTrabajo>) -> String {
    [
        region(marcas::CABECERA, &plantillas::cabecera()),
        region(marcas::NUCLEO_COMUN, &plantillas::nucleo_comun()),
        region(marcas::ENTORNO, &preparar_entorno(espacio_de_trabajo)),
    ]
    .join("\n\n")
        + "\n"
}

/// Texto completo de CLAUDE.md: el puntero, dentro de su región para que el archivo se reconozca
/// como gestionado.
#[must_use]
pub fn puntero_claude() -> String {
    region(marcas::PUNTERO, &plantillas::puntero_claude()) + "\n"
}

/// Lo que la aplicación escribió en versiones anteriores y no es del proyecto, para no recogerlo
/// al pasar al archivo único. `instrucciones_personalizadas_antiguas` es la plantilla de CLAUDE.md
/// que el usuario guardó con una versión anterior, si existe.
#[must_use]
pub fn textos_de_la_aplicacion(
    instrucciones_personalizadas_antiguas: Option<&str>,
) -> TextosDeLaAplicacion {
    let mut plantillas_completas = plantillas::plantillas_antiguas();
    if let Some(antiguas) =
        instrucciones_personalizadas_antiguas.filter(|t| !t.chars().all(char::is_whitespace))
    {
        plantillas_completas.push(formato::normalizar_para_comparar(antiguas));
    }
    TextosDeLaAplicacion {
        plantillas_completas,
        textos_sueltos: plantillas::textos_anteriores_de_la_aplicacion(),
    }
}

/// Bloque de entorno: un encabezado que dice de dónde sale y, debajo, el texto de las
/// instrucciones del espacio de trabajo con cualquier marca de región desactivada, para que un
/// archivo ajeno no pueda romper la estructura de este.
fn preparar_entorno(espacio_de_trabajo: Option<&InstruccionesDelEspacioDeTrabajo>) -> String {
    let Some(espacio) = espacio_de_trabajo else {
        return ENTORNO_SIN_CONFIGURAR.to_owned();
    };
    let normalizado = formato::normalizar(&espacio.texto);
    let texto = normalizado.trim_matches('\n');
    if texto.chars().all(char::is_whitespace) {
        return ENTORNO_SIN_CONFIGURAR.to_owned();
    }
    let encabezado =
        plantillas::encabezado_entorno().replace(MARCADOR_RUTA, &espacio.ruta.to_string_lossy());
    format!(
        "{}\n\n{}",
        formato::normalizar(&encabezado),
        marcas::neutralizar(texto)
    )
}

/// Región con sus marcas de apertura y cierre, sin salto final.
fn region(id: &str, contenido: &str) -> String {
    format!("{}\n{contenido}\n{}", marcas::inicio(id), marcas::fin(id))
}
