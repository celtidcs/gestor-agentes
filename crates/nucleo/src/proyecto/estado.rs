//! Estado de las instrucciones de un proyecto para la lista de proyectos: la lectura de sus
//! archivos, en [`ArchivosDeInstrucciones::leer`], y la decisión, sin disco, en
//! [`EstadoDelProyecto::segun`].

use std::fs;
use std::path::Path;

use super::{archivos, catalogo};
use crate::instrucciones::marcas;
use crate::textos;

/// Estado de las instrucciones de un proyecto, para la lista de proyectos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoDelProyecto {
    /// No hay ningún archivo de instrucciones.
    SinConfigurar,
    /// AGENTS.md tiene el núcleo común y el proyecto tiene su archivo propio.
    Configurado,
    /// Hay instrucciones, pero de antes del archivo único: se pondrán al día al lanzar un agente.
    FormatoAntiguo,
}

/// Lo que se sabe de los archivos de instrucciones de un proyecto para calcular su estado. Es
/// una foto de lo leído, sin invariante que guardar, y por eso sus campos son públicos.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArchivosDeInstrucciones {
    /// Texto de AGENTS.md, si existe.
    pub reglas: Option<String>,
    /// Si existe algún otro archivo de instrucciones (CLAUDE.md o GEMINI.md).
    pub hay_otros: bool,
    /// Si existe .gestor/instrucciones-del-proyecto.md.
    pub hay_archivo_propio: bool,
}

impl ArchivosDeInstrucciones {
    /// Lee del disco lo necesario de un proyecto. Lo que no se puede leer cuenta como ausente.
    #[must_use]
    pub fn leer(carpeta_proyecto: &Path) -> Self {
        let existe = |relativa: &str| archivos::ruta(carpeta_proyecto, relativa).is_file();
        Self {
            reglas: fs::read(archivos::ruta(carpeta_proyecto, catalogo::RUTA_REGLAS))
                .ok()
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned()),
            hay_otros: existe(catalogo::RUTA_PUNTERO_CLAUDE)
                || existe(catalogo::RUTA_GEMINI_ANTERIOR),
            hay_archivo_propio: existe(catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO),
        }
    }
}

impl EstadoDelProyecto {
    /// Calcula el estado leyendo la carpeta del proyecto.
    #[must_use]
    pub fn de(carpeta_proyecto: &Path) -> Self {
        Self::segun(&ArchivosDeInstrucciones::leer(carpeta_proyecto))
    }

    /// Calcula el estado a partir de lo leído, sin tocar el disco: configurado si AGENTS.md tiene
    /// la marca de inicio del núcleo común en una línea propia (se admiten espacios alrededor) y
    /// además existe el archivo propio del proyecto; sin configurar si no hay ningún archivo de
    /// instrucciones; formato antiguo en cualquier otro caso.
    #[must_use]
    pub fn segun(archivos: &ArchivosDeInstrucciones) -> Self {
        let Some(reglas) = archivos.reglas.as_deref() else {
            return if archivos.hay_otros {
                Self::FormatoAntiguo
            } else {
                Self::SinConfigurar
            };
        };
        let marca = marcas::inicio(marcas::NUCLEO_COMUN);
        // Sin BOM y con \r, \n o \r\n como separador de línea.
        let reglas = reglas.trim_start_matches('\u{FEFF}');
        let tiene_nucleo = reglas
            .split(['\r', '\n'])
            .any(|linea| linea.trim() == marca);
        if tiene_nucleo && archivos.hay_archivo_propio {
            Self::Configurado
        } else {
            Self::FormatoAntiguo
        }
    }

    /// Texto que muestra la lista de proyectos.
    #[must_use]
    pub const fn texto(self) -> &'static str {
        match self {
            Self::SinConfigurar => textos::ESTADO_SIN_CONFIGURAR,
            Self::Configurado => textos::ESTADO_CONFIGURADO,
            Self::FormatoAntiguo => textos::ESTADO_FORMATO_ANTIGUO,
        }
    }
}
