//! Lo que distingue a cada herramienta: dónde guarda sus cuentas y cómo se elige una.

use std::path::{Path, PathBuf};

/// Herramienta de consola cuyas cuentas sabe buscar la biblioteca.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Herramienta {
    /// Claude Code, de Anthropic.
    ClaudeCode,
    /// Codex CLI, de OpenAI.
    CodexCli,
    /// Antigravity CLI (`agy`), de Google.
    Agy,
}

/// Variable con la que Claude Code usa otra carpeta de configuración. Fuente:
/// <https://code.claude.com/docs/en/settings> («To keep the home-directory files somewhere
/// else, set CLAUDE_CONFIG_DIR») y <https://code.claude.com/docs/en/env-vars>.
const VARIABLE_CLAUDE: &str = "CLAUDE_CONFIG_DIR";

/// Variable con la que Codex CLI usa otra carpeta. Fuente:
/// <https://learn.chatgpt.com/docs/config-file/environment-variables> («Sets the root for
/// Codex state, including config, auth…», por defecto `~/.codex`).
const VARIABLE_CODEX: &str = "CODEX_HOME";

/// Archivos que solo existen en una carpeta que ya ha usado Claude Code: su configuración
/// global, sus credenciales y sus ajustes.
const ARCHIVOS_DE_CUENTA_CLAUDE: [&str; 3] = [".claude.json", ".credentials.json", "settings.json"];

/// Archivos que solo existen en una carpeta que ya ha usado Codex CLI.
const ARCHIVOS_DE_CUENTA_CODEX: [&str; 2] = ["config.toml", "auth.json"];

impl Herramienta {
    /// Todas las herramientas, en el orden en que se presentan.
    pub const TODAS: [Herramienta; 3] = [Self::ClaudeCode, Self::CodexCli, Self::Agy];

    /// Carpeta de la cuenta que usa la herramienta sin ninguna variable.
    ///
    /// Antigravity CLI no tiene carpeta propia de cuenta documentada; se toma `~/.gemini`,
    /// que es donde está su configuración MCP (<https://antigravity.google/docs/mcp>).
    pub fn carpeta_por_defecto(self, personal: &Path) -> PathBuf {
        personal.join(match self {
            Self::ClaudeCode => ".claude",
            Self::CodexCli => ".codex",
            Self::Agy => ".gemini",
        })
    }

    /// Carpeta cuya existencia delata que la herramienta está instalada con su cuenta por
    /// defecto. En Antigravity CLI es su subcarpeta propia: `~/.gemini` también la crean otras
    /// herramientas de Google.
    pub(crate) fn carpeta_que_delata(self, personal: &Path) -> PathBuf {
        match self {
            Self::Agy => self.carpeta_por_defecto(personal).join("antigravity-cli"),
            Self::ClaudeCode | Self::CodexCli => self.carpeta_por_defecto(personal),
        }
    }

    /// Variable de entorno con la que se elige otra cuenta, si la herramienta tiene una
    /// documentada. Antigravity CLI no la tiene (no consta en
    /// <https://antigravity.google/docs/cli/reference>).
    pub fn variable_de_cuenta(self) -> Option<&'static str> {
        match self {
            Self::ClaudeCode => Some(VARIABLE_CLAUDE),
            Self::CodexCli => Some(VARIABLE_CODEX),
            Self::Agy => None,
        }
    }

    /// Si se le pueden dar de alta cuentas a mano: solo a las herramientas que eligen la cuenta
    /// con una variable de entorno. Sin ella, su consola no usaría la carpeta añadida.
    pub fn admite_alta_manual(self) -> bool {
        self.variable_de_cuenta().is_some()
    }

    /// Las herramientas que admiten alta manual, en el orden de [`Herramienta::TODAS`]: las que
    /// un programa ofrece en su formulario de alta.
    pub fn con_alta_manual() -> Vec<Herramienta> {
        Self::TODAS
            .into_iter()
            .filter(|herramienta| herramienta.admite_alta_manual())
            .collect()
    }

    /// Archivos cuya presencia indica que una carpeta es una cuenta de esta herramienta.
    pub(crate) fn archivos_de_cuenta(self) -> &'static [&'static str] {
        match self {
            Self::ClaudeCode => &ARCHIVOS_DE_CUENTA_CLAUDE,
            Self::CodexCli => &ARCHIVOS_DE_CUENTA_CODEX,
            Self::Agy => &[],
        }
    }

    /// Archivo donde la herramienta lee sus servidores MCP para la cuenta de `carpeta`.
    ///
    /// - Claude Code: `~/.claude.json` en la cuenta por defecto
    ///   (<https://code.claude.com/docs/en/mcp>) y `<carpeta>/.claude.json` en las demás. Lo
    ///   segundo no consta en la documentación; se ha observado en cuentas reales.
    /// - Codex CLI: `<carpeta>/config.toml`.
    /// - Antigravity CLI: `<carpeta>/config/mcp_config.json`.
    pub(crate) fn archivo_de_configuracion(
        self,
        carpeta: &Path,
        por_defecto: bool,
        personal: &Path,
    ) -> PathBuf {
        match self {
            Self::ClaudeCode if por_defecto => personal.join(".claude.json"),
            Self::ClaudeCode => carpeta.join(".claude.json"),
            Self::CodexCli => carpeta.join("config.toml"),
            Self::Agy => carpeta.join("config").join("mcp_config.json"),
        }
    }
}
