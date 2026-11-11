//! Apoyo común de las barreras del nucleo: carpeta temporal que se borra sola y dobles de las
//! dependencias del inicializador.
//!
//! Cada archivo de pruebas usa solo una parte de este módulo; lo que no usa no es código muerto
//! del producto, de ahí la excepción.
#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use nucleo::git::{EjecutorDeProcesos, ProgramaNoDisponible, ResultadoDeProceso, ServicioGit};
use nucleo::proyecto::archivos;
use nucleo::proyecto::informe::InformeDeInicializacion;
use nucleo::proyecto::inicializador::{ConfiguracionDeInstrucciones, InicializadorDeProyecto};
use nucleo::proyecto::localizador::{FuenteDeEntorno, InstruccionesDelEspacioDeTrabajo};
use nucleo::proyecto::momento::{Momento, RelojFijo};

/// Contador para que dos carpetas creadas en el mismo instante no coincidan.
static CONTADOR: AtomicU64 = AtomicU64::new(0);

/// Carpeta temporal propia de una prueba, directamente en la carpeta temporal del sistema. Se
/// borra al salir de ámbito. No se anida bajo una carpeta común: el Drop solo puede borrar la
/// suya, y una carpeta madre compartida quedaría siempre tras las pruebas.
pub struct CarpetaTemporal {
    ruta: PathBuf,
}

impl CarpetaTemporal {
    /// Crea una carpeta nueva y vacía.
    pub fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let numero = CONTADOR.fetch_add(1, Ordering::Relaxed);
        let ruta =
            std::env::temp_dir().join(format!("gestor-pruebas-{}-{nanos}-{numero}", process::id()));
        let creada = fs::create_dir_all(&ruta);
        assert!(
            creada.is_ok(),
            "no se pudo crear {}: {creada:?}",
            ruta.display()
        );
        Self { ruta }
    }

    /// Ruta de la carpeta.
    pub fn ruta(&self) -> &Path {
        &self.ruta
    }

    /// Ruta de un archivo dentro de la carpeta, con la ruta relativa en barras normales.
    pub fn archivo(&self, relativa: &str) -> PathBuf {
        archivos::ruta(&self.ruta, relativa)
    }

    /// Crea una subcarpeta y devuelve su ruta.
    pub fn crear_carpeta(&self, relativa: &str) -> PathBuf {
        let ruta = self.archivo(relativa);
        let creada = fs::create_dir_all(&ruta);
        assert!(creada.is_ok(), "{creada:?}");
        ruta
    }

    /// Escribe un archivo de texto tal cual.
    pub fn escribir(&self, relativa: &str, texto: &str) {
        self.escribir_bytes(relativa, texto.as_bytes());
    }

    /// Escribe un archivo con los bytes indicados, creando su carpeta.
    pub fn escribir_bytes(&self, relativa: &str, bytes: &[u8]) {
        let ruta = self.archivo(relativa);
        if let Some(carpeta) = ruta.parent() {
            let creada = fs::create_dir_all(carpeta);
            assert!(creada.is_ok(), "{creada:?}");
        }
        let escrito = fs::write(&ruta, bytes);
        assert!(escrito.is_ok(), "{escrito:?}");
    }

    /// Lee un archivo como texto; vacío si no existe.
    pub fn leer(&self, relativa: &str) -> String {
        fs::read_to_string(self.archivo(relativa)).unwrap_or_default()
    }

    /// Lee un archivo como bytes; vacío si no existe.
    pub fn leer_bytes(&self, relativa: &str) -> Vec<u8> {
        fs::read(self.archivo(relativa)).unwrap_or_default()
    }

    /// Indica si existe un archivo.
    pub fn existe(&self, relativa: &str) -> bool {
        self.archivo(relativa).is_file()
    }
}

impl Drop for CarpetaTemporal {
    fn drop(&mut self) {
        // Si falla el borrado, lo que queda está en la carpeta temporal del sistema, que se
        // limpia al terminar la serie de pruebas.
        let _ = fs::remove_dir_all(&self.ruta);
    }
}

/// Momento fijo de las pruebas, el mismo que usaba la 2.3.2: 2026-10-26 15:30:00.
pub fn momento_fijo() -> Momento {
    Momento::new(2026, 10, 26, 15, 30, 0).unwrap_or_else(|error| unreachable!("{error}"))
}

/// Reloj fijo de las pruebas.
pub fn reloj_fijo() -> RelojFijo {
    RelojFijo(momento_fijo())
}

/// Configuración con la plantilla antigua que se quiera.
#[derive(Default)]
pub struct ConfiguracionFalsa {
    pub instrucciones_antiguas: RefCell<Option<String>>,
}

impl ConfiguracionDeInstrucciones for ConfiguracionFalsa {
    fn leer_instrucciones_antiguas(&self) -> Option<String> {
        self.instrucciones_antiguas.borrow().clone()
    }
}

/// Fuente de entorno con un texto fijo y una ruta ficticia; sin texto, no hay entorno.
#[derive(Default)]
pub struct FuenteDeEntornoFalsa {
    pub texto: RefCell<String>,
}

/// Ruta ficticia de las instrucciones del espacio de trabajo falsas.
pub const RUTA_FICTICIA: &str = "/espacio/CLAUDE.md";

impl FuenteDeEntorno for FuenteDeEntornoFalsa {
    fn buscar(&self, _raiz_proyecto: &Path) -> Option<InstruccionesDelEspacioDeTrabajo> {
        let texto = self.texto.borrow().clone();
        (!texto.is_empty()).then(|| InstruccionesDelEspacioDeTrabajo {
            ruta: PathBuf::from(RUTA_FICTICIA),
            texto,
        })
    }
}

/// Git falso: responde a rev-parse según el estado y crea el repositorio con init.
pub struct EjecutorFalso {
    pub dentro_de_repositorio: Cell<bool>,
    pub git_instalado: Cell<bool>,
    pub llamadas: RefCell<Vec<Vec<String>>>,
}

impl Default for EjecutorFalso {
    fn default() -> Self {
        Self {
            dentro_de_repositorio: Cell::new(false),
            git_instalado: Cell::new(true),
            llamadas: RefCell::new(Vec::new()),
        }
    }
}

impl EjecutorDeProcesos for EjecutorFalso {
    fn ejecutar(
        &self,
        programa: &str,
        argumentos: &[OsString],
        _carpeta: &Path,
    ) -> Result<ResultadoDeProceso, ProgramaNoDisponible> {
        if !self.git_instalado.get() {
            return Err(ProgramaNoDisponible::NoSePudoArrancar {
                programa: programa.to_owned(),
                origen: std::io::Error::other("git no está instalado"),
            });
        }
        let lista: Vec<String> = argumentos
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let es_consulta = lista.iter().any(|a| a == "rev-parse");
        self.llamadas.borrow_mut().push(lista);
        if es_consulta {
            return Ok(if self.dentro_de_repositorio.get() {
                resultado(0, "true\n", "")
            } else {
                resultado(128, "", "fatal: not a git repository")
            });
        }
        self.dentro_de_repositorio.set(true);
        Ok(resultado(0, "Initialized empty Git repository", ""))
    }
}

fn resultado(codigo: i32, salida: &str, error: &str) -> ResultadoDeProceso {
    ResultadoDeProceso {
        codigo_de_salida: codigo,
        salida: salida.to_owned(),
        error: error.to_owned(),
    }
}

/// Dobles de un inicializador completo, para construirlo en cada prueba.
#[derive(Default)]
pub struct Entorno {
    pub configuracion: ConfiguracionFalsa,
    pub fuente: FuenteDeEntornoFalsa,
    pub git: EjecutorFalso,
}

impl Entorno {
    /// Inicializa la carpeta con los dobles, con la fuente de entorno indicada o la falsa.
    pub fn inicializar(
        &self,
        raiz: &Path,
        lanzada: Option<nucleo::agentes::IdentidadAgente>,
        fuente: Option<&dyn FuenteDeEntorno>,
    ) -> InformeDeInicializacion {
        let reloj = reloj_fijo();
        InicializadorDeProyecto::new(
            &self.configuracion,
            fuente.unwrap_or(&self.fuente),
            ServicioGit::new(&self.git),
            &reloj,
        )
        .inicializar(raiz, lanzada)
    }
}
