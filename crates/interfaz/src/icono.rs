//! Icono de la ventana. Es el mismo dibujo que la 2.3.2 (gestor-claude.png, 256 × 256), que
//! viaja dentro del ejecutable para no depender de ningún archivo junto a él. El icono del propio
//! .exe en Windows lo pone build.rs con gestor-claude.ico.

use eframe::egui::IconData;

/// PNG del icono, incluido en el ejecutable al compilar.
pub const ICONO_PNG: &[u8] = include_bytes!("../assets/gestor-claude.png");

/// El PNG del icono no se ha podido decodificar. Solo puede pasar si el archivo incluido está
/// dañado; quien lo reciba abre la ventana sin icono.
#[derive(Debug, thiserror::Error)]
#[error("no se puede leer el icono de la ventana: {0}")]
pub struct ErrorDeIcono(String);

/// Convierte un PNG en el icono de la ventana, en RGBA de 8 bits por canal.
///
/// # Errors
///
/// Devuelve [`ErrorDeIcono`] con el motivo si los bytes no son un PNG válido.
pub fn decodificar(png: &[u8]) -> Result<IconData, ErrorDeIcono> {
    eframe::icon_data::from_png_bytes(png).map_err(|error| ErrorDeIcono(error.to_string()))
}
