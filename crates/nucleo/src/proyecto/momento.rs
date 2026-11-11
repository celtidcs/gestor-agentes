//! Momento de reloj con la precisión que necesita la aplicación (segundos) y el reloj del que se
//! obtiene. El reloj se inyecta para que el inicializador se pueda probar con una hora fija, igual
//! que el ejemplo oraculo, que fija 2026-11-10 12:00:00.

use std::fmt;

/// Fecha y hora local, sin zona horaria, con precisión de segundos.
///
/// Solo se usa para nombrar respaldos y fechar el encabezado de un bloque de legado. Los campos se
/// validan al construirlo, así que siempre representa una fecha y hora existentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Momento {
    anio: u16,
    mes: u8,
    dia: u8,
    hora: u8,
    minuto: u8,
    segundo: u8,
}

/// Fecha u hora que no existe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("momento inválido: {anio:04}-{mes:02}-{dia:02} {hora:02}:{minuto:02}:{segundo:02}")]
pub struct MomentoInvalido {
    anio: u16,
    mes: u8,
    dia: u8,
    hora: u8,
    minuto: u8,
    segundo: u8,
}

/// Último año representable con cuatro cifras, que es lo que admiten los formatos de la 2.3.2.
const ULTIMO_ANIO: u16 = 9999;

impl Momento {
    /// Primer momento representable, 0001-01-01 00:00:00. Es el que da el reloj del sistema si
    /// este marcara una fecha que los formatos de la 2.3.2 no pueden escribir.
    pub const MINIMO: Self = Self {
        anio: 1,
        mes: 1,
        dia: 1,
        hora: 0,
        minuto: 0,
        segundo: 0,
    };

    /// Crea un momento validando la fecha (calendario gregoriano) y la hora.
    ///
    /// # Errors
    ///
    /// Devuelve [`MomentoInvalido`] si la fecha no existe, como el 30 de febrero, o la hora se sale
    /// de rango.
    pub fn new(
        anio: u16,
        mes: u8,
        dia: u8,
        hora: u8,
        minuto: u8,
        segundo: u8,
    ) -> Result<Self, MomentoInvalido> {
        let valido = (1..=ULTIMO_ANIO).contains(&anio)
            && (1..=12).contains(&mes)
            && dia >= 1
            && dia <= dias_del_mes(anio, mes)
            && hora < 24
            && minuto < 60
            && segundo < 60;
        if valido {
            Ok(Self {
                anio,
                mes,
                dia,
                hora,
                minuto,
                segundo,
            })
        } else {
            Err(MomentoInvalido {
                anio,
                mes,
                dia,
                hora,
                minuto,
                segundo,
            })
        }
    }

    /// Fecha en formato «yyyy-MM-dd», la del encabezado de un bloque de legado.
    #[must_use]
    pub fn fecha_iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.anio, self.mes, self.dia)
    }

    /// Marca «yyyyMMdd-HHmmss» del nombre de un respaldo o un archivado; ordena cronológicamente.
    #[must_use]
    pub fn marca_de_tiempo(&self) -> String {
        format!(
            "{:04}{:02}{:02}-{:02}{:02}{:02}",
            self.anio, self.mes, self.dia, self.hora, self.minuto, self.segundo
        )
    }
}

impl fmt::Display for Momento {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:02}:{:02}:{:02}",
            self.fecha_iso(),
            self.hora,
            self.minuto,
            self.segundo
        )
    }
}

/// Días de un mes del calendario gregoriano.
const fn dias_del_mes(anio: u16, mes: u8) -> u8 {
    match mes {
        2 if es_bisiesto(anio) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Año bisiesto del calendario gregoriano.
const fn es_bisiesto(anio: u16) -> bool {
    (anio.is_multiple_of(4) && !anio.is_multiple_of(100)) || anio.is_multiple_of(400)
}

/// Fuente de la hora actual.
pub trait Reloj {
    /// Momento actual en hora local.
    fn ahora(&self) -> Momento;
}

/// Reloj que siempre da el mismo momento. Sirve para las pruebas y para el oráculo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelojFijo(pub Momento);

impl Reloj for RelojFijo {
    fn ahora(&self) -> Momento {
        self.0
    }
}

/// Reloj del sistema en hora local. Es el que usa la aplicación.
#[derive(Debug, Clone, Copy, Default)]
pub struct RelojDelSistema;

impl Reloj for RelojDelSistema {
    fn ahora(&self) -> Momento {
        use chrono::{Datelike, Timelike};
        let ahora = chrono::Local::now();
        // chrono garantiza mes, día, hora, minuto y segundo dentro de rango; el segundo intercalar
        // (60) se recorta a 59. Un año fuera de u16 no es representable y se trata igual.
        let anio = u16::try_from(ahora.year()).unwrap_or(u16::MAX);
        let segundo = ahora.second().min(59);
        Momento::new(
            anio,
            octeto(ahora.month()),
            octeto(ahora.day()),
            octeto(ahora.hour()),
            octeto(ahora.minute()),
            octeto(segundo),
        )
        .unwrap_or(Momento::MINIMO)
    }
}

/// Convierte un campo de fecha de chrono, que siempre cabe, en octeto.
fn octeto(valor: u32) -> u8 {
    u8::try_from(valor).unwrap_or(u8::MAX)
}
