//! Modelo de los perfiles de agente y su validación, sin tocar el disco.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::agentes::TipoAgente;
use crate::textos;

/// Herramienta de un perfil. Las tres primeras leen archivos de instrucciones; Programator solo
/// se lanza.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HerramientaDelPerfil {
    /// Claude Code.
    Claude,
    /// Codex.
    Codex,
    /// Gemini, con agy, la CLI de Antigravity.
    Gemini,
    /// El arnés Programator.
    Programator,
}

impl HerramientaDelPerfil {
    /// Tipo de agente de instrucciones, si la herramienta lo es.
    #[must_use]
    pub const fn tipo_agente(self) -> Option<TipoAgente> {
        match self {
            Self::Claude => Some(TipoAgente::Claude),
            Self::Codex => Some(TipoAgente::Codex),
            Self::Gemini => Some(TipoAgente::Gemini),
            Self::Programator => None,
        }
    }

    /// Nombre visible.
    #[must_use]
    pub const fn nombre(self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini",
            Self::Programator => "Programator",
        }
    }

    /// Bytes del archivo de instrucciones que lee la herramienta, si se sabe que tiene límite.
    #[must_use]
    pub const fn limite_de_instrucciones_por_defecto(self) -> Option<u64> {
        match self {
            Self::Codex => Some(LIMITE_DE_INSTRUCCIONES_CODEX),
            Self::Gemini => Some(LIMITE_DE_INSTRUCCIONES_GEMINI),
            Self::Claude | Self::Programator => None,
        }
    }
}

/// Bytes que Antigravity lee de cada archivo de reglas: «Antigravity truncates any single rule
/// file that exceeds 24,000 bytes» (https://antigravity.google/docs/rules/). Se midió
/// con agy 1.5.5, con marcas a ambos lados del límite.
pub const LIMITE_DE_INSTRUCCIONES_GEMINI: u64 = 24_000;

/// Bytes de AGENTS.md que lee Codex por defecto: project_doc_max_bytes, «32 KiB by default»
/// (https://learn.chatgpt.com/docs/agent-configuration/agents-md).
pub const LIMITE_DE_INSTRUCCIONES_CODEX: u64 = 32_768;

/// Perfil de agente, con los nombres de campo de la 2.x. Los campos no son públicos (núcleo,
/// 5.3): se leen con sus métodos, se construyen con [`PerfilDeAgente::new`] y los métodos `con_*`,
/// y la interfaz solo los cambia con operaciones del dominio como
/// [`PerfilDeAgente::fijar_ejecutable`]. La validez de una lista de perfiles se comprueba al
/// cargarla y al guardarla.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerfilDeAgente {
    /// Herramienta.
    #[serde(rename = "Tipo")]
    tipo: HerramientaDelPerfil,
    /// Número de instancia, desde 1.
    #[serde(rename = "Instancia", default = "primera_instancia")]
    instancia: u32,
    /// Carpeta de configuración de la herramienta (CLAUDE_CONFIG_DIR o CODEX_HOME). Admite «~».
    #[serde(rename = "DirectorioConfiguracion", default)]
    directorio_configuracion: Option<String>,
    /// Ejecutable fijado. Obligatorio para Programator; opcional para el resto. Admite «~».
    #[serde(
        rename = "Ejecutable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    ejecutable: Option<String>,
    /// Argumentos que se pasan al programa, por ejemplo `--model <id>` para agy o
    /// `-c project_doc_max_bytes=65536` para codex. El lanzador decide a qué herramientas los
    /// pasa.
    #[serde(rename = "Argumentos", default, skip_serializing_if = "Vec::is_empty")]
    argumentos: Vec<String>,
    /// Bytes del archivo de instrucciones que lee la herramienta, si no es el de su tipo (ver
    /// [`HerramientaDelPerfil::limite_de_instrucciones_por_defecto`]). Si un perfil de Codex sube
    /// project_doc_max_bytes con sus argumentos, tiene que subir esto a la vez, o el aviso de
    /// instrucciones cortadas mentirá.
    #[serde(
        rename = "LimiteDeInstrucciones",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    limite_de_instrucciones: Option<u64>,
}

const fn primera_instancia() -> u32 {
    1
}

impl PerfilDeAgente {
    /// Perfil sin carpeta ni ejecutable.
    #[must_use]
    pub const fn new(tipo: HerramientaDelPerfil, instancia: u32) -> Self {
        Self {
            tipo,
            instancia,
            directorio_configuracion: None,
            ejecutable: None,
            argumentos: Vec::new(),
            limite_de_instrucciones: None,
        }
    }

    /// El mismo perfil con la carpeta de configuración `carpeta`.
    #[must_use]
    pub fn con_directorio(self, carpeta: &str) -> Self {
        Self {
            directorio_configuracion: Some(carpeta.to_owned()),
            ..self
        }
    }

    /// El mismo perfil con el ejecutable `ruta` fijado, tal cual.
    #[must_use]
    pub fn con_ejecutable(self, ruta: &str) -> Self {
        Self {
            ejecutable: Some(ruta.to_owned()),
            ..self
        }
    }

    /// El mismo perfil con estos argumentos para el programa.
    #[must_use]
    pub fn con_argumentos<T: Into<String>>(self, argumentos: impl IntoIterator<Item = T>) -> Self {
        Self {
            argumentos: argumentos.into_iter().map(Into::into).collect(),
            ..self
        }
    }

    /// El mismo perfil con un límite de instrucciones propio de `bytes`.
    #[must_use]
    pub fn con_limite_de_instrucciones(self, bytes: u64) -> Self {
        Self {
            limite_de_instrucciones: Some(bytes),
            ..self
        }
    }

    /// Herramienta del perfil.
    #[must_use]
    pub const fn tipo(&self) -> HerramientaDelPerfil {
        self.tipo
    }

    /// Número de instancia; desde 1 en un perfil válido.
    #[must_use]
    pub const fn instancia(&self) -> u32 {
        self.instancia
    }

    /// Carpeta de configuración tal como está guardada, sin expandir «~».
    #[must_use]
    pub fn directorio_configuracion(&self) -> Option<&str> {
        self.directorio_configuracion.as_deref()
    }

    /// Ejecutable fijado tal como está guardado, sin expandir «~».
    #[must_use]
    pub fn ejecutable(&self) -> Option<&str> {
        self.ejecutable.as_deref()
    }

    /// Argumentos que se pasan al programa.
    #[must_use]
    pub fn argumentos(&self) -> &[String] {
        &self.argumentos
    }

    /// Quita el ejecutable fijado, para que se busque en sus ubicaciones por defecto.
    pub(super) fn quitar_ejecutable(&mut self) {
        self.ejecutable = None;
    }

    /// El mismo perfil con la carpeta de configuración de `otro` si este no la tiene.
    #[must_use]
    pub(super) fn con_directorio_de(self, otro: Self) -> Self {
        Self {
            directorio_configuracion: self
                .directorio_configuracion
                .or(otro.directorio_configuracion),
            ..self
        }
    }

    /// Fija la ruta del ejecutable que ha escrito el usuario, sin los espacios de los extremos.
    /// Una ruta vacía o en blanco deja el perfil sin ejecutable, para que se busque en sus
    /// ubicaciones por defecto.
    pub fn fijar_ejecutable(&mut self, ruta: &str) {
        let ruta = ruta.trim();
        self.ejecutable = (!ruta.is_empty()).then(|| ruta.to_owned());
    }

    /// Nombre de la identidad y valor de GESTOR_AGENTE, como «Claude 2» o «Programator 1».
    #[must_use]
    pub fn nombre(&self) -> String {
        format!("{} {}", self.tipo.nombre(), self.instancia)
    }

    /// Bytes del archivo de instrucciones que lee la herramienta: el del perfil o, si no lo
    /// lleva, el de su tipo. Nada si no se conoce límite.
    #[must_use]
    pub fn limite_de_instrucciones(&self) -> Option<u64> {
        self.limite_de_instrucciones
            .or_else(|| self.tipo.limite_de_instrucciones_por_defecto())
    }

    /// Carpeta de configuración con «~» sustituido por la carpeta del usuario; nada si está vacía.
    #[must_use]
    pub fn directorio_configuracion_expandido(&self, carpeta_usuario: &Path) -> Option<PathBuf> {
        expandir(self.directorio_configuracion.as_deref(), carpeta_usuario)
    }

    /// Ejecutable con «~» sustituido por la carpeta del usuario; nada si está vacío.
    #[must_use]
    pub fn ejecutable_expandido(&self, carpeta_usuario: &Path) -> Option<PathBuf> {
        expandir(self.ejecutable.as_deref(), carpeta_usuario)
    }

    /// Perfiles de una instalación nueva: los de la 2.3.2 más Programator, sin ejecutable hasta
    /// que el usuario lo indique.
    #[must_use]
    pub fn por_defecto() -> Vec<Self> {
        vec![
            Self::new(HerramientaDelPerfil::Claude, 1).con_directorio("~/.claude"),
            Self::new(HerramientaDelPerfil::Claude, 2).con_directorio("~/.claude2"),
            Self::new(HerramientaDelPerfil::Codex, 1),
            Self::new(HerramientaDelPerfil::Gemini, 1),
            Self::new(HerramientaDelPerfil::Programator, 1),
        ]
    }
}

/// Sustituye «~» o «~/» y «~\» iniciales por la carpeta del usuario, como la 2.3.2.
fn expandir(valor: Option<&str>, carpeta_usuario: &Path) -> Option<PathBuf> {
    let valor = valor.filter(|v| !v.chars().all(char::is_whitespace))?;
    if valor == "~" {
        return Some(carpeta_usuario.to_path_buf());
    }
    match valor
        .strip_prefix("~/")
        .or_else(|| valor.strip_prefix("~\\"))
    {
        Some(resto) => Some(carpeta_usuario.join(resto)),
        None => Some(PathBuf::from(valor)),
    }
}

/// Motivo por el que una lista de perfiles no es válida. El almacén le añade la ruta del archivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PerfilesInvalidos {
    /// Un perfil no es válido por sí solo; lleva el motivo.
    Perfil(String),
    /// Dos perfiles tienen la misma identidad; lleva su nombre.
    Repetido(String),
}

/// Comprueba una lista de perfiles: instancia desde 1, límite de instrucciones distinto de 0 e
/// identidades sin repetir. Se detiene en el primer fallo, en el orden de la lista.
pub(super) fn validar(perfiles: &[PerfilDeAgente]) -> Result<(), PerfilesInvalidos> {
    let mut vistas = HashSet::new();
    for perfil in perfiles {
        if perfil.instancia == 0 {
            return Err(PerfilesInvalidos::Perfil(textos::instancia_desde_uno(
                perfil.tipo.nombre(),
            )));
        }
        if perfil.limite_de_instrucciones == Some(0) {
            return Err(PerfilesInvalidos::Perfil(
                textos::limite_de_instrucciones_cero(&perfil.nombre()),
            ));
        }
        let nombre = perfil.nombre();
        if !vistas.insert(nombre.clone()) {
            return Err(PerfilesInvalidos::Repetido(nombre));
        }
    }
    Ok(())
}
