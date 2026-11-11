//! Orden de la lista de proyectos y comparación de rutas.
//!
//! La 2.3.2 ordenaba con la cultura del usuario sin distinguir mayúsculas, para que «Álvaro»
//! fuera junto a «Alba». Rust no trae las tablas de ordenación de cada idioma, así que aquí se
//! reproduce lo que importa en español: se ignoran mayúsculas y tildes, y la «ñ» va entre la «n»
//! y la «o». Es una ordenación visual y no decide la identidad de nada.

use std::cmp::Ordering;

/// Clave de orden de un nombre. Compara primero las letras sin tildes ni mayúsculas, después las
/// letras con sus tildes y, al final, el texto exacto, para que el orden sea total y estable.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClaveDeOrden {
    primaria: Vec<u32>,
    secundaria: Vec<char>,
    exacta: String,
}

impl ClaveDeOrden {
    /// Clave de `nombre`.
    #[must_use]
    pub fn de(nombre: &str) -> Self {
        let minusculas: Vec<char> = nombre.chars().flat_map(char::to_lowercase).collect();
        Self {
            primaria: minusculas
                .iter()
                .map(|letra| peso_primario(*letra))
                .collect(),
            secundaria: minusculas,
            exacta: nombre.to_owned(),
        }
    }
}

/// Compara dos nombres para la lista de proyectos.
#[must_use]
pub fn comparar_nombres(a: &str, b: &str) -> Ordering {
    ClaveDeOrden::de(a).cmp(&ClaveDeOrden::de(b))
}

/// Peso de una letra en minúscula. Cada carácter vale el doble de su código, de modo que queda
/// un hueco impar libre detrás de cada uno: la «ñ» ocupa el de la «n».
fn peso_primario(letra: char) -> u32 {
    if letra == 'ñ' {
        return u32::from('n') * 2 + 1;
    }
    u32::from(sin_tilde(letra)) * 2
}

/// Letra base de las letras con tilde, diéresis o cedilla de las lenguas de la península y del
/// francés, que son las que pueden aparecer en los nombres de las carpetas de los proyectos.
const fn sin_tilde(letra: char) -> char {
    match letra {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        otra => otra,
    }
}

/// Indica si dos rutas son la misma sin distinguir mayúsculas, como el
/// StringComparison.OrdinalIgnoreCase de la 2.3.2: se compara carácter a carácter y no se
/// normaliza nada más.
#[must_use]
pub fn misma_ruta(a: &str, b: &str) -> bool {
    a.chars().count() == b.chars().count()
        && a.chars()
            .zip(b.chars())
            .all(|(x, y)| x == y || x.to_uppercase().eq(y.to_uppercase()))
}
