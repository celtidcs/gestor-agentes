//! Operaciones de disco sobre los archivos de un proyecto: lectura con detección de formato,
//! escritura atómica, creación solo si falta y respaldo antes de una sustitución.
//!
//! Todo lo que se escribe aquí queda dentro de la carpeta del proyecto. Los respaldos van a
//! .gestor/respaldos/, que git ignora porque .gestor/ entero está en .gitignore.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::momento::Momento;
use crate::instrucciones::formato::FormatoDeTexto;

/// Carpeta de respaldos, relativa a la raíz del proyecto.
pub const CARPETA_RESPALDOS: &str = ".gestor/respaldos";

/// Sufijo del archivo temporal que se usa para escribir de forma atómica.
const SUFIJO_TEMPORAL: &str = ".gestor-tmp";

/// Combina la raíz del proyecto con una ruta relativa escrita con barras normales.
#[must_use]
pub fn ruta(raiz: &Path, ruta_relativa: &str) -> PathBuf {
    ruta_relativa
        .split('/')
        .fold(raiz.to_path_buf(), |acumulada, parte| acumulada.join(parte))
}

/// Lee un archivo de texto detectando su formato; nada si no existe.
///
/// # Errors
///
/// Devuelve el error de disco si el archivo existe pero no se puede leer.
pub fn leer(ruta: &Path) -> io::Result<Option<(String, FormatoDeTexto)>> {
    match fs::read(ruta) {
        Ok(bytes) => Ok(Some(FormatoDeTexto::leer(&bytes))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Escribe un archivo de forma atómica: primero en un temporal junto al destino y después lo
/// mueve encima, de modo que un corte a mitad nunca deja el archivo a medias. Crea la carpeta si
/// falta.
///
/// # Errors
///
/// Devuelve el error de disco si no se puede escribir.
pub fn escribir(ruta: &Path, texto_normalizado: &str, formato: FormatoDeTexto) -> io::Result<()> {
    if let Some(carpeta) = ruta.parent().filter(|c| !c.as_os_str().is_empty()) {
        fs::create_dir_all(carpeta)?;
    }
    let mut temporal = ruta.as_os_str().to_owned();
    temporal.push(SUFIJO_TEMPORAL);
    let temporal = PathBuf::from(temporal);
    fs::write(&temporal, formato.codificar(texto_normalizado))?;
    fs::rename(&temporal, ruta)
}

/// Crea un archivo con el formato por defecto solo si todavía no existe. Devuelve verdadero si lo
/// ha creado y falso si ya existía, en cuyo caso no lo toca.
///
/// # Errors
///
/// Devuelve el error de disco si no se puede comprobar o escribir.
pub fn crear_si_no_existe(ruta: &Path, texto_normalizado: &str) -> io::Result<bool> {
    if ruta.try_exists()? {
        return Ok(false);
    }
    escribir(ruta, texto_normalizado, FormatoDeTexto::default())?;
    Ok(true)
}

/// Ruta relativa del respaldo de un archivo para un momento dado, dentro de .gestor/respaldos/ y
/// con barras normales: «.gestor/respaldos/CLAUDE.md.20261110-120000.bak».
#[must_use]
pub fn ruta_de_respaldo(ruta_relativa: &str, momento: Momento) -> String {
    let nombre = ruta_relativa.rsplit('/').next().unwrap_or(ruta_relativa);
    format!(
        "{CARPETA_RESPALDOS}/{nombre}.{}.bak",
        momento.marca_de_tiempo()
    )
}

/// Copia un archivo del proyecto a su ruta de respaldo, byte a byte, sustituyendo un respaldo
/// anterior con el mismo nombre.
///
/// # Errors
///
/// Devuelve el error de disco si la copia falla.
pub fn respaldar(raiz: &Path, ruta_relativa: &str, ruta_relativa_respaldo: &str) -> io::Result<()> {
    let destino = ruta(raiz, ruta_relativa_respaldo);
    if let Some(carpeta) = destino.parent() {
        fs::create_dir_all(carpeta)?;
    }
    fs::copy(ruta(raiz, ruta_relativa), destino).map(|_| ())
}
