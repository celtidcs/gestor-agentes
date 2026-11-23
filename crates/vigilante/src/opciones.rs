//! Línea de órdenes del vigilante. Solo interpreta: no toca disco, entorno ni procesos.
//!
//! Uso:
//! ```text
//! gestor-vigilante (--una-vez | --hasta-cambio | --persistente --entrega codex)
//!                  [--agente "Claude 2"] [--vida-segundos N]
//! ```
//! Las opciones de la segunda línea de abajo las pone el propio vigilante al quedarse en segundo
//! plano en modo persistente; no hace falta escribirlas: `--hijo`, `--dueno PID:ARRANQUE`,
//! `--codex RUTA`, `--hilo ID`. Las pruebas usan además `--gestor RUTA` (la carpeta .gestor/) y
//! `--intervalo-segundos N`.

use std::fmt;
use std::path::PathBuf;

/// Vida máxima de --hasta-cambio, en segundos: una jornada de 8 horas, como en los guiones de la
/// 3.5.1. Es un tope de seguridad; al cumplirlo despierta al agente una vez y este lo relanza.
pub const VIDA_HASTA_CAMBIO_SEGUNDOS: u64 = 28_800;

/// Segundos entre dos pasadas, como en los guiones de la 3.5.1: un encargo se ve en menos de ese
/// tiempo sin cargar el equipo, porque cada pasada solo lee unos pocos archivos pequeños.
pub const INTERVALO_SEGUNDOS: u64 = 10;

/// Modo de trabajo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modo {
    /// Una pasada y termina: dice lo que ha cambiado o «sin cambios».
    UnaVez,
    /// Espera hasta un cambio con etiqueta (encargo o mutación viva) y termina. Para las
    /// herramientas que despiertan al agente cuando acaba una tarea en segundo plano.
    HastaCambio,
    /// Se queda en segundo plano toda la sesión y entrega cada aviso en la sesión del agente.
    Persistente,
    /// Anota la sesión de Codex (CODEX_THREAD_ID) para el vigilante que lanzó Gestor Agentes desde
    /// la terminal, y termina.
    AnotarHilo,
}

/// Cómo se entrega un aviso en modo persistente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDeEntrega {
    /// Con `codex queue`, en la sesión de Codex que lanzó el vigilante.
    Codex,
}

/// Proceso identificado por su número y su hora de arranque (segundos desde 1970), para no
/// confundirlo con otro que reciba el mismo número.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcesoMarcado {
    /// Número de proceso.
    pub pid: u32,
    /// Hora de arranque.
    pub arranque: u64,
}

/// Opciones ya interpretadas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opciones {
    /// Modo.
    pub modo: Modo,
    /// Identidad pasada con --agente, si se pasó.
    pub agente: Option<String>,
    /// Vida máxima en segundos, si se pasó.
    pub vida_segundos: Option<u64>,
    /// Entrega del modo persistente.
    pub entrega: Option<TipoDeEntrega>,
    /// Si es el proceso que ya se ha quedado en segundo plano.
    pub hijo: bool,
    /// Proceso del agente al que sirve, en el modo persistente.
    pub dueno: Option<ProcesoMarcado>,
    /// Ejecutable de Codex, si se pasó.
    pub codex: Option<PathBuf>,
    /// Sesión (hilo) de Codex, si se pasó.
    pub hilo: Option<String>,
    /// Carpeta .gestor/ del proyecto, si se pasó; si no, la de la ubicación del ejecutable.
    pub gestor: Option<PathBuf>,
    /// Segundos entre pasadas; por defecto [`INTERVALO_SEGUNDOS`]. Las pruebas lo acortan.
    pub intervalo_segundos: u64,
    /// Terminal que abrió Codex, cuando el vigilante persistente lo lanza Gestor Agentes desde ella
    /// y no la sesión de Codex (que en su sandbox no puede ver los procesos).
    pub desde_terminal: Option<u32>,
}

/// Error al interpretar las opciones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpcionesInvalidas(pub String);

impl fmt::Display for OpcionesInvalidas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Interpreta los argumentos, sin el nombre del programa.
///
/// # Errors
///
/// [`OpcionesInvalidas`] con el motivo, si falta el modo, hay dos, una opción no se conoce, le
/// falta su valor o el valor no vale.
pub fn interpretar(argumentos: &[String]) -> Result<Opciones, OpcionesInvalidas> {
    let mut modo = None;
    let mut opciones = Opciones {
        modo: Modo::UnaVez,
        agente: None,
        vida_segundos: None,
        entrega: None,
        hijo: false,
        dueno: None,
        codex: None,
        hilo: None,
        gestor: None,
        intervalo_segundos: INTERVALO_SEGUNDOS,
        desde_terminal: None,
    };
    let mut resto = argumentos.iter();
    while let Some(argumento) = resto.next() {
        let mut valor = |nombre: &str| {
            resto
                .next()
                .cloned()
                .ok_or_else(|| OpcionesInvalidas(format!("falta el valor de {nombre}")))
        };
        match argumento.as_str() {
            "--una-vez" => fijar_modo(&mut modo, Modo::UnaVez)?,
            "--hasta-cambio" => fijar_modo(&mut modo, Modo::HastaCambio)?,
            "--persistente" => fijar_modo(&mut modo, Modo::Persistente)?,
            "--anotar-hilo" => fijar_modo(&mut modo, Modo::AnotarHilo)?,
            "--desde-terminal" => {
                opciones.desde_terminal = Some(numero_de_proceso(&valor("--desde-terminal")?)?);
            }
            "--agente" => opciones.agente = Some(valor("--agente")?),
            "--vida-segundos" => {
                opciones.vida_segundos = Some(entero_positivo(
                    "--vida-segundos",
                    &valor("--vida-segundos")?,
                )?);
            }
            "--entrega" => opciones.entrega = Some(entrega(&valor("--entrega")?)?),
            "--hijo" => opciones.hijo = true,
            "--dueno" => opciones.dueno = Some(proceso_marcado(&valor("--dueno")?)?),
            "--codex" => opciones.codex = Some(PathBuf::from(valor("--codex")?)),
            "--hilo" => opciones.hilo = Some(valor("--hilo")?),
            "--gestor" => opciones.gestor = Some(PathBuf::from(valor("--gestor")?)),
            "--intervalo-segundos" => {
                opciones.intervalo_segundos =
                    entero_positivo("--intervalo-segundos", &valor("--intervalo-segundos")?)?;
            }
            otro => return Err(OpcionesInvalidas(format!("opción desconocida: {otro}"))),
        }
    }
    opciones.modo = modo.ok_or_else(|| {
        OpcionesInvalidas(String::from(
            "falta el modo: --una-vez, --hasta-cambio, --persistente o --anotar-hilo",
        ))
    })?;
    if opciones.modo == Modo::Persistente && opciones.entrega.is_none() {
        return Err(OpcionesInvalidas(String::from(
            "--persistente necesita --entrega codex",
        )));
    }
    if opciones.desde_terminal.is_some() && opciones.modo != Modo::Persistente {
        return Err(OpcionesInvalidas(String::from(
            "--desde-terminal solo vale con --persistente",
        )));
    }
    Ok(opciones)
}

/// Número de proceso: un entero de 1 en adelante, solo con cifras.
fn numero_de_proceso(texto: &str) -> Result<u32, OpcionesInvalidas> {
    texto
        .parse::<u32>()
        .ok()
        .filter(|n| *n >= 1 && texto.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| {
            OpcionesInvalidas(format!(
                "--desde-terminal necesita un número de proceso: {texto}"
            ))
        })
}

/// Fija el modo, o falla si ya había uno.
fn fijar_modo(modo: &mut Option<Modo>, nuevo: Modo) -> Result<(), OpcionesInvalidas> {
    if modo.is_some() {
        return Err(OpcionesInvalidas(String::from(
            "solo se puede pedir un modo: --una-vez, --hasta-cambio, --persistente o --anotar-hilo",
        )));
    }
    *modo = Some(nuevo);
    Ok(())
}

/// Un entero de 1 en adelante, escrito solo con cifras.
fn entero_positivo(opcion: &str, texto: &str) -> Result<u64, OpcionesInvalidas> {
    texto
        .parse::<u64>()
        .ok()
        .filter(|n| *n >= 1 && texto.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| OpcionesInvalidas(format!("{opcion} necesita un entero positivo: {texto}")))
}

/// Tipo de entrega por su nombre.
fn entrega(texto: &str) -> Result<TipoDeEntrega, OpcionesInvalidas> {
    match texto {
        "codex" => Ok(TipoDeEntrega::Codex),
        otro => Err(OpcionesInvalidas(format!("entrega desconocida: {otro}"))),
    }
}

/// `PID:ARRANQUE`.
fn proceso_marcado(texto: &str) -> Result<ProcesoMarcado, OpcionesInvalidas> {
    let mal = || OpcionesInvalidas(format!("--dueno necesita PID:ARRANQUE: {texto}"));
    let (pid, arranque) = texto.split_once(':').ok_or_else(mal)?;
    Ok(ProcesoMarcado {
        pid: pid.parse().map_err(|_| mal())?,
        arranque: arranque.parse().map_err(|_| mal())?,
    })
}
