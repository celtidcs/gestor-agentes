//! Decide qué hacer con un archivo que la aplicación escribe entero (AGENTS.md, CLAUDE.md y los
//! documentos de consulta): dejarlo, crearlo o sustituirlo, y si antes hay que guardar una copia.
//!
//! Es lógica pura. La copia se pide cuando lo que hay no es lo último que escribió la aplicación,
//! porque entonces alguien lo editó a mano o lo escribió otra versión, y su texto no debe perderse
//! sin dejar rastro.

use super::{formato, huella};

/// Lo que hay que hacer con el archivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sustitucion {
    /// Ya tiene el texto vigente.
    SinCambios,
    /// No existe y se crea.
    Crear,
    /// Existe con otro texto y se sustituye.
    Sustituir {
        /// Si antes hay que guardar una copia exacta, porque su texto no es lo último que
        /// escribió la aplicación.
        respaldar: bool,
    },
}

/// Decide qué hacer con un archivo gestionado entero.
///
/// `existente` es su texto actual, si existe. `vigente` es el que debe tener. `huella_escrita` es
/// la huella de lo último que escribió la aplicación en él, si la tiene registrada.
#[must_use]
pub fn decidir(
    existente: Option<&str>,
    vigente: &str,
    huella_escrita: Option<&str>,
) -> Sustitucion {
    let Some(existente) = existente else {
        return Sustitucion::Crear;
    };
    let existente = formato::normalizar(existente);
    if existente == vigente {
        return Sustitucion::SinCambios;
    }
    let es_lo_ultimo_escrito = huella_escrita.is_some_and(|h| h == huella::calcular(&existente));
    Sustitucion::Sustituir {
        respaldar: !es_lo_ultimo_escrito,
    }
}
