//! Gestor Agentes: la ventana de escritorio y la lógica que la gobierna.
//!
//! La lógica (lista de proyectos, perfiles, diálogos, trabajos, fechas y textos) no depende de
//! egui y se prueba en tests/ sin abrir ninguna ventana. Solo [`tema`] y [`ventana`] pintan, y
//! solo [`sistema`] toca el disco, el reloj y los procesos.

pub mod aplicacion;
pub mod arranque;
pub mod aviso_de_version;
pub use nucleo::fecha;
pub mod icono;
pub use nucleo::orden;
pub mod perfiles;
pub mod rutas;
pub mod sistema;
pub mod tema;
pub mod textos;
pub mod trabajos;
pub mod ventana;
