//! Lleva al archivo de .gestor/ los archivos del sistema de trabajo anterior que la aplicación
//! reconoce por su nombre, para que no convivan con los vigentes (regla de cero basura del
//! núcleo). Nunca borra nada: mueve, y si en el destino ya hay un archivo con ese nombre, añade
//! al nuevo una marca de tiempo.
//!
//! Son el canal antiguo de Gemini (.gestor/gemini/canal.md y sus compañeros), GEMINI.md, que desde
//! la 3.0.0 no se usa porque Gemini lee AGENTS.md, y la memoria compartida del sistema de relevo.
//! Pueden guardar una tarea viva, así que cada movimiento se anota en el informe y el que los
//! mueve se encarga de que el archivo propio del proyecto apunte a ellos.

use std::fs;
use std::io;
use std::path::Path;

use super::archivos;
use super::catalogo;
use super::informe::{InformeDeInicializacion, TipoDeCambio};
use super::momento::Momento;
use crate::instrucciones::migracion::Archivado;
use crate::textos;

/// Carpeta del canal de Gemini anterior al canal común.
pub const CARPETA_GEMINI_ANTIGUA: &str = ".gestor/gemini";

/// Destino del canal antiguo de Gemini.
pub const CARPETA_ARCHIVO_GEMINI: &str = ".gestor/archivo/canal-gemini-antiguo";

/// Destino de los demás archivos del sistema anterior.
pub const CARPETA_ARCHIVO_SISTEMA_ANTERIOR: &str = ".gestor/archivo/sistema-anterior";

/// Archivos del canal antiguo de Gemini dentro de [`CARPETA_GEMINI_ANTIGUA`].
const ARCHIVOS_GEMINI_ANTIGUOS: [&str; 4] = [
    "canal.md",
    "historico.md",
    "session.log",
    "INSTRUCCIONES_GEMINI.md",
];

/// Archivos sueltos del sistema anterior, con su ruta relativa al proyecto.
const ARCHIVOS_SUELTOS: [&str; 2] = [
    catalogo::RUTA_GEMINI_ANTERIOR,
    catalogo::RUTA_MEMORIA_COMPARTIDA,
];

/// Mueve al archivo todos los archivos del sistema anterior que haya y devuelve dónde ha dejado
/// cada uno, en orden. Cada movimiento queda en el informe.
///
/// # Errors
///
/// Devuelve el primer error de disco; lo movido hasta entonces queda movido y anotado.
pub fn archivar(
    raiz: &Path,
    momento: Momento,
    informe: &mut InformeDeInicializacion,
) -> io::Result<Vec<Archivado>> {
    let mut archivados = Vec::new();
    for nombre in ARCHIVOS_GEMINI_ANTIGUOS {
        let origen = format!("{CARPETA_GEMINI_ANTIGUA}/{nombre}");
        let destino = format!("{CARPETA_ARCHIVO_GEMINI}/{nombre}");
        archivados.extend(mover(raiz, &origen, &destino, momento, informe)?);
    }
    for origen in ARCHIVOS_SUELTOS {
        let nombre = origen.rsplit('/').next().unwrap_or(origen);
        let destino = format!("{CARPETA_ARCHIVO_SISTEMA_ANTERIOR}/{nombre}");
        archivados.extend(mover(raiz, origen, &destino, momento, informe)?);
    }
    Ok(archivados)
}

/// Mueve un archivo si existe, sin pisar otro que ya esté en el destino.
fn mover(
    raiz: &Path,
    origen_relativo: &str,
    destino_relativo: &str,
    momento: Momento,
    informe: &mut InformeDeInicializacion,
) -> io::Result<Option<Archivado>> {
    let origen = archivos::ruta(raiz, origen_relativo);
    if !origen.is_file() {
        return Ok(None);
    }

    let mut destino_relativo = destino_relativo.to_owned();
    let mut destino = archivos::ruta(raiz, &destino_relativo);
    if destino.is_file() {
        destino_relativo = format!("{destino_relativo}.{}", momento.marca_de_tiempo());
        destino = archivos::ruta(raiz, &destino_relativo);
    }
    if let Some(carpeta) = destino.parent() {
        fs::create_dir_all(carpeta)?;
    }
    fs::rename(&origen, &destino)?;
    informe.anotar_cambio(
        origen_relativo,
        TipoDeCambio::Archivado,
        Some(textos::movido_al_archivo(&destino_relativo)),
    );
    Ok(Some(Archivado {
        origen: origen_relativo.to_owned(),
        destino: destino_relativo,
    }))
}
