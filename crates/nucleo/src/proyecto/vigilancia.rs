//! Lectura de lo que deja el vigilante de cada agente en .gestor/<nombre>/vigilante/, para el panel
//! del equipo de la ventana: si hay un vigilante en marcha (vivo.json) y cuántos encargos le ha
//! avisado desde el último acuse RECIBIDO de su bandeja (avisos.log). Lógica pura: los textos y las
//! horas llegan como argumentos; comprobar que el proceso sigue vivo le toca a quien llama.

use chrono::{NaiveDateTime, NaiveTime};
use serde_json::Value;

use super::revision_del_canal::latido_de;

/// Carpeta del recuerdo del vigilante dentro de la carpeta del agente.
pub const CARPETA_DEL_VIGILANTE: &str = "vigilante";

/// Archivo con el proceso del vigilante en marcha.
pub const ARCHIVO_VIVO: &str = "vivo.json";

/// Registro de los avisos que ha dado el vigilante.
pub const ARCHIVO_DE_AVISOS: &str = "avisos.log";

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

/// Etiqueta con que avisos.log anota un encargo.
const ETIQUETA_DE_ENCARGO: &str = "encargo";

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
        pid: u32::try_from(valor.get("pid")?.as_u64()?).ok()?,
        arranque: valor.get("arranque")?.as_u64()?,
        modo: valor.get("modo")?.as_str()?.to_owned(),
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
            etiquetas.contains(ETIQUETA_DE_ENCARGO) && desde.is_none_or(|desde| *hora > desde)
        })
        .count()
}
