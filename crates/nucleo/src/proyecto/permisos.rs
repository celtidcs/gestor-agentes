//! Permisos del arranque: lo que cada herramienta tiene autorizado por defecto en el proyecto.
//!
//! El arranque de AGENTS.md pide siempre lo mismo, casi todo lectura: saber la identidad, la fecha,
//! listar, ver el estado de git, escribir en .gestor/ y lanzar el vigilante del canal. Para que el
//! agente no lo pregunte en cada sesión, la preparación deja esas reglas, y solo esas, en el archivo
//! de permisos del proyecto de cada herramienta. Lo que el usuario ya tenga en ese archivo se
//! conserva; solo se añaden las reglas que faltan.
//!
//! Claude Code: `.claude/settings.local.json`, lista `permissions.allow`. Formato y sintaxis de las
//! reglas según https://code.claude.com/docs/en/permissions.md y
//! https://code.claude.com/docs/en/settings.md (consultadas el 2026-11-16): el archivo local del
//! proyecto se aplica sin el diálogo de confianza mientras git no lo siga; `Bash(orden *)` acepta
//! la orden sola y con argumentos; Monitor usa las mismas reglas que Bash; `Edit(/ruta/**)` se ancla
//! en la raíz del proyecto y cubre también Write.

use serde_json::{Map, Value};

use crate::textos;

/// Archivo de permisos de Claude Code en el proyecto, relativo a la raíz.
pub const ARCHIVO_DE_CLAUDE: &str = ".claude/settings.local.json";

/// Reglas de Claude Code para el arranque, en el orden en que se añaden.
pub const REGLAS_DE_CLAUDE: [&str; 11] = [
    "Bash(echo $GESTOR_AGENTE $GESTOR_PROYECTO $PWD)",
    "Bash(date *)",
    "Bash(ls *)",
    "Bash(git status *)",
    "Bash(./.gestor/vigilante/gestor-vigilante *)",
    "PowerShell(Write-Output $env:GESTOR_AGENTE $env:GESTOR_PROYECTO $PWD.Path)",
    "PowerShell(Get-Date *)",
    "PowerShell(Get-ChildItem *)",
    "PowerShell(git status *)",
    "PowerShell(./.gestor/vigilante/gestor-vigilante *)",
    "Edit(/.gestor/**)",
];

/// Lo que hay que hacer con el archivo de permisos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fusion {
    /// Ya tiene todas las reglas: no se toca.
    SinCambios,
    /// Texto nuevo del archivo, con las reglas que faltaban añadidas.
    Escribir(String),
    /// El archivo existe pero no es un objeto JSON con la forma esperada: no se toca, y se avisa
    /// con este motivo.
    Ilegible(String),
}

/// Añade a `actual` (el texto del archivo, si existe) las `reglas` que falten en
/// `permissions.allow`, conservando el resto del archivo y el orden de sus claves.
#[must_use]
pub fn fusionar(actual: Option<&str>, reglas: &[&str]) -> Fusion {
    let mut raiz = match actual.map(str::trim) {
        None | Some("") => Map::new(),
        Some(texto) => match serde_json::from_str::<Value>(texto) {
            Ok(Value::Object(mapa)) => mapa,
            Ok(_) => return Fusion::Ilegible(String::from(textos::PERMISOS_NO_ES_OBJETO)),
            Err(error) => return Fusion::Ilegible(error.to_string()),
        },
    };
    let permisos = raiz
        .entry("permissions")
        .or_insert_with(|| Value::Object(Map::new()));
    let Value::Object(permisos) = permisos else {
        return Fusion::Ilegible(String::from(textos::PERMISOS_SIN_OBJETO_DE_PERMISOS));
    };
    let permitidas = permisos
        .entry("allow")
        .or_insert_with(|| Value::Array(Vec::new()));
    let Value::Array(permitidas) = permitidas else {
        return Fusion::Ilegible(String::from(textos::PERMISOS_SIN_LISTA_DE_PERMITIDOS));
    };
    let faltan: Vec<Value> = reglas
        .iter()
        .filter(|regla| !permitidas.iter().any(|v| v.as_str() == Some(regla)))
        .map(|regla| Value::String((*regla).to_owned()))
        .collect();
    if faltan.is_empty() && actual.is_some_and(|t| !t.trim().is_empty()) {
        return Fusion::SinCambios;
    }
    permitidas.extend(faltan);
    match serde_json::to_string_pretty(&Value::Object(raiz)) {
        Ok(texto) => Fusion::Escribir(texto + "\n"),
        Err(error) => Fusion::Ilegible(error.to_string()),
    }
}
