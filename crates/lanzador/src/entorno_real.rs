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

    // Misma regla que nucleo::cuentas_de_agentes::carpeta_personal (la de deteccion-agentes); se
    // repite aquí porque el lanzador no depende de nucleo, y las dos se prueban por separado.
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

    fn proyecto_de_agy(&self, carpeta_del_proyecto: &Path) -> crate::agy::ProyectoDeAgy {
        let Some(personal) = self.carpeta_personal() else {
            return crate::agy::ProyectoDeAgy::Ilegible;
        };
        let carpeta = crate::agy::CARPETA_DE_PROYECTOS
            .iter()
            .fold(personal, |ruta, parte| ruta.join(parte));
        let Ok(entradas) = std::fs::read_dir(carpeta) else {
            return crate::agy::ProyectoDeAgy::Ilegible;
        };
        let mut rutas: Vec<PathBuf> = entradas
            .filter_map(Result::ok)
            .map(|entrada| entrada.path())
            .filter(|ruta| ruta.extension().is_some_and(|ext| ext == "json"))
            .collect();
        rutas.sort();
        let textos: Vec<String> = rutas
            .iter()
            .filter_map(|ruta| std::fs::read_to_string(ruta).ok())
            .collect();
        crate::agy::buscar_proyecto(&textos, carpeta_del_proyecto)
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
