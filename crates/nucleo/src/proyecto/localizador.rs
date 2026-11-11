//! Localiza las instrucciones del espacio de trabajo de un proyecto: el CLAUDE.md, AGENTS.md o
//! GEMINI.md más cercano en las carpetas superiores que no sea un archivo gestionado por la
//! aplicación. Su texto se copia en la región de entorno de AGENTS.md.
//!
//! Solo se descartan los archivos con marcas propias de la aplicación; un bloque con marcas de otro
//! identificador, como el que el usuario delimita a mano, no impide usar el archivo (corrección
//! de la 2.3.2).

use std::path::{Path, PathBuf};

use super::archivos;
use crate::instrucciones::marcas;

/// Nombres que se buscan en cada carpeta superior, en este orden.
const NOMBRES_CANDIDATOS: [&str; 3] = ["CLAUDE.md", "AGENTS.md", "GEMINI.md"];

/// Instrucciones del espacio de trabajo encontradas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstruccionesDelEspacioDeTrabajo {
    /// Ruta absoluta del archivo de origen.
    pub ruta: PathBuf,
    /// Texto normalizado con \n.
    pub texto: String,
}

/// Origen de las instrucciones del espacio de trabajo. Se inyecta para probar el inicializador
/// sin depender de las carpetas superiores reales.
pub trait FuenteDeEntorno {
    /// Busca las instrucciones del espacio de trabajo del proyecto, si las hay.
    fn buscar(&self, raiz_proyecto: &Path) -> Option<InstruccionesDelEspacioDeTrabajo>;
}

/// Fuente real: recorre las carpetas superiores del proyecto en el disco.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalizadorDeInstrucciones;

impl FuenteDeEntorno for LocalizadorDeInstrucciones {
    fn buscar(&self, raiz_proyecto: &Path) -> Option<InstruccionesDelEspacioDeTrabajo> {
        let proyecto = std::path::absolute(raiz_proyecto).ok()?;
        proyecto.ancestors().skip(1).find_map(|carpeta| {
            NOMBRES_CANDIDATOS
                .iter()
                .find_map(|nombre| leer_si_es_del_espacio_de_trabajo(&carpeta.join(nombre)))
        })
    }
}

/// Lee un candidato y lo acepta si existe, tiene texto y no lleva marcas propias de la
/// aplicación. Un archivo que no se puede leer se trata como si no existiera.
fn leer_si_es_del_espacio_de_trabajo(ruta: &Path) -> Option<InstruccionesDelEspacioDeTrabajo> {
    let (texto, _) = archivos::leer(ruta).ok().flatten()?;
    let utilizable =
        !texto.chars().all(char::is_whitespace) && !marcas::contiene_marcas_propias(&texto);
    utilizable.then(|| InstruccionesDelEspacioDeTrabajo {
        ruta: ruta.to_path_buf(),
        texto,
    })
}
