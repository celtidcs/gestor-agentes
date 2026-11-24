//! Apartados de un documento del canal, como «## Mutaciones vivas» o «## Papeles» de estado.md.
//!
//! Un apartado empieza en su título, con o sin espacios a los lados, y acaba en el siguiente
//! título, aunque lleve espacios delante, o al final del texto.

/// Carácter con que empieza un título.
const MARCA_DE_TITULO: char = '#';

/// Si una línea es un título.
#[must_use]
pub fn es_titulo(linea: &str) -> bool {
    linea.trim_start().starts_with(MARCA_DE_TITULO)
}

/// Líneas del apartado `titulo`, sin el título, tal como están, o nada si no está.
#[must_use]
pub fn lineas_crudas<'a>(texto: &'a str, titulo: &str) -> Option<Vec<&'a str>> {
    let mut todas = texto.lines();
    todas.find(|linea| linea.trim() == titulo)?;
    Some(todas.take_while(|linea| !es_titulo(linea)).collect())
}

/// Líneas con contenido del apartado `titulo`, sin espacios a los lados, o nada si no está.
#[must_use]
pub fn lineas<'a>(texto: &'a str, titulo: &str) -> Option<Vec<&'a str>> {
    lineas_crudas(texto, titulo).map(|lineas| {
        lineas
            .into_iter()
            .map(str::trim)
            .filter(|linea| !linea.is_empty())
            .collect()
    })
}

/// Posición del título `titulo` en `lineas` y del final de su apartado (el siguiente título o el
/// final), para quien tiene que insertar líneas en él.
#[must_use]
pub fn limites(lineas: &[&str], titulo: &str) -> Option<(usize, usize)> {
    let inicio = lineas.iter().position(|linea| linea.trim() == titulo)?;
    let fin = lineas[inicio + 1..]
        .iter()
        .position(|linea| es_titulo(linea))
        .map_or(lineas.len(), |i| inicio + 1 + i);
    Some((inicio, fin))
}

/// Si el texto tiene el apartado `titulo`.
#[must_use]
pub fn tiene(texto: &str, titulo: &str) -> bool {
    texto.lines().any(|linea| linea.trim() == titulo)
}
