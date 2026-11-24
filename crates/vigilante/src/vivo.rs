//! Rastro que deja el vigilante para que Gestor Agentes y el Jefe sepan quién está vigilando, en
//! .gestor/<nombre>/:
//! - vigilante.pid: la marca del único vigilante vivo de ese agente (el relevo);
//! - vigilante/vivo.json: quién vigila, desde cuándo y con qué proceso; se borra al terminar bien;
//! - vigilante/avisos.log: una línea por aviso, con la hora en segundos desde 1970, las etiquetas,
//!   los archivos y el resultado de la entrega, separados por tabuladores.
//!
//! vivo.json solo se escribe al arrancar, para no reescribir el disco en cada pasada: la ventana
//! comprueba con el número y la hora de arranque que el proceso sigue vivo.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use canal::contrato::claves_de_vivo;
use canal::identidad::Identidad;

use crate::opciones::{Modo, ProcesoMarcado};

pub use canal::contrato::{ARCHIVO_DE_AVISOS, ARCHIVO_DE_RELEVO, ARCHIVO_VIVO};

/// Segundos desde 1970 en este momento, o 0 si el reloj del sistema está antes de 1970.
#[must_use]
pub fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duracion| duracion.as_secs())
}

/// Marca única de este vigilante: su proceso y los nanosegundos del arranque, porque el número de
/// proceso puede repetirse.
#[must_use]
pub fn marca_propia() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duracion| duracion.subsec_nanos());
    format!("gestor-vigilante {} {nanos}", std::process::id())
}

/// Ruta del archivo de relevo del agente.
#[must_use]
pub fn ruta_de_relevo(gestor: &Path, nombre: &str) -> PathBuf {
    gestor.join(nombre).join(ARCHIVO_DE_RELEVO)
}

/// Marca escrita en el archivo de relevo, o vacío si no se puede leer.
#[must_use]
pub fn marca_escrita(ruta: &Path) -> String {
    fs::read_to_string(ruta)
        .map(|texto| texto.trim().to_owned())
        .unwrap_or_default()
}

/// Nombre del modo en vivo.json.
const fn nombre_del_modo(modo: Modo) -> &'static str {
    match modo {
        Modo::UnaVez => "una-vez",
        Modo::HastaCambio => "hasta-cambio",
        Modo::Persistente => "persistente",
        Modo::AnotarHilo => "anotar-hilo",
    }
}

/// Texto de vivo.json. La identidad solo lleva letras, cifras y un espacio ([`Identidad`]), así
/// que no hace falta escapar nada.
#[must_use]
pub fn texto_vivo(
    identidad: &Identidad,
    proceso: ProcesoMarcado,
    modo: Modo,
    desde: u64,
) -> String {
    format!(
        "{{\"{}\":\"{identidad}\",\"{}\":{},\"{}\":{},\"{}\":\"{}\",\"{}\":{desde}}}\n",
        claves_de_vivo::IDENTIDAD,
        claves_de_vivo::PID,
        proceso.pid,
        claves_de_vivo::ARRANQUE,
        proceso.arranque,
        claves_de_vivo::MODO,
        nombre_del_modo(modo),
        claves_de_vivo::DESDE,
    )
}

/// Escribe vivo.json.
///
/// # Errors
///
/// El error de disco.
pub fn escribir_vivo(carpeta: &Path, texto: &str) -> io::Result<()> {
    fs::create_dir_all(carpeta)?;
    fs::write(carpeta.join(ARCHIVO_VIVO), texto)
}

/// Borra vivo.json si sigue siendo el de este proceso.
pub fn borrar_vivo(carpeta: &Path, texto: &str) {
    let ruta = carpeta.join(ARCHIVO_VIVO);
    if fs::read_to_string(&ruta).is_ok_and(|actual| actual == texto) {
        // Si no se puede borrar, la ventana verá que el proceso ha muerto.
        let _ = fs::remove_file(ruta);
    }
}

/// Añade una línea a avisos.log.
///
/// # Errors
///
/// El error de disco.
pub fn anotar_aviso(
    carpeta: &Path,
    hora: u64,
    etiquetas: &str,
    archivos: &[String],
    entrega: &str,
) -> io::Result<()> {
    fs::create_dir_all(carpeta)?;
    let mut registro = OpenOptions::new()
        .create(true)
        .append(true)
        .open(carpeta.join(ARCHIVO_DE_AVISOS))?;
    let limpia = |texto: &str| texto.replace(['\t', '\n', '\r'], " ");
    writeln!(
        registro,
        "{hora}\t{}\t{}\t{}",
        limpia(etiquetas),
        limpia(&archivos.join(",")),
        limpia(entrega)
    )
}
