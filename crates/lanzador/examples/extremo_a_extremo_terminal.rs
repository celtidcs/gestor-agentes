//! Prueba de extremo a extremo de los agentes en terminal: codex y agy en terminal y, en
//! Linux, Programator dentro de un emulador de terminal.
//!
//! Para cada caso ejecuta el plan real, con el entorno real y `ejecutar`, en una carpeta de
//! proyecto con un nombre difícil. En lugar del programa de verdad abre un pequeño guion fijado en
//! el perfil que anota lo que ve dentro de la terminal (GESTOR_AGENTE, CODEX_HOME, carpeta actual
//! y cada argumento, uno por línea) y cierra la terminal. Después compara lo anotado con lo
//! esperado. No instala ni ejecuta codex ni agy, y no usa ninguna cuenta.
//!
//! Uso: cargo run -p lanzador --example extremo_a_extremo_terminal
//! Sale con código 0 si todo coincide y con 1 si no. En Linux necesita un servidor X (DISPLAY),
//! por ejemplo Xvfb, y uno de los emuladores de terminal que admite el lanzador.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use lanzador::{ejecutar, planificar, EntornoReal, Herramienta, PeticionDeLanzamiento, Sistema};

/// Nombre de la carpeta del proyecto: espacios, acentos, comillas rectas y tipográficas, punto y
/// coma y $, para que la prueba pase también por los escapes.
const NOMBRE_PROYECTO: &str = "Proyecto ’de’ prueba; $x 'ñ'";
/// Argumento hostil del perfil, que tiene que llegar intacto al programa.
const ARGUMENTO_HOSTIL: &str = "valor 'con' ’comillas’; $(touch PWNED) & echo";
/// Marca que el guion escribe en lugar de una variable que no está fijada.
const SIN_VALOR: &str = "-";
/// Tiempo máximo de espera a que la terminal anote lo que ve.
const ESPERA_MAXIMA: Duration = Duration::from_secs(60);
/// Intervalo entre comprobaciones del archivo anotado.
const INTERVALO: Duration = Duration::from_millis(250);

/// Un caso de la prueba.
struct Caso {
    nombre: &'static str,
    herramienta: Herramienta,
    identidad: &'static str,
    con_codex_home: bool,
    argumentos: Vec<String>,
}

fn casos() -> Vec<Caso> {
    let mut casos = vec![
        Caso {
            nombre: "codex",
            herramienta: Herramienta::Codex,
            identidad: "Codex 9",
            con_codex_home: true,
            argumentos: vec![
                "-c".to_owned(),
                "project_doc_max_bytes=65536".to_owned(),
                ARGUMENTO_HOSTIL.to_owned(),
            ],
        },
        Caso {
            nombre: "agy",
            herramienta: Herramienta::Gemini,
            identidad: "Gemini 9",
            con_codex_home: false,
            argumentos: vec![
                "--effort".to_owned(),
                "high".to_owned(),
                ARGUMENTO_HOSTIL.to_owned(),
            ],
        },
    ];
    if cfg!(not(windows)) {
        casos.push(Caso {
            nombre: "programator",
            herramienta: Herramienta::Programator,
            identidad: "Programator 9",
            con_codex_home: false,
            argumentos: Vec::new(),
        });
    }
    casos
}

fn main() -> ExitCode {
    let raiz = std::env::temp_dir().join(format!("lanzador-e2e-terminal-{}", std::process::id()));
    let mut fallos = 0;
    for caso in casos() {
        match comprobar_en(&raiz.join(caso.nombre), &caso) {
            Ok(()) => println!("{}: OK", caso.nombre),
            Err(motivo) => {
                fallos += 1;
                println!("{}: FALLO: {motivo}", caso.nombre);
            }
        }
    }
    let _ = std::fs::remove_dir_all(&raiz);
    if fallos == 0 {
        println!("EXTREMO A EXTREMO TERMINAL: OK");
        ExitCode::SUCCESS
    } else {
        println!("EXTREMO A EXTREMO TERMINAL: {fallos} fallos");
        ExitCode::FAILURE
    }
}

fn comprobar_en(raiz: &Path, caso: &Caso) -> Result<(), String> {
    let proyecto = raiz.join(NOMBRE_PROYECTO);
    let configuracion = raiz.join("configuracion");
    let anotado = raiz.join("anotado.txt");
    let carpeta_programa = raiz.join("programa");
    std::fs::create_dir_all(&proyecto).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&carpeta_programa).map_err(|e| e.to_string())?;
    let guion = escribir_guion(&carpeta_programa, &anotado)?;

    let peticion = PeticionDeLanzamiento {
        herramienta: caso.herramienta,
        identidad: caso.identidad.to_owned(),
        carpeta_proyecto: proyecto.clone(),
        nombre_proyecto: NOMBRE_PROYECTO.to_owned(),
        directorio_configuracion: caso.con_codex_home.then(|| configuracion.clone()),
        ejecutable: Some(guion),
        argumentos_extra: caso.argumentos.clone(),
    };
    let plan = match planificar(&peticion, Sistema::actual(), &EntornoReal) {
        Ok(plan) => plan,
        Err(error) => return Err(error.to_string()),
    };
    println!("{}: programa {}", caso.nombre, plan.programa.display());
    ejecutar(&plan).map_err(|e| e.to_string())?;

    let (carpeta_esperada, argumentos_esperados) = if caso.herramienta == Herramienta::Programator {
        (
            carpeta_programa.clone(),
            vec!["--ruta".to_owned(), ruta_texto(&proyecto)?.to_owned()],
        )
    } else {
        (proyecto.clone(), caso.argumentos.clone())
    };
    let codex_home = if caso.con_codex_home {
        ruta_texto(&configuracion)?.to_owned()
    } else {
        SIN_VALOR.to_owned()
    };
    let mut esperadas = vec![
        caso.identidad.to_owned(),
        codex_home,
        ruta_texto(&carpeta_esperada)?.to_owned(),
    ];
    esperadas.extend(argumentos_esperados);

    let texto = esperar(&anotado, esperadas.len())?;
    let vistas: Vec<&str> = texto.lines().collect();
    if vistas != esperadas {
        return Err(format!(
            "se esperaba {esperadas:?} y la terminal vio {vistas:?}"
        ));
    }
    if raiz.join("PWNED").exists() || proyecto.join("PWNED").exists() {
        return Err("el argumento hostil llegó a ejecutarse".to_owned());
    }
    if caso.con_codex_home && !configuracion.is_dir() {
        return Err("no se creó la carpeta de CODEX_HOME".to_owned());
    }
    Ok(())
}

fn ruta_texto(ruta: &Path) -> Result<&str, String> {
    ruta.to_str()
        .ok_or_else(|| format!("ruta no Unicode: {}", ruta.display()))
}

/// Escribe el guion que hace de programa: anota GESTOR_AGENTE, CODEX_HOME, la carpeta actual y
/// sus argumentos, uno por línea, y cierra la terminal.
fn escribir_guion(carpeta: &Path, anotado: &Path) -> Result<PathBuf, String> {
    let (nombre, contenido) = if cfg!(windows) {
        let destino = ruta_texto(anotado)?.replace('\'', "''");
        (
            "falso-programa.ps1",
            format!(
                "$codex = if ($env:CODEX_HOME) {{ $env:CODEX_HOME }} else {{ '{SIN_VALOR}' }}\r\n\
                 $lineas = @($env:GESTOR_AGENTE, $codex, (Get-Location).ProviderPath) + $args\r\n\
                 [IO.File]::WriteAllText('{destino}', ($lineas -join \"`n\"))\r\n\
                 [Environment]::Exit(0)\r\n"
            ),
        )
    } else {
        (
            "falso-programa.sh",
            format!(
                "#!/bin/sh\n\
                 printf '%s\\n' \"$GESTOR_AGENTE\" \"${{CODEX_HOME:-{SIN_VALOR}}}\" \"$(pwd)\" \"$@\" > '{}'\n\
                 kill \"$PPID\"\n",
                ruta_texto(anotado)?.replace('\'', "'\\''")
            ),
        )
    };
    let ruta = carpeta.join(nombre);
    std::fs::write(&ruta, contenido).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ruta, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }
    Ok(ruta)
}

/// Espera a que aparezca el archivo anotado con al menos `lineas` líneas y lo lee.
fn esperar(anotado: &Path, lineas: usize) -> Result<String, String> {
    let inicio = Instant::now();
    while inicio.elapsed() < ESPERA_MAXIMA {
        if let Ok(texto) = std::fs::read_to_string(anotado) {
            if texto.lines().count() >= lineas {
                return Ok(texto);
            }
        }
        std::thread::sleep(INTERVALO);
    }
    let parcial = std::fs::read_to_string(anotado).unwrap_or_default();
    Err(format!(
        "la terminal no anotó {lineas} líneas en {} s; anotado: {parcial:?}",
        ESPERA_MAXIMA.as_secs()
    ))
}
