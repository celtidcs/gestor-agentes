//! Decisión pura de cómo se abre cada herramienta.
//!
//! Traducción del lanzador de la 2.3.2 y sus búsquedas de ejecutables, más Programator, con
//! codex y agy en terminal y sin aplicaciones de escritorio desde la 3.0.0. Nada de aquí toca el sistema: todo pasa por [`EntornoDelSistema`].

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::terminal::{self, Asignacion, TERMINALES_LINUX};
use crate::textos;
use crate::{
    EntornoDelSistema, ErrorDeLanzamiento, Herramienta, PeticionDeLanzamiento, PlanDeLanzamiento,
    Sistema, VARIABLE_IDENTIDAD,
};

/// Nombre del ejecutable de Claude Code, sin extensión.
const NOMBRE_CLAUDE: &str = "claude";
/// Nombre del ejecutable de Codex CLI, sin extensión.
const NOMBRE_CODEX: &str = "codex";
/// Extensiones de Claude en Windows, por orden: npm instala Claude Code con un lanzador .cmd.
const EXTENSIONES_CLAUDE_WINDOWS: [&str; 2] = [".exe", ".cmd"];
/// Extensiones de Codex CLI en Windows, por orden. Se acepta .cmd (npm instala codex con un
/// lanzador .cmd), porque la ruta del proyecto no va en los argumentos: la carpeta se fija en el
/// guion con Set-Location. Los argumentos del perfil sí pasan, y los escribe el usuario.
const EXTENSIONES_CODEX_WINDOWS: [&str; 2] = [".exe", ".cmd"];
/// Nombre del ejecutable de agy, la CLI de Antigravity, sin extensión.
const NOMBRE_AGY: &str = "agy";
/// Extensiones de agy en Windows: su instalador oficial deja un agy.exe.
const EXTENSIONES_AGY_WINDOWS: [&str; 1] = [".exe"];
/// Carpetas bajo %LOCALAPPDATA% donde el instalador oficial de agy deja su programa en Windows
/// (https://antigravity.google/docs/getting-started?tab=cli; comprobado con agy 1.5.5 en
/// Windows).
const CARPETAS_AGY_WINDOWS: [&str; 2] = ["agy", "bin"];
/// Carpetas bajo %LOCALAPPDATA% donde el instalador oficial de Codex CLI deja codex.exe en
/// Windows: «$defaultVisibleBinDir = Join-Path $env:LOCALAPPDATA "Programs\OpenAI\Codex\bin"»
/// (https://releases.openai.com/codex/install.ps1, leído al escribir esto). El instalador también la
/// añade al PATH del usuario, pero un programa abierto antes de instalar no lo ve, y así se
/// encuentra igual.
const CARPETAS_CODEX_WINDOWS: [&str; 4] = ["Programs", "OpenAI", "Codex", "bin"];
/// Carpetas del sistema donde se busca un ejecutable en Linux después de ~/.local/bin.
const CARPETAS_SISTEMA_LINUX: [&str; 2] = ["/usr/local/bin", "/usr/bin"];
/// Separador entre la identidad y el proyecto en el título de la terminal, como en la 2.3.2.
const SEPARADOR_TITULO: &str = " · ";
/// Variable que, vacía, evita que Claude Code se crea anidado si la app se abrió desde una
/// sesión suya.
const VARIABLE_CLAUDECODE: &str = "CLAUDECODE";
/// Variable con la carpeta de configuración de Claude Code.
const VARIABLE_CONFIGURACION_CLAUDE: &str = "CLAUDE_CONFIG_DIR";
/// Variable con la carpeta de configuración de Codex.
const VARIABLE_CONFIGURACION_CODEX: &str = "CODEX_HOME";
/// Opción con la que Programator recibe la carpeta del proyecto.
const OPCION_RUTA_PROGRAMATOR: &str = "--ruta";
/// Opción de agy que abre la sesión interactiva con un primer mensaje: «-i Short alias for
/// --prompt-interactive», «Run an initial prompt interactively and continue the session»
/// (agy --help, comprobado el 2026-11-13). Claude Code y codex
/// reciben el mensaje como argumento posicional: «claude "query"», «Start interactive session
/// with initial prompt» (https://code.claude.com/docs/en/cli-reference) y «Usage: codex
/// [OPTIONS] [PROMPT]» (codex --help, el mismo día).
const OPCION_MENSAJE_AGY: &str = "-i";

/// Véase [`crate::planificar`].
pub(crate) fn planificar(
    peticion: &PeticionDeLanzamiento,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
    match peticion.herramienta {
        Herramienta::Claude => plan_de_claude(peticion, sistema, entorno),
        Herramienta::Codex => plan_de_codex(peticion, sistema, entorno),
        Herramienta::Gemini => plan_de_gemini(peticion, sistema, entorno),
        Herramienta::Programator => plan_de_programator(peticion, sistema, entorno),
    }
}

/// Véase [`crate::detectar_ejecutable`].
pub(crate) fn detectar_ejecutable(
    herramienta: Herramienta,
    ruta_fijada: Option<&Path>,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Option<PathBuf> {
    if let Some(ruta) = ruta_fijada {
        return entorno.existe(ruta).then(|| ruta.to_path_buf());
    }
    match herramienta {
        Herramienta::Claude => {
            buscar_ejecutable(NOMBRE_CLAUDE, &EXTENSIONES_CLAUDE_WINDOWS, sistema, entorno)
        }
        Herramienta::Codex => buscar_con_instalador(
            &ProgramaConInstalador {
                nombre: NOMBRE_CODEX,
                extensiones_windows: &EXTENSIONES_CODEX_WINDOWS,
                carpetas_del_instalador: &CARPETAS_CODEX_WINDOWS,
            },
            sistema,
            entorno,
        ),
        Herramienta::Gemini => buscar_con_instalador(
            &ProgramaConInstalador {
                nombre: NOMBRE_AGY,
                extensiones_windows: &EXTENSIONES_AGY_WINDOWS,
                carpetas_del_instalador: &CARPETAS_AGY_WINDOWS,
            },
            sistema,
            entorno,
        ),
        Herramienta::Programator => None,
    }
}

/// El ejecutable que abre la petición, con la misma búsqueda que [`detectar_ejecutable`], o el
/// error que explica por qué no lo hay.
fn ejecutable_de(
    peticion: &PeticionDeLanzamiento,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Result<PathBuf, ErrorDeLanzamiento> {
    detectar_ejecutable(
        peticion.herramienta,
        peticion.ejecutable.as_deref(),
        sistema,
        entorno,
    )
    .ok_or_else(|| sin_ejecutable(peticion, sistema))
}

/// Error cuando no hay ejecutable: el perfil de Programator sin ruta está incompleto; una ruta
/// fijada que no existe se nombra en el mensaje; y si no se encuentra el programa, se dice dónde
/// se ha buscado.
fn sin_ejecutable(peticion: &PeticionDeLanzamiento, sistema: Sistema) -> ErrorDeLanzamiento {
    let herramienta = textos::nombre(peticion.herramienta).to_owned();
    match &peticion.ejecutable {
        Some(fijado) => ErrorDeLanzamiento::NoEncontrada {
            herramienta,
            detalle: textos::ejecutable_inexistente(fijado),
        },
        None if peticion.herramienta == Herramienta::Programator => {
            ErrorDeLanzamiento::PerfilIncompleto {
                identidad: peticion.identidad.clone(),
                detalle: textos::PROGRAMATOR_SIN_EJECUTABLE.to_owned(),
            }
        }
        None => ErrorDeLanzamiento::NoEncontrada {
            herramienta,
            detalle: textos::donde_se_busco(peticion.herramienta, sistema).to_owned(),
        },
    }
}

/// Lo que se abre en una terminal: título de la ventana, variables, carpeta de trabajo, programa,
/// argumentos y carpetas que hay que crear antes.
struct OrdenDeTerminal<'a> {
    titulo: String,
    variables: Vec<Asignacion<'a>>,
    carpeta: &'a Path,
    programa: &'a Path,
    argumentos: Vec<&'a str>,
    carpetas_a_crear: Vec<PathBuf>,
}

/// Plan que abre una orden en una terminal: PowerShell en Windows y el primer emulador
/// disponible en Linux. Las rutas se escriben en el guion, así que tienen que ser Unicode.
fn plan_en_terminal(
    orden: OrdenDeTerminal<'_>,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
    let carpeta = como_texto(orden.carpeta)?;
    let programa = como_texto(orden.programa)?;
    let (programa_terminal, argumentos) = match sistema {
        Sistema::Windows => {
            let guion = terminal::guion_powershell(
                &orden.titulo,
                &orden.variables,
                carpeta,
                programa,
                &orden.argumentos,
            );
            (
                PathBuf::from(terminal::POWERSHELL),
                terminal::argumentos_powershell(&guion),
            )
        }
        Sistema::Linux => {
            let guion =
                terminal::guion_bash(&orden.variables, carpeta, programa, &orden.argumentos);
            let emulador = TERMINALES_LINUX
                .iter()
                .copied()
                .find(|t| entorno.buscar_en_path(t.programa(), &[]).is_some())
                .unwrap_or(terminal::TerminalLinux::Xterm);
            (
                PathBuf::from(emulador.programa()),
                emulador.argumentos(&orden.titulo, &guion),
            )
        }
    };
    Ok(PlanDeLanzamiento {
        programa: programa_terminal,
        argumentos,
        variables: Vec::new(),
        carpeta: orden.carpeta.to_path_buf(),
        carpetas_a_crear: orden.carpetas_a_crear,
    })
}

/// Título de la ventana de terminal: «<identidad> · <proyecto>», como en la 2.3.2.
fn titulo(peticion: &PeticionDeLanzamiento) -> String {
    format!(
        "{}{SEPARADOR_TITULO}{}",
        peticion.identidad, peticion.nombre_proyecto
    )
}

/// La asignación de GESTOR_AGENTE para el guion de la terminal.
fn asignacion_de_identidad(peticion: &PeticionDeLanzamiento) -> Asignacion<'_> {
    Asignacion {
        nombre: VARIABLE_IDENTIDAD,
        valor: peticion.identidad.as_str(),
    }
}

/// Claude Code en una terminal: PowerShell en Windows y el primer emulador disponible en Linux.
/// Si el programa no se encuentra, es un error. La 2.3.2 confiaba entonces en que el intérprete
/// lo encontrase por su nombre, pero desde la 3.0.0 la detección y el lanzamiento usan la misma
/// búsqueda, y un botón solo se muestra si esa búsqueda encuentra el programa.
fn plan_de_claude(
    peticion: &PeticionDeLanzamiento,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
    let ejecutable = ejecutable_de(peticion, sistema, entorno)?;
    let configuracion = peticion
        .directorio_configuracion
        .as_deref()
        .map(como_texto)
        .transpose()?;
    let mut variables = vec![
        asignacion_de_identidad(peticion),
        Asignacion {
            nombre: VARIABLE_CLAUDECODE,
            valor: "",
        },
    ];
    if let Some(carpeta) = configuracion {
        variables.push(Asignacion {
            nombre: VARIABLE_CONFIGURACION_CLAUDE,
            valor: carpeta,
        });
    }
    plan_en_terminal(
        OrdenDeTerminal {
            titulo: titulo(peticion),
            variables,
            carpeta: &peticion.carpeta_proyecto,
            programa: &ejecutable,
            argumentos: mensaje_inicial(peticion).into_iter().collect(),
            carpetas_a_crear: peticion.directorio_configuracion.iter().cloned().collect(),
        },
        sistema,
        entorno,
    )
}

/// Una ruta como texto para el guion de la terminal, o el error si no es Unicode válido.
fn como_texto(ruta: &Path) -> Result<&str, ErrorDeLanzamiento> {
    ruta.to_str()
        .ok_or_else(|| ErrorDeLanzamiento::RutaNoUnicode {
            ruta: ruta.to_path_buf(),
        })
}

/// Codex CLI en una terminal, en la carpeta del proyecto, con GESTOR_AGENTE, CODEX_HOME si el
/// perfil la trae y los argumentos del perfil.
fn plan_de_codex(
    peticion: &PeticionDeLanzamiento,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
    let ejecutable = ejecutable_de(peticion, sistema, entorno)?;
    let configuracion = peticion
        .directorio_configuracion
        .as_deref()
        .map(como_texto)
        .transpose()?;
    let mut variables = vec![asignacion_de_identidad(peticion)];
    if let Some(carpeta) = configuracion {
        variables.push(Asignacion {
            nombre: VARIABLE_CONFIGURACION_CODEX,
            valor: carpeta,
        });
    }
    plan_en_terminal(
        OrdenDeTerminal {
            titulo: titulo(peticion),
            variables,
            carpeta: &peticion.carpeta_proyecto,
            programa: &ejecutable,
            argumentos: argumentos_del_perfil(peticion)
                .into_iter()
                .chain(mensaje_inicial(peticion))
                .collect(),
            carpetas_a_crear: peticion.directorio_configuracion.iter().cloned().collect(),
        },
        sistema,
        entorno,
    )
}

/// Gemini con agy en una terminal, en la carpeta del proyecto, con GESTOR_AGENTE (agy la pasa a
/// las órdenes de su agente: comprobado con agy 1.5.5) y los argumentos del perfil. agy no tiene
/// variable documentada para su carpeta de configuración, así que no se fija ninguna.
fn plan_de_gemini(
    peticion: &PeticionDeLanzamiento,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
    let ejecutable = ejecutable_de(peticion, sistema, entorno)?;
    plan_en_terminal(
        OrdenDeTerminal {
            titulo: titulo(peticion),
            variables: vec![asignacion_de_identidad(peticion)],
            carpeta: &peticion.carpeta_proyecto,
            programa: &ejecutable,
            argumentos: argumentos_del_perfil(peticion)
                .into_iter()
                .chain(
                    mensaje_inicial(peticion)
                        .into_iter()
                        .flat_map(|mensaje| [OPCION_MENSAJE_AGY, mensaje]),
                )
                .collect(),
            carpetas_a_crear: Vec::new(),
        },
        sistema,
        entorno,
    )
}

/// Programa que en Windows tiene un instalador oficial que lo deja en una carpeta conocida bajo
/// %LOCALAPPDATA%.
struct ProgramaConInstalador<'a> {
    /// Nombre del ejecutable, sin extensión.
    nombre: &'a str,
    /// Extensiones que se prueban en Windows, por orden; la primera es la del ejecutable que deja
    /// el instalador.
    extensiones_windows: &'a [&'a str],
    /// Carpetas, bajo %LOCALAPPDATA%, donde lo deja su instalador.
    carpetas_del_instalador: &'a [&'a str],
}

/// Busca un programa con instalador oficial (agy y Codex CLI): en Windows, primero donde lo deja
/// ese instalador, bajo %LOCALAPPDATA%; después, como cualquier otro ejecutable (~/.local/bin y el
/// PATH, y en Linux también /usr/local/bin y /usr/bin). Es el mismo orden que deja el instalador
/// de Codex, que pone su carpeta delante en el PATH si encuentra otra instalación.
fn buscar_con_instalador(
    programa: &ProgramaConInstalador<'_>,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Option<PathBuf> {
    let instalado = match (sistema, programa.extensiones_windows.first()) {
        (Sistema::Windows, Some(extension)) => entorno.carpeta_de_datos_locales().map(|local| {
            programa
                .carpetas_del_instalador
                .iter()
                .fold(local, |ruta, parte| ruta.join(parte))
                .join(format!("{}{extension}", programa.nombre))
        }),
        _ => None,
    };
    instalado.filter(|ruta| entorno.existe(ruta)).or_else(|| {
        buscar_ejecutable(
            programa.nombre,
            programa.extensiones_windows,
            sistema,
            entorno,
        )
    })
}

/// Mensaje inicial de la petición, si lo hay y no está en blanco.
fn mensaje_inicial(peticion: &PeticionDeLanzamiento) -> Option<&str> {
    peticion
        .mensaje_inicial
        .as_deref()
        .filter(|mensaje| !mensaje.trim().is_empty())
}

/// Los argumentos del perfil como texto, para el guion de la terminal.
fn argumentos_del_perfil(peticion: &PeticionDeLanzamiento) -> Vec<&str> {
    peticion
        .argumentos_extra
        .iter()
        .map(String::as_str)
        .collect()
}

/// Programator con `--ruta <carpeta del proyecto>`, desde la carpeta de su ejecutable para que
/// encuentre su programator.toml. Es un programa de consola: en Windows se abre en una consola
/// nueva y en Linux dentro de un emulador de terminal, porque allí no hay consola que crear.
fn plan_de_programator(
    peticion: &PeticionDeLanzamiento,
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
    let perfil_incompleto = |detalle: &str| ErrorDeLanzamiento::PerfilIncompleto {
        identidad: peticion.identidad.clone(),
        detalle: detalle.to_owned(),
    };
    let ejecutable = ejecutable_de(peticion, sistema, entorno)?;
    let carpeta = ejecutable
        .parent()
        .filter(|c| !c.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .ok_or_else(|| perfil_incompleto(textos::PROGRAMATOR_SIN_CARPETA))?;
    if sistema == Sistema::Linux {
        return plan_en_terminal(
            OrdenDeTerminal {
                titulo: titulo(peticion),
                variables: vec![asignacion_de_identidad(peticion)],
                carpeta: &carpeta,
                programa: &ejecutable,
                argumentos: vec![
                    OPCION_RUTA_PROGRAMATOR,
                    como_texto(&peticion.carpeta_proyecto)?,
                ],
                carpetas_a_crear: Vec::new(),
            },
            sistema,
            entorno,
        );
    }
    Ok(PlanDeLanzamiento {
        programa: ejecutable,
        argumentos: vec![
            OsString::from(OPCION_RUTA_PROGRAMATOR),
            peticion.carpeta_proyecto.clone().into_os_string(),
        ],
        variables: vec![variable_de_identidad(peticion)],
        carpeta,
        carpetas_a_crear: Vec::new(),
    })
}

/// La variable GESTOR_AGENTE con la identidad de la petición.
fn variable_de_identidad(peticion: &PeticionDeLanzamiento) -> (String, OsString) {
    (
        VARIABLE_IDENTIDAD.to_owned(),
        OsString::from(&peticion.identidad),
    )
}

/// Busca un ejecutable como la 2.3.2. En Windows: ~/.local/bin y después cada carpeta del PATH,
/// con las extensiones dadas. En Linux: ~/.local/bin, /usr/local/bin y /usr/bin, y después el
/// PATH (la 2.3.2 solo hacía esto último para Codex; para Claude caía al nombre a secas, que el
/// intérprete busca igualmente en el PATH).
fn buscar_ejecutable(
    nombre: &str,
    extensiones_windows: &[&str],
    sistema: Sistema,
    entorno: &dyn EntornoDelSistema,
) -> Option<PathBuf> {
    let local = entorno
        .carpeta_personal()
        .map(|personal| personal.join(".local").join("bin"));
    match sistema {
        Sistema::Windows => local
            .and_then(|carpeta| {
                extensiones_windows
                    .iter()
                    .map(|ext| carpeta.join(format!("{nombre}{ext}")))
                    .find(|candidato| entorno.existe(candidato))
            })
            .or_else(|| entorno.buscar_en_path(nombre, extensiones_windows)),
        Sistema::Linux => local
            .into_iter()
            .chain(CARPETAS_SISTEMA_LINUX.iter().map(PathBuf::from))
            .map(|carpeta| carpeta.join(nombre))
            .find(|candidato| entorno.existe(candidato))
            .or_else(|| entorno.buscar_en_path(nombre, &[])),
    }
}
