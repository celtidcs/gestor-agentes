//! Avisos de la última preparación del proyecto, guardados para los agentes.
//!
//! Los avisos (una bandeja sin podar, un latido viejo, un apartado que falta en el estado del
//! canal...) los tiene que atender un agente, no el usuario. Por eso no abren ninguna ventana:
//! cada preparación los deja en [`RUTA_RELATIVA`], que el arranque de AGENTS.md manda leer, y
//! borra el archivo si ya no hay ninguno, para que nunca quede un aviso resuelto.

use std::fs;
use std::io;
use std::path::Path;

use super::archivos;
use super::momento::Momento;
use crate::instrucciones::formato::FormatoDeTexto;
use crate::textos;

/// Archivo con los avisos de la última preparación, relativo a la raíz del proyecto.
pub const RUTA_RELATIVA: &str = ".gestor/avisos-de-preparacion.md";

/// Texto del archivo para estos avisos, o nada si no hay ninguno.
#[must_use]
pub fn componer(avisos: &[String], momento: Momento) -> Option<String> {
    if avisos.is_empty() {
        return None;
    }
    let mut texto = textos::cabecera_de_avisos(&momento.to_string());
    for aviso in avisos {
        texto.push_str("- ");
        texto.push_str(aviso);
        texto.push('\n');
    }
    Some(texto)
}

/// Escribe el archivo con los avisos o lo borra si no hay ninguno.
///
/// # Errors
///
/// Devuelve el error de disco si no se puede escribir ni borrar.
pub fn guardar(raiz: &Path, avisos: &[String], momento: Momento) -> io::Result<()> {
    let ruta = archivos::ruta(raiz, RUTA_RELATIVA);
    match componer(avisos, momento) {
        Some(texto) => archivos::escribir(&ruta, &texto, FormatoDeTexto::default()),
        None => match fs::remove_file(&ruta) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            otro => otro,
        },
    }
}
