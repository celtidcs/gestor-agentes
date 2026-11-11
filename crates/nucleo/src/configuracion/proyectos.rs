//! Proyecto registrado en la lista de la aplicación.

use serde::{Deserialize, Serialize};

/// Proyecto registrado, con los nombres de campo de la 2.x. Los campos no son públicos (núcleo,
/// 5.3): se leen con sus métodos y la última apertura solo cambia con
/// [`ProyectoRegistrado::anotar_apertura`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProyectoRegistrado {
    /// Nombre visible.
    #[serde(rename = "Name", default)]
    nombre: String,
    /// Carpeta del proyecto.
    #[serde(rename = "Path", default)]
    ruta: String,
    /// Última apertura en ISO 8601, tal como la escribió quien la guardó. Se conserva como texto
    /// para no reinterpretar lo que escribió la 2.x.
    #[serde(rename = "LastOpened", default)]
    ultima_apertura: String,
}

impl ProyectoRegistrado {
    /// Proyecto con su nombre visible, su carpeta y su última apertura en ISO 8601.
    #[must_use]
    pub fn new(
        nombre: impl Into<String>,
        ruta: impl Into<String>,
        ultima_apertura: impl Into<String>,
    ) -> Self {
        Self {
            nombre: nombre.into(),
            ruta: ruta.into(),
            ultima_apertura: ultima_apertura.into(),
        }
    }

    /// Nombre visible.
    #[must_use]
    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    /// Carpeta del proyecto.
    #[must_use]
    pub fn ruta(&self) -> &str {
        &self.ruta
    }

    /// Última apertura en ISO 8601, tal como se guardó.
    #[must_use]
    pub fn ultima_apertura(&self) -> &str {
        &self.ultima_apertura
    }

    /// Anota `momento`, en ISO 8601, como última apertura del proyecto.
    pub fn anotar_apertura(&mut self, momento: String) {
        self.ultima_apertura = momento;
    }
}
