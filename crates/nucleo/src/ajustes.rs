//! Ajustes de funcionamiento que el usuario puede cambiar sin recompilar, en datos/ajustes.json.
//!
//! Es un archivo nuevo de la 3.0.0 y no uno de los de la 2.x: config.json es una lista de
//! proyectos con el formato de la 2.x y no admite claves sueltas. Cada ajuste tiene su valor por
//! defecto declarado aquí; una clave que falta toma ese valor, y un valor que no tiene sentido se
//! rechaza al cargar, con un error que lo dice, en vez de corregirse en silencio.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::actualizaciones;
use crate::textos;

/// Nombre del archivo de ajustes dentro de la carpeta de datos.
pub const ARCHIVO_AJUSTES: &str = "ajustes.json";

/// Segundos que se espera a git por defecto: los mismos 30 que esperaba la 2.3.2.
pub const TIEMPO_MAXIMO_GIT_POR_DEFECTO: u64 = 30;

/// Tamaño por defecto, en bytes, a partir del cual se avisa de que una bandeja del canal está sin
/// podar. El núcleo pide unas ochenta líneas vivas por bandeja (apartado 3.2); a unos doscientos
/// bytes por línea de prosa son unos 16 KiB.
pub const LIMITE_DE_BANDEJA_POR_DEFECTO: u64 = 16 * 1024;

/// Horas sin latido a partir de las cuales una reserva se da por huérfana. Una jornada de trabajo
/// cabe de sobra: un agente que no ha escrito en doce horas ya no está trabajando en lo que reservó.
pub const LIMITE_DE_LATIDO_POR_DEFECTO: u64 = 12;

/// Si se consulta al arrancar si hay una versión nueva: sí por defecto, y se puede desactivar.
pub const COMPROBAR_ACTUALIZACIONES_POR_DEFECTO: bool = true;

/// Repositorio público donde se publican las versiones de Gestor Agentes.
pub const REPOSITORIO_DE_ACTUALIZACIONES_POR_DEFECTO: &str = "celtidcs/gestor-agentes";

/// Segundos que se espera a GitHub por defecto. La consulta va en su propio hilo y no retrasa
/// nada; cinco segundos bastan a una petición pequeña y cortan pronto una red que no responde.
pub const TIEMPO_MAXIMO_ACTUALIZACION_POR_DEFECTO: u64 = 5;

/// Ajustes de funcionamiento. Invariante: los tiempos y el límite son mayores que cero y el
/// repositorio tiene la forma propietario/nombre, lo que asegura [`Ajustes::validar`]; por eso los
/// campos no son públicos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Si se consulta al arrancar la última versión publicada.
    #[serde(
        rename = "ComprobarActualizaciones",
        default = "comprobar_actualizaciones_por_defecto"
    )]
    comprobar_actualizaciones: bool,
    /// Repositorio de GitHub donde se buscan las versiones, como propietario/nombre.
    #[serde(
        rename = "RepositorioDeActualizaciones",
        default = "repositorio_de_actualizaciones_por_defecto"
    )]
    repositorio_de_actualizaciones: String,
    /// Segundos que se espera a la respuesta de GitHub.
    #[serde(
        rename = "TiempoMaximoActualizacionSegundos",
        default = "tiempo_maximo_actualizacion_por_defecto"
    )]
    tiempo_maximo_actualizacion_segundos: u64,
    /// Horas sin latido tras las que se avisa de que una reserva puede estar huérfana.
    #[serde(
        rename = "LimiteDeLatidoHoras",
        default = "limite_de_latido_por_defecto"
    )]
    limite_de_latido_horas: u64,
}

const fn tiempo_maximo_git_por_defecto() -> u64 {
    TIEMPO_MAXIMO_GIT_POR_DEFECTO
}

const fn limite_de_bandeja_por_defecto() -> u64 {
    LIMITE_DE_BANDEJA_POR_DEFECTO
}

const fn limite_de_latido_por_defecto() -> u64 {
    LIMITE_DE_LATIDO_POR_DEFECTO
}

const fn comprobar_actualizaciones_por_defecto() -> bool {
    COMPROBAR_ACTUALIZACIONES_POR_DEFECTO
}

fn repositorio_de_actualizaciones_por_defecto() -> String {
    REPOSITORIO_DE_ACTUALIZACIONES_POR_DEFECTO.to_owned()
}

const fn tiempo_maximo_actualizacion_por_defecto() -> u64 {
    TIEMPO_MAXIMO_ACTUALIZACION_POR_DEFECTO
}

impl Default for Ajustes {
    fn default() -> Self {
        Self {
            tiempo_maximo_git_segundos: TIEMPO_MAXIMO_GIT_POR_DEFECTO,
            limite_de_bandeja_bytes: LIMITE_DE_BANDEJA_POR_DEFECTO,
            comprobar_actualizaciones: COMPROBAR_ACTUALIZACIONES_POR_DEFECTO,
            repositorio_de_actualizaciones: repositorio_de_actualizaciones_por_defecto(),
            tiempo_maximo_actualizacion_segundos: TIEMPO_MAXIMO_ACTUALIZACION_POR_DEFECTO,
            limite_de_latido_horas: LIMITE_DE_LATIDO_POR_DEFECTO,
        }
    }
}

impl Ajustes {
    /// Comprueba la invariante de unos ajustes recién leídos.
    ///
    /// # Errors
    ///
    /// Devuelve el motivo del rechazo si algún valor es cero o el repositorio no tiene la forma
    /// propietario/nombre.
    pub fn validar(self) -> Result<Self, &'static str> {
        if self.tiempo_maximo_git_segundos == 0 {
            return Err(textos::AJUSTE_TIEMPO_CERO);
        }
        if self.limite_de_bandeja_bytes == 0 {
            return Err(textos::AJUSTE_LIMITE_CERO);
        }
        if self.limite_de_latido_horas == 0 {
            return Err(textos::AJUSTE_LATIDO_CERO);
        }
        if self.tiempo_maximo_actualizacion_segundos == 0 {
            return Err(textos::AJUSTE_TIEMPO_ACTUALIZACION_CERO);
        }
        if !actualizaciones::repositorio_valido(&self.repositorio_de_actualizaciones) {
            return Err(textos::AJUSTE_REPOSITORIO_INVALIDO);
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

    /// Horas sin latido tras las que una reserva se da por huérfana.
    #[must_use]
    pub const fn limite_de_latido_horas(&self) -> u64 {
        self.limite_de_latido_horas
    }

    /// Si se consulta al arrancar la última versión publicada.
    #[must_use]
    pub const fn comprobar_actualizaciones(&self) -> bool {
        self.comprobar_actualizaciones
    }

    /// Repositorio de GitHub donde se buscan las versiones, como propietario/nombre.
    #[must_use]
    pub fn repositorio_de_actualizaciones(&self) -> &str {
        &self.repositorio_de_actualizaciones
    }

    /// Tiempo máximo que se espera a la respuesta de GitHub.
    #[must_use]
    pub const fn tiempo_maximo_actualizacion(&self) -> Duration {
        Duration::from_secs(self.tiempo_maximo_actualizacion_segundos)
    }
}
