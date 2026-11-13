//! Prueba de extremo a extremo del lanzador de Claude.
//!
//! Ejecuta el plan real, con el entorno real y `ejecutar`, en una carpeta de proyecto con un
//! nombre difícil. En lugar de Claude Code abre un pequeño guion que anota lo que ve dentro de la
//! terminal (título en Windows, GESTOR_AGENTE, CLAUDE_CONFIG_DIR y carpeta actual) en un archivo
//! y cierra la terminal. Después compara lo anotado con lo esperado.
//!
//! Uso: cargo run -p lanzador --example extremo_a_extremo
//! Sale con código 0 si todo coincide y con 1 si no. En Linux necesita un servidor X (DISPLAY),
//! por ejemplo Xvfb, y uno de los emuladores de terminal que admite el lanzador.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use lanzador::{ejecutar, planificar, EntornoReal, Herramienta, PeticionDeLanzamiento, Sistema};

/// Identidad de prueba.
const IDENTIDAD: &str = "Claude 9";
/// Nombre de la carpeta del proyecto: espacios, acentos, comillas rectas y tipográficas, punto y
/// coma y $, para que el extremo a extremo pase también por los escapes.
const NOMBRE_PROYECTO: &str = "Proyecto ’de’ prueba; $x 'ñ'";
/// Tiempo máximo de espera a que la terminal anote lo que ve.
const ESPERA_MAXIMA: Duration = Duration::from_secs(60);
/// Intervalo entre comprobaciones del archivo anotado.
const INTERVALO: Duration = Duration::from_millis(250);

fn main() -> ExitCode {
    match comprobar() {
        Ok(()) => {
            println!("EXTREMO A EXTREMO: OK");
            ExitCode::SUCCESS
        }
        Err(motivo) => {
            println!("EXTREMO A EXTREMO: FALLO: {motivo}");
            ExitCode::FAILURE
        }
    }
}

fn comprobar() -> Result<(), String> {
    let raiz = std::env::temp_dir().join(format!("lanzador-e2e-{}", std::process::id()));
    let resultado = comprobar_en(&raiz);
    let _ = std::fs::remove_dir_all(&raiz);
    resultado
}

fn comprobar_en(raiz: &Path) -> Result<(), String> {
    let proyecto = raiz.join(NOMBRE_PROYECTO);
    let configuracion = raiz.join("configuracion-claude9");
    let anotado = raiz.join("anotado.txt");
    std::fs::create_dir_all(&proyecto).map_err(|e| e.to_string())?;
    let guion = escribir_guion(raiz, &anotado)?;

    let peticion = PeticionDeLanzamiento {
        herramienta: Herramienta::Claude,
        identidad: IDENTIDAD.to_owned(),
        carpeta_proyecto: proyecto.clone(),
        nombre_proyecto: NOMBRE_PROYECTO.to_owned(),
        directorio_configuracion: Some(configuracion.clone()),
        ejecutable: Some(guion),
        argumentos_extra: Vec::new(),
        mensaje_inicial: None,
    };
    let plan = match planificar(&peticion, Sistema::actual(), &EntornoReal) {
        Ok(plan) => plan,
        Err(error) => return Err(error.to_string()),
    };
    println!("programa: {}", plan.programa.display());
    ejecutar(&plan).map_err(|e| e.to_string())?;

    let texto = esperar(&anotado)?;
    let lineas: Vec<&str> = texto.lines().collect();
    let esperado_titulo = format!("{IDENTIDAD} · {NOMBRE_PROYECTO}");
    let esperadas = [
        ("título", esperado_titulo.as_str()),
        ("GESTOR_AGENTE", IDENTIDAD),
        ("CLAUDE_CONFIG_DIR", ruta_texto(&configuracion)?),
        ("carpeta", ruta_texto(&proyecto)?),
    ];
    for (indice, (nombre, esperado)) in esperadas.iter().enumerate() {
        let visto = lineas.get(indice).copied().unwrap_or("");
        if nombre == &"título" && cfg!(not(windows)) {
            println!("título: no se puede leer desde dentro de la terminal en Linux; se omite");
            continue;
        }
        if visto != *esperado {
            return Err(format!(
                "{nombre}: se esperaba «{esperado}» y la terminal vio «{visto}»"
            ));
        }
        println!("{nombre}: «{visto}» correcto");
    }
    if !configuracion.is_dir() {
        return Err("no se creó la carpeta de configuración".to_owned());
    }
    println!("carpeta de configuración creada: correcto");
    Ok(())
}

fn ruta_texto(ruta: &Path) -> Result<&str, String> {
    ruta.to_str()
        .ok_or_else(|| format!("ruta no Unicode: {}", ruta.display()))
}

/// Escribe el guion que hace de Claude Code: anota título, variables y carpeta, una por línea, y
/// cierra la terminal.
fn escribir_guion(raiz: &Path, anotado: &Path) -> Result<PathBuf, String> {
    let destino = ruta_texto(anotado)?.replace('\'', "''");
    let (nombre, contenido) = if cfg!(windows) {
        (
            "falso-claude.ps1",
            format!(
                "$lineas = @([Console]::Title, $env:GESTOR_AGENTE, $env:CLAUDE_CONFIG_DIR, \
                 (Get-Location).ProviderPath)\r\n\
                 [IO.File]::WriteAllText('{destino}', ($lineas -join \"`n\"))\r\n\
                 [Environment]::Exit(0)\r\n"
            ),
        )
    } else {
        (
            "falso-claude.sh",
            format!(
                "#!/bin/sh\n\
                 printf '%s\\n' '-' \"$GESTOR_AGENTE\" \"$CLAUDE_CONFIG_DIR\" \"$(pwd)\" > '{}'\n\
                 kill \"$PPID\"\n",
                ruta_texto(anotado)?.replace('\'', "'\\''")
            ),
        )
    };
    let ruta = raiz.join(nombre);
    std::fs::write(&ruta, contenido).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ruta, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }
    Ok(ruta)
}

/// Espera a que aparezca el archivo anotado y lo lee.
fn esperar(anotado: &Path) -> Result<String, String> {
    let inicio = Instant::now();
    while inicio.elapsed() < ESPERA_MAXIMA {
        if let Ok(texto) = std::fs::read_to_string(anotado) {
            if texto.lines().count() >= 4 {
                return Ok(texto);
            }
        }
        std::thread::sleep(INTERVALO);
    }
    Err(format!(
        "la terminal no anotó nada en {} s",
        ESPERA_MAXIMA.as_secs()
    ))
}
