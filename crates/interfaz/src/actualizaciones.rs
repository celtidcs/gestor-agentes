//! Aviso de versión nueva al arrancar. Una sola consulta, en un hilo propio para no retrasar la
//! ventana ni ocupar el hilo de trabajos: si GitHub no responde a tiempo, la red falla o la
//! respuesta no se entiende, no se avisa de nada. Las decisiones son de
//! [`nucleo::actualizaciones`]; aquí solo se compone la consulta y se entrega el resultado.

use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use nucleo::actualizaciones::{self, Publicacion};
use nucleo::ajustes::Ajustes;

/// Nombre del hilo de la consulta, para reconocerlo en un depurador.
const NOMBRE_DEL_HILO: &str = "gestor-agentes-version";

/// Versión de esta compilación, la de Cargo.toml.
pub const VERSION_INSTALADA: &str = env!("CARGO_PKG_VERSION");

/// Hace la petición HTTP. Se inyecta para probar la consulta sin red.
pub trait ConsultorDePublicaciones {
    /// Pide `direccion` con la cabecera User-Agent `agente` y espera como mucho `tiempo`.
    /// Devuelve el cuerpo de una respuesta correcta, o nada ante cualquier fallo.
    fn consultar(&self, direccion: &str, agente: &str, tiempo: Duration) -> Option<String>;
}

/// Lo que necesita la consulta, copiado de los ajustes para llevarlo al hilo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consulta {
    repositorio: String,
    tiempo: Duration,
}

impl Consulta {
    /// La consulta que piden los ajustes, o nada si están desactivadas.
    #[must_use]
    pub fn desde(ajustes: &Ajustes) -> Option<Self> {
        ajustes.comprobar_actualizaciones().then(|| Self {
            repositorio: ajustes.repositorio_de_actualizaciones().to_owned(),
            tiempo: ajustes.tiempo_maximo_actualizacion(),
        })
    }
}

/// Consulta la última versión publicada y devuelve la publicación si es más nueva que
/// `version_instalada`.
#[must_use]
pub fn comprobar(
    consulta: &Consulta,
    version_instalada: &str,
    consultor: &dyn ConsultorDePublicaciones,
) -> Option<Publicacion> {
    let direccion = actualizaciones::direccion_de_la_consulta(&consulta.repositorio)?;
    let agente = actualizaciones::agente_de_usuario(version_instalada);
    let cuerpo = consultor.consultar(&direccion, &agente, consulta.tiempo)?;
    actualizaciones::novedad(
        version_instalada,
        actualizaciones::leer_publicacion(&cuerpo, &consulta.repositorio),
    )
}

/// Lanza la consulta en un hilo propio, si los ajustes la piden, y devuelve por dónde llegará la
/// publicación nueva. `avisar` se llama al dejarla, para que la ventana se repinte. Si no hay
/// consulta, o el hilo no se puede crear, el canal queda cerrado y nunca llega nada.
#[must_use]
pub fn en_segundo_plano(
    consulta: Option<Consulta>,
    consultor: Box<dyn ConsultorDePublicaciones + Send>,
    avisar: Box<dyn Fn() + Send>,
) -> Receiver<Publicacion> {
    let (salida, entrada) = mpsc::channel();
    if let Some(consulta) = consulta {
        // Un hilo que no se puede crear solo cuesta el aviso: la aplicación sigue igual.
        let _ = thread::Builder::new()
            .name(NOMBRE_DEL_HILO.to_owned())
            .spawn(move || {
                if let Some(publicacion) =
                    comprobar(&consulta, VERSION_INSTALADA, consultor.as_ref())
                {
                    if salida.send(publicacion).is_ok() {
                        avisar();
                    }
                }
            });
    }
    entrada
}
