//! Encargos del canal: los bloques de una bandeja que llevan «Para» seguido de una lista de
//! destinatarios.
//!
//! Reglas:
//! - «Para» va con mayúscula y no forma parte de otra palabra. Puede ir a mitad de línea, detrás de
//!   la fecha o de un título de markdown.
//! - Los destinatarios van detrás, separados por comas, «y» o «e». La lista acaba en el primer
//!   «:», «;», «.» o «—». Lo que va entre paréntesis no cuenta.
//! - Un destinatario vale con su número («Codex 1») y, si es la instancia 1, también sin él
//!   («Codex»).
//! - Un bloque va desde la línea del «Para» hasta la primera línea en blanco, la siguiente línea
//!   con encargo, un título, una marca de latido o el final. Su huella ([`crate::huella`])
//!   identifica el encargo aunque cambie de sitio en la bandeja.

use crate::huella;
use crate::identidad::Identidad;

/// Palabra que abre un encargo.
const PALABRA_DEL_ENCARGO: &str = "Para ";
/// Marca con la que empieza cada intervención de una bandeja.
const MARCA_DE_LATIDO: &str = "LATIDO:";
/// Caracteres que cierran la lista de destinatarios.
const FIN_DE_LISTA: [char; 4] = [':', ';', '.', '—'];
/// Separadores de destinatarios dentro de la lista, además de la coma.
const CONJUNCIONES: [&str; 2] = [" y ", " e "];

/// Un encargo leído de una bandeja.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encargo {
    /// Primera línea del bloque, sin espacios a los lados, para enseñarla en un aviso.
    pub primera_linea: String,
    /// Huella del bloque entero.
    pub huella: u64,
    /// Fecha y hora del último LATIDO anterior al encargo en su bandeja, si lo hay.
    pub latido: Option<String>,
}

/// Destinatarios de todos los «Para» de una línea, como se escribieron y en orden.
#[must_use]
pub fn destinatarios(linea: &str) -> Vec<String> {
    let mut encontrados = Vec::new();
    let mut resto = linea;
    while let Some(posicion) = resto.find(PALABRA_DEL_ENCARGO) {
        let antes = resto[..posicion].chars().next_back();
        let tras = &resto[posicion + PALABRA_DEL_ENCARGO.len()..];
        if antes.is_none_or(|c| !c.is_alphanumeric()) {
            encontrados.extend(lista_de(tras));
        }
        resto = tras;
    }
    encontrados
}

/// Si la línea encarga algo a `identidad`.
#[must_use]
pub fn es_para(linea: &str, identidad: &Identidad) -> bool {
    destinatarios(linea)
        .iter()
        .any(|destinatario| nombra_a(destinatario, identidad))
}

/// Encargos de un texto dirigidos a `identidad`, en orden.
#[must_use]
pub fn encargos_para(texto: &str, identidad: &Identidad) -> Vec<Encargo> {
    let lineas: Vec<&str> = texto.lines().collect();
    let mut encargos = Vec::new();
    let mut latido = None;
    for (inicio, linea) in lineas.iter().enumerate() {
        if let Some(valor) = latido_de(linea) {
            latido = Some(valor);
        }
        if !es_para(linea, identidad) {
            continue;
        }
        let fin = lineas[inicio + 1..]
            .iter()
            .position(|siguiente| cierra_el_bloque(siguiente))
            .map_or(lineas.len(), |desplazamiento| inicio + 1 + desplazamiento);
        let bloque = lineas[inicio..fin].join("\n");
        encargos.push(Encargo {
            primera_linea: linea.trim().to_owned(),
            huella: huella::de_texto(&bloque),
            latido: latido.clone(),
        });
    }
    encargos
}

/// Fecha y hora de una línea de latido («LATIDO: 2026-11-22 13:17:48»), si lo es.
#[must_use]
pub fn latido_de(linea: &str) -> Option<String> {
    linea
        .trim()
        .strip_prefix(MARCA_DE_LATIDO)
        .map(|resto| resto.trim().to_owned())
        .filter(|valor| !valor.is_empty())
}

/// Si una línea acaba el bloque de un encargo anterior.
fn cierra_el_bloque(linea: &str) -> bool {
    let limpia = linea.trim();
    limpia.is_empty()
        || limpia.starts_with('#')
        || limpia.starts_with(MARCA_DE_LATIDO)
        || !destinatarios(linea).is_empty()
}

/// Destinatarios del texto que sigue a «Para ».
fn lista_de(texto: &str) -> Vec<String> {
    let sin_parentesis = quitar_parentesis(texto);
    let lista = sin_parentesis
        .split(|c| FIN_DE_LISTA.contains(&c))
        .next()
        .unwrap_or_default();
    lista
        .split(',')
        .flat_map(partir_por_conjunciones)
        .map(|destinatario| destinatario.trim().trim_matches('*').trim().to_owned())
        .filter(|destinatario| !destinatario.is_empty())
        .collect()
}

/// Parte un trozo de la lista por las conjunciones.
fn partir_por_conjunciones(trozo: &str) -> Vec<&str> {
    let mut partes = vec![trozo];
    for conjuncion in CONJUNCIONES {
        partes = partes
            .into_iter()
            .flat_map(|parte| parte.split(conjuncion))
            .collect();
    }
    partes
}

/// El texto sin lo que va entre paréntesis.
fn quitar_parentesis(texto: &str) -> String {
    let mut profundidad = 0_usize;
    texto
        .chars()
        .filter(|c| match c {
            '(' => {
                profundidad += 1;
                false
            }
            ')' => {
                profundidad = profundidad.saturating_sub(1);
                false
            }
            _ => profundidad == 0,
        })
        .collect()
}

/// Si un destinatario, como se escribió, nombra a `identidad`.
fn nombra_a(destinatario: &str, identidad: &Identidad) -> bool {
    let mut palabras = destinatario.split_whitespace();
    let Some(herramienta) = palabras.next() else {
        return false;
    };
    if !herramienta.eq_ignore_ascii_case(identidad.herramienta()) {
        return false;
    }
    match palabras.next().map(str::parse::<u32>) {
        Some(Ok(numero)) => numero == identidad.instancia(),
        _ => identidad.instancia() == 1,
    }
}
