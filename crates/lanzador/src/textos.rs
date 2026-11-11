//! Textos que el lanzador da al usuario, compuestos en un solo sitio.

use crate::{Herramienta, Sistema};

/// Nombre de Claude Code en los avisos.
const NOMBRE_CLAUDE: &str = "Claude Code";

/// Nombre de Codex CLI, el programa codex en terminal, en los avisos.
const NOMBRE_CODEX_CLI: &str = "Codex CLI";

/// Nombre de agy, la CLI de Antigravity, en los avisos.
const NOMBRE_AGY: &str = "agy (Antigravity CLI)";

/// Nombre de Programator en los avisos.
const NOMBRE_PROGRAMATOR: &str = "Programator";

/// Nombre de la herramienta en los avisos.
pub(crate) const fn nombre(herramienta: Herramienta) -> &'static str {
    match herramienta {
        Herramienta::Claude => NOMBRE_CLAUDE,
        Herramienta::Codex => NOMBRE_CODEX_CLI,
        Herramienta::Gemini => NOMBRE_AGY,
        Herramienta::Programator => NOMBRE_PROGRAMATOR,
    }
}

/// Detalle del error cuando no se encuentra un programa en Linux (claude, codex o agy).
const NO_ENCONTRADO_LINUX: &str = "no está en ~/.local/bin, /usr/local/bin, /usr/bin ni en el PATH";

/// Detalle del error cuando no se encuentra Claude Code en Windows.
const CLAUDE_NO_ENCONTRADO: &str = "no está claude.exe ni claude.cmd en ~/.local/bin ni en el PATH";

/// Detalle del error cuando no se encuentra Codex CLI en Windows.
const CODEX_CLI_NO_ENCONTRADO: &str =
    "no está en %LOCALAPPDATA%\\Programs\\OpenAI\\Codex\\bin, ni como codex.exe o codex.cmd en \
     ~/.local/bin o en el PATH";

/// Detalle del error cuando no se encuentra agy en Windows.
const AGY_NO_ENCONTRADO: &str =
    "no está en %LOCALAPPDATA%\\agy\\bin, en ~/.local/bin ni en el PATH";

/// Detalle del error de Programator sin ruta fijada. No se usa como «dónde se buscó», porque
/// Programator no se busca: ese caso es [`PROGRAMATOR_SIN_EJECUTABLE`].
const PROGRAMATOR_NO_SE_BUSCA: &str = "no se busca: su ruta se fija en el perfil";

/// Dónde se ha buscado el programa de la herramienta, para el error de no encontrado.
pub(crate) const fn donde_se_busco(herramienta: Herramienta, sistema: Sistema) -> &'static str {
    match (herramienta, sistema) {
        (Herramienta::Programator, _) => PROGRAMATOR_NO_SE_BUSCA,
        (_, Sistema::Linux) => NO_ENCONTRADO_LINUX,
        (Herramienta::Claude, Sistema::Windows) => CLAUDE_NO_ENCONTRADO,
        (Herramienta::Codex, Sistema::Windows) => CODEX_CLI_NO_ENCONTRADO,
        (Herramienta::Gemini, Sistema::Windows) => AGY_NO_ENCONTRADO,
    }
}

/// Detalle del error cuando el perfil de Programator no tiene ejecutable.
pub(crate) const PROGRAMATOR_SIN_EJECUTABLE: &str = "falta la ruta del ejecutable de Programator";

/// Detalle del error cuando el ejecutable de Programator no tiene carpeta.
pub(crate) const PROGRAMATOR_SIN_CARPETA: &str =
    "la ruta del ejecutable de Programator no tiene carpeta";

/// Detalle del error cuando el ejecutable fijado en el perfil no existe.
pub(crate) fn ejecutable_inexistente(ruta: &std::path::Path) -> String {
    format!("no existe {}", ruta.display())
}
