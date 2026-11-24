//! Lo visto por el vigilante, en disco, en .gestor/<nombre>/vigilante/:
//! - archivos.txt: una línea «<huella> <nombre>» por archivo vigilado;
//! - encargos.txt: una huella por línea, la de cada encargo ya avisado;
//! - mutaciones.txt: la huella de las mutaciones vivas avisadas, o vacío.
//!
//! Las huellas van en hexadecimal ([`canal::huella::en_hexadecimal`]). Un archivo que no se
//! entiende se trata como si faltara: el vigilante vuelve a empezar con una primera pasada.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use canal::contrato;
use canal::huella;

use crate::pasada::Visto;

/// Encargos avisados que se guardan como mucho: los más recientes. Con unos pocos encargos por
/// hora, son semanas de trabajo; uno más antiguo que se mueva de sitio volvería a avisar una vez.
const ENCARGOS_GUARDADOS: usize = 2000;

/// Huella y nombre de cada archivo vigilado.
const ARCHIVOS: &str = "archivos.txt";
/// Huellas de los encargos avisados.
const ENCARGOS: &str = "encargos.txt";
/// Huella de las mutaciones vivas avisadas.
const MUTACIONES: &str = "mutaciones.txt";

/// Carpeta del recuerdo de un agente.
#[must_use]
pub fn carpeta(gestor: &Path, nombre: &str) -> PathBuf {
    gestor.join(nombre).join(contrato::CARPETA_DEL_VIGILANTE)
}

/// Lee lo visto, o nada si falta o no se entiende.
#[must_use]
pub fn leer(carpeta: &Path) -> Option<Visto> {
    let archivos_txt = fs::read_to_string(carpeta.join(ARCHIVOS)).ok()?;
    let mut visto = Visto::default();
    for linea in archivos_txt.lines().filter(|l| !l.trim().is_empty()) {
        let (huella, nombre) = linea.split_once(' ')?;
        visto
            .archivos
            .insert(nombre.to_owned(), u64::from_str_radix(huella, 16).ok()?);
    }
    let encargos_txt = fs::read_to_string(carpeta.join(ENCARGOS)).unwrap_or_default();
    for linea in encargos_txt.lines().filter(|l| !l.trim().is_empty()) {
        visto
            .encargos
            .insert(u64::from_str_radix(linea.trim(), 16).ok()?);
    }
    let mutaciones_txt = fs::read_to_string(carpeta.join(MUTACIONES)).unwrap_or_default();
    let mutaciones = mutaciones_txt.trim();
    visto.mutaciones = if mutaciones.is_empty() {
        None
    } else {
        Some(u64::from_str_radix(mutaciones, 16).ok()?)
    };
    Some(visto)
}

/// Guarda lo visto. Escribe cada archivo entero de una vez. Los encargos conservan el orden en que
/// se guardaron, para quedarse con los más recientes.
///
/// # Errors
///
/// El error de disco al crear la carpeta o escribir un archivo.
pub fn guardar(carpeta: &Path, visto: &Visto) -> io::Result<()> {
    fs::create_dir_all(carpeta)?;
    let orden_de_encargos: Vec<u64> = fs::read_to_string(carpeta.join(ENCARGOS))
        .unwrap_or_default()
        .lines()
        .filter_map(|linea| u64::from_str_radix(linea.trim(), 16).ok())
        .collect();
    let archivos: String = visto
        .archivos
        .iter()
        .map(|(nombre, h)| format!("{} {nombre}\n", huella::en_hexadecimal(*h)))
        .collect();
    fs::write(carpeta.join(ARCHIVOS), archivos)?;
    let encargos: String = recientes(&visto.encargos, &orden_de_encargos)
        .iter()
        .map(|h| format!("{}\n", huella::en_hexadecimal(*h)))
        .collect();
    fs::write(carpeta.join(ENCARGOS), encargos)?;
    let mutaciones = visto
        .mutaciones
        .map(|h| huella::en_hexadecimal(h) + "\n")
        .unwrap_or_default();
    fs::write(carpeta.join(MUTACIONES), mutaciones)
}

/// Los encargos que se guardan: los de `orden` (del más antiguo al más reciente) que siguen en
/// `encargos`, y luego los demás, quedándose con los [`ENCARGOS_GUARDADOS`] últimos.
fn recientes(encargos: &BTreeSet<u64>, orden: &[u64]) -> Vec<u64> {
    let mut lista: Vec<u64> = orden
        .iter()
        .copied()
        .filter(|h| encargos.contains(h))
        .collect();
    let ya: BTreeSet<u64> = lista.iter().copied().collect();
    lista.extend(encargos.iter().copied().filter(|h| !ya.contains(h)));
    let sobran = lista.len().saturating_sub(ENCARGOS_GUARDADOS);
    lista.split_off(sobran)
}
