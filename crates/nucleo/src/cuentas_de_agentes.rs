//! El único módulo de Gestor Agentes que habla con la biblioteca deteccion-agentes.
//!
//! Copiado de examples/integracion_de_referencia.rs de la biblioteca, como manda su norma de
//! integración (N3). Es el único que llama a `buscar_cuentas` y `con_cuentas_manuales`; el resto
//! del programa solo usa los tipos `Cuenta`, `Herramienta`, `OrigenDeLaCuenta` y
//! `AjusteDeEntorno` y sus métodos. La traducción de las cuentas a los perfiles de Gestor
//! («Claude 2») está en [`crate::cuentas`].
//!
//! De la referencia faltan tres funciones que Gestor no necesita: `orden_de_consola` (Gestor
//! abre la consola con un guion de terminal y escribe en él el ajuste de entorno de la cuenta, ver
//! el lanzador), `archivo_para_registrar_mcp` (Gestor no registra servidores MCP) y
//! `nombre_para_mostrar` (Gestor presenta cada cuenta por la identidad de su perfil).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use deteccion_agentes::{
    buscar_cuentas, con_cuentas_manuales, validar_alta_manual, Cuenta, Herramienta, RechazoDeAlta,
};

/// Variable que guarda la carpeta personal en cada sistema.
fn variable_de_la_carpeta_personal() -> &'static str {
    if cfg!(windows) {
        "USERPROFILE"
    } else {
        "HOME"
    }
}

/// Una cuenta dada de alta a mano, tal como la guarda Gestor: la herramienta y la carpeta de un
/// perfil que fija su carpeta de configuración.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuentaManual {
    /// Herramienta de la cuenta. Solo una de `Herramienta::con_alta_manual()`; las demás las
    /// descarta la biblioteca.
    pub herramienta: Herramienta,
    /// Carpeta de configuración que eligió la persona.
    pub carpeta: PathBuf,
}

/// La carpeta personal de quien ejecuta el programa, o `None` si el sistema no la da.
///
/// Sin carpeta personal no se busca nada: Gestor no añade cuentas. El programa real pasa
/// `|nombre| std::env::var_os(nombre)` como `leer_variable`; las pruebas, una función propia.
pub fn carpeta_personal(leer_variable: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    leer_variable(variable_de_la_carpeta_personal())
        .filter(|valor| !valor.is_empty())
        .map(PathBuf::from)
}

/// Todas las cuentas del equipo: las que encuentra la biblioteca y las dadas de alta a mano.
///
/// Se llama cada vez que Gestor va a mostrar las cuentas (al buscar los agentes), nunca en cada
/// fotograma, porque lee el disco. La biblioteca descarta sola las manuales que no se pueden
/// usar: aquí no se filtra nada.
pub fn cuentas_del_equipo(
    personal: &Path,
    leer_variable: impl Fn(&str) -> Option<OsString>,
    manuales: &[CuentaManual],
) -> Vec<Cuenta> {
    let manuales: Vec<(Herramienta, PathBuf)> = manuales
        .iter()
        .map(|manual| (manual.herramienta, manual.carpeta.clone()))
        .collect();
    con_cuentas_manuales(buscar_cuentas(personal, leer_variable), &manuales)
}

/// Comprueba una cuenta que la persona quiere dar de alta a mano, antes de guardarla.
///
/// `cuentas` son las que Gestor ya tiene. Con el rechazo, Gestor dice a la persona el motivo con
/// sus propios textos.
///
/// # Errors
///
/// El [`RechazoDeAlta`] de la biblioteca.
pub fn validar_alta(nueva: &CuentaManual, cuentas: &[Cuenta]) -> Result<(), RechazoDeAlta> {
    validar_alta_manual(nueva.herramienta, &nueva.carpeta, cuentas)
}
