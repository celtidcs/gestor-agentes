//! Lógica pura sobre .gestor/canal/estado.md que la aplicación aplica al preparar un proyecto:
//! añadir los apartados obligatorios que falten y anotar el papel que el Director elige en la
//! ventana al lanzar un agente. No toca el disco: recibe el texto y devuelve el nuevo.
//!
//! Los agentes escriben el apartado de papeles a su manera, así que la aplicación no reescribe
//! nada: solo añade una línea al final del apartado. El arranque de AGENTS.md dice que, si un
//! agente aparece varias veces, vale la línea más reciente.

use std::io;
use std::path::Path;

use super::archivos;
use super::informe::{InformeDeInicializacion, TipoDeCambio};
use super::momento::Momento;
use super::revision_del_canal::APARTADOS_OBLIGATORIOS;
use crate::agentes::CARPETA_CANAL;
use crate::instrucciones::formato::FormatoDeTexto;

/// Ruta de estado.md, relativa al proyecto.
fn ruta_del_estado() -> String {
    format!("{CARPETA_CANAL}/estado.md")
}

/// Apartado de los papeles.
const APARTADO_PAPELES: &str = "## Papeles";

/// Cuerpo con que se añade un apartado que falta, como en la plantilla de estado.md.
const CUERPO_VACIO: &str = "Ninguna.";

/// Cuerpo con que se añade el apartado de papeles, como en la plantilla de estado.md.
const CUERPO_DE_PAPELES: &str = "- Jefe de proyecto: sin asignar\n- Ayudantes: ninguno anotado";

/// Texto que distingue las líneas de papel que anota la aplicación.
const MARCA_DE_GESTOR: &str = "en Gestor Agentes al lanzarlo";

/// Papel de un agente en el proyecto. Lo decide el Director.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Papel {
    /// Jefe de proyecto: solo hay uno.
    Jefe,
    /// Ayudante del Jefe.
    Ayudante,
}

impl Papel {
    /// Nombre del papel tal como lo pregunta el arranque de AGENTS.md.
    #[must_use]
    pub const fn nombre(self) -> &'static str {
        match self {
            Self::Jefe => "Jefe de proyecto",
            Self::Ayudante => "ayudante",
        }
    }
}

/// Añade al final de estado.md los apartados obligatorios que falten, en el orden de la plantilla.
/// Devuelve el texto nuevo y los apartados añadidos, o nada si no falta ninguno.
#[must_use]
pub fn completar_apartados(estado: &str) -> Option<(String, Vec<&'static str>)> {
    let faltan: Vec<&'static str> = APARTADOS_OBLIGATORIOS
        .iter()
        .copied()
        .filter(|titulo| !tiene_apartado(estado, titulo))
        .collect();
    if faltan.is_empty() {
        return None;
    }
    let mut texto = estado.trim_end_matches('\n').to_owned();
    for titulo in &faltan {
        let cuerpo = if *titulo == APARTADO_PAPELES {
            CUERPO_DE_PAPELES
        } else {
            CUERPO_VACIO
        };
        texto.push_str(&format!("\n\n{titulo}\n\n{cuerpo}"));
    }
    texto.push('\n');
    Some((texto, faltan))
}

/// Línea con que la aplicación anota el papel elegido. `momento` es la fecha y la hora del
/// sistema, como las escriben los agentes.
#[must_use]
pub fn linea_de_papel(identidad: &str, papel: Papel, momento: &str) -> String {
    let sustituye = match papel {
        Papel::Jefe => " Sustituye a cualquier Jefe anotado antes.",
        Papel::Ayudante => "",
    };
    format!(
        "- {identidad}, {momento}: **{}** (elegido por el Director {MARCA_DE_GESTOR}).{sustituye}",
        papel.nombre()
    )
}

/// Añade la línea del papel al final del apartado de papeles, que se crea si falta. No cambia nada
/// si la última línea que la aplicación anotó para esa identidad ya da ese papel y sigue vigente.
#[must_use]
pub fn anotar_papel(estado: &str, identidad: &str, papel: Papel, momento: &str) -> Option<String> {
    let base = completar_apartados(estado).map_or_else(|| estado.to_owned(), |(texto, _)| texto);
    if papel_vigente_anotado_por_gestor(&base, identidad) == Some(papel) {
        return (base != estado).then_some(base);
    }
    let lineas: Vec<&str> = base.lines().collect();
    let (inicio, fin) = limites_de_papeles(&lineas)?;
    let ultima_con_texto = (inicio..fin)
        .rev()
        .find(|&i| !lineas[i].trim().is_empty())
        .unwrap_or(inicio);
    let nueva = linea_de_papel(identidad, papel, momento);
    let mut resultado: Vec<String> = lineas.iter().map(|l| (*l).to_owned()).collect();
    if ultima_con_texto == inicio {
        // Apartado sin nada: la línea va tras una en blanco, como en la plantilla.
        resultado.insert(inicio + 1, String::new());
        resultado.insert(inicio + 2, nueva);
    } else {
        resultado.insert(ultima_con_texto + 1, nueva);
    }
    Some(resultado.join("\n") + "\n")
}

/// El papel de la última línea que la aplicación anotó para esa identidad, si la hay.
#[must_use]
pub fn papel_anotado_por_gestor(estado: &str, identidad: &str) -> Option<Papel> {
    let prefijo = format!("- {identidad}, ");
    estado
        .lines()
        .rev()
        .find(|linea| linea.starts_with(&prefijo) && linea.contains(MARCA_DE_GESTOR))
        .and_then(|linea| {
            [Papel::Jefe, Papel::Ayudante]
                .into_iter()
                .find(|papel| linea.contains(&format!("**{}**", papel.nombre())))
        })
}

/// Posición del título «## Papeles» y del final de su apartado (el siguiente título o el final del
/// texto).
fn limites_de_papeles(lineas: &[&str]) -> Option<(usize, usize)> {
    let inicio = lineas
        .iter()
        .position(|linea| linea.trim() == APARTADO_PAPELES)?;
    let fin = lineas[inicio + 1..]
        .iter()
        .position(|linea| linea.trim_start().starts_with('#'))
        .map_or(lineas.len(), |i| inicio + 1 + i);
    Some((inicio, fin))
}

/// El papel de la última línea que la aplicación anotó para esa identidad, si sigue vigente: no
/// hay después otra línea de esa identidad (la más reciente manda, paso 6 del arranque) ni, si es
/// Jefe, otra línea que nombre un Jefe (solo hay uno). Solo cuentan las líneas del apartado de
/// papeles: una reserva u otra anotación de la misma identidad no relevan su papel.
fn papel_vigente_anotado_por_gestor(estado: &str, identidad: &str) -> Option<Papel> {
    let prefijo = format!("- {identidad}, ");
    let todas: Vec<&str> = estado.lines().collect();
    let (inicio, fin) = limites_de_papeles(&todas)?;
    let lineas = &todas[inicio + 1..fin];
    let posicion = lineas
        .iter()
        .rposition(|linea| linea.starts_with(&prefijo) && linea.contains(MARCA_DE_GESTOR))?;
    let papel = papel_anotado_por_gestor(lineas[posicion], identidad)?;
    let jefe = Papel::Jefe.nombre();
    let relevado = lineas[posicion + 1..].iter().any(|linea| {
        linea.starts_with(&prefijo)
            || (papel == Papel::Jefe && linea.starts_with("- ") && linea.contains(jefe))
    });
    (!relevado).then_some(papel)
}

/// Si estado.md tiene el apartado, con o sin espacios a los lados del título.
fn tiene_apartado(estado: &str, titulo: &str) -> bool {
    estado.lines().any(|linea| linea.trim() == titulo)
}

/// Añade a estado.md, en disco, los apartados obligatorios que falten. Si estado.md no existe no
/// hace nada: lo crea la estructura de .gestor/.
///
/// # Errors
///
/// El error de disco al leer o escribir estado.md.
pub fn completar_en_disco(raiz: &Path, informe: &mut InformeDeInicializacion) -> io::Result<()> {
    let relativa = ruta_del_estado();
    let ruta = archivos::ruta(raiz, &relativa);
    let Some((estado, formato)) = archivos::leer(&ruta)? else {
        return Ok(());
    };
    if let Some((texto, anadidos)) = completar_apartados(&estado) {
        archivos::escribir(&ruta, &texto, formato)?;
        informe.anotar_cambio(
            &relativa,
            TipoDeCambio::Actualizado,
            Some(format!("apartados añadidos: {}", anadidos.join(", "))),
        );
    }
    Ok(())
}

/// Anota en estado.md, en disco, el papel que el Director ha elegido en la ventana para la
/// identidad que se lanza. Devuelve si ha cambiado algo.
///
/// # Errors
///
/// El error de disco al leer o escribir estado.md.
pub fn anotar_en_disco(
    raiz: &Path,
    identidad: &str,
    papel: Papel,
    momento: Momento,
) -> io::Result<bool> {
    let ruta = archivos::ruta(raiz, &ruta_del_estado());
    let (estado, formato) =
        archivos::leer(&ruta)?.unwrap_or_else(|| (String::new(), FormatoDeTexto::default()));
    let fecha_y_hora = momento.como_fecha_y_hora().map_or_else(
        || momento.fecha_iso(),
        |f| f.format(FORMATO_DE_MOMENTO).to_string(),
    );
    match anotar_papel(&estado, identidad, papel, &fecha_y_hora) {
        Some(texto) => {
            archivos::escribir(&ruta, &texto, formato)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Fecha y hora como las escriben los agentes en el canal.
const FORMATO_DE_MOMENTO: &str = "%Y-%m-%d %H:%M:%S";
