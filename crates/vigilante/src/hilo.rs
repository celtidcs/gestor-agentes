//! Sesión de Codex anotada en el proyecto. Dentro de su sandbox, Codex no puede ver los procesos,
//! así que el vigilante persistente lo lanza Gestor Agentes desde la terminal y Codex solo le deja
//! aquí su CODEX_THREAD_ID, con `--anotar-hilo`. Lo que decide ([`hilo_valido`]) no toca el disco.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Nombre del archivo con la sesión, en la carpeta del vigilante del agente.
pub const ARCHIVO_DEL_HILO: &str = "hilo.txt";

/// Ruta del archivo de la sesión en la carpeta del vigilante `carpeta`.
#[must_use]
pub fn ruta(carpeta: &Path) -> PathBuf {
    carpeta.join(ARCHIVO_DEL_HILO)
}

/// La sesión anotada en `texto`, si vale: una sola palabra, no vacía, escrita (`modificado`, en
/// segundos desde 1970) después de que arrancara el vigilante (`arranque`). Una anotación anterior
/// es de una sesión de Codex ya cerrada.
#[must_use]
pub fn hilo_valido(texto: &str, modificado: u64, arranque: u64) -> Option<String> {
    let hilo = texto.trim();
    if hilo.is_empty() || hilo.chars().any(char::is_whitespace) || modificado < arranque {
        return None;
    }
    Some(hilo.to_owned())
}

/// Escribe `hilo` en el archivo de la sesión de `carpeta`, creando la carpeta si falta.
///
/// # Errors
///
/// El error del disco.
pub fn anotar(carpeta: &Path, hilo: &str) -> io::Result<()> {
    fs::create_dir_all(carpeta)?;
    fs::write(ruta(carpeta), format!("{hilo}\n"))
}

/// Texto y hora de modificación (segundos desde 1970) del archivo de la sesión, si se puede leer.
#[must_use]
pub fn leer(carpeta: &Path) -> Option<(String, u64)> {
    let ruta = ruta(carpeta);
    let texto = fs::read_to_string(&ruta).ok()?;
    let modificado = fs::metadata(&ruta)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some((texto, modificado))
}
