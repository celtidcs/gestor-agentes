//! Implementación real de [`EntornoDelSistema`], la que usa la aplicación.
//!
//! Es la única parte del lanzador que consulta el sistema al planificar y al detectar: variables
//! de entorno y existencia de archivos. Se mantiene lo más fina posible; las decisiones están en
//! el módulo de planificación, que se prueba con un doble.

use std::path::{Path, PathBuf};

use crate::EntornoDelSistema;

/// Entorno del sistema en el que se ejecuta el programa.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntornoReal;

impl EntornoDelSistema for EntornoReal {
    fn buscar_en_path(&self, nombre: &str, extensiones: &[&str]) -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;
        std::env::split_paths(&path)
            .filter(|carpeta| !carpeta.as_os_str().is_empty())
            .find_map(|carpeta| {
                if extensiones.is_empty() {
                    let candidato = carpeta.join(nombre);
                    es_ejecutable(&candidato).then_some(candidato)
                } else {
                    extensiones
                        .iter()
                        .map(|ext| carpeta.join(format!("{nombre}{ext}")))
                        .find(|candidato| candidato.is_file())
                }
            })
    }

    fn carpeta_personal(&self) -> Option<PathBuf> {
        let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        std::env::var_os(variable)
            .filter(|valor| !valor.is_empty())
            .map(PathBuf::from)
    }

    fn carpeta_de_datos_locales(&self) -> Option<PathBuf> {
        if !cfg!(windows) {
            return None;
        }
        std::env::var_os("LOCALAPPDATA")
            .filter(|valor| !valor.is_empty())
            .map(PathBuf::from)
    }

    fn existe(&self, ruta: &Path) -> bool {
        ruta.is_file()
    }
}

/// Indica si una ruta es un archivo ejecutable (en Linux, con algún permiso de ejecución, como
/// exige `which`, que era lo que usaba la 2.3.2).
fn es_ejecutable(ruta: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        ruta.metadata()
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        ruta.is_file()
    }
}
