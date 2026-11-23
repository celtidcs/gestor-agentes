//! Acuses de recibo: si el agente ya dejó constancia en su bandeja de haber leído un encargo.
//!
//! Un encargo se nombra con una etiqueta corta en su primera línea, como «PING-1» o «VERIF-360D».
//! Cuenta como acusado si la bandeja del agente tiene, en una intervención con un LATIDO igual o
//! posterior al del encargo, una línea con esa etiqueta y la palabra «recibido» en cualquier forma
//! de mayúsculas («RECIBIDO 14:48:22: … PING-1», «PING-1 recibido 13:27:10»). Así un vigilante que
//! arranca tarde no repite un aviso que el agente ya atendió (caso del 2026-11-22 a las 14:48).

use crate::encargos;

/// Palabra del acuse, en minúsculas.
const PALABRA_DE_ACUSE: &str = "recibido";

/// Caracteres que cierran la lista de destinatarios de un «Para», como en [`crate::encargos`].
const FIN_DE_LISTA: [char; 4] = [':', ';', '.', '—'];

/// Largo mínimo de una etiqueta: «M1» o «T4» son demasiado comunes para fiarse.
const LARGO_MINIMO_DE_ETIQUETA: usize = 3;

/// Largo de «AAAA-MM-DD HH:MM:SS», con el que se comparan los latidos.
const LARGO_DE_FECHA_Y_HORA: usize = 19;

/// Etiqueta del encargo en su primera línea: la primera palabra, tras la lista de destinatarios,
/// escrita en mayúsculas y cifras con al menos una cifra o un guion («PING-1», «VERIF-360D»).
#[must_use]
pub fn etiqueta(primera_linea: &str) -> Option<String> {
    let tras_para = primera_linea.split_once("Para ")?.1;
    let resto = tras_para.split_once(|c| FIN_DE_LISTA.contains(&c))?.1;
    palabras(resto)
        .find(|palabra| es_etiqueta(palabra))
        .map(str::to_owned)
}

/// Si la bandeja `propia` acusa el encargo `etiqueta`, cuyo latido es `latido_del_encargo`.
#[must_use]
pub fn acusado(etiqueta: &str, latido_del_encargo: Option<&str>, propia: &str) -> bool {
    let desde = latido_del_encargo.map(normalizar);
    let mut latido_actual: Option<String> = None;
    for linea in propia.lines() {
        if let Some(valor) = encargos::latido_de(linea) {
            latido_actual = Some(normalizar(&valor));
            continue;
        }
        let a_tiempo = match (&desde, &latido_actual) {
            (None, _) => true,
            (Some(desde), Some(actual)) => actual >= desde,
            (Some(_), None) => false,
        };
        if a_tiempo
            && linea.to_lowercase().contains(PALABRA_DE_ACUSE)
            && palabras(linea).any(|palabra| palabra == etiqueta)
        {
            return true;
        }
    }
    false
}

/// Palabras de una línea: tramos de letras, cifras y guiones.
fn palabras(texto: &str) -> impl Iterator<Item = &str> {
    texto
        .split(|c: char| !(c.is_alphanumeric() || c == '-'))
        .filter(|palabra| !palabra.is_empty())
}

/// Si una palabra tiene forma de etiqueta.
fn es_etiqueta(palabra: &str) -> bool {
    palabra.chars().count() >= LARGO_MINIMO_DE_ETIQUETA
        && palabra.chars().any(char::is_uppercase)
        && palabra
            .chars()
            .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '-')
        && palabra.chars().any(|c| c.is_ascii_digit() || c == '-')
}

/// Fecha y hora de un latido, sin lo que lleve detrás.
fn normalizar(latido: &str) -> String {
    latido.trim().chars().take(LARGO_DE_FECHA_Y_HORA).collect()
}
