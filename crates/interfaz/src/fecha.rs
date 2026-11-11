//! Fechas de la última apertura: cómo se muestran y cómo se escriben.
//!
//! La lista de proyectos guarda la última apertura como texto ISO 8601, igual que la 2.x. La
//! zona horaria se recibe como parámetro para que el formato se pruebe sin depender de la zona
//! del equipo.

use chrono::{DateTime, NaiveDateTime, TimeZone, Timelike};

/// Formato con que se muestra la última apertura, el mismo de la 2.3.2 («dd/MM/yyyy HH:mm»).
const FORMATO_VISIBLE: &str = "%d/%m/%Y %H:%M";

/// Formato de una fecha sin desfase, como la escribe .NET para un DateTime sin zona.
const FORMATO_SIN_DESFASE: &str = "%Y-%m-%dT%H:%M:%S%.f";

/// Cien nanosegundos: la unidad («tick») de las fracciones de segundo de .NET, que escribe siete
/// cifras decimales.
const NANOSEGUNDOS_POR_TICK: u32 = 100;

/// Mayor fracción de siete cifras. Un segundo intercalar de chrono tiene más de mil millones de
/// nanosegundos y se recorta a este valor.
const FRACCION_MAXIMA: u32 = 9_999_999;

/// Texto visible de una última apertura guardada como ISO 8601.
///
/// Con desfase, se pasa a la zona indicada, como hacía .NET al leer un DateTime local. Sin
/// desfase, se muestra tal cual. Si el texto no es una fecha válida devuelve una cadena vacía, y
/// la fila se ve sin fecha en lugar de impedir que se cargue la lista.
#[must_use]
pub fn mostrar<Zona>(iso: &str, zona: &Zona) -> String
where
    Zona: TimeZone,
    Zona::Offset: std::fmt::Display,
{
    let iso = iso.trim();
    if let Ok(con_desfase) = DateTime::parse_from_rfc3339(iso) {
        return con_desfase
            .with_timezone(zona)
            .format(FORMATO_VISIBLE)
            .to_string();
    }
    NaiveDateTime::parse_from_str(iso, FORMATO_SIN_DESFASE)
        .map(|sin_desfase| sin_desfase.format(FORMATO_VISIBLE).to_string())
        .unwrap_or_default()
}

/// Texto ISO 8601 con siete decimales y desfase («2026-11-10T18:20:05.1234567+01:00»), como lo
/// escribe la 2.3.2, para que una lista guardada por la 3.0.0 la siga leyendo la 2.x.
///
/// chrono no tiene un especificador de siete decimales, así que la fracción se compone aparte.
#[must_use]
pub fn iso<Zona>(momento: &DateTime<Zona>) -> String
where
    Zona: TimeZone,
    Zona::Offset: std::fmt::Display,
{
    let fraccion = (momento.nanosecond() / NANOSEGUNDOS_POR_TICK).min(FRACCION_MAXIMA);
    format!(
        "{}.{fraccion:07}{}",
        momento.format("%Y-%m-%dT%H:%M:%S"),
        momento.format("%:z")
    )
}
