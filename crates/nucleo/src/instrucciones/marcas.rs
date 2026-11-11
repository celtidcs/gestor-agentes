//! Construcción y reconocimiento de las marcas que delimitan una región gestionada dentro de un
//! archivo de texto, con la forma «<!-- ID:INICIO -->» y «<!-- ID:FIN -->».
//!
//! Las marcas son comentarios HTML para que no se vean al mostrar el Markdown. Una marca solo
//! cuenta si ocupa una línea completa (espacios aparte): mencionarla dentro de una frase no abre
//! ni cierra nada. Así lo exige el núcleo común en su regla de calibrar las mediciones.
//!
//! El patrón de marca
//! se reconoce a mano, sin dependencia de expresiones regulares, con las mismas reglas: espacios
//! Unicode opcionales en los extremos y alrededor del identificador, un identificador de
//! mayúsculas ASCII y guiones con segmentos separados por dos puntos, y el tipo INICIO o FIN.

/// Región con la cabecera de identidad del archivo único de reglas (hasta la 3.0.0, la cabecera
/// propia de cada herramienta).
pub const CABECERA: &str = "GESTOR:CABECERA";

/// Región del núcleo común, con el identificador que ya usaban las versiones anteriores.
pub const NUCLEO_COMUN: &str = "NUCLEO-COMUN";

/// Región con el bloque de entorno local.
pub const ENTORNO: &str = "GESTOR:ENTORNO";

/// Región con el puntero de CLAUDE.md (hasta la 3.0.0, el aviso de compatibilidad de
/// memoria-compartida.md).
pub const PUNTERO: &str = "GESTOR:PUNTERO";

/// Bloque con la norma de la unidad de trabajo que se puede copiar a mano en los archivos de
/// instrucciones de los proyectos. No es una marca propia de la aplicación: se
/// reconoce solo para no tomar ese bloque por texto del proyecto al migrar, porque la orden ya
/// llega con las normas de la carpeta proyecto.
pub const UNIDAD_DE_TRABAJO: &str = "UNIDAD-DE-TRABAJO";

/// Bloque de texto antiguo conservado para que lo revisen los agentes. No es una región
/// gestionada: la aplicación lo crea una vez y después no vuelve a tocarlo.
pub const LEGADO: &str = "GESTOR:LEGADO";

/// Prefijo con el que se desactivan las marcas de un texto conservado como legado.
const PREFIJO_NEUTRALIZADO: &str = "legado ";

/// Identificadores de las marcas que crea y mantiene la aplicación. Un archivo que lleve alguna
/// es un archivo gestionado; las marcas con otros identificadores son de terceros (por ejemplo,
/// un bloque que el usuario delimita a mano) y no lo convierten en uno.
const IDENTIFICADORES_PROPIOS: [&str; 5] = [CABECERA, NUCLEO_COMUN, ENTORNO, PUNTERO, LEGADO];

/// Tipo de apertura en el texto de una marca.
const TIPO_INICIO: &str = "INICIO";

/// Tipo de cierre en el texto de una marca.
const TIPO_FIN: &str = "FIN";

/// Devuelve la marca de apertura de una región, por ejemplo «<!-- NUCLEO-COMUN:INICIO -->».
#[must_use]
pub fn inicio(id: &str) -> String {
    format!("<!-- {id}:{TIPO_INICIO} -->")
}

/// Devuelve la marca de cierre de una región.
#[must_use]
pub fn fin(id: &str) -> String {
    format!("<!-- {id}:{TIPO_FIN} -->")
}

/// Indica si una línea, sin espacios en los extremos, es exactamente la marca de apertura de la
/// región indicada.
#[must_use]
pub fn es_inicio(linea: &str, id: &str) -> bool {
    linea.trim() == inicio(id)
}

/// Indica si una línea, sin espacios en los extremos, es exactamente la marca de cierre de la
/// región indicada.
#[must_use]
pub fn es_fin(linea: &str, id: &str) -> bool {
    linea.trim() == fin(id)
}

/// Desactiva todas las marcas de región que aparezcan como línea completa en un texto con saltos
/// normalizados a \n, convirtiendo cada una en «<!-- legado ID:INICIO -->». Se usa al envolver
/// contenido antiguo como legado, para que sus marcas no se confundan con las regiones vivas.
#[must_use]
pub fn neutralizar(texto: &str) -> String {
    texto
        .split('\n')
        .map(|linea| match reconocer(linea) {
            Some((id, tipo)) => format!("<!-- {PREFIJO_NEUTRALIZADO}{id}:{tipo} -->"),
            None => linea.to_owned(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Indica si un texto contiene alguna marca de región, de cualquier identificador, como línea
/// completa.
#[must_use]
pub fn contiene_marcas(texto: &str) -> bool {
    texto.split('\n').any(|linea| reconocer(linea).is_some())
}

/// Indica si un texto contiene, como línea completa, alguna marca propia de la aplicación
/// (cabecera, núcleo común, entorno, puntero o legado). Sirve para no confundir los archivos de un
/// proyecto gestionado con las instrucciones del espacio de trabajo, que pueden delimitar bloques
/// con marcas de otro identificador.
#[must_use]
pub fn contiene_marcas_propias(texto: &str) -> bool {
    texto.split('\n').any(|linea| {
        IDENTIFICADORES_PROPIOS
            .iter()
            .any(|id| es_inicio(linea, id) || es_fin(linea, id))
    })
}

/// Reconoce una marca de cualquier identificador en una línea completa y devuelve su
/// identificador y su tipo (INICIO o FIN).
fn reconocer(linea: &str) -> Option<(&str, &str)> {
    let interior = linea
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    let (id, tipo) = interior.rsplit_once(':')?;
    let tipo_valido = tipo == TIPO_INICIO || tipo == TIPO_FIN;
    (tipo_valido && es_identificador_valido(id)).then_some((id, tipo))
}

/// Un identificador son uno o más segmentos separados por dos puntos; cada segmento empieza por
/// una mayúscula ASCII y sigue con mayúsculas ASCII o guiones.
fn es_identificador_valido(id: &str) -> bool {
    id.split(':').all(|segmento| {
        let mut caracteres = segmento.chars();
        caracteres
            .next()
            .is_some_and(|primero| primero.is_ascii_uppercase())
            && caracteres.all(|c| c.is_ascii_uppercase() || c == '-')
    })
}
