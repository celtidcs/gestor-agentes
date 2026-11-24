//! Aviso de versión nueva al arrancar, y última versión publicada de Programator.
//!
//! Es el único módulo de Gestor Agentes que habla con la biblioteca común aviso-version-github,
//! como pide su norma de integración (bibliotecas/aviso-version-github/documentacion), y sigue
//! su integración de referencia. Cada comprobación se hace una vez, al arrancar, en un hilo con
//! nombre, y el resultado llega por un canal que AvisosAlArrancar pasa a la aplicación sin esperar. Si está
//! desactivada no se fabrica ninguna consulta. Todo fallo se deja en el registro de la aplicación,
//! la salida de errores: ni se calla ni se enseña al usuario, que simplemente no recibe aviso.
//!
//! Los ajustes de la biblioteca salen de las claves de ajustes.json que la aplicación usa desde
//! la 3.x (ComprobarActualizaciones, RepositorioDeActualizaciones, RepositorioDeProgramator y
//! TiempoMaximoActualizacionSegundos), para no romper la configuración de nadie.

use std::fmt;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use std::thread;

use aviso_version_github::{
    comprobar, proxy_del_sistema, ultima_version, ureq, ConsultaDeVersion, ConsultaHttps,
    FalloDeComprobacion, OrigenDeLasVersiones, ProgramaQueConsulta,
};
use nucleo::actualizaciones::{self, AvisoDeVersionNueva, Version};
use nucleo::ajustes::{self, Ajustes};
use nucleo::fecha;

use crate::aplicacion::Aplicacion;
use crate::textos;

/// Nombre con que Gestor Agentes se presenta en la cabecera User-Agent.
pub const NOMBRE_DEL_PROGRAMA: &str = "gestor-agentes";

/// Repositorio donde se publican las versiones de Gestor Agentes: el valor de fábrica de
/// RepositorioDeActualizaciones en ajustes.json.
pub const REPOSITORIO_DE_VERSIONES_POR_DEFECTO: &str =
    ajustes::REPOSITORIO_DE_ACTUALIZACIONES_POR_DEFECTO;

/// Versión de esta compilación, la de Cargo.toml.
pub const VERSION_INSTALADA: &str = env!("CARGO_PKG_VERSION");

/// Nombre del hilo que comprueba la versión de Gestor Agentes.
const NOMBRE_DEL_HILO: &str = "aviso-de-version";

/// Nombre del hilo que pide la última versión de Programator.
const NOMBRE_DEL_HILO_DE_PROGRAMATOR: &str = "aviso-de-version-programator";

/// Dónde se dejan los fallos. El real escribe en la salida de errores; las pruebas los guardan.
pub type Registro = Arc<dyn Fn(&dyn fmt::Display) + Send + Sync>;

/// Lo que se llama al entregar un resultado, para que la ventana se repinte.
pub type Avisar = Box<dyn Fn() + Send>;

/// La línea que deja en el registro un fallo: el momento, el comienzo común de todas y el
/// `fallo`, que ya dice qué comprobación falló y con qué repositorio.
#[must_use]
pub fn linea_de_registro(momento: &str, fallo: &dyn fmt::Display) -> String {
    format!(
        "{momento} {} {fallo}",
        textos::REGISTRO_DEL_AVISO_DE_VERSION
    )
}

/// El registro de la aplicación: la salida de errores, como el resto de sus avisos técnicos,
/// con la fecha y la hora locales de cada fallo.
#[must_use]
pub fn registro_real() -> Registro {
    Arc::new(|fallo: &dyn fmt::Display| {
        eprintln!(
            "{}",
            linea_de_registro(&fecha::iso(&chrono::Local::now()), fallo)
        );
    })
}

/// La consulta HTTPS que usa `proxy`, o ninguno, y se presenta como Gestor Agentes en la
/// versión `version`. Si esa identificación no es válida, se registra y la consulta sale solo
/// con el nombre de la biblioteca.
#[must_use]
pub fn consulta_https(
    origen: &OrigenDeLasVersiones,
    proxy: Option<ureq::Proxy>,
    version: &str,
    registrar: &Registro,
) -> ConsultaHttps {
    match ProgramaQueConsulta::nuevo(NOMBRE_DEL_PROGRAMA, version) {
        Ok(programa) => ConsultaHttps::para_programa(origen, proxy, &programa),
        Err(fallo) => {
            registrar(&format_args!(
                "{}: {fallo}",
                textos::IDENTIFICACION_NO_VALIDA
            ));
            ConsultaHttps::nueva(origen, proxy)
        }
    }
}

/// La consulta real: HTTPS, con el proxy del sistema y Gestor Agentes identificado con la
/// versión de esta compilación.
#[must_use]
pub fn consulta_real(origen: &OrigenDeLasVersiones) -> Box<dyn ConsultaDeVersion> {
    Box::new(consulta_https(
        origen,
        proxy_del_sistema(),
        VERSION_INSTALADA,
        &registro_real(),
    ))
}

/// Qué se comprueba: de qué programa y en qué repositorio. Encabeza cada fallo registrado.
struct Comprobacion {
    programa: &'static str,
    repositorio: String,
}

impl fmt::Display for Comprobacion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&textos::que_se_comprueba(self.programa, &self.repositorio))
    }
}

/// Origen de las versiones de la `comprobacion` con el tiempo de espera de los ajustes, o nada
/// si está desactivada o la biblioteca no da por buenos los valores, fallo que se registra.
fn origen(
    ajustes: &Ajustes,
    comprobacion: &Comprobacion,
    registrar: &Registro,
) -> Option<OrigenDeLasVersiones> {
    if !ajustes.comprobar_actualizaciones() {
        return None;
    }
    let de_la_biblioteca = actualizaciones::ajustes_de_version(
        &comprobacion.repositorio,
        ajustes.tiempo_maximo_actualizacion().as_secs(),
    );
    match OrigenDeLasVersiones::desde_ajustes(&de_la_biblioteca) {
        Ok(origen) => Some(origen),
        Err(fallo) => {
            registrar(&format_args!("{comprobacion}: {fallo}"));
            None
        }
    }
}

/// Comprueba en un hilo propio si hay una versión de Gestor Agentes más nueva que
/// `version_local` y devuelve por dónde llegará el aviso. La consulta la fabrica `fabricar`: la
/// real es [`consulta_real`]. Si está desactivada, no se llama a `fabricar` y el canal queda
/// cerrado.
pub fn arrancar(
    ajustes: &Ajustes,
    version_local: impl Into<String>,
    fabricar: impl FnOnce(&OrigenDeLasVersiones) -> Box<dyn ConsultaDeVersion>,
    registrar: Registro,
    avisar: Avisar,
) -> Receiver<AvisoDeVersionNueva> {
    let comprobacion = Comprobacion {
        programa: textos::TITULO,
        repositorio: ajustes.repositorio_de_actualizaciones().to_owned(),
    };
    let Some(origen) = origen(ajustes, &comprobacion, &registrar) else {
        return cerrado();
    };
    let consulta = fabricar(&origen);
    let version_local = version_local.into();
    lanzar(
        NOMBRE_DEL_HILO,
        comprobacion,
        registrar,
        avisar,
        move || comprobar(&version_local, &origen, consulta.as_ref()),
    )
}

/// Como [`arrancar`], pero entrega la última versión publicada de Programator, sea cual sea,
/// para compararla con la del Programator de cada perfil.
pub fn arrancar_programator(
    ajustes: &Ajustes,
    fabricar: impl FnOnce(&OrigenDeLasVersiones) -> Box<dyn ConsultaDeVersion>,
    registrar: Registro,
    avisar: Avisar,
) -> Receiver<Version> {
    let comprobacion = Comprobacion {
        programa: textos::PROGRAMATOR,
        repositorio: ajustes.repositorio_de_programator().to_owned(),
    };
    let Some(origen) = origen(ajustes, &comprobacion, &registrar) else {
        return cerrado();
    };
    let consulta = fabricar(&origen);
    lanzar(
        NOMBRE_DEL_HILO_DE_PROGRAMATOR,
        comprobacion,
        registrar,
        avisar,
        move || ultima_version(&origen, consulta.as_ref()).map(Some),
    )
}

/// Un canal por el que nunca llegará nada.
fn cerrado<T>() -> Receiver<T> {
    mpsc::channel().1
}

/// Hace `pedir` en un hilo llamado `nombre`, entrega lo que dé y registra su fallo, encabezado
/// por la `comprobacion`. Si el hilo no se puede crear, se registra y el canal queda cerrado.
fn lanzar<T: Send + 'static>(
    nombre: &str,
    comprobacion: Comprobacion,
    registrar: Registro,
    avisar: Avisar,
    pedir: impl FnOnce() -> Result<Option<T>, FalloDeComprobacion> + Send + 'static,
) -> Receiver<T> {
    let (salida, entrada) = mpsc::channel();
    let registrar_en_el_hilo = Arc::clone(&registrar);
    let contexto = comprobacion.to_string();
    let hilo = thread::Builder::new()
        .name(nombre.to_owned())
        .spawn(move || match pedir() {
            // Si la ventana ya se cerró, nadie espera el resultado: no es un fallo.
            Ok(Some(resultado)) => {
                if salida.send(resultado).is_ok() {
                    avisar();
                }
            }
            Ok(None) => {}
            Err(fallo) => registrar_en_el_hilo(&format_args!("{contexto}: {fallo}")),
        });
    if let Err(error) = hilo {
        registrar(&format_args!("{comprobacion}: {error}"));
    }
    entrada
}

/// Lo que llega de las dos comprobaciones del arranque, con la versión instalada con que se
/// comparó, para que el aviso diga esa misma. La ventana lo guarda y en cada fotograma llama a
/// [`AvisosAlArrancar::entregar`].
pub struct AvisosAlArrancar {
    instalada: String,
    de_version: Option<Receiver<AvisoDeVersionNueva>>,
    de_programator: Option<Receiver<Version>>,
}

impl AvisosAlArrancar {
    /// Sin comprobaciones: nunca llega nada.
    #[must_use]
    pub const fn ninguno() -> Self {
        Self {
            instalada: String::new(),
            de_version: None,
            de_programator: None,
        }
    }

    /// Lo que llegue por estos canales; `instalada` es la versión con que se comparó el aviso.
    #[must_use]
    pub fn desde_canales(
        instalada: impl Into<String>,
        de_version: Receiver<AvisoDeVersionNueva>,
        de_programator: Receiver<Version>,
    ) -> Self {
        Self {
            instalada: instalada.into(),
            de_version: Some(de_version),
            de_programator: Some(de_programator),
        }
    }

    /// Arranca las dos comprobaciones, la de Gestor Agentes contra `instalada`, cada una con la
    /// consulta que dé `fabricar` y su propio `avisar`.
    pub fn arrancar(
        ajustes: &Ajustes,
        instalada: &str,
        fabricar: impl Fn(&OrigenDeLasVersiones) -> Box<dyn ConsultaDeVersion>,
        registrar: &Registro,
        avisar: impl Fn() -> Avisar,
    ) -> Self {
        let de_version = arrancar(
            ajustes,
            instalada,
            &fabricar,
            Arc::clone(registrar),
            avisar(),
        );
        let de_programator =
            arrancar_programator(ajustes, &fabricar, Arc::clone(registrar), avisar());
        Self::desde_canales(instalada, de_version, de_programator)
    }

    /// Pasa a la `aplicacion` lo que haya llegado: el aviso de versión nueva, como diálogo con
    /// la versión instalada con que se comparó, y la última versión de Programator. Cada una
    /// llega una sola vez; después, o si su comprobación terminó sin nada, se deja de mirar.
    pub fn entregar(&mut self, aplicacion: &mut Aplicacion) {
        if let Some(aviso) = recoger_una_vez(&mut self.de_version) {
            aplicacion.informar(
                textos::TITULO_ACTUALIZACION,
                textos::actualizacion_disponible(&aviso.version, &self.instalada, &aviso.enlace),
            );
        }
        if let Some(version) = recoger_una_vez(&mut self.de_programator) {
            aplicacion.fijar_programator_publicado(version);
        }
    }

    /// Si todavía se espera algo de alguna de las dos comprobaciones.
    #[must_use]
    pub const fn esperando(&self) -> bool {
        self.de_version.is_some() || self.de_programator.is_some()
    }
}

/// Lo que haya llegado por un canal que solo trae un valor. Al recibirlo, o si el canal se cerró
/// sin nada, se deja de mirar.
fn recoger_una_vez<T>(canal: &mut Option<Receiver<T>>) -> Option<T> {
    let recibido = match canal.as_ref()?.try_recv() {
        Ok(valor) => Some(valor),
        Err(TryRecvError::Empty) => return None,
        Err(TryRecvError::Disconnected) => None,
    };
    *canal = None;
    recibido
}
