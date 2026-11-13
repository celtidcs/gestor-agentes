//! Revisión del canal de un proyecto: lógica pura que lee el texto de .gestor/canal/estado.md y el
//! de las bandejas y dice qué no cuadra y cómo está el equipo. No toca el disco ni el
//! reloj: el texto y la hora llegan como argumentos.
//!
//! Lo que se comprueba es solo estructural, porque estos archivos los escriben modelos de lenguaje
//! y un análisis más fino sería frágil: los apartados obligatorios de estado.md, las mutaciones
//! vivas, las reservas de una misma ruta por dos agentes y las reservas de un agente que ya no da
//! señales. Las líneas de reserva siguen el formato de la plantilla de estado.md:
//! `- <Agente>, <fecha hora>: <ruta>[, <ruta>…] — <motivo>`.

use chrono::NaiveDateTime;

/// Apartados que estado.md tiene que tener, tal como los escribe su plantilla.
pub const APARTADOS_OBLIGATORIOS: [&str; 3] =
    ["## Papeles", "## Reservas activas", "## Mutaciones vivas"];

/// Apartado de los papeles del equipo.
const APARTADO_PAPELES: &str = "## Papeles";
/// Apartado de las reservas de archivos.
const APARTADO_RESERVAS: &str = "## Reservas activas";
/// Apartado de las mutaciones de control aplicadas.
const APARTADO_MUTACIONES: &str = "## Mutaciones vivas";
/// Apartado de las autoexclusiones de funciones.
const APARTADO_AUTOEXCLUSIONES: &str = "## Autoexclusiones vigentes";

/// Texto con el que la plantilla dice que un apartado está vacío.
const VACIO: &str = "Ninguna.";

/// Principio de la línea del Jefe en el apartado de papeles de la plantilla.
const LINEA_DEL_JEFE: &str = "- Jefe de proyecto:";

/// Lo que sigue al Jefe en la plantilla cuando todavía no hay ninguno.
const JEFE_SIN_ASIGNAR: &str = "sin asignar";

/// Marca con la que empieza cada intervención de una bandeja.
const MARCA_DE_LATIDO: &str = "LATIDO:";

/// Formatos de fecha y hora que se aceptan tras la marca de latido: con y sin segundos.
const FORMATOS_DE_LATIDO: [&str; 2] = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M"];

/// Separador entre las rutas y el motivo de una reserva, como en la plantilla.
const SEPARADOR_DEL_MOTIVO: &str = " — ";

/// Segundos de una hora.
const SEGUNDOS_POR_HORA: i64 = 3600;

/// Bandeja del canal: nombre del archivo y su texto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bandeja {
    /// Nombre del archivo dentro de .gestor/canal/, como «claude2.md».
    pub nombre: String,
    /// Texto completo.
    pub texto: String,
}

/// Algo del canal que no cuadra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvisoDelCanal {
    /// A estado.md le falta un apartado obligatorio.
    FaltaApartado(&'static str),
    /// Hay mutaciones de control aplicadas.
    MutacionesVivas,
    /// Una línea de reserva no sigue el formato de la plantilla.
    ReservaIlegible(String),
    /// Dos agentes reservan la misma ruta, o uno una carpeta que contiene la ruta del otro.
    ReservaRepetida {
        /// Ruta en conflicto.
        ruta: String,
        /// Los dos agentes.
        agentes: (String, String),
    },
    /// La reserva es de un agente sin latido reciente: ninguno legible, o más antiguo que el límite.
    ReservaSinLatido {
        /// Agente de la reserva.
        agente: String,
        /// Horas desde su último latido, si lo tiene.
        horas: Option<i64>,
    },
}

/// Reserva leída de estado.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reserva {
    /// Agente que reserva, como «Claude 2».
    pub agente: String,
    /// Rutas reservadas.
    pub rutas: Vec<String>,
}

/// Último latido de un agente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Latido {
    /// Identificador del agente, el nombre de su bandeja sin «.md», como «claude2».
    pub identificador: String,
    /// Fecha y hora de su último latido legible.
    pub momento: NaiveDateTime,
}

/// Cómo está el equipo de un proyecto, para enseñarlo en la ventana.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResumenDelEquipo {
    /// Jefe de proyecto anotado, si lo hay.
    pub jefe: Option<String>,
    /// Último latido de cada bandeja que tiene alguno, por orden de nombre.
    pub latidos: Vec<Latido>,
    /// Número de reservas activas.
    pub reservas: usize,
    /// Si hay mutaciones vivas.
    pub mutaciones_vivas: bool,
    /// Si hay autoexclusiones vigentes.
    pub autoexclusiones: bool,
}

/// Revisa el canal y devuelve lo que no cuadra, en el orden en que se comprueba.
#[must_use]
pub fn revisar(
    estado: &str,
    bandejas: &[Bandeja],
    ahora: NaiveDateTime,
    limite_horas: u64,
) -> Vec<AvisoDelCanal> {
    let mut avisos: Vec<AvisoDelCanal> = APARTADOS_OBLIGATORIOS
        .iter()
        .filter(|apartado| apartado_de(estado, apartado).is_none())
        .map(|apartado| AvisoDelCanal::FaltaApartado(apartado))
        .collect();
    if hay_contenido(estado, APARTADO_MUTACIONES) {
        avisos.push(AvisoDelCanal::MutacionesVivas);
    }
    let (reservas, ilegibles) = leer_reservas(estado);
    avisos.extend(ilegibles.into_iter().map(AvisoDelCanal::ReservaIlegible));
    avisos.extend(repetidas(&reservas));
    let latidos = latidos(bandejas);
    let limite = i64::try_from(limite_horas).unwrap_or(i64::MAX);
    let mut vistos: Vec<&str> = Vec::new();
    for reserva in &reservas {
        if vistos.contains(&reserva.agente.as_str()) {
            continue;
        }
        vistos.push(&reserva.agente);
        let ultimo = identificador_de(&reserva.agente).and_then(|identificador| {
            latidos
                .iter()
                .find(|l| l.identificador == identificador)
                .map(|l| l.momento)
        });
        let horas = ultimo.map(|momento| (ahora - momento).num_seconds() / SEGUNDOS_POR_HORA);
        if horas.is_none_or(|h| h > limite) {
            avisos.push(AvisoDelCanal::ReservaSinLatido {
                agente: reserva.agente.clone(),
                horas,
            });
        }
    }
    avisos
}

/// Resume el equipo del proyecto a partir de estado.md y de las bandejas.
#[must_use]
pub fn resumir(estado: &str, bandejas: &[Bandeja]) -> ResumenDelEquipo {
    let jefe = apartado_de(estado, APARTADO_PAPELES).and_then(|lineas| {
        lineas
            .iter()
            .find_map(|l| l.strip_prefix(LINEA_DEL_JEFE))
            .map(|resto| resto.trim().trim_end_matches('.').to_owned())
            .filter(|jefe| !jefe.is_empty() && !jefe.starts_with(JEFE_SIN_ASIGNAR))
    });
    ResumenDelEquipo {
        jefe,
        latidos: latidos(bandejas),
        reservas: leer_reservas(estado).0.len(),
        mutaciones_vivas: hay_contenido(estado, APARTADO_MUTACIONES),
        autoexclusiones: hay_contenido(estado, APARTADO_AUTOEXCLUSIONES),
    }
}

/// Identificador de bandeja de un nombre de agente: «Claude 2» da «claude2» y «Codex 1» o
/// «Codex», «codex». Nada si el nombre no tiene esa forma.
#[must_use]
pub fn identificador_de(nombre: &str) -> Option<String> {
    let mut partes = nombre.split_whitespace();
    let herramienta = partes.next()?.to_lowercase();
    if !herramienta.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let numero = match partes.next() {
        Some(texto) => texto.parse::<u32>().ok().filter(|n| *n >= 1)?,
        None => 1,
    };
    if partes.next().is_some() {
        return None;
    }
    Some(if numero == 1 {
        herramienta
    } else {
        format!("{herramienta}{numero}")
    })
}

/// Último latido legible de cada bandeja, por orden de nombre.
#[must_use]
pub fn latidos(bandejas: &[Bandeja]) -> Vec<Latido> {
    let mut latidos: Vec<Latido> = bandejas
        .iter()
        .filter_map(|bandeja| {
            let momento = bandeja.texto.lines().rev().find_map(latido_de)?;
            Some(Latido {
                identificador: bandeja.nombre.trim_end_matches(".md").to_lowercase(),
                momento,
            })
        })
        .collect();
    latidos.sort_by(|a, b| a.identificador.cmp(&b.identificador));
    latidos
}

/// Fecha y hora de una línea «LATIDO: <fecha hora>», si la tiene.
fn latido_de(linea: &str) -> Option<NaiveDateTime> {
    let resto = linea.trim().strip_prefix(MARCA_DE_LATIDO)?.trim();
    FORMATOS_DE_LATIDO.iter().find_map(|formato| {
        // La hora puede ir seguida de texto: se prueba con los caracteres justos de cada formato.
        let largo = resto
            .char_indices()
            .nth(longitud_de(formato))
            .map_or(resto.len(), |(i, _)| i);
        NaiveDateTime::parse_from_str(&resto[..largo], formato).ok()
    })
}

/// Caracteres que ocupa una fecha escrita con `formato`.
fn longitud_de(formato: &str) -> usize {
    // Cada campo ocupa dos caracteres en el formato y escribe dos cifras, salvo %Y, que escribe
    // cuatro: la fecha mide lo que el formato más dos.
    formato.len() + 2
}

/// Líneas del apartado `titulo` de estado.md, sin el título, o nada si no está. Acaba en el
/// siguiente título.
fn apartado_de<'a>(estado: &'a str, titulo: &str) -> Option<Vec<&'a str>> {
    let mut lineas = estado.lines();
    lineas.find(|l| l.trim() == titulo)?;
    Some(
        lineas
            .take_while(|l| !l.starts_with('#'))
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect(),
    )
}

/// Si el apartado existe y dice algo distinto de «Ninguna.».
fn hay_contenido(estado: &str, titulo: &str) -> bool {
    apartado_de(estado, titulo).is_some_and(|lineas| lineas.iter().any(|l| *l != VACIO))
}

/// Reservas legibles de estado.md y las líneas de reserva que no siguen el formato.
fn leer_reservas(estado: &str) -> (Vec<Reserva>, Vec<String>) {
    let mut reservas = Vec::new();
    let mut ilegibles = Vec::new();
    // Solo las líneas de lista son reservas: el resto del apartado es explicación o «Ninguna.».
    for linea in apartado_de(estado, APARTADO_RESERVAS).unwrap_or_default() {
        if !linea.starts_with("- ") {
            continue;
        }
        match leer_reserva(linea) {
            Some(reserva) => reservas.push(reserva),
            None => ilegibles.push(linea.to_owned()),
        }
    }
    (reservas, ilegibles)
}

/// Lee `- <Agente>, <fecha hora>: <rutas> — <motivo>`.
fn leer_reserva(linea: &str) -> Option<Reserva> {
    let resto = linea.strip_prefix("- ")?;
    let (agente, resto) = resto.split_once(", ")?;
    identificador_de(agente)?;
    // La hora lleva dos puntos: las rutas empiezan tras el primer «: ».
    let (_, rutas) = resto.split_once(": ")?;
    let rutas = rutas
        .split_once(SEPARADOR_DEL_MOTIVO)
        .map_or(rutas, |(rutas, _)| rutas);
    let rutas: Vec<String> = rutas
        .split(", ")
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(str::to_owned)
        .collect();
    (!rutas.is_empty()).then(|| Reserva {
        agente: agente.trim().to_owned(),
        rutas,
    })
}

/// Rutas que reservan dos agentes distintos: la misma, o una carpeta (terminada en «/») que
/// contiene la del otro.
fn repetidas(reservas: &[Reserva]) -> Vec<AvisoDelCanal> {
    let mut avisos = Vec::new();
    for (i, a) in reservas.iter().enumerate() {
        for b in reservas.iter().skip(i + 1) {
            if a.agente == b.agente {
                continue;
            }
            for ra in &a.rutas {
                for rb in &b.rutas {
                    if let Some(ruta) = en_conflicto(ra, rb) {
                        avisos.push(AvisoDelCanal::ReservaRepetida {
                            ruta: ruta.to_owned(),
                            agentes: (a.agente.clone(), b.agente.clone()),
                        });
                    }
                }
            }
        }
    }
    avisos
}

/// La ruta en conflicto entre dos reservas, si la hay: la más concreta de las dos.
fn en_conflicto<'a>(a: &'a str, b: &'a str) -> Option<&'a str> {
    if a == b {
        return Some(a);
    }
    if a.ends_with('/') && b.starts_with(a) {
        return Some(b);
    }
    if b.ends_with('/') && a.starts_with(b) {
        return Some(a);
    }
    None
}
