//! Aviso de versión nueva: lo propio de Gestor Agentes sobre la biblioteca común
//! aviso-version-github, que está en bibliotecas/, compartida con otros programas.
//!
//! La consulta la hace el módulo aviso_de_version de la interfaz, el único que llama a la
//! biblioteca, como pide su norma de integración. Aquí solo están, sin red, los ajustes de la
//! biblioteca que salen de ajustes.json, la validación de sus repositorios al leerlo y el tipo
//! de las versiones, con el que se compara el Programator de cada perfil.

use aviso_version_github::OrigenDeLasVersiones;

pub use aviso_version_github::{
    AjustesDeVersionNueva, AvisoDeVersionNueva, VersionSemantica as Version,
};

/// Ajustes de la biblioteca para `repositorio`, con el servidor de github.com y el tiempo de
/// espera de Gestor Agentes. Los demás valores, los de la biblioteca por defecto.
#[must_use]
pub fn ajustes_de_version(repositorio: &str, segundos_de_espera: u64) -> AjustesDeVersionNueva {
    AjustesDeVersionNueva {
        segundos_de_espera,
        ..AjustesDeVersionNueva::para_repositorio(repositorio)
    }
}

/// Indica si `repositorio` tiene la forma propietario/nombre que admite la biblioteca.
#[must_use]
pub fn repositorio_valido(repositorio: &str) -> bool {
    OrigenDeLasVersiones::desde_ajustes(&AjustesDeVersionNueva::para_repositorio(repositorio))
        .is_ok()
}
