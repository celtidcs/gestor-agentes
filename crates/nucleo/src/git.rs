//! Servicio de git del proyecto: asegura que el proyecto está bajo git y que .gestor/ está en
//! .gitignore. Las órdenes se lanzan con un [`EjecutorDeProcesos`] inyectado, con los argumentos
//! como lista y la opción -c safe.directory en cada una, sin tocar nunca la configuración global
//! de git.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::proyecto::archivos;
use crate::proyecto::informe::{InformeDeInicializacion, TipoDeCambio};
use crate::textos;

/// Programa de git.
pub const PROGRAMA_GIT: &str = "git";

/// Archivo de exclusiones de git, relativo a la raíz del proyecto.
pub const ARCHIVO_IGNORADOS: &str = ".gitignore";

/// Entrada que excluye .gestor/ del control de versiones.
pub const ENTRADA_GESTOR: &str = ".gestor/";

/// Comentario que precede a la entrada al añadirla.
const COMENTARIO_ENTRADA: &str =
    "# Canal, diarios y actas de los agentes (Gestor Agentes): no se versionan.";

/// Formas de escribir la exclusión que ya cuentan como presentes.
const ENTRADAS_EQUIVALENTES: [&str; 6] = [
    ".gestor",
    ".gestor/",
    "/.gestor",
    "/.gestor/",
    ".gestor/*",
    ".gestor/**",
];

/// Código de salida que se anota cuando el proceso termina sin código, lo que solo pasa en Unix
/// si lo mata una señal (std::process::ExitStatus::code devuelve None). Es negativo para que no
/// coincida con ningún código de salida normal, que en Unix va de 0 a 255.
pub const CODIGO_SIN_CODIGO_DE_SALIDA: i32 = -1;

/// Intervalo con que se comprueba si el proceso ha terminado.
const INTERVALO_DE_ESPERA: Duration = Duration::from_millis(20);

/// Resultado de un proceso terminado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoDeProceso {
    /// Código de salida; -1 si el proceso terminó por una señal.
    pub codigo_de_salida: i32,
    /// Salida estándar, decodificada como UTF-8 con sustitución.
    pub salida: String,
    /// Salida de error, decodificada como UTF-8 con sustitución.
    pub error: String,
}

/// No se pudo ejecutar el programa o no respondió a tiempo.
#[derive(Debug, thiserror::Error)]
pub enum ProgramaNoDisponible {
    /// El sistema no encontró o no pudo arrancar el programa.
    #[error("No se encuentra {programa}: {origen}")]
    NoSePudoArrancar {
        /// Programa que se intentó arrancar.
        programa: String,
        /// Error del sistema.
        #[source]
        origen: io::Error,
    },
    /// El programa no terminó dentro del tiempo máximo y se ha detenido.
    #[error("{programa} no respondió en {segundos} segundos.")]
    SinRespuesta {
        /// Programa que no respondió.
        programa: String,
        /// Segundos que se esperó.
        segundos: u64,
    },
}

/// Ejecuta un programa externo y espera a que termine. Se inyecta para probar el servicio sin
/// lanzar git.
pub trait EjecutorDeProcesos {
    /// Ejecuta el programa con sus argumentos en la carpeta indicada.
    ///
    /// # Errors
    ///
    /// Devuelve [`ProgramaNoDisponible`] si no se puede arrancar o no termina a tiempo.
    fn ejecutar(
        &self,
        programa: &str,
        argumentos: &[OsString],
        carpeta: &Path,
    ) -> Result<ResultadoDeProceso, ProgramaNoDisponible>;
}

/// Ejecutor real, con std::process, sin ventana en Windows y con tiempo máximo.
#[derive(Debug, Clone, Copy)]
pub struct EjecutorDelSistema {
    tiempo_maximo: Duration,
}

impl EjecutorDelSistema {
    /// Ejecutor que para cada proceso que no termine en `tiempo_maximo`, normalmente el de
    /// [`crate::ajustes::Ajustes::tiempo_maximo_git`].
    #[must_use]
    pub const fn con_tiempo_maximo(tiempo_maximo: Duration) -> Self {
        Self { tiempo_maximo }
    }
}

impl Default for EjecutorDelSistema {
    /// Ejecutor con el tiempo máximo por defecto de los ajustes.
    fn default() -> Self {
        Self::con_tiempo_maximo(Duration::from_secs(
            crate::ajustes::TIEMPO_MAXIMO_GIT_POR_DEFECTO,
        ))
    }
}

impl EjecutorDeProcesos for EjecutorDelSistema {
    fn ejecutar(
        &self,
        programa: &str,
        argumentos: &[OsString],
        carpeta: &Path,
    ) -> Result<ResultadoDeProceso, ProgramaNoDisponible> {
        let mut orden = Command::new(programa);
        orden
            .args(argumentos)
            .current_dir(carpeta)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        sin_ventana(&mut orden);
        let hijo = orden
            .spawn()
            .map_err(|origen| ProgramaNoDisponible::NoSePudoArrancar {
                programa: programa.to_owned(),
                origen,
            })?;
        esperar_con_tiempo_maximo(hijo, programa, self.tiempo_maximo)
    }
}

/// En Windows, git es un programa de consola: sin esta opción abriría una ventana negra.
#[cfg(windows)]
fn sin_ventana(orden: &mut Command) {
    use std::os::windows::process::CommandExt;
    /// CREATE_NO_WINDOW, de la documentación de Process Creation Flags de Microsoft
    /// (https://learn.microsoft.com/windows/win32/procthread/process-creation-flags).
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    orden.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
const fn sin_ventana(_orden: &mut Command) {}

/// Lee las dos salidas en hilos aparte, para que el hijo no se bloquee con una tubería llena, y
/// espera hasta el tiempo máximo; si se agota, detiene el proceso.
fn esperar_con_tiempo_maximo(
    mut hijo: Child,
    programa: &str,
    tiempo_maximo: Duration,
) -> Result<ResultadoDeProceso, ProgramaNoDisponible> {
    let lector_salida = hijo.stdout.take().map(leer_en_hilo);
    let lector_error = hijo.stderr.take().map(leer_en_hilo);
    let limite = Instant::now() + tiempo_maximo;

    let estado = loop {
        match hijo.try_wait() {
            Ok(Some(estado)) => break Some(estado),
            Ok(None) if Instant::now() < limite => thread::sleep(INTERVALO_DE_ESPERA),
            Ok(None) | Err(_) => break None,
        }
    };

    let Some(estado) = estado else {
        // Si ya terminó entre la comprobación y la orden de parar, kill falla y da igual.
        let _ = hijo.kill();
        let _ = hijo.wait();
        return Err(ProgramaNoDisponible::SinRespuesta {
            programa: programa.to_owned(),
            segundos: tiempo_maximo.as_secs(),
        });
    };

    Ok(ResultadoDeProceso {
        codigo_de_salida: estado.code().unwrap_or(CODIGO_SIN_CODIGO_DE_SALIDA),
        salida: recoger(lector_salida),
        error: recoger(lector_error),
    })
}

/// Lee una tubería entera en un hilo.
fn leer_en_hilo<R: Read + Send + 'static>(mut tuberia: R) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        // Un error de lectura deja lo leído hasta entonces; la salida solo se usa para informar.
        let _ = tuberia.read_to_end(&mut bytes);
        bytes
    })
}

/// Recoge lo que leyó un hilo, como texto.
fn recoger(lector: Option<thread::JoinHandle<Vec<u8>>>) -> String {
    lector
        .and_then(|hilo| hilo.join().ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

/// Servicio de git con su ejecutor.
pub struct ServicioGit<'a> {
    ejecutor: &'a dyn EjecutorDeProcesos,
}

impl<'a> ServicioGit<'a> {
    /// Crea el servicio con el ejecutor indicado.
    #[must_use]
    pub const fn new(ejecutor: &'a dyn EjecutorDeProcesos) -> Self {
        Self { ejecutor }
    }

    /// Comprueba que la carpeta está dentro de un repositorio y, si no, lo crea con git init. Si
    /// git no está disponible, lo anota como aviso; si git init falla, como error.
    pub fn asegurar_repositorio(&self, raiz: &Path, informe: &mut InformeDeInicializacion) {
        let consulta = self.ejecutor.ejecutar(
            PROGRAMA_GIT,
            &argumentos(raiz, &["rev-parse", "--is-inside-work-tree"]),
            raiz,
        );
        let resultado = consulta.and_then(|consulta| {
            if consulta.codigo_de_salida == 0 && consulta.salida.trim() == "true" {
                return Ok(None);
            }
            self.ejecutor
                .ejecutar(PROGRAMA_GIT, &argumentos(raiz, &["init"]), raiz)
                .map(Some)
        });
        match resultado {
            Ok(None) => {}
            Ok(Some(creacion)) if creacion.codigo_de_salida != 0 => {
                informe.anotar_error(textos::git_init_fallido(creacion.error.trim()));
            }
            Ok(Some(_)) => informe.anotar_cambio(".git", TipoDeCambio::RepositorioCreado, None),
            Err(error) => informe.anotar_aviso(textos::git_no_comprobado(&error.to_string())),
        }
    }
}

/// Añade .gestor/ a .gitignore si no está, conservando el formato del archivo.
///
/// # Errors
///
/// Devuelve el error de disco si no se puede leer o escribir .gitignore.
pub fn asegurar_ignorado(raiz: &Path, informe: &mut InformeDeInicializacion) -> io::Result<()> {
    let ruta = archivos::ruta(raiz, ARCHIVO_IGNORADOS);
    let leido = archivos::leer(&ruta)?;
    let Some(nuevo) = anadir_entrada_gestor(leido.as_ref().map(|(texto, _)| texto.as_str())) else {
        return Ok(());
    };
    let formato = leido
        .as_ref()
        .map(|(_, formato)| *formato)
        .unwrap_or_default();
    archivos::escribir(&ruta, &nuevo, formato)?;
    let tipo = if leido.is_some() {
        TipoDeCambio::Actualizado
    } else {
        TipoDeCambio::Creado
    };
    informe.anotar_cambio(
        ARCHIVO_IGNORADOS,
        tipo,
        Some(String::from(textos::DETALLE_GITIGNORE)),
    );
    Ok(())
}

/// Texto de .gitignore con la exclusión de .gestor/ añadida al final, o nada si ya estaba en
/// alguna de sus formas equivalentes.
#[must_use]
pub fn anadir_entrada_gestor(texto_actual: Option<&str>) -> Option<String> {
    let texto = texto_actual.unwrap_or_default();
    let ya_esta = texto
        .split('\n')
        .any(|linea| ENTRADAS_EQUIVALENTES.contains(&linea.trim()));
    if ya_esta {
        return None;
    }
    let bloque = format!("{COMENTARIO_ENTRADA}\n{ENTRADA_GESTOR}\n");
    if texto.is_empty() {
        return Some(bloque);
    }
    let separador = if texto.ends_with('\n') { "" } else { "\n" };
    Some(format!("{texto}{separador}\n{bloque}"))
}

/// Argumentos de una orden de git con -c safe.directory=<raíz con barras normales> delante.
fn argumentos(raiz: &Path, orden: &[&str]) -> Vec<OsString> {
    let raiz_con_barras = raiz.to_string_lossy().replace('\\', "/");
    let mut lista = vec![
        OsString::from("-c"),
        OsString::from(format!("safe.directory={raiz_con_barras}")),
    ];
    lista.extend(orden.iter().map(OsString::from));
    lista
}
