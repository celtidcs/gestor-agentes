//! Perfiles de agente vistos desde la interfaz: etiqueta del botón, identidad que se prepara,
//! petición que se pasa al lanzador y disponibilidad de cada botón.

use std::path::{Path, PathBuf};

use lanzador::{Herramienta, PeticionDeLanzamiento};
use nucleo::agentes::IdentidadAgente;
use nucleo::configuracion::{HerramientaDelPerfil, PerfilDeAgente};

use crate::textos;

/// Carpeta que se usa en lugar de la personal cuando no se conoce. Con ella, «~/.claude» queda
/// como «~/.claude» literal y el fallo se ve en lugar de acabar en una carpeta relativa.
const SIN_CARPETA_PERSONAL: &str = "~";

/// Primera instancia de Claude o Codex que necesita su propia carpeta de cuenta. La 1 no la
/// exige: recién instalada la herramienta, su carpeta por defecto aún no existe y se crea al
/// abrirla por primera vez.
const PRIMERA_INSTANCIA_CON_CUENTA_PROPIA: u32 = 2;

/// Si un perfil se puede abrir en este equipo, y si no, por qué.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disponibilidad {
    /// Se abre con ese ejecutable.
    Disponible(PathBuf),
    /// No se encuentra el programa: ni la ruta fijada en el perfil ni en su búsqueda automática.
    SinEjecutable,
    /// El programa está, pero falta la carpeta de la cuenta del perfil, como ~/.claude2.
    SinCuenta(PathBuf),
}

impl Disponibilidad {
    /// Indica si el botón del perfil se muestra.
    #[must_use]
    pub const fn disponible(&self) -> bool {
        matches!(self, Self::Disponible(_))
    }
}

/// Texto del botón de un perfil: «Abrir · Claude 2». Todos se abren en terminal, así que ya no
/// se dice dónde.
#[must_use]
pub fn etiqueta(perfil: &PerfilDeAgente) -> String {
    textos::etiqueta_de_perfil(&perfil.nombre())
}

/// Herramienta del lanzador que corresponde a la del perfil.
#[must_use]
pub const fn herramienta(herramienta: HerramientaDelPerfil) -> Herramienta {
    match herramienta {
        HerramientaDelPerfil::Claude => Herramienta::Claude,
        HerramientaDelPerfil::Codex => Herramienta::Codex,
        HerramientaDelPerfil::Gemini => Herramienta::Gemini,
        HerramientaDelPerfil::Programator => Herramienta::Programator,
    }
}

/// Identidad de agente que se prepara en el proyecto. Programator no es un agente con archivos
/// propios y no tiene; tampoco un perfil con instancia 0, que la configuración ya rechaza.
#[must_use]
pub fn identidad_a_preparar(perfil: &PerfilDeAgente) -> Option<IdentidadAgente> {
    let tipo = perfil.tipo().tipo_agente()?;
    IdentidadAgente::new(tipo, perfil.instancia()).ok()
}

/// Carpeta personal que se usa para sustituir «~», o «~» si no se conoce.
fn personal_o_tilde(carpeta_personal: Option<&Path>) -> PathBuf {
    carpeta_personal.map_or_else(|| PathBuf::from(SIN_CARPETA_PERSONAL), Path::to_path_buf)
}

/// Petición de lanzamiento del perfil en un proyecto, con «~» sustituida por la carpeta personal.
/// Lleva el mensaje inicial del núcleo, para que el agente ejecute el arranque de AGENTS.md al
/// abrirse.
#[must_use]
pub fn peticion(
    perfil: &PerfilDeAgente,
    nombre_proyecto: &str,
    carpeta_proyecto: &Path,
    carpeta_personal: Option<&Path>,
) -> PeticionDeLanzamiento {
    let personal = personal_o_tilde(carpeta_personal);
    PeticionDeLanzamiento {
        herramienta: herramienta(perfil.tipo()),
        identidad: perfil.nombre(),
        carpeta_proyecto: carpeta_proyecto.to_path_buf(),
        nombre_proyecto: nombre_proyecto.to_owned(),
        directorio_configuracion: perfil.directorio_configuracion_expandido(&personal),
        ejecutable: perfil.ejecutable_expandido(&personal),
        argumentos_extra: perfil.argumentos().to_vec(),
        mensaje_inicial: Some(nucleo::plantillas::mensaje_inicial()),
    }
}

/// Ruta fijada del ejecutable del perfil, con «~» sustituida; nada si se busca solo.
#[must_use]
pub fn ejecutable_fijado(
    perfil: &PerfilDeAgente,
    carpeta_personal: Option<&Path>,
) -> Option<PathBuf> {
    perfil.ejecutable_expandido(&personal_o_tilde(carpeta_personal))
}

/// Carpeta de cuenta que el perfil necesita que exista, con «~» sustituida: la de Claude o Codex
/// desde la instancia 2, si el perfil fija una. Nada si el perfil no la exige.
#[must_use]
pub fn carpeta_de_cuenta_exigida(
    perfil: &PerfilDeAgente,
    carpeta_personal: Option<&Path>,
) -> Option<PathBuf> {
    let tiene_cuentas = matches!(
        perfil.tipo(),
        HerramientaDelPerfil::Claude | HerramientaDelPerfil::Codex
    );
    if !tiene_cuentas || perfil.instancia() < PRIMERA_INSTANCIA_CON_CUENTA_PROPIA {
        return None;
    }
    perfil.directorio_configuracion_expandido(&personal_o_tilde(carpeta_personal))
}

/// Disponibilidad de un perfil a partir de lo que se ha encontrado: el ejecutable detectado (ver
/// `lanzador::detectar_ejecutable`) y, si el perfil exige carpeta de cuenta, esa carpeta y si
/// existe. Sin ejecutable no hay botón; con él, falta además la cuenta si su carpeta no existe.
#[must_use]
pub fn disponibilidad(
    ejecutable: Option<PathBuf>,
    cuenta_exigida: Option<(PathBuf, bool)>,
) -> Disponibilidad {
    let Some(ejecutable) = ejecutable else {
        return Disponibilidad::SinEjecutable;
    };
    match cuenta_exigida {
        Some((carpeta, false)) => Disponibilidad::SinCuenta(carpeta),
        _ => Disponibilidad::Disponible(ejecutable),
    }
}

/// Archivo de instrucciones del proyecto cuyo tamaño se vigila al abrir con el perfil: el de su
/// tipo de agente, solo si la herramienta tiene un límite conocido.
#[must_use]
pub fn archivo_vigilado(perfil: &PerfilDeAgente) -> Option<&'static str> {
    perfil.limite_de_instrucciones()?;
    Some(perfil.tipo().tipo_agente()?.archivo_instrucciones())
}

/// Aviso de instrucciones cortadas, si el archivo vigilado mide más de lo que lee la
/// herramienta. `tamano` es lo que mide ya preparado; nada si no se pudo medir.
#[must_use]
pub fn aviso_de_tamano(perfil: &PerfilDeAgente, tamano: Option<u64>) -> Option<String> {
    let limite = perfil.limite_de_instrucciones()?;
    let archivo = archivo_vigilado(perfil)?;
    let tamano = tamano?;
    (tamano > limite)
        .then(|| textos::instrucciones_cortadas(archivo, tamano, perfil.tipo().nombre(), limite))
}
