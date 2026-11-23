//! Proyecto de Antigravity (agy) que corresponde a una carpeta.
//!
//! agy abre cada sesión en un proyecto suyo, que se elige con `--project <id o nombre>` (ayuda de
//! agy 1.5.12, `agy --help`). Sin esa opción usa su proyecto por defecto, aunque se abra en otra
//! carpeta: así trabajó un agente en otro proyecto el 2026-11-18. Cada proyecto es un JSON en
//! ~/.gemini/config/projects/ con su `id` y sus carpetas en `projectResources.resources[]`, como
//! `folderUri` o como `gitFolder.folderUri`. Formato observado en el equipo, no documentado.

use std::path::Path;

use serde_json::Value;

/// Carpeta de los proyectos de agy, relativa a la carpeta personal.
pub const CARPETA_DE_PROYECTOS: [&str; 3] = [".gemini", "config", "projects"];

/// Prefijo de las direcciones de archivo.
const ESQUEMA_DE_ARCHIVO: &str = "file://";

/// Lo que se sabe del proyecto de agy de una carpeta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProyectoDeAgy {
    /// La carpeta tiene proyecto: su `id`.
    Encontrado(String),
    /// Los proyectos se leen y ninguno tiene la carpeta: agy puede crearlo con `--new-project`.
    Falta,
    /// No se entiende ningún proyecto (no hay, o su formato ha cambiado). Crear uno en cada
    /// lanzamiento llenaría agy de proyectos repetidos, así que no se abre.
    Ilegible,
}

/// Busca el proyecto de la carpeta entre los textos de los JSON de proyectos de agy y dice si
/// está, si falta o si no se entiende ninguno.
#[must_use]
pub fn buscar_proyecto(proyectos: &[String], carpeta: &Path) -> ProyectoDeAgy {
    if let Some(id) = proyecto_de_la_carpeta(proyectos, carpeta) {
        return ProyectoDeAgy::Encontrado(id);
    }
    if proyectos.iter().any(|texto| se_entiende(texto)) {
        ProyectoDeAgy::Falta
    } else {
        ProyectoDeAgy::Ilegible
    }
}

/// Indica si un JSON tiene la forma de un proyecto de agy: `id` de texto y
/// `projectResources.resources` como lista.
fn se_entiende(texto: &str) -> bool {
    serde_json::from_str::<Value>(texto).is_ok_and(|valor| {
        valor.get("id").is_some_and(Value::is_string)
            && valor
                .get("projectResources")
                .and_then(|recursos| recursos.get("resources"))
                .is_some_and(Value::is_array)
    })
}

/// Busca, entre los textos de los JSON de proyectos de agy, el `id` del primero que tiene la
/// carpeta dada entre sus recursos. Los JSON que no se entienden se ignoran.
#[must_use]
pub fn proyecto_de_la_carpeta(proyectos: &[String], carpeta: &Path) -> Option<String> {
    let buscada = normalizar(&carpeta.to_string_lossy());
    proyectos.iter().find_map(|texto| {
        let valor: Value = serde_json::from_str(texto).ok()?;
        let id = valor.get("id")?.as_str()?.to_owned();
        let recursos = valor
            .get("projectResources")?
            .get("resources")?
            .as_array()?;
        recursos
            .iter()
            .filter_map(direccion_del_recurso)
            .any(|direccion| normalizar(&ruta_de_la_direccion(direccion)) == buscada)
            .then_some(id)
    })
}

/// La dirección de un recurso, sea carpeta suelta o carpeta de git.
fn direccion_del_recurso(recurso: &Value) -> Option<&str> {
    recurso
        .get("folderUri")
        .or_else(|| recurso.get("gitFolder")?.get("folderUri"))?
        .as_str()
}

/// Ruta de una dirección file://, con los %XX descodificados. En Windows quita la barra de
/// delante de la letra de unidad (file:///m:/...).
fn ruta_de_la_direccion(direccion: &str) -> String {
    let sin_esquema = direccion
        .strip_prefix(ESQUEMA_DE_ARCHIVO)
        .unwrap_or(direccion);
    let ruta = descodificar(sin_esquema);
    match ruta.strip_prefix('/') {
        Some(resto) if tiene_letra_de_unidad(resto) => resto.to_owned(),
        _ => ruta,
    }
}

/// Descodifica las secuencias %XX. Una secuencia mal formada se deja tal cual.
fn descodificar(texto: &str) -> String {
    let bytes = texto.as_bytes();
    let mut salida = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let par = bytes.get(i + 1..i + 3).and_then(|par| {
            std::str::from_utf8(par)
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        });
        match (bytes[i], par) {
            (b'%', Some(byte)) => {
                salida.push(byte);
                i += 3;
            }
            (byte, _) => {
                salida.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&salida).into_owned()
}

/// Forma comparable de una ruta: barras normales y sin barra final. Una ruta con letra de unidad
/// va en minúsculas, porque agy guarda la letra en minúscula y Windows no distingue mayúsculas.
fn normalizar(ruta: &str) -> String {
    let ruta = ruta.replace('\\', "/");
    let ruta = ruta.trim_end_matches('/');
    if tiene_letra_de_unidad(ruta) {
        ruta.to_lowercase()
    } else {
        ruta.to_owned()
    }
}

/// Indica si una ruta empieza por una letra de unidad, como «m:».
fn tiene_letra_de_unidad(ruta: &str) -> bool {
    let mut caracteres = ruta.chars();
    matches!(
        (caracteres.next(), caracteres.next()),
        (Some(letra), Some(':')) if letra.is_ascii_alphabetic()
    )
}
