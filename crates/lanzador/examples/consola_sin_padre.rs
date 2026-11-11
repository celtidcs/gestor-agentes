//! Prueba de la terminal de Claude abierta desde un proceso sin consola, como lo es Gestor
//! Agentes al abrirse desde el menú Inicio.
//!
//! En Windows se compila como aplicación de ventana, así que no tiene consola ni manejadores
//! estándar. Lanza con `ejecutar` un plan real de Claude cuyo «ejecutable» es un guion que anota en
//! un archivo si la entrada y la salida de la PowerShell nueva están redirigidas, y el título de
//! su ventana, y después la cierra. El propio ejemplo escribe en otro archivo si el lanzamiento
//! salió bien, porque no tiene consola donde escribir.
//!
//! Uso, desde una sesión sin tuberías heredadas (por ejemplo, con Start-Process):
//! consola_sin_padre.exe <carpeta de trabajo>
//! Deja en la carpeta anotado.txt (IsOutputRedirected, IsInputRedirected y título, una línea cada
//! uno) y lanzamiento.txt (OK o el error).

#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use lanzador::{ejecutar, planificar, EntornoReal, Herramienta, PeticionDeLanzamiento, Sistema};

fn main() {
    let Some(raiz) = std::env::args_os().nth(1).map(PathBuf::from) else {
        return;
    };
    let resultado = lanzar(&raiz);
    let texto = match resultado {
        Ok(()) => "OK".to_owned(),
        Err(motivo) => motivo,
    };
    let _ = std::fs::write(raiz.join("lanzamiento.txt"), texto);
}

fn lanzar(raiz: &Path) -> Result<(), String> {
    let proyecto = raiz.join("Proyecto de prueba");
    std::fs::create_dir_all(&proyecto).map_err(|e| e.to_string())?;
    let anotado = raiz.join("anotado.txt");
    let destino = anotado
        .to_str()
        .ok_or("ruta no Unicode")?
        .replace('\'', "''");
    let guion = raiz.join("falso-claude.ps1");
    std::fs::write(
        &guion,
        format!(
            "$lineas = @([Console]::IsOutputRedirected, [Console]::IsInputRedirected, \
             [Console]::Title)\r\n\
             [IO.File]::WriteAllText('{destino}', ($lineas -join \"`n\"))\r\n\
             [Environment]::Exit(0)\r\n"
        ),
    )
    .map_err(|e| e.to_string())?;
    let peticion = PeticionDeLanzamiento {
        herramienta: Herramienta::Claude,
        identidad: "Claude 9".to_owned(),
        carpeta_proyecto: proyecto,
        nombre_proyecto: "Proyecto de prueba".to_owned(),
        directorio_configuracion: None,
        ejecutable: Some(guion),
        argumentos_extra: Vec::new(),
    };
    match planificar(&peticion, Sistema::actual(), &EntornoReal) {
        Ok(plan) => ejecutar(&plan).map_err(|e| e.to_string()),
        Err(error) => Err(error.to_string()),
    }
}
