//! Herramientas de agente e identidad de cada instancia dentro de un proyecto.
//!
//! De la identidad se derivan, siguiendo el núcleo común de instrucciones, los archivos que
//! pertenecen en exclusiva a esa instancia: su bandeja en .gestor/canal/ y su carpeta de diario,
//! acta y registro de funciones. La primera instancia de cada herramienta no lleva número en esas
//! rutas (claude.md, .gestor/claude/) para conservar la compatibilidad con los proyectos ya
//! preparados; las siguientes sí (claude2.md, .gestor/claude2/).

use std::num::NonZeroU32;

/// Carpeta de gestión del proyecto, relativa a su raíz.
pub const CARPETA_GESTOR: &str = ".gestor";

/// Carpeta del canal de comunicación, relativa a la raíz del proyecto.
pub const CARPETA_CANAL: &str = ".gestor/canal";

/// Herramienta de agente para la que la aplicación mantiene un archivo de instrucciones canónico
/// en la raíz de cada proyecto. Programator no está aquí porque no lee archivos de instrucciones:
/// solo se lanza (ver el crate lanzador).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TipoAgente {
    /// Claude Code, de Anthropic. Lee CLAUDE.md, que le hace cargar AGENTS.md.
    Claude,
    /// Codex, de OpenAI. Lee AGENTS.md.
    Codex,
    /// Gemini en Google Antigravity, que lee AGENTS.md. Que Gemini CLI lo lea sin configurarlo no
    /// está comprobado.
    Gemini,
}

impl TipoAgente {
    /// Todas las herramientas, en el orden en que la 2.3.2 las recorría.
    pub const TODOS: [Self; 3] = [Self::Claude, Self::Codex, Self::Gemini];

    /// Nombre visible de la herramienta, como se escribe en la identidad.
    #[must_use]
    pub const fn nombre(self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini",
        }
    }

    /// Nombre en minúsculas que se usa en las rutas y en los nombres de plantilla.
    #[must_use]
    pub const fn identificador_base(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
        }
    }

    /// Archivo de reglas que lee la herramienta por su cuenta: AGENTS.md para Codex y Gemini, y
    /// CLAUDE.md para Claude, que es solo el puntero que le hace cargar AGENTS.md. Es el archivo
    /// cuyo tamaño importa cuando la herramienta corta lo que lee.
    #[must_use]
    pub const fn archivo_instrucciones(self) -> &'static str {
        match self {
            Self::Claude => "CLAUDE.md",
            Self::Codex | Self::Gemini => "AGENTS.md",
        }
    }
}

/// Identidad de una instancia de agente, como «Claude 2» o «Codex 3». La instancia empieza en 1,
/// y el tipo lo garantiza: no se puede construir una identidad con instancia 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdentidadAgente {
    tipo: TipoAgente,
    instancia: NonZeroU32,
}

/// Dato de identidad rechazado en la frontera.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErrorDeIdentidad {
    /// La instancia es 0; se cuenta desde 1.
    #[error("{}", crate::textos::instancia_desde_uno(tipo))]
    InstanciaCero {
        /// Herramienta a la que se quiso dar la instancia 0.
        tipo: &'static str,
    },
}

impl IdentidadAgente {
    /// Crea una identidad validando la instancia.
    ///
    /// # Errors
    ///
    /// Devuelve [`ErrorDeIdentidad::InstanciaCero`] si la instancia es 0.
    pub fn new(tipo: TipoAgente, instancia: u32) -> Result<Self, ErrorDeIdentidad> {
        NonZeroU32::new(instancia)
            .map(|instancia| Self { tipo, instancia })
            .ok_or(ErrorDeIdentidad::InstanciaCero {
                tipo: tipo.nombre(),
            })
    }

    /// Identidad de la primera instancia de una herramienta.
    #[must_use]
    pub const fn primera(tipo: TipoAgente) -> Self {
        Self {
            tipo,
            instancia: NonZeroU32::MIN,
        }
    }

    /// Herramienta del agente.
    #[must_use]
    pub const fn tipo(&self) -> TipoAgente {
        self.tipo
    }

    /// Número de instancia, empezando por 1.
    #[must_use]
    pub const fn instancia(&self) -> u32 {
        self.instancia.get()
    }

    /// Nombre visible y valor de GESTOR_AGENTE, por ejemplo «Claude 2».
    #[must_use]
    pub fn nombre(&self) -> String {
        format!("{} {}", self.tipo.nombre(), self.instancia)
    }

    /// Identificador en minúsculas usado en las rutas: «claude» para la primera instancia y
    /// «claude2» para la segunda.
    #[must_use]
    pub fn identificador(&self) -> String {
        let base = self.tipo.identificador_base();
        if self.instancia == NonZeroU32::MIN {
            base.to_owned()
        } else {
            format!("{base}{}", self.instancia)
        }
    }

    /// Ruta relativa de la bandeja de esta instancia, con barras normales.
    #[must_use]
    pub fn ruta_bandeja(&self) -> String {
        format!("{CARPETA_CANAL}/{}.md", self.identificador())
    }

    /// Ruta relativa de la carpeta de diario, acta y registro de funciones.
    #[must_use]
    pub fn ruta_carpeta_propia(&self) -> String {
        format!("{CARPETA_GESTOR}/{}", self.identificador())
    }

    /// Archivo de instrucciones que carga la herramienta de esta identidad.
    #[must_use]
    pub const fn archivo_instrucciones(&self) -> &'static str {
        self.tipo.archivo_instrucciones()
    }
}
