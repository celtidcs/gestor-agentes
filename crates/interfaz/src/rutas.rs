//! Carpetas que necesita la aplicación al arrancar, calculadas sin tocar el disco.
//!
//! Las variables de entorno se reciben como función para poder probar cada caso sin cambiar el
//! entorno del proceso.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use lanzador::Sistema;

/// Carpeta de datos junto al ejecutable, porque la aplicación es portable.
pub const CARPETA_DE_DATOS: &str = "datos";

/// Carpeta de datos de la aplicación: datos/ en la carpeta del ejecutable. Nada si el ejecutable
/// no tiene carpeta.
#[must_use]
pub fn carpeta_de_datos(ejecutable: &Path) -> Option<PathBuf> {
    Some(ejecutable.parent()?.join(CARPETA_DE_DATOS))
}

/// Carpeta de datos de aplicación del usuario, la misma que .NET devuelve para
/// SpecialFolder.ApplicationData y donde la 2.x guardaba su configuración: %APPDATA% en Windows;
/// en Linux, $XDG_CONFIG_HOME si es una ruta absoluta y, si no, $HOME/.config.
///
/// Nada si las variables necesarias no están o están vacías.
#[must_use]
pub fn datos_de_aplicacion(
    sistema: Sistema,
    variable: impl Fn(&str) -> Option<OsString>,
) -> Option<PathBuf> {
    let no_vacia = |nombre: &str| variable(nombre).filter(|valor| !valor.is_empty());
    match sistema {
        Sistema::Windows => no_vacia("APPDATA").map(PathBuf::from),
        Sistema::Linux => no_vacia("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            // has_root y no is_absolute, para que «/x» cuente como absoluta también cuando las
            // pruebas de la rama de Linux se ejecutan en Windows.
            .filter(|ruta| ruta.has_root())
            .or_else(|| no_vacia("HOME").map(|casa| PathBuf::from(casa).join(".config"))),
    }
}
