//! Ajustes de funcionamiento que el usuario puede cambiar sin recompilar, en datos/ajustes.json.
//!
//! Es un archivo nuevo de la 3.0.0 y no uno de los de la 2.x: config.json es una lista de
//! proyectos con el formato de la 2.x y no admite claves sueltas. Cada ajuste tiene su valor por
//! defecto declarado aquí; una clave que falta toma ese valor, y un valor que no tiene sentido se
//! rechaza al cargar, con un error que lo dice, en vez de corregirse en silencio.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::textos;

/// Nombre del archivo de ajustes dentro de la carpeta de datos.
pub const ARCHIVO_AJUSTES: &str = "ajustes.json";

/// Segundos que se espera a git por defecto: los mismos 30 que esperaba la 2.3.2.
pub const TIEMPO_MAXIMO_GIT_POR_DEFECTO: u64 = 30;

/// Tamaño por defecto, en bytes, a partir del cual se avisa de que una bandeja del canal está sin
/// podar. El núcleo pide unas ochenta líneas vivas por bandeja (apartado 3.2); a unos doscientos
/// bytes por línea de prosa son unos 16 KiB.
pub const LIMITE_DE_BANDEJA_POR_DEFECTO: u64 = 16 * 1024;

/// Ajustes de funcionamiento. Invariante: los dos valores son mayores que cero, lo que asegura
/// [`Ajustes::validar`]; por eso los campos no son públicos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ajustes {
    /// Segundos que se espera a una orden de git antes de pararla.
    #[serde(
        rename = "TiempoMaximoGitSegundos",
        default = "tiempo_maximo_git_por_defecto"
    )]
    tiempo_maximo_git_segundos: u64,
    /// Bytes a partir de los cuales una bandeja del canal se considera sin podar.
    #[serde(
        rename = "LimiteDeBandejaBytes",
        default = "limite_de_bandeja_por_defecto"
    )]
    limite_de_bandeja_bytes: u64,
}

const fn tiempo_maximo_git_por_defecto() -> u64 {
    TIEMPO_MAXIMO_GIT_POR_DEFECTO
}

const fn limite_de_bandeja_por_defecto() -> u64 {
    LIMITE_DE_BANDEJA_POR_DEFECTO
}

impl Default for Ajustes {
    fn default() -> Self {
        Self {
            tiempo_maximo_git_segundos: TIEMPO_MAXIMO_GIT_POR_DEFECTO,
            limite_de_bandeja_bytes: LIMITE_DE_BANDEJA_POR_DEFECTO,
        }
    }
}

impl Ajustes {
    /// Comprueba la invariante de unos ajustes recién leídos.
    ///
    /// # Errors
    ///
    /// Devuelve el motivo del rechazo si algún valor es cero.
    pub const fn validar(self) -> Result<Self, &'static str> {
        if self.tiempo_maximo_git_segundos == 0 {
            return Err(textos::AJUSTE_TIEMPO_CERO);
        }
        if self.limite_de_bandeja_bytes == 0 {
            return Err(textos::AJUSTE_LIMITE_CERO);
        }
        Ok(self)
    }

    /// Tiempo máximo que se espera a git.
    #[must_use]
    pub const fn tiempo_maximo_git(&self) -> Duration {
        Duration::from_secs(self.tiempo_maximo_git_segundos)
    }

    /// Tamaño en bytes a partir del cual se avisa de que una bandeja está sin podar.
    #[must_use]
    pub const fn limite_de_bandeja(&self) -> u64 {
        self.limite_de_bandeja_bytes
    }
}
