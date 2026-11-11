//! Copia de la configuración de la 2.x a la carpeta de datos de la 3.0.0.

use std::fs;
use std::path::{Path, PathBuf};

use super::almacen::Configuracion;
use super::error::{disco, ErrorDeConfiguracion};
use super::{ARCHIVOS_DE_CONFIGURACION, ARCHIVO_PERFILES, ARCHIVO_PROYECTOS};

/// Nombre de la carpeta de configuración de la 2.x dentro de la carpeta de datos de aplicación
/// del usuario (%APPDATA% en Windows, $XDG_CONFIG_HOME o ~/.config en Linux).
pub const CARPETA_DE_LA_2X: &str = "GestorClaude";

/// Carpeta de configuración de la 2.x a partir de la carpeta de datos de aplicación del usuario,
/// que la interfaz obtiene del sistema (%APPDATA% en Windows; $XDG_CONFIG_HOME o ~/.config en
/// Linux, que es lo que devuelve .NET para SpecialFolder.ApplicationData).
#[must_use]
pub fn carpeta_de_la_2x(datos_de_aplicacion: &Path) -> PathBuf {
    datos_de_aplicacion.join(CARPETA_DE_LA_2X)
}

impl Configuracion {
    /// Copia la configuración de la 2.x desde `origen` a esta carpeta, sin borrar el original y
    /// sin sobrescribir nada que ya exista aquí. Solo actúa si aquí todavía no hay lista de
    /// proyectos ni perfiles. Devuelve los nombres de los archivos copiados.
    ///
    /// # Errors
    ///
    /// Devuelve [`ErrorDeConfiguracion::Disco`] si una copia falla.
    pub fn migrar_desde(&self, origen: &Path) -> Result<Vec<&'static str>, ErrorDeConfiguracion> {
        let carpeta = self.carpeta();
        let ya_hay_configuracion = [ARCHIVO_PROYECTOS, ARCHIVO_PERFILES]
            .iter()
            .any(|archivo| carpeta.join(archivo).exists());
        if ya_hay_configuracion || !origen.is_dir() {
            return Ok(Vec::new());
        }

        let mut copiados = Vec::new();
        for archivo in ARCHIVOS_DE_CONFIGURACION {
            let desde = origen.join(archivo);
            if !desde.is_file() {
                continue;
            }
            fs::create_dir_all(carpeta).map_err(|origen| disco(carpeta, origen))?;
            let hacia = carpeta.join(archivo);
            fs::copy(&desde, &hacia).map_err(|origen| disco(&hacia, origen))?;
            copiados.push(archivo);
        }
        Ok(copiados)
    }
}
