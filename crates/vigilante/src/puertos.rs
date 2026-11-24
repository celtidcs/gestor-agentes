//! Lo que el modo persistente necesita del mundo: la hora, la espera entre pasadas y la entrega
//! del aviso. En producción son el reloj del sistema, `thread::sleep` y `codex queue`; las pruebas
//! ponen los suyos.

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::entrega::{self, ResultadoDeEntrega};
use crate::vivo;

/// Hora actual.
pub trait Reloj {
    /// Segundos desde 1970.
    fn ahora(&self) -> u64;
}

/// Espera entre dos pasadas.
pub trait Espera {
    /// Espera `duracion`.
    fn esperar(&self, duracion: Duration);
}

/// Entrega de un aviso en la sesión del agente.
pub trait Entregador {
    /// Entrega `mensaje` y dice cómo fue.
    fn entregar(&self, mensaje: &str) -> ResultadoDeEntrega;
}

/// Reloj del sistema.
pub struct RelojDelSistema;

impl Reloj for RelojDelSistema {
    fn ahora(&self) -> u64 {
        vivo::ahora()
    }
}

/// Espera de verdad, con el hilo dormido.
pub struct EsperaDelSistema;

impl Espera for EsperaDelSistema {
    fn esperar(&self, duracion: Duration) {
        thread::sleep(duracion);
    }
}

/// Entrega con `codex queue` en una sesión de Codex.
pub struct EntregaConCodex {
    /// Ejecutable de Codex.
    pub codex: PathBuf,
    /// Raíz del proyecto.
    pub raiz: PathBuf,
    /// Sesión de Codex.
    pub hilo: String,
}

impl Entregador for EntregaConCodex {
    fn entregar(&self, mensaje: &str) -> ResultadoDeEntrega {
        entrega::entregar(&self.codex, &self.raiz, &self.hilo, mensaje)
    }
}
