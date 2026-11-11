//! Núcleo de Gestor Agentes, sin interfaz ni dependencias de plataforma.
//!
//! Contiene las identidades de los agentes, la configuración portable y sus ajustes, la lógica
//! pura de las instrucciones ([`instrucciones`]), la preparación de un proyecto, que es la parte
//! que toca el disco ([`proyecto`]), el servicio de git y la detección de cuentas. La arquitectura
//! está descrita en documentation/architecture.md.

pub mod agentes;
pub mod ajustes;
pub mod configuracion;
pub mod cuentas;
pub mod git;
pub mod instrucciones;
pub mod plantillas;
pub mod proyecto;
pub mod textos;
