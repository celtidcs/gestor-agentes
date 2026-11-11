//! Colores de la 2.3.2 (su paleta oscura) y su aplicación a egui.

use eframe::egui::{self, Color32};

/// Fondo de la ventana.
pub const FONDO: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x2e);
/// Fondo de la tabla, del núcleo común y de las pestañas sin seleccionar.
pub const SUPERFICIE: Color32 = Color32::from_rgb(0x31, 0x32, 0x44);
/// Cabeceras, fila seleccionada y pestaña activa.
pub const SUPERFICIE_ALTA: Color32 = Color32::from_rgb(0x45, 0x47, 0x5a);
/// Fila o pestaña bajo el puntero.
pub const SUPERFICIE_SENALADA: Color32 = Color32::from_rgb(0x3b, 0x3d, 0x52);
/// Texto principal.
pub const TEXTO: Color32 = Color32::from_rgb(0xcd, 0xd6, 0xf4);
/// Texto secundario y cabeceras.
pub const TEXTO_SECUNDARIO: Color32 = Color32::from_rgb(0xa6, 0xad, 0xc8);
/// Barra de estado y texto de los botones desactivados.
pub const TEXTO_APAGADO: Color32 = Color32::from_rgb(0x6c, 0x70, 0x86);
/// Botón de acción principal.
pub const MORADO: Color32 = Color32::from_rgb(0xcb, 0xa6, 0xf7);
/// Botón secundario.
pub const VERDE: Color32 = Color32::from_rgb(0xa6, 0xe3, 0xa1);
/// Botones de peligro.
pub const ROJO: Color32 = Color32::from_rgb(0xf3, 0x8b, 0xa8);

/// Tamaño del encabezado, como el FontSize 24 de la 2.3.2.
pub const TAMANO_ENCABEZADO: f32 = 24.0;
/// Tamaño del texto de botones y tabla (FontSize 13).
pub const TAMANO_TEXTO: f32 = 13.0;
/// Tamaño de la explicación, del núcleo y de la barra de estado (FontSize 12).
pub const TAMANO_PEQUENO: f32 = 12.0;
/// Alto máximo del texto de un diálogo antes de desplazarse (AltoMaximoDelMensaje de la 2.3.2).
pub const ALTO_MAXIMO_DIALOGO: f32 = 420.0;
/// Ancho de los diálogos (Width 460 de la 2.3.2).
pub const ANCHO_DIALOGO: f32 = 460.0;
/// Margen exterior de la ventana (Margin 20).
pub const MARGEN: i8 = 20;
/// Margen interior de los paneles de contenido (el de la 2.3.2).
pub const MARGEN_INTERIOR: i8 = 12;
/// Tamaño inicial de la ventana (900 por 600).
pub const TAMANO_VENTANA: [f32; 2] = [900.0, 600.0];
/// Ancho del campo de la ruta de un ejecutable en la pestaña de agentes. Nuevo en la 3.0.0:
/// con la ventana a su tamaño inicial de 900 puntos caben el nombre, el estado, este
/// campo y los dos botones de la fila.
pub const ANCHO_RUTA: f32 = 320.0;

/// Aplica la paleta a los estilos de egui.
pub fn aplicar(contexto: &egui::Context) {
    let mut visuales = egui::Visuals::dark();
    visuales.panel_fill = FONDO;
    visuales.window_fill = SUPERFICIE;
    visuales.extreme_bg_color = SUPERFICIE;
    visuales.faint_bg_color = SUPERFICIE_SENALADA;
    visuales.override_text_color = Some(TEXTO);
    visuales.selection.bg_fill = SUPERFICIE_ALTA;
    visuales.widgets.hovered.weak_bg_fill = SUPERFICIE_SENALADA;
    visuales.widgets.active.weak_bg_fill = SUPERFICIE_ALTA;
    contexto.set_visuals(visuales);
}

/// Color de cada botón de perfil: alternan morado y verde, como en la 2.3.2.
#[must_use]
pub const fn color_de_perfil(posicion: usize) -> Color32 {
    if posicion.is_multiple_of(2) {
        MORADO
    } else {
        VERDE
    }
}
