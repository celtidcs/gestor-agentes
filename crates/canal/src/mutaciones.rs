//! Apartado «## Mutaciones vivas» de .gestor/canal/estado.md.
//!
//! El apartado empieza en su título (con o sin espacios delante) y acaba en el siguiente título,
//! aunque lleve espacios delante. Está vacío si sus líneas, quitando las que están en blanco, solo
//! dicen «Ninguna.», con o sin viñeta y con o sin una nota entre paréntesis delante, como
//! «- (Añadido por Claude 1, 2026-11-18 13:18.) Ninguna.».

use crate::apartados;
use crate::huella;

/// Título del apartado.
pub const TITULO: &str = "## Mutaciones vivas";

/// Palabra con la que la plantilla dice que el apartado está vacío, sin el punto.
const VACIO: &str = "Ninguna";

/// Líneas con contenido del apartado, sin espacios a los lados, o nada si no está.
#[must_use]
pub fn lineas(estado: &str) -> Option<Vec<&str>> {
    apartados::lineas(estado, TITULO)
}

/// Si hay alguna mutación viva: el apartado existe y alguna línea no es la de vacío.
#[must_use]
pub fn hay_vivas(estado: &str) -> bool {
    lineas(estado).is_some_and(|lineas| lineas.iter().any(|linea| !es_vacio(linea)))
}

/// Huella de las líneas de mutaciones vivas, o nada si no hay ninguna. Cambia solo si cambia una
/// mutación, no cuando cambia otra parte de estado.md.
#[must_use]
pub fn huella_de_las_vivas(estado: &str) -> Option<u64> {
    let vivas: Vec<&str> = lineas(estado)?
        .into_iter()
        .filter(|linea| !es_vacio(linea))
        .collect();
    (!vivas.is_empty()).then(|| huella::de_texto(&vivas.join("\n")))
}

/// Si una línea del apartado dice que está vacío.
#[must_use]
pub fn es_vacio(linea: &str) -> bool {
    let sin_vineta = linea
        .trim()
        .strip_prefix('-')
        .map_or(linea.trim(), str::trim_start);
    let sin_nota = if sin_vineta.starts_with('(') {
        sin_vineta
            .find(')')
            .map_or(sin_vineta, |cierre| sin_vineta[cierre + 1..].trim_start())
    } else {
        sin_vineta
    };
    let sin_punto = sin_nota.trim_end().trim_end_matches('.');
    sin_punto == VACIO
}
