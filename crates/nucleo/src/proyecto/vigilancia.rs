//! Lectura de lo que deja el vigilante de cada agente en .gestor/<nombre>/vigilante/, para el panel
//! del equipo de la ventana: si hay un vigilante en marcha (vivo.json) y cuántos encargos le ha
//! avisado desde el último acuse RECIBIDO de su bandeja (avisos.log). Lógica pura: los textos y las
//! horas llegan como argumentos; comprobar que el proceso sigue vivo le toca a quien llama.

use canal::contrato::{
    claves_de_vivo, CARPETA_GESTOR, ETIQUETA_DE_ENCARGO, EXTENSION_DEL_CANAL,
    SEPARADOR_DE_ETIQUETAS,
};
use chrono::{NaiveDateTime, NaiveTime};
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::revision_del_canal::latido_de;

pub use canal::contrato::{ARCHIVO_DE_AVISOS, ARCHIVO_VIVO, CARPETA_DEL_VIGILANTE};

/// Principio de la línea de acuse en la bandeja (apartado 3 del núcleo común).
const MARCA_DE_ACUSE: &str = "RECIBIDO ";

/// Formato de la hora del acuse.
const FORMATO_DEL_ACUSE: &str = "%Y-%m-%d %H:%M:%S";

/// Largo de una hora escrita con [`FORMATO_DEL_ACUSE`].
const LARGO_DEL_ACUSE: usize = 19;

/// Formato del acuse que solo lleva la hora, como «RECIBIDO 07:17:33».
const FORMATO_DE_LA_HORA: &str = "%H:%M:%S";

/// Largo de una hora escrita con [`FORMATO_DE_LA_HORA`].
const LARGO_DE_LA_HORA: usize = 8;

/// Cómo está el vigilante de un agente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoDelVigilante {
    /// vivo.json apunta a un proceso que sigue en marcha.
    Vigilando,
    /// vivo.json apunta a un proceso que ya no está: el vigilante murió sin borrarlo.
    Caido,
}

/// Vigilancia de un agente para el panel del equipo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VigilanciaDeAgente {
    /// Identificador del agente, el nombre de su bandeja sin «.md», como «claude2».
    pub identificador: String,
    /// Estado del vigilante, o nada si no hay vivo.json (sin vigilante en marcha, o un proyecto de
    /// una versión anterior).
    pub vigilante: Option<EstadoDelVigilante>,
    /// Encargos avisados desde su último acuse RECIBIDO.
    pub encargos_sin_acuse: usize,
}

/// Vigilante en marcha según vivo.json.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VigilanteAnotado {
    /// Número del proceso.
    pub pid: u32,
    /// Hora de arranque del proceso, en segundos desde 1970, para no confundirlo con otro que
    /// reutilice el número.
    pub arranque: u64,
    /// Modo: «una-vez», «hasta-cambio» o «persistente».
    pub modo: String,
}

/// Lee vivo.json. Nada si no se entiende.
#[must_use]
pub fn leer_vivo(texto: &str) -> Option<VigilanteAnotado> {
    let valor: Value = serde_json::from_str(texto).ok()?;
    Some(VigilanteAnotado {
        pid: u32::try_from(valor.get(claves_de_vivo::PID)?.as_u64()?).ok()?,
        arranque: valor.get(claves_de_vivo::ARRANQUE)?.as_u64()?,
        modo: valor.get(claves_de_vivo::MODO)?.as_str()?.to_owned(),
    })
}

/// Hora del último acuse RECIBIDO de una bandeja, si tiene alguno legible.
///
/// El acuse lleva fecha y hora, o solo la hora: el núcleo común manda empezar cada intervención con
/// su LATIDO fechado, así que un acuse con solo la hora toma la fecha del LATIDO anterior más
/// cercano. Sin LATIDO antes, ese acuse no cuenta.
#[must_use]
pub fn ultimo_acuse(bandeja: &str) -> Option<NaiveDateTime> {
    let mut fecha_del_latido = None;
    let mut ultimo = None;
    for linea in bandeja.lines() {
        if let Some(latido) = latido_de(linea) {
            fecha_del_latido = Some(latido.date());
            continue;
        }
        let Some(resto) = linea.trim_start().strip_prefix(MARCA_DE_ACUSE) else {
            continue;
        };
        let hora = hora_completa(resto).or_else(|| {
            let hora =
                NaiveTime::parse_from_str(resto.get(..LARGO_DE_LA_HORA)?, FORMATO_DE_LA_HORA)
                    .ok()?;
            Some(fecha_del_latido?.and_time(hora))
        });
        ultimo = ultimo.max(hora);
    }
    ultimo
}

/// Fecha y hora completas al principio del acuse, si las tiene.
fn hora_completa(resto: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(resto.get(..LARGO_DEL_ACUSE)?, FORMATO_DEL_ACUSE).ok()
}

/// Encargos que avisos.log anota después de `desde` (segundos desde 1970), o todos si no hay
/// acuse. Las líneas que no se entienden no cuentan.
#[must_use]
pub fn encargos_sin_acuse(avisos: &str, desde: Option<i64>) -> usize {
    avisos
        .lines()
        .filter_map(|linea| {
            let mut campos = linea.split('\t');
            let hora: i64 = campos.next()?.parse().ok()?;
            let etiquetas = campos.next()?;
            Some((hora, etiquetas))
        })
        .filter(|(hora, etiquetas)| {
            etiquetas
                .split(SEPARADOR_DE_ETIQUETAS)
                .any(|etiqueta| etiqueta == ETIQUETA_DE_ENCARGO)
                && desde.is_none_or(|desde| *hora > desde)
        })
        .count()
}

/// Identificador del agente de una bandeja, su nombre sin «.md», como «claude2».
#[must_use]
pub fn identificador_de_bandeja(bandeja: &str) -> Option<&str> {
    bandeja.strip_suffix(EXTENSION_DEL_CANAL)
}

/// Carpeta del recuerdo del vigilante del agente `identificador` en el proyecto de `raiz`.
#[must_use]
pub fn carpeta_del_recuerdo(raiz: &Path, identificador: &str) -> PathBuf {
    raiz.join(CARPETA_GESTOR)
        .join(identificador)
        .join(CARPETA_DEL_VIGILANTE)
}

/// Lo que se sabe de la vigilancia de un agente, leído por quien llama.
pub struct DatosDeVigilancia<'a> {
    /// Identificador del agente.
    pub identificador: &'a str,
    /// Texto de vivo.json, si existe.
    pub vivo: Option<&'a str>,
    /// Texto de avisos.log, si existe.
    pub avisos: Option<&'a str>,
    /// Hora del último acuse, en segundos desde 1970, si hay alguno.
    pub ultimo_acuse: Option<i64>,
}

/// Vigilancia de un agente para el panel, o nada si no tiene vigilante anotado ni encargos sin
/// acuse. `esta_vivo` dice si el proceso anotado (número y hora de arranque) sigue en marcha.
#[must_use]
pub fn vigilancia_de_agente(
    datos: &DatosDeVigilancia<'_>,
    esta_vivo: &mut dyn FnMut(&VigilanteAnotado) -> bool,
) -> Option<VigilanciaDeAgente> {
    let vigilante = datos.vivo.and_then(leer_vivo).map(|anotado| {
        if esta_vivo(&anotado) {
            EstadoDelVigilante::Vigilando
        } else {
            EstadoDelVigilante::Caido
        }
    });
    let encargos_sin_acuse = datos
        .avisos
        .map_or(0, |avisos| encargos_sin_acuse(avisos, datos.ultimo_acuse));
    (vigilante.is_some() || encargos_sin_acuse > 0).then(|| VigilanciaDeAgente {
        identificador: datos.identificador.to_owned(),
        vigilante,
        encargos_sin_acuse,
    })
}
