//! Una cuenta encontrada y lo que hace falta para usarla.

use crate::Herramienta;
use std::path::{Path, PathBuf};

/// De dónde sale una cuenta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OrigenDeLaCuenta {
    /// La carpeta que usa la herramienta sin ninguna variable.
    PorDefecto,
    /// Una carpeta de la carpeta personal con los archivos de una cuenta.
    Busqueda,
    /// La carpeta que señala la variable de entorno de la herramienta.
    Variable,
    /// Una carpeta dada de alta a mano por la persona.
    Manual,
}

/// Una cuenta de una herramienta: la herramienta y su carpeta de configuración.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cuenta {
    /// Herramienta a la que pertenece.
    pub herramienta: Herramienta,
    /// Carpeta de configuración de la cuenta.
    pub carpeta: PathBuf,
    /// De dónde sale.
    pub origen: OrigenDeLaCuenta,
}

/// Cambio del entorno con que se abre la consola de una cuenta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AjusteDeEntorno {
    /// Poner `variable` con la carpeta de la cuenta.
    Poner {
        /// Nombre de la variable.
        variable: &'static str,
        /// Carpeta de la cuenta.
        valor: PathBuf,
    },
    /// Quitar `variable`, para que la consola use su cuenta por defecto aunque el programa que
    /// la abre la tenga puesta.
    Quitar {
        /// Nombre de la variable.
        variable: &'static str,
    },
}

impl AjusteDeEntorno {
    /// Aplica el ajuste a la orden con que se abrirá la consola.
    pub fn aplicar_a(&self, orden: &mut std::process::Command) {
        match self {
            Self::Poner { variable, valor } => {
                orden.env(variable, valor);
            }
            Self::Quitar { variable } => {
                orden.env_remove(variable);
            }
        }
    }
}

impl Cuenta {
    /// Indica si es la cuenta que la herramienta usa sin ninguna variable.
    pub fn es_la_por_defecto(&self) -> bool {
        self.origen == OrigenDeLaCuenta::PorDefecto
    }

    /// Nombre con que se distingue de las demás cuentas de la misma herramienta: el de su
    /// carpeta. `None` en la cuenta por defecto, que se presenta solo con el de la herramienta.
    pub fn nombre_visible(&self) -> Option<String> {
        (!self.es_la_por_defecto()).then(|| {
            self.carpeta
                .file_name()
                .unwrap_or(self.carpeta.as_os_str())
                .to_string_lossy()
                .into_owned()
        })
    }

    /// Archivo donde se registra un servidor MCP para esta cuenta.
    pub fn archivo_de_configuracion(&self, personal: &Path) -> PathBuf {
        self.herramienta
            .archivo_de_configuracion(&self.carpeta, self.es_la_por_defecto(), personal)
    }

    /// Cómo hay que ajustar el entorno de la consola para que use esta cuenta, o `None` si la
    /// herramienta no tiene variable de cuenta.
    pub fn ajuste_de_entorno(&self) -> Option<AjusteDeEntorno> {
        let variable = self.herramienta.variable_de_cuenta()?;
        Some(if self.es_la_por_defecto() {
            AjusteDeEntorno::Quitar { variable }
        } else {
            AjusteDeEntorno::Poner {
                variable,
                valor: self.carpeta.clone(),
            }
        })
    }
}
