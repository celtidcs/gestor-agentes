//! Configuración portable de la aplicación: la lista de proyectos y los perfiles de agente.
//!
//! Vive en la carpeta datos/ junto al ejecutable, que la interfaz calcula y pasa
//! aquí; el nucleo no adivina rutas. Los archivos y su esquema son los de la 2.x, con los mismos
//! nombres de campo, para que la migración sea una copia: config.json, perfiles-agentes.json,
//! perfiles-agentes.version y claude-instructions.txt. La 3.0.0 añade el perfil de Programator
//! con su ejecutable, que es configuración de cada equipo y nunca va en el código.
//!
//! Cada submódulo tiene una responsabilidad: `perfiles` es el modelo de los perfiles y su
//! validación, sin disco; `esquema` pone al día los perfiles de versiones anteriores, sin disco;
//! `proyectos` es la fila de la lista de proyectos; `almacen` lee y escribe los archivos de
//! datos/; y `desde_2x` copia la configuración de la 2.x.

mod almacen;
mod desde_2x;
mod error;
mod esquema;
mod perfiles;
mod proyectos;

pub use almacen::Configuracion;
pub use desde_2x::{carpeta_de_la_2x, CARPETA_DE_LA_2X};
pub use error::ErrorDeConfiguracion;
pub use perfiles::{
    HerramientaDelPerfil, PerfilDeAgente, LIMITE_DE_INSTRUCCIONES_CODEX,
    LIMITE_DE_INSTRUCCIONES_GEMINI,
};
pub use proyectos::ProyectoRegistrado;

/// Lista de proyectos.
pub const ARCHIVO_PROYECTOS: &str = "config.json";

/// Perfiles de agente.
pub const ARCHIVO_PERFILES: &str = "perfiles-agentes.json";

/// Versión del esquema de los perfiles ya aplicada.
pub const ARCHIVO_VERSION_PERFILES: &str = "perfiles-agentes.version";

/// Plantilla de CLAUDE.md que guardaba la versión 1, si existe.
pub const ARCHIVO_INSTRUCCIONES_ANTIGUAS: &str = "claude-instructions.txt";

/// Archivos que forman la configuración, en el orden en que se migran.
pub const ARCHIVOS_DE_CONFIGURACION: [&str; 4] = [
    ARCHIVO_PROYECTOS,
    ARCHIVO_PERFILES,
    ARCHIVO_VERSION_PERFILES,
    ARCHIVO_INSTRUCCIONES_ANTIGUAS,
];
