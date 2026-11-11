//! Fallos de la configuración, con la ruta del archivo afectado.

use std::io;
use std::path::{Path, PathBuf};

/// Fallo de configuración, con la ruta del archivo.
#[derive(Debug, thiserror::Error)]
pub enum ErrorDeConfiguracion {
    /// No se pudo leer o escribir un archivo.
    #[error("{ruta}: {origen}")]
    Disco {
        /// Archivo afectado.
        ruta: PathBuf,
        /// Error del sistema.
        #[source]
        origen: io::Error,
    },
    /// El archivo no tiene un formato válido.
    #[error("{ruta} no tiene un formato válido: {motivo}")]
    FormatoInvalido {
        /// Archivo afectado.
        ruta: PathBuf,
        /// Explicación del analizador.
        motivo: String,
    },
    /// Un perfil no es válido.
    #[error("{ruta}: perfil inválido ({motivo}).")]
    PerfilInvalido {
        /// Archivo de perfiles.
        ruta: PathBuf,
        /// Qué falla.
        motivo: String,
    },
    /// Un ajuste tiene un valor que no tiene sentido.
    #[error("{ruta}: ajuste inválido ({motivo}).")]
    AjusteInvalido {
        /// Archivo de ajustes.
        ruta: PathBuf,
        /// Qué falla.
        motivo: &'static str,
    },
    /// Dos perfiles tienen la misma identidad.
    #[error("{ruta}: el perfil «{nombre}» aparece dos veces.")]
    PerfilRepetido {
        /// Archivo de perfiles.
        ruta: PathBuf,
        /// Identidad repetida.
        nombre: String,
    },
}

/// Error de disco con la ruta afectada.
pub(super) fn disco(ruta: &Path, origen: io::Error) -> ErrorDeConfiguracion {
    ErrorDeConfiguracion::Disco {
        ruta: ruta.to_path_buf(),
        origen,
    }
}
