//! Forma comparable de una ruta escrita como texto, para saber si dos textos nombran la misma
//! carpeta aunque uno use barras invertidas o acabe en barra.

/// Barra con que se escriben las rutas comparables.
const BARRA: char = '/';

/// Barra invertida de Windows.
const BARRA_INVERTIDA: char = '\\';

/// Si las mayúsculas cuentan al comparar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mayusculas {
    /// Se distinguen, como en Linux.
    Cuentan,
    /// No se distinguen, como en Windows: la ruta se pasa a minúsculas.
    NoCuentan,
}

/// La ruta con barras normales, sin barra final y, si las mayúsculas no cuentan, en minúsculas.
#[must_use]
pub fn forma_comparable(ruta: &str, mayusculas: Mayusculas) -> String {
    let ruta = ruta.replace(BARRA_INVERTIDA, &BARRA.to_string());
    let ruta = ruta.trim_end_matches(BARRA);
    match mayusculas {
        Mayusculas::Cuentan => ruta.to_owned(),
        Mayusculas::NoCuentan => ruta.to_lowercase(),
    }
}
