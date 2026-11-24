//! Lectura del canal de Gestor Agentes (.gestor/canal/), sin disco, sin reloj y sin dependencias.
//!
//! La comparten el vigilante del canal y el núcleo, para que los dos entiendan igual:
//! - la identidad de un agente ([`identidad`]);
//! - los encargos dirigidos a él ([`encargos`]);
//! - el apartado de mutaciones vivas de estado.md ([`mutaciones`]);
//! - las huellas con que se reconoce un texto ya visto ([`huella`]);
//! - los acuses de recibo de una bandeja ([`acuses`]);
//! - los apartados de un documento ([`apartados`]);
//! - los nombres de carpetas, archivos, variables y opciones que comparten la aplicación, el
//!   lanzador y el vigilante ([`contrato`]);
//! - la forma comparable de una ruta ([`rutas`]).

pub mod acuses;
pub mod apartados;
pub mod contrato;
pub mod encargos;
pub mod huella;
pub mod identidad;
pub mod mutaciones;
pub mod rutas;
