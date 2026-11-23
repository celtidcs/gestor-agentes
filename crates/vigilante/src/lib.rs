//! Vigilante del canal de Gestor Agentes (ejecutable gestor-vigilante).
//!
//! Gestor Agentes lo copia en .gestor/vigilante/ de cada proyecto. Vigila .gestor/canal/ por un
//! agente y le avisa de los encargos dirigidos a él y de las mutaciones vivas nuevas. Sustituye a
//! los guiones vigilar-canal.ps1 y vigilar-canal.sh: un solo proceso pequeño, sin intérprete.
//!
//! Piezas:
//! - [`opciones`]: la línea de órdenes, sin tocar nada.
//! - [`pasada`]: qué ha cambiado y qué etiquetas lleva, sin disco.
//! - [`recuerdo`]: lo visto en la pasada anterior, en disco.
//! - [`procesos`]: el árbol de procesos, para terminar cuando muere quien lo lanzó.
//! - [`entrega`]: cómo se hace llegar el aviso a la sesión de Codex.
//! - [`vivo`]: el latido de la vigilancia que lee la ventana de Gestor Agentes.
//! - [`ejecucion`]: los modos, que juntan todo lo anterior.

pub mod ejecucion;
pub mod entrega;
pub mod hilo;
pub mod opciones;
pub mod pasada;
pub mod procesos;
pub mod recuerdo;
pub mod vivo;
