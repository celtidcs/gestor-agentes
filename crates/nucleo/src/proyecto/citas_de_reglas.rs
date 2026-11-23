//! Aviso de código del proyecto que cita los archivos de reglas que la aplicación reescribe en
//! cada lanzamiento (AGENTS.md, CLAUDE.md y GEMINI.md). Una prueba que lea uno de ellos se rompe
//! cuando cambia la plantilla, sin que nadie haya tocado el proyecto (pasó en un proyecto con la 3.5.1).
//!
//! Solo se miran archivos de código, por su extensión, y se saltan las carpetas de compilación,
//! de dependencias y ocultas. El recorrido tiene topes para que la preparación siga siendo rápida.

use std::fs;
use std::io;
use std::path::Path;

use super::catalogo::{RUTA_GEMINI_ANTERIOR, RUTA_PUNTERO_CLAUDE, RUTA_REGLAS};

/// Archivos que se buscan en el código.
pub const ARCHIVOS_CITADOS: [&str; 3] = [RUTA_REGLAS, RUTA_PUNTERO_CLAUDE, RUTA_GEMINI_ANTERIOR];

/// Extensiones de código que se miran, en minúsculas.
const EXTENSIONES_DE_CODIGO: [&str; 22] = [
    "rs", "py", "js", "mjs", "cjs", "ts", "tsx", "jsx", "go", "java", "kt", "cs", "c", "cc", "cpp",
    "h", "hpp", "rb", "php", "ps1", "sh", "swift",
];

/// Carpetas que nunca se recorren, además de las que empiezan por punto.
const CARPETAS_SALTADAS: [&str; 7] = [
    "target",
    "node_modules",
    "dist",
    "build",
    "out",
    "vendor",
    "__pycache__",
];

/// Archivos como mucho que se leen: un proyecto mayor se mira solo en parte.
const MAXIMO_DE_ARCHIVOS: usize = 5_000;

/// Tamaño máximo de un archivo que se lee, en bytes: uno mayor no es código escrito a mano.
const MAXIMO_DE_BYTES: u64 = 1_048_576;

/// Archivos que se citan como mucho en el aviso.
pub const MAXIMO_DE_CITAS: usize = 10;

/// Rutas relativas, con barras normales, de los archivos de código que citan alguno de
/// [`ARCHIVOS_CITADOS`], por orden de recorrido y como mucho [`MAXIMO_DE_CITAS`].
///
/// # Errors
///
/// El error de disco al leer la raíz. Las carpetas y archivos que no se pueden leer se saltan.
pub fn buscar(raiz: &Path) -> io::Result<Vec<String>> {
    let mut citas = Vec::new();
    let mut leidos = 0;
    let mut pendientes = vec![raiz.to_path_buf()];
    let mut primera = true;
    while let Some(carpeta) = pendientes.pop() {
        let entradas = match fs::read_dir(&carpeta) {
            Ok(entradas) => entradas,
            Err(error) if primera => return Err(error),
            Err(_) => continue,
        };
        primera = false;
        let mut hijas: Vec<_> = entradas.filter_map(Result::ok).map(|e| e.path()).collect();
        hijas.sort();
        for ruta in hijas.into_iter().rev() {
            let nombre = ruta
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if ruta.is_dir() {
                if !nombre.starts_with('.') && !CARPETAS_SALTADAS.contains(&nombre.as_str()) {
                    pendientes.push(ruta);
                }
                continue;
            }
            if !es_codigo(&nombre) || !ruta.metadata().is_ok_and(|m| m.len() <= MAXIMO_DE_BYTES) {
                continue;
            }
            leidos += 1;
            if leidos > MAXIMO_DE_ARCHIVOS {
                return Ok(citas);
            }
            let Ok(bytes) = fs::read(&ruta) else {
                continue;
            };
            if cita_alguno(&String::from_utf8_lossy(&bytes)) {
                let relativa = ruta.strip_prefix(raiz).unwrap_or(&ruta);
                citas.push(relativa.to_string_lossy().replace('\\', "/"));
                if citas.len() >= MAXIMO_DE_CITAS {
                    return Ok(citas);
                }
            }
        }
    }
    Ok(citas)
}

/// Si un texto cita alguno de los archivos de reglas por su nombre.
#[must_use]
pub fn cita_alguno(texto: &str) -> bool {
    ARCHIVOS_CITADOS
        .iter()
        .any(|archivo| texto.contains(archivo))
}

/// Si un nombre de archivo es de código, por su extensión.
fn es_codigo(nombre: &str) -> bool {
    nombre
        .rsplit_once('.')
        .is_some_and(|(_, ext)| EXTENSIONES_DE_CODIGO.contains(&ext.to_lowercase().as_str()))
}
