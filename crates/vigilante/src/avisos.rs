//! Avisos pendientes de entregar en el modo persistente. Lógica pura: junta lo que traen varias
//! pasadas, sin repetir etiquetas ni archivos, y dice cuándo toca entregarlo
//! ([`entrega::toca_entregar`]). Las horas llegan como argumentos, en segundos desde 1970.

use canal::contrato::{
    ETIQUETA_DE_ENCARGO, ETIQUETA_DE_MUTACION, ETIQUETA_DE_RESPUESTA, SEPARADOR_DE_ETIQUETAS,
};

use canal::identidad::Identidad;

use crate::entrega::{self, ResultadoDeEntrega};
use crate::pasada::{Cambio, Resultado};
use crate::puertos::Entregador;

/// Etiquetas en el orden en que se escriben en avisos.log.
const ORDEN_DE_ETIQUETAS: [&str; 3] = [
    ETIQUETA_DE_ENCARGO,
    ETIQUETA_DE_RESPUESTA,
    ETIQUETA_DE_MUTACION,
];

/// Etiquetas de un cambio.
fn etiquetas_del_cambio(cambio: &Cambio) -> Vec<&'static str> {
    let mut etiquetas = Vec::new();
    if cambio.respuesta_de_programator() {
        etiquetas.push(ETIQUETA_DE_RESPUESTA);
    } else if cambio.encargo {
        etiquetas.push(ETIQUETA_DE_ENCARGO);
    }
    if cambio.mutacion {
        etiquetas.push(ETIQUETA_DE_MUTACION);
    }
    etiquetas
}

/// Un lote de avisos listo para entregar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lote {
    /// Etiquetas, separadas por comas, en el orden de avisos.log.
    pub etiquetas: String,
    /// Archivos del canal, en el orden en que llegaron.
    pub archivos: Vec<String>,
}

impl Lote {
    /// Etiquetas y archivos de los cambios etiquetados de una sola pasada.
    #[must_use]
    pub fn de_resultado(resultado: &Resultado) -> Self {
        let mut pendientes = AvisosPendientes::default();
        pendientes.anadir(resultado);
        pendientes.lote()
    }
}

/// Avisos que esperan su entrega.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AvisosPendientes {
    etiquetas: Vec<&'static str>,
    archivos: Vec<String>,
    ultima_entrega: Option<u64>,
}

impl AvisosPendientes {
    /// Añade los cambios etiquetados de una pasada, sin repetir etiquetas ni archivos.
    pub fn anadir(&mut self, resultado: &Resultado) {
        for cambio in resultado.cambios.iter().filter(|c| c.etiquetado()) {
            for etiqueta in etiquetas_del_cambio(cambio) {
                if !self.etiquetas.contains(&etiqueta) {
                    self.etiquetas.push(etiqueta);
                }
            }
            if !self.archivos.contains(&cambio.archivo) {
                self.archivos.push(cambio.archivo.clone());
            }
        }
    }

    /// Si no hay nada pendiente.
    #[must_use]
    pub fn esta_vacio(&self) -> bool {
        self.archivos.is_empty()
    }

    /// Lo pendiente, sin vaciarlo.
    fn lote(&self) -> Lote {
        let etiquetas: Vec<&str> = ORDEN_DE_ETIQUETAS
            .iter()
            .copied()
            .filter(|etiqueta| self.etiquetas.contains(etiqueta))
            .collect();
        Lote {
            etiquetas: etiquetas.join(&SEPARADOR_DE_ETIQUETAS.to_string()),
            archivos: self.archivos.clone(),
        }
    }

    /// Si hay algo pendiente y ya toca entregar a la hora `ahora`, lo devuelve, lo vacía y anota
    /// la entrega.
    pub fn tomar_si_toca(&mut self, ahora: u64) -> Option<Lote> {
        if self.esta_vacio() || !entrega::toca_entregar(self.ultima_entrega, ahora) {
            return None;
        }
        let lote = self.lote();
        self.etiquetas.clear();
        self.archivos.clear();
        self.ultima_entrega = Some(ahora);
        Some(lote)
    }
}

/// Si toca a la hora `ahora`, entrega lo pendiente con `entregador` en un solo aviso para
/// `identidad`, y devuelve el lote y cómo fue la entrega.
pub fn entregar_si_toca(
    pendientes: &mut AvisosPendientes,
    ahora: u64,
    identidad: &Identidad,
    entregador: &dyn Entregador,
) -> Option<(Lote, ResultadoDeEntrega)> {
    let lote = pendientes.tomar_si_toca(ahora)?;
    let mensaje = entrega::texto_del_aviso(identidad, &lote.archivos);
    let resultado = entregador.entregar(&mensaje);
    Some((lote, resultado))
}
