//! Aprendizaje común entre proyectos (órdenes del Director del 2026-11-18).
//!
//! Las soluciones comprobadas en un proyecto se guardan como fichas en aprendizaje-comun/, en la
//! raíz del repositorio de Gestor Agentes, y viajan dentro del ejecutable. Al preparar cada
//! proyecto, la aplicación escribe .gestor/consulta/aprendizaje-comun.md con las fichas
//! comprobadas. Y recoge, en su carpeta de datos, las reglas aprendidas que un proyecto marca como
//! candidatas a común, para que el Director decida cuáles pasan a ficha.

use std::fmt::Write as _;

/// Ruta del documento de consulta, relativa al proyecto.
pub const RUTA_DEL_DOCUMENTO: &str = ".gestor/consulta/aprendizaje-comun.md";

/// Archivo de la carpeta de datos de la aplicación donde se juntan las candidatas.
pub const ARCHIVO_DE_CANDIDATAS: &str = "candidatas-comunes.md";

/// Marca con que termina una regla aprendida que su proyecto propone como común.
pub const MARCA_DE_CANDIDATA: &str = "(candidata a común)";

/// Apartado de las reglas aprendidas en las instrucciones del proyecto.
const APARTADO_DE_REGLAS: &str = "## Reglas aprendidas";

/// Fichas, por nombre de archivo. Una prueba comprueba que están todas las de la carpeta.
pub const FICHAS: [(&str, &str); 5] = [
    (
        "acuse-recibido.md",
        include_str!("../../../aprendizaje-comun/acuse-recibido.md"),
    ),
    (
        "codex-queue.md",
        include_str!("../../../aprendizaje-comun/codex-queue.md"),
    ),
    (
        "para-un-destinatario.md",
        include_str!("../../../aprendizaje-comun/para-un-destinatario.md"),
    ),
    (
        "rustdoc-avisos.md",
        include_str!("../../../aprendizaje-comun/rustdoc-avisos.md"),
    ),
    (
        "sed-y-crlf.md",
        include_str!("../../../aprendizaje-comun/sed-y-crlf.md"),
    ),
];

/// Campos que lleva toda ficha, en este orden, tras el título.
pub const CAMPOS: [&str; 7] = [
    "Estado",
    "Problema",
    "Cómo se reconoce",
    "Solución",
    "Evidencia",
    "Herramientas y versiones",
    "Aplica a",
];

/// Estados de una ficha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoDeFicha {
    /// Sin evidencia todavía.
    Propuesta,
    /// Comprobada en un proyecto.
    Comprobada,
    /// Comprobada en varios proyectos.
    ComprobadaEnVarios,
    /// Ya no se aplica.
    Retirada,
}

impl EstadoDeFicha {
    /// Lee el estado tal como se escribe en la ficha.
    #[must_use]
    pub fn leer(texto: &str) -> Option<Self> {
        match texto.trim() {
            "propuesta" => Some(Self::Propuesta),
            "comprobada" => Some(Self::Comprobada),
            "comprobada en varios" => Some(Self::ComprobadaEnVarios),
            "retirada" => Some(Self::Retirada),
            _ => None,
        }
    }

    /// Si la ficha va al documento de consulta de los proyectos.
    #[must_use]
    pub const fn se_publica(self) -> bool {
        matches!(self, Self::Comprobada | Self::ComprobadaEnVarios)
    }
}

/// Ficha leída.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ficha {
    /// Título, sin la almohadilla.
    pub titulo: String,
    /// Estado.
    pub estado: EstadoDeFicha,
    /// Texto completo, sin espacios al final.
    pub texto: String,
}

/// Lee una ficha: título en la primera línea con «# » y una línea «Campo: valor» por cada campo de
/// [`CAMPOS`], en su orden. Nada si no tiene esa forma.
#[must_use]
pub fn leer_ficha(texto: &str) -> Option<Ficha> {
    let mut lineas = texto.lines().filter(|linea| !linea.trim().is_empty());
    let titulo = lineas.next()?.strip_prefix("# ")?.trim().to_owned();
    let mut estado = None;
    for campo in CAMPOS {
        let linea = lineas.next()?;
        let valor = linea.strip_prefix(campo)?.strip_prefix(':')?.trim();
        if valor.is_empty() {
            return None;
        }
        if campo == CAMPOS[0] {
            estado = EstadoDeFicha::leer(valor);
        }
    }
    Some(Ficha {
        titulo,
        estado: estado?,
        texto: texto.trim_end().to_owned(),
    })
}

/// Texto de .gestor/consulta/aprendizaje-comun.md: una cabecera y las fichas que se publican, en
/// orden de archivo.
#[must_use]
pub fn documento() -> String {
    let mut texto = String::from(
        "# Aprendizaje común\n\n\
         Soluciones comprobadas en otros proyectos del equipo. Gestor Agentes escribe este archivo \
         entero en cada lanzamiento: no lo edites. Una solución nueva se propone como regla \
         aprendida del proyecto que termina en «(candidata a común)».\n",
    );
    for ficha in FICHAS
        .iter()
        .filter_map(|(_, contenido)| leer_ficha(contenido))
        .filter(|ficha| ficha.estado.se_publica())
    {
        let _ = write!(texto, "\n{}\n", ficha.texto.replacen("# ", "## ", 1));
    }
    texto
}

/// Reglas aprendidas que el texto de instrucciones del proyecto marca como candidatas a común, sin
/// la viñeta ni la marca.
#[must_use]
pub fn candidatas(instrucciones: &str) -> Vec<String> {
    canal::apartados::lineas(instrucciones, APARTADO_DE_REGLAS)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|l| l.strip_suffix(MARCA_DE_CANDIDATA))
        .map(|l| {
            l.trim_start_matches('-')
                .trim()
                .trim_end_matches('.')
                .trim()
        })
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Añade a `actual` (el texto de candidatas-comunes.md, si existe) las candidatas que falten, con
/// su proyecto y la fecha. Nada si no falta ninguna. Una regla ya anotada no se repite aunque
/// venga de otro proyecto. Los paréntesis del nombre del proyecto se escriben como corchetes.
#[must_use]
pub fn anadir_candidatas(
    actual: Option<&str>,
    candidatas: &[String],
    proyecto: &str,
    fecha: &str,
) -> Option<String> {
    let base = actual.unwrap_or("");
    let mut anotadas: Vec<&str> = base.lines().filter_map(regla_anotada).collect();
    let mut faltan: Vec<&String> = Vec::new();
    for regla in candidatas {
        if !anotadas.contains(&regla.as_str()) {
            anotadas.push(regla);
            faltan.push(regla);
        }
    }
    if faltan.is_empty() {
        return None;
    }
    let mut texto = if base.trim().is_empty() {
        String::from(
            "# Candidatas a aprendizaje común\n\n\
             Reglas aprendidas que los proyectos proponen como comunes. El Director decide cuáles \
             pasan a ficha en aprendizaje-comun/.\n",
        )
    } else {
        base.trim_end_matches('\n').to_owned() + "\n"
    };
    let proyecto = proyecto_sin_separador(proyecto);
    for regla in faltan {
        let _ = writeln!(texto, "- {regla}{SEPARADOR_DEL_ORIGEN}{proyecto}, {fecha})");
    }
    Some(texto)
}

/// El nombre del proyecto con sus paréntesis cambiados por corchetes. Así el separador del origen
/// no puede aparecer en él y el último de la línea es siempre el que separa la regla.
fn proyecto_sin_separador(proyecto: &str) -> String {
    proyecto.replace('(', "[").replace(')', "]")
}

/// Viñeta de cada candidata anotada.
const VINETA: &str = "- ";

/// Lo que separa la regla de su proyecto y fecha en candidatas-comunes.md: «- regla. (proyecto,
/// fecha)». La regla puede llevar paréntesis propios, así que vale el último separador.
const SEPARADOR_DEL_ORIGEN: &str = ". (";

/// La regla entera de una línea de candidatas-comunes.md, sin la viñeta ni el origen.
fn regla_anotada(linea: &str) -> Option<&str> {
    linea
        .strip_prefix(VINETA)?
        .rsplit_once(SEPARADOR_DEL_ORIGEN)
        .map(|(regla, _)| regla)
}
