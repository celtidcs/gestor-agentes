//! Divide un archivo en texto libre y regiones gestionadas, reconociendo las marcas solo como
//! líneas completas.
//!
//! El análisis es estricto: una marca de apertura repetida, una región sin cerrar, un cierre sin
//! apertura o una marca dentro de otra región se tratan como error. Ante cualquier duda sobre la
//! forma del archivo es preferible no escribir nada y avisar que arriesgarse a borrar texto.

use std::collections::HashSet;

use super::marcas;

/// Contenido de una región gestionada encontrada en el archivo, sin sus marcas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionEncontrada {
    /// Identificador de la región.
    pub id: String,
    /// Líneas entre la marca de inicio y la de fin, sin saltos.
    pub lineas: Vec<String>,
}

impl RegionEncontrada {
    /// Contenido de la región como un único texto con saltos \n.
    #[must_use]
    pub fn contenido(&self) -> String {
        self.lineas.join("\n")
    }
}

/// Trozo de un archivo analizado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segmento {
    /// Texto fuera de cualquier región gestionada. La aplicación nunca lo modifica.
    TextoLibre(Vec<String>),
    /// Contenido de una región gestionada.
    Region(RegionEncontrada),
}

/// Problema de marcas que impide modificar un archivo. El texto de cada variante es el mismo que
/// mostraba la 2.3.2 en el informe.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErrorDeMarcas {
    /// Una marca de otra región gestionada aparece dentro de una región abierta.
    #[error("línea {linea}: marca de {intrusa} dentro de la región {abierta}")]
    MarcaIntrusa {
        /// Número de línea, contando desde 1.
        linea: usize,
        /// Región de la marca intrusa.
        intrusa: String,
        /// Región que estaba abierta.
        abierta: String,
    },
    /// Una región se abre por segunda vez.
    #[error("línea {linea}: la región {id} aparece dos veces")]
    RegionRepetida {
        /// Número de línea, contando desde 1.
        linea: usize,
        /// Región repetida.
        id: String,
    },
    /// Una marca de cierre aparece sin su apertura.
    #[error("línea {linea}: cierre de la región {id} sin su apertura")]
    CierreSinApertura {
        /// Número de línea, contando desde 1.
        linea: usize,
        /// Región del cierre.
        id: String,
    },
    /// El archivo termina con una región abierta.
    #[error("la región {id} no tiene marca de cierre")]
    SinCierre {
        /// Región sin cerrar.
        id: String,
    },
}

/// Analiza un texto con saltos normalizados a \n.
///
/// `ids_gestionados` son los identificadores de las regiones que se reconocen; las marcas de
/// cualquier otro identificador son texto libre. Si el análisis es válido, los segmentos
/// reconstruyen el archivo exactamente con [`componer`].
///
/// # Errors
///
/// Devuelve [`ErrorDeMarcas`] si las marcas no forman regiones bien delimitadas.
pub fn analizar(texto: &str, ids_gestionados: &[&str]) -> Result<Vec<Segmento>, ErrorDeMarcas> {
    let mut segmentos = Vec::new();
    let mut texto_libre: Vec<String> = Vec::new();
    let mut contenido_region: Vec<String> = Vec::new();
    let mut regiones_vistas: HashSet<&str> = HashSet::new();
    let mut region_abierta: Option<&str> = None;

    for (indice, linea) in texto.split('\n').enumerate() {
        let numero_linea = indice + 1;

        if let Some(abierta) = region_abierta {
            if marcas::es_fin(linea, abierta) {
                segmentos.push(Segmento::Region(RegionEncontrada {
                    id: abierta.to_owned(),
                    lineas: std::mem::take(&mut contenido_region),
                }));
                region_abierta = None;
                continue;
            }
            if let Some(intrusa) = buscar_marca(linea, ids_gestionados) {
                return Err(ErrorDeMarcas::MarcaIntrusa {
                    linea: numero_linea,
                    intrusa: intrusa.to_owned(),
                    abierta: abierta.to_owned(),
                });
            }
            contenido_region.push(linea.to_owned());
            continue;
        }

        if let Some(apertura) = ids_gestionados
            .iter()
            .copied()
            .find(|id| marcas::es_inicio(linea, id))
        {
            if !regiones_vistas.insert(apertura) {
                return Err(ErrorDeMarcas::RegionRepetida {
                    linea: numero_linea,
                    id: apertura.to_owned(),
                });
            }
            volcar_texto_libre(&mut segmentos, &mut texto_libre);
            region_abierta = Some(apertura);
            continue;
        }

        if let Some(cierre) = ids_gestionados
            .iter()
            .copied()
            .find(|id| marcas::es_fin(linea, id))
        {
            return Err(ErrorDeMarcas::CierreSinApertura {
                linea: numero_linea,
                id: cierre.to_owned(),
            });
        }

        texto_libre.push(linea.to_owned());
    }

    if let Some(abierta) = region_abierta {
        return Err(ErrorDeMarcas::SinCierre {
            id: abierta.to_owned(),
        });
    }

    volcar_texto_libre(&mut segmentos, &mut texto_libre);
    Ok(segmentos)
}

/// Indica si los segmentos contienen al menos una región gestionada.
#[must_use]
pub fn tiene_regiones(segmentos: &[Segmento]) -> bool {
    segmentos.iter().any(|s| matches!(s, Segmento::Region(_)))
}

/// Reconstruye el texto de un archivo a partir de sus segmentos, con las marcas de cada región y
/// saltos \n.
#[must_use]
pub fn componer(segmentos: &[Segmento]) -> String {
    let mut lineas: Vec<String> = Vec::new();
    for segmento in segmentos {
        match segmento {
            Segmento::Region(region) => {
                lineas.push(marcas::inicio(&region.id));
                lineas.extend(region.lineas.iter().cloned());
                lineas.push(marcas::fin(&region.id));
            }
            Segmento::TextoLibre(texto) => lineas.extend(texto.iter().cloned()),
        }
    }
    lineas.join("\n")
}

/// Primera región gestionada cuya marca, de apertura o de cierre, es la línea.
fn buscar_marca<'a>(linea: &str, ids: &[&'a str]) -> Option<&'a str> {
    ids.iter()
        .copied()
        .find(|id| marcas::es_inicio(linea, id) || marcas::es_fin(linea, id))
}

/// Pasa el texto libre acumulado a un segmento, si lo hay.
fn volcar_texto_libre(segmentos: &mut Vec<Segmento>, texto_libre: &mut Vec<String>) {
    if texto_libre.is_empty() {
        return;
    }
    segmentos.push(Segmento::TextoLibre(std::mem::take(texto_libre)));
}
