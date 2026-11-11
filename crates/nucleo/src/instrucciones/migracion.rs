//! Paso de un proyecto al archivo único de instrucciones: recoge el texto propio del proyecto que
//! haya en sus archivos de instrucciones anteriores (CLAUDE.md, AGENTS.md y GEMINI.md) y compone
//! con él el archivo propio del proyecto, .gestor/instrucciones-del-proyecto.md.
//!
//! Es lógica pura: recibe los textos ya leídos y devuelve el texto nuevo, sin tocar el disco. La
//! regla que la gobierna es no perder nada del proyecto. Se descarta solo lo que se reconoce con
//! seguridad como escrito por la aplicación (sus regiones, sus plantillas completas y los textos
//! que dejaba fuera de las regiones) y el bloque de la unidad de trabajo, que ya llega con las
//! normas de la carpeta proyecto. Todo lo demás pasa, sin párrafos repetidos, a un bloque pendiente
//! de consolidar que revisa el primer agente que arranca. Si un archivo tiene las marcas dañadas,
//! su texto pasa entero.

use std::collections::HashSet;

use crate::instrucciones::analizador::{self, Segmento};
use crate::instrucciones::{formato, marcas};

/// Regiones que se reconocen en un archivo de instrucciones anterior. Todas se descartan salvo
/// el legado, cuyo contenido es texto del proyecto pendiente de revisar.
const REGIONES_RECONOCIDAS: [&str; 6] = [
    marcas::CABECERA,
    marcas::NUCLEO_COMUN,
    marcas::ENTORNO,
    marcas::PUNTERO,
    marcas::UNIDAD_DE_TRABAJO,
    marcas::LEGADO,
];

/// Apertura de un marcador de plantilla, como {{FECHA}}.
const APERTURA_DE_MARCADOR: &str = "{{";

/// Un archivo de instrucciones anterior, ya leído y con los saltos normalizados a \n.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchivoAnterior<'a> {
    /// Ruta relativa al proyecto, por ejemplo «GEMINI.md».
    pub ruta_relativa: &'a str,
    /// Texto completo del archivo.
    pub texto: &'a str,
}

/// Lo que la aplicación escribió en versiones anteriores y no es del proyecto.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextosDeLaAplicacion {
    /// Plantillas completas de archivo, normalizadas con
    /// [`formato::normalizar_para_comparar`]. Un archivo igual a una de ellas no aporta nada.
    pub plantillas_completas: Vec<String>,
    /// Textos que la aplicación dejaba fuera de sus regiones. Cada uno de sus párrafos se
    /// reconoce y se descarta; si un párrafo lleva un marcador {{...}}, basta con que coincida
    /// lo que hay antes del marcador.
    pub textos_sueltos: Vec<String>,
}

/// Texto propio del proyecto recogido de sus archivos anteriores.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextoPropio {
    /// Párrafos en el orden en que aparecen, sin repetidos y con las marcas de región
    /// desactivadas para que no rompan el bloque que los contiene.
    pub parrafos: Vec<String>,
    /// Archivos cuyas marcas estaban dañadas; su texto se ha recogido entero.
    pub archivos_con_marcas_danadas: Vec<String>,
}

impl TextoPropio {
    /// Indica si no se ha recogido nada.
    #[must_use]
    pub fn esta_vacio(&self) -> bool {
        self.parrafos.is_empty()
    }
}

/// Archivo del sistema anterior que la aplicación ha llevado al archivo de .gestor/.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archivado {
    /// Ruta relativa que tenía.
    pub origen: String,
    /// Ruta relativa donde está ahora.
    pub destino: String,
}

/// Recoge el texto propio del proyecto de sus archivos de instrucciones anteriores, en el orden
/// recibido.
#[must_use]
pub fn recoger(archivos: &[ArchivoAnterior<'_>], conocidos: &TextosDeLaAplicacion) -> TextoPropio {
    let parrafos_de_la_aplicacion = parrafos_reconocibles(conocidos);
    let mut vistos: HashSet<String> = HashSet::new();
    let mut propio = TextoPropio::default();

    for archivo in archivos {
        let texto = formato::normalizar(archivo.texto);
        if es_plantilla_completa(&texto, conocidos) {
            continue;
        }
        let lineas = match analizador::analizar(&texto, &REGIONES_RECONOCIDAS) {
            Ok(segmentos) => lineas_del_proyecto(segmentos),
            Err(_) => {
                propio
                    .archivos_con_marcas_danadas
                    .push(archivo.ruta_relativa.to_owned());
                texto.split('\n').map(str::to_owned).collect()
            }
        };
        for parrafo in en_parrafos(&lineas) {
            let clave = formato::normalizar_para_comparar(&parrafo);
            if es_de_la_aplicacion(&clave, &parrafos_de_la_aplicacion) || !vistos.insert(clave) {
                continue;
            }
            propio.parrafos.push(marcas::neutralizar(&parrafo));
        }
    }
    propio
}

/// Compone el archivo propio del proyecto: el texto inicial y, si hay algo que revisar, el bloque
/// pendiente de consolidar con los párrafos recogidos y la lista de archivos archivados.
///
/// `encabezado` es el encabezado del bloque, ya con su fecha. `aviso_de_archivados` es la frase
/// que presenta la lista de archivos archivados. Devuelve el texto con salto de línea final.
#[must_use]
pub fn componer_instrucciones_del_proyecto(
    inicial: &str,
    encabezado: &str,
    aviso_de_archivados: &str,
    propio: &TextoPropio,
    archivados: &[Archivado],
) -> String {
    let inicial = inicial.trim_end_matches('\n');
    if propio.esta_vacio() && archivados.is_empty() {
        return format!("{inicial}\n");
    }

    let mut partes: Vec<String> = vec![encabezado.trim_end_matches('\n').to_owned()];
    partes.extend(propio.parrafos.iter().cloned());
    if !archivados.is_empty() {
        let lista: Vec<String> = archivados
            .iter()
            .map(|a| format!("- {}, ahora en {}", a.origen, a.destino))
            .collect();
        partes.push(format!("{aviso_de_archivados}\n\n{}", lista.join("\n")));
    }
    format!(
        "{inicial}\n\n{}\n{}\n{}\n",
        marcas::inicio(marcas::LEGADO),
        partes.join("\n\n"),
        marcas::fin(marcas::LEGADO)
    )
}

/// Indica si el archivo entero es una plantilla antigua de la aplicación o está vacío.
fn es_plantilla_completa(texto: &str, conocidos: &TextosDeLaAplicacion) -> bool {
    let normalizado = formato::normalizar_para_comparar(texto);
    normalizado.is_empty() || conocidos.plantillas_completas.contains(&normalizado)
}

/// Líneas que pertenecen al proyecto: el texto libre y el contenido de los bloques de legado.
fn lineas_del_proyecto(segmentos: Vec<Segmento>) -> Vec<String> {
    segmentos
        .into_iter()
        .flat_map(|segmento| match segmento {
            Segmento::TextoLibre(lineas) => lineas,
            Segmento::Region(region) if region.id == marcas::LEGADO => {
                let mut lineas = region.lineas;
                lineas.push(String::new());
                lineas
            }
            Segmento::Region(_) => vec![String::new()],
        })
        .collect()
}

/// Agrupa líneas en párrafos separados por líneas en blanco, sin espacios finales.
fn en_parrafos(lineas: &[String]) -> Vec<String> {
    lineas
        .split(|linea| linea.trim().is_empty())
        .filter(|grupo| !grupo.is_empty())
        .map(|grupo| {
            grupo
                .iter()
                .map(|linea| linea.trim_end())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect()
}

/// Párrafos de los textos sueltos de la aplicación, normalizados para comparar.
fn parrafos_reconocibles(conocidos: &TextosDeLaAplicacion) -> Vec<String> {
    conocidos
        .textos_sueltos
        .iter()
        .flat_map(|texto| {
            let lineas: Vec<String> = formato::normalizar(texto)
                .split('\n')
                .map(str::to_owned)
                .collect();
            en_parrafos(&lineas)
        })
        .map(|parrafo| formato::normalizar_para_comparar(&parrafo))
        .filter(|parrafo| !parrafo.is_empty())
        .collect()
}

/// Indica si un párrafo, normalizado para comparar, es uno de los de la aplicación. Si el de la
/// aplicación lleva un marcador, basta con que coincida todo lo que hay antes del marcador.
fn es_de_la_aplicacion(parrafo: &str, de_la_aplicacion: &[String]) -> bool {
    de_la_aplicacion
        .iter()
        .any(|conocido| match conocido.split_once(APERTURA_DE_MARCADOR) {
            Some((prefijo, _)) if !prefijo.trim().is_empty() => parrafo.starts_with(prefijo),
            _ => parrafo == conocido,
        })
}
