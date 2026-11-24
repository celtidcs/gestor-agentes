//! Validación de una cuenta que la persona quiere dar de alta a mano.

use crate::busqueda::misma_carpeta;
use crate::{Cuenta, Herramienta};
use std::fmt;
use std::path::Path;

/// Por qué no se puede dar de alta a mano una cuenta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RechazoDeAlta {
    /// La herramienta no elige la cuenta con una variable de entorno: su consola no usaría la
    /// carpeta añadida ([`Herramienta::admite_alta_manual`]).
    SinVariableDeCuenta,
    /// Otra cuenta de la misma herramienta ya usa esa carpeta. Lleva esa cuenta.
    CarpetaYaUsada(Cuenta),
}

impl fmt::Display for RechazoDeAlta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SinVariableDeCuenta => {
                write!(f, "la herramienta no tiene cuentas separadas por carpeta")
            }
            Self::CarpetaYaUsada(cuenta) => write!(
                f,
                "la carpeta ya es la de otra cuenta ({})",
                cuenta.carpeta.display()
            ),
        }
    }
}

impl std::error::Error for RechazoDeAlta {}

/// Comprueba, antes de guardarla, si se puede dar de alta a mano la cuenta de `herramienta` en
/// `carpeta`, frente a las `cuentas` que ya hay (las encontradas y las manuales).
///
/// Solo cuentan las cuentas de la misma herramienta: una carpeta usada por otra herramienta no
/// impide el alta. Las carpetas se comparan como en la búsqueda: sin distinguir la barra
/// final ni, en Windows, el tipo de barra ni mayúsculas de minúsculas.
///
/// # Errors
///
/// [`RechazoDeAlta::SinVariableDeCuenta`] si la herramienta no admite alta manual, y
/// [`RechazoDeAlta::CarpetaYaUsada`] con la cuenta que ya usa la carpeta.
pub fn validar_alta_manual(
    herramienta: Herramienta,
    carpeta: &Path,
    cuentas: &[Cuenta],
) -> Result<(), RechazoDeAlta> {
    if !herramienta.admite_alta_manual() {
        return Err(RechazoDeAlta::SinVariableDeCuenta);
    }
    match cuentas
        .iter()
        .find(|cuenta| cuenta.herramienta == herramienta && misma_carpeta(&cuenta.carpeta, carpeta))
    {
        Some(cuenta) => Err(RechazoDeAlta::CarpetaYaUsada(cuenta.clone())),
        None => Ok(()),
    }
}
