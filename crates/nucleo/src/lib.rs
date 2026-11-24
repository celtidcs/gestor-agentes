//! Núcleo de Gestor Agentes, sin interfaz ni dependencias de plataforma.
//!
//! Contiene las identidades de los agentes, la configuración portable y sus ajustes, la lógica
//! pura de las instrucciones ([`instrucciones`]), la preparación de un proyecto, que es la parte
//! que toca el disco ([`proyecto`]), el servicio de git y la traducción a perfiles de las cuentas que encuentra la biblioteca común
//! deteccion-agentes ([`cuentas`], que la llama solo a través de [`cuentas_de_agentes`]). La arquitectura
//! está descrita en documentation/architecture.md. [`actualizaciones`] adapta, sin red, el aviso
//! de versión nueva a la biblioteca común aviso-version-github. [`fecha`] y [`orden`] dan la forma
//! de las fechas de la lista de proyectos y el orden, la comparación y el nombre de sus carpetas.

pub mod actualizaciones;
pub mod agentes;
pub mod ajustes;
pub mod aprendizaje;
pub mod configuracion;
pub mod cuentas;
pub mod cuentas_de_agentes;
pub mod fecha;
pub mod git;
pub mod instrucciones;
pub mod migracion_de_claude;
pub mod orden;
pub mod plantillas;
pub mod proyecto;
pub mod textos;
pub mod version_de_programa;
