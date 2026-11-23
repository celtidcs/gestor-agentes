//! Lectura del canal de Gestor Agentes (.gestor/canal/), sin disco, sin reloj y sin dependencias.
//!
//! La comparten el vigilante del canal y el núcleo, para que los dos entiendan igual:
//! - la identidad de un agente ([`identidad`]);
//! - los encargos dirigidos a él ([`encargos`]);
//! - el apartado de mutaciones vivas de estado.md ([`mutaciones`]);
//! - las huellas con que se reconoce un texto ya visto ([`huella`]).

pub mod acuses;
pub mod encargos;
pub mod huella;
pub mod identidad;
pub mod mutaciones;
