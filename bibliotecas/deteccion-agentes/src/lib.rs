//! Busca en el equipo las cuentas de los agentes de programación de consola.
//!
//! Una misma herramienta puede tener varias cuentas en un equipo: cada una vive en su propia
//! carpeta de configuración, y la herramienta elige cuál usar por una variable de entorno
//! (`CLAUDE_CONFIG_DIR` en Claude Code, `CODEX_HOME` en Codex CLI). La biblioteca no conoce
//! ninguna cuenta concreta: las encuentra mirando la carpeta personal y esas variables, y
//! añade las que la persona haya dado de alta a mano.
//!
//! Uso. Se llama a [`buscar_cuentas`] con la carpeta personal y una función que lea variables
//! de entorno, y después a [`con_cuentas_manuales`] con las cuentas guardadas por el programa,
//! que descarta las que no se pueden usar. Antes de guardar una cuenta nueva dada de alta a
//! mano, [`validar_alta_manual`] dice si se puede y, si no, por qué; el formulario de alta ofrece
//! las herramientas de [`Herramienta::con_alta_manual`].
//! De cada [`Cuenta`] se obtiene el archivo donde registrar un servidor MCP
//! ([`Cuenta::archivo_de_configuracion`]) y el ajuste de entorno con que abrir su consola
//! ([`Cuenta::ajuste_de_entorno`]).
//!
//! Errores. La búsqueda no falla: una carpeta que no se puede leer simplemente no aporta
//! cuentas.

#![deny(missing_docs)]
#![cfg_attr(test, deny(dead_code, unused_imports, unused_mut, unused_variables))]

mod alta;
mod busqueda;
mod cuenta;
mod herramienta;

pub use alta::{validar_alta_manual, RechazoDeAlta};
pub use busqueda::{buscar_cuentas, con_cuentas_manuales};
pub use cuenta::{AjusteDeEntorno, Cuenta, OrigenDeLaCuenta};
pub use herramienta::Herramienta;

#[cfg(test)]
mod pruebas;
