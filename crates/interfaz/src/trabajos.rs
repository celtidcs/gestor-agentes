//! Trabajos que no pueden correr en el hilo de la ventana: elegir carpeta o ejecutable, preparar y
//! abrir un proyecto, borrar su carpeta y detectar los agentes del equipo.
//!
//! [`ejecutar`] decide qué se hace con cada trabajo usando un [`Motor`] inyectado, así que se
//! prueba sin disco ni procesos. [`Trabajador`] solo lo lleva a un hilo aparte.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

use lanzador::{ErrorDeLanzamiento, Herramienta, PeticionDeLanzamiento};
use nucleo::actualizaciones::Version;
use nucleo::agentes::IdentidadAgente;
use nucleo::configuracion::{HerramientaDelPerfil, PerfilDeAgente};
use nucleo::cuentas::{self, AltaRechazada, CarpetaPersonal};
use nucleo::proyecto::informe::{InformeDeInicializacion, TipoDeCambio};
use nucleo::proyecto::papeles::Papel;

use crate::perfiles::{self, CarpetaDelAgente, Disponibilidad};
use crate::textos;

/// Nombre del hilo de trabajo, para los volcados y los depuradores.
const NOMBRE_DEL_HILO: &str = "gestor-agentes-trabajos";

/// Estado del canal, donde se anota el papel elegido.
const RUTA_DEL_ESTADO: &str = ".gestor/canal/estado.md";

/// Trabajo que la ventana encarga.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trabajo {
    /// Abrir el selector de carpetas del sistema.
    ElegirCarpeta,
    /// Preparar el proyecto y abrir en él el agente del perfil.
    Abrir {
        /// Nombre visible del proyecto.
        nombre: String,
        /// Carpeta del proyecto.
        ruta: String,
        /// Perfil que se abre.
        perfil: PerfilDeAgente,
        /// Papel que el Director ha elegido en la ventana, que se anota en estado.md. Nada si no ha
        /// elegido: el agente lo pregunta al arrancar.
        papel: Option<Papel>,
    },
    /// Borrar la carpeta del proyecto con todo su contenido.
    Borrar {
        /// Nombre visible del proyecto.
        nombre: String,
        /// Carpeta que se borra.
        ruta: String,
    },
    /// Añadir a los perfiles las cuentas instaladas que falten y ver cuáles se pueden abrir.
    Detectar {
        /// Perfiles guardados.
        perfiles: Vec<PerfilDeAgente>,
    },
    /// Abrir el selector de archivos para la ruta del ejecutable de un perfil.
    ElegirEjecutable {
        /// Posición del perfil.
        indice: usize,
    },
    /// Elegir la carpeta de una cuenta, darla de alta y volver a buscar los agentes.
    DarDeAltaCuenta {
        /// Herramienta de la cuenta (Claude o Codex).
        herramienta: HerramientaDelPerfil,
        /// Perfiles guardados.
        perfiles: Vec<PerfilDeAgente>,
    },
}

/// Cómo terminó el lanzamiento.
#[derive(Debug)]
pub enum Lanzamiento {
    /// El agente se ha abierto.
    Abierto,
    /// No se ha podido abrir.
    Fallido(ErrorDeLanzamiento),
}

/// Resultado de un trabajo, que vuelve al hilo de la ventana.
#[derive(Debug)]
pub enum ResultadoDeTrabajo {
    /// Carpeta elegida, o nada si el usuario canceló.
    CarpetaElegida(Option<PathBuf>),
    /// Proyecto preparado e intento de abrir el agente.
    Abierto {
        /// Nombre visible del proyecto.
        nombre: String,
        /// Carpeta del proyecto.
        ruta: String,
        /// Perfil que se abrió. Va en una caja para que esta variante no haga grande todo el
        /// resultado (clippy::large_enum_variant).
        perfil: Box<PerfilDeAgente>,
        /// Lo que hizo la preparación.
        informe: InformeDeInicializacion,
        /// Bytes que mide, ya preparado, el archivo de instrucciones que vigila el perfil (ver
        /// [`perfiles::archivo_vigilado`]); nada si no se vigila o no se pudo medir.
        tamano_instrucciones: Option<u64>,
        /// Cómo terminó el lanzamiento.
        lanzamiento: Lanzamiento,
    },
    /// Intento de borrar la carpeta del proyecto.
    Borrado {
        /// Nombre visible del proyecto.
        nombre: String,
        /// Carpeta que se intentó borrar.
        ruta: String,
        /// Error del sistema, si lo hubo.
        error: Option<String>,
    },
    /// Agentes detectados.
    Detectado {
        /// Los perfiles guardados más las cuentas nuevas, en su orden.
        perfiles: Vec<PerfilDeAgente>,
        /// Disponibilidad de cada perfil, en el mismo orden.
        disponibilidad: Vec<Disponibilidad>,
        /// Carpeta donde vive cada perfil y si existe, en el mismo orden; nada si no se sabe.
        carpetas: Vec<Option<CarpetaDelAgente>>,
        /// Cuántos perfiles son nuevos y hay que guardar.
        anadidos: usize,
    },
    /// La cuenta no se ha dado de alta: nada si el usuario canceló, o el motivo del rechazo.
    CuentaNoAnadida(Option<AltaRechazada>),
    /// Ejecutable elegido para un perfil, o nada si el usuario canceló.
    EjecutableElegido {
        /// Posición del perfil.
        indice: usize,
        /// Archivo elegido.
        ruta: Option<PathBuf>,
    },
}

/// Efectos sobre el sistema que necesitan los trabajos. Se inyecta para probar [`ejecutar`].
pub trait Motor {
    /// Muestra el selector de carpetas y devuelve la elegida.
    fn elegir_carpeta(&self) -> Option<PathBuf>;
    /// Muestra el selector de archivos para un ejecutable y devuelve el elegido.
    fn elegir_ejecutable(&self) -> Option<PathBuf>;
    /// Muestra el selector de carpetas para la carpeta de una cuenta y devuelve la elegida.
    fn elegir_carpeta_de_cuenta(&self) -> Option<PathBuf>;
    /// Versión que declara el ejecutable en su recurso de versión, si la lleva (ver
    /// [`nucleo::version_de_programa::version_en_recurso`]).
    fn version_del_ejecutable(&self, ruta: &Path) -> Option<Version>;
    /// Prepara el proyecto para la identidad indicada.
    fn preparar(&self, raiz: &Path, identidad: Option<IdentidadAgente>) -> InformeDeInicializacion;

    /// Anota en estado.md del proyecto el papel elegido para esa identidad. Devuelve si ha cambiado
    /// algo.
    ///
    /// # Errors
    ///
    /// El error de disco al leer o escribir estado.md.
    fn anotar_papel(&self, raiz: &Path, identidad: &str, papel: Papel) -> std::io::Result<bool>;
    /// Carpeta personal del usuario, para sustituir «~» en los perfiles.
    fn carpeta_personal(&self) -> Option<PathBuf>;
    /// Decide cómo se abre la petición.
    ///
    /// # Errors
    ///
    /// Los de [`lanzador::planificar`].
    fn planificar(
        &self,
        peticion: &PeticionDeLanzamiento,
    ) -> Result<lanzador::PlanDeLanzamiento, ErrorDeLanzamiento>;
    /// Ejecutable de la herramienta: la ruta fijada si existe, o el que encuentre su búsqueda
    /// automática si no hay ruta fijada. Es [`lanzador::detectar_ejecutable`], la misma búsqueda
    /// que usa [`Self::planificar`].
    fn detectar_ejecutable(
        &self,
        herramienta: Herramienta,
        ruta_fijada: Option<&Path>,
    ) -> Option<PathBuf>;
    /// Carpetas de la carpeta personal que pueden ser de una cuenta (ver
    /// [`nucleo::cuentas::explorar`]).
    fn explorar_cuentas(&self, carpeta_personal: &Path) -> Vec<CarpetaPersonal>;
    /// Indica si existe la carpeta.
    fn existe_carpeta(&self, ruta: &Path) -> bool;
    /// Abre el plan.
    ///
    /// # Errors
    ///
    /// Los de [`lanzador::ejecutar`].
    fn ejecutar(&self, plan: &lanzador::PlanDeLanzamiento) -> Result<(), ErrorDeLanzamiento>;
    /// Borra una carpeta con todo su contenido.
    ///
    /// # Errors
    ///
    /// El error del sistema.
    fn borrar_carpeta(&self, ruta: &Path) -> std::io::Result<()>;
    /// Bytes que mide un archivo; nada si no existe o no se puede leer.
    fn tamano(&self, ruta: &Path) -> Option<u64>;
}

/// Hace un trabajo. El proyecto se prepara siempre antes de abrirlo, y se abre aunque la
/// preparación haya tenido errores, como en la 2.3.2: el informe los cuenta después.
#[must_use]
pub fn ejecutar(trabajo: Trabajo, motor: &dyn Motor) -> ResultadoDeTrabajo {
    match trabajo {
        Trabajo::ElegirCarpeta => ResultadoDeTrabajo::CarpetaElegida(motor.elegir_carpeta()),
        Trabajo::Abrir {
            nombre,
            ruta,
            perfil,
            papel,
        } => {
            let carpeta = PathBuf::from(&ruta);
            let mut informe = motor.preparar(&carpeta, perfiles::identidad_a_preparar(&perfil));
            let tamano_instrucciones = perfiles::archivo_vigilado(&perfil)
                .and_then(|archivo| motor.tamano(&carpeta.join(archivo)));
            let personal = motor.carpeta_personal();
            let peticion = perfiles::peticion(&perfil, &nombre, &carpeta, personal.as_deref());
            if let Some(papel) = papel {
                match motor.anotar_papel(&carpeta, &peticion.identidad, papel) {
                    Ok(true) => informe.anotar_cambio(
                        RUTA_DEL_ESTADO,
                        TipoDeCambio::Actualizado,
                        Some(textos::papel_anotado(&peticion.identidad, papel)),
                    ),
                    Ok(false) => {}
                    Err(error) => {
                        informe.anotar_error(format!("{RUTA_DEL_ESTADO}: {error}"));
                    }
                }
            }
            let lanzamiento = lanzar(&peticion, motor);
            ResultadoDeTrabajo::Abierto {
                nombre,
                ruta,
                perfil: Box::new(perfil),
                informe,
                tamano_instrucciones,
                lanzamiento,
            }
        }
        Trabajo::Borrar { nombre, ruta } => {
            let error = motor
                .borrar_carpeta(Path::new(&ruta))
                .err()
                .map(|error| error.to_string());
            ResultadoDeTrabajo::Borrado {
                nombre,
                ruta,
                error,
            }
        }
        Trabajo::Detectar { perfiles } => detectar(perfiles, motor),
        Trabajo::ElegirEjecutable { indice } => ResultadoDeTrabajo::EjecutableElegido {
            indice,
            ruta: motor.elegir_ejecutable(),
        },
        Trabajo::DarDeAltaCuenta {
            herramienta,
            perfiles,
        } => dar_de_alta_cuenta(herramienta, perfiles, motor),
    }
}

/// Planifica y abre.
fn lanzar(peticion: &PeticionDeLanzamiento, motor: &dyn Motor) -> Lanzamiento {
    match motor
        .planificar(peticion)
        .and_then(|plan| motor.ejecutar(&plan))
    {
        Ok(()) => Lanzamiento::Abierto,
        Err(error) => Lanzamiento::Fallido(error),
    }
}

/// Añade las cuentas instaladas que falten (sin carpeta personal no se buscan) y calcula la
/// disponibilidad de cada perfil.
fn detectar(mut lista: Vec<PerfilDeAgente>, motor: &dyn Motor) -> ResultadoDeTrabajo {
    let personal = motor.carpeta_personal();
    let anadidos = personal.as_deref().map_or(0, |carpeta| {
        cuentas::incorporar(
            &mut lista,
            cuentas::perfiles_de_cuentas(&motor.explorar_cuentas(carpeta)),
        )
    });
    let (disponibilidad, carpetas) = lista
        .iter()
        .map(|perfil| detectar_perfil(perfil, personal.as_deref(), motor))
        .unzip();
    ResultadoDeTrabajo::Detectado {
        perfiles: lista,
        disponibilidad,
        carpetas,
        anadidos,
    }
}

/// Pide la carpeta de la cuenta, la da de alta (ver [`cuentas::dar_de_alta`]) y vuelve a buscar
/// los agentes, contando el perfil nuevo entre los que hay que guardar.
fn dar_de_alta_cuenta(
    herramienta: HerramientaDelPerfil,
    mut lista: Vec<PerfilDeAgente>,
    motor: &dyn Motor,
) -> ResultadoDeTrabajo {
    let Some(carpeta) = motor.elegir_carpeta_de_cuenta() else {
        return ResultadoDeTrabajo::CuentaNoAnadida(None);
    };
    let personal = motor.carpeta_personal().unwrap_or_default();
    if let Err(rechazo) = cuentas::dar_de_alta(&mut lista, herramienta, &carpeta, &personal) {
        return ResultadoDeTrabajo::CuentaNoAnadida(Some(rechazo));
    }
    match detectar(lista, motor) {
        ResultadoDeTrabajo::Detectado {
            perfiles,
            disponibilidad,
            carpetas,
            anadidos,
        } => ResultadoDeTrabajo::Detectado {
            perfiles,
            disponibilidad,
            carpetas,
            anadidos: anadidos + 1,
        },
        otro => otro,
    }
}

/// Disponibilidad de un perfil y carpeta donde vive. La carpeta se mira una sola vez en el disco
/// y sirve también de cuenta exigida cuando el perfil la exige.
fn detectar_perfil(
    perfil: &PerfilDeAgente,
    personal: Option<&Path>,
    motor: &dyn Motor,
) -> (Disponibilidad, Option<CarpetaDelAgente>) {
    let fijado = perfiles::ejecutable_fijado(perfil, personal);
    let ejecutable =
        motor.detectar_ejecutable(perfiles::herramienta(perfil.tipo()), fijado.as_deref());
    let version = match (&ejecutable, perfil.tipo()) {
        (Some(programa), HerramientaDelPerfil::Programator) => {
            motor.version_del_ejecutable(programa)
        }
        _ => None,
    };
    let carpeta = perfiles::carpeta_del_agente(perfil, personal).map(|ruta| {
        let existe = motor.existe_carpeta(&ruta);
        CarpetaDelAgente::new(ruta, existe).con_version(version)
    });
    let cuenta = perfiles::carpeta_de_cuenta_exigida(perfil, personal).map(|exigida| {
        let existe = carpeta
            .as_ref()
            .filter(|vista| vista.ruta() == exigida)
            .map_or_else(|| motor.existe_carpeta(&exigida), CarpetaDelAgente::existe);
        (exigida, existe)
    });
    (perfiles::disponibilidad(ejecutable, cuenta), carpeta)
}

/// El hilo de trabajo ya no existe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("el hilo de trabajo se ha detenido")]
pub struct TrabajadorPerdido;

/// Hilo de trabajo que hace los trabajos de uno en uno y avisa a la ventana al terminar cada uno.
pub struct Trabajador {
    encargos: Sender<Trabajo>,
    resultados: Receiver<ResultadoDeTrabajo>,
}

impl Trabajador {
    /// Arranca el hilo. `avisar` se llama tras cada resultado, para que la ventana se repinte y
    /// lo recoja.
    ///
    /// # Errors
    ///
    /// El error del sistema si no puede crear el hilo.
    pub fn arrancar(
        motor: Box<dyn Motor + Send>,
        avisar: Box<dyn Fn() + Send>,
    ) -> std::io::Result<Self> {
        let (encargos, entrada) = mpsc::channel::<Trabajo>();
        let (salida, resultados) = mpsc::channel();
        thread::Builder::new()
            .name(NOMBRE_DEL_HILO.to_owned())
            .spawn(move || {
                // El bucle acaba cuando la ventana suelta su extremo del canal.
                for trabajo in entrada {
                    if salida.send(ejecutar(trabajo, motor.as_ref())).is_err() {
                        break;
                    }
                    avisar();
                }
            })?;
        Ok(Self {
            encargos,
            resultados,
        })
    }

    /// Encarga un trabajo.
    ///
    /// # Errors
    ///
    /// [`TrabajadorPerdido`] si el hilo ya no existe.
    pub fn encargar(&self, trabajo: Trabajo) -> Result<(), TrabajadorPerdido> {
        self.encargos.send(trabajo).map_err(|_| TrabajadorPerdido)
    }

    /// Recoge un resultado terminado, sin esperar.
    ///
    /// # Errors
    ///
    /// [`TrabajadorPerdido`] si el hilo ya no existe y no queda nada que recoger.
    pub fn recoger(&self) -> Result<Option<ResultadoDeTrabajo>, TrabajadorPerdido> {
        match self.resultados.try_recv() {
            Ok(resultado) => Ok(Some(resultado)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(TrabajadorPerdido),
        }
    }
}
