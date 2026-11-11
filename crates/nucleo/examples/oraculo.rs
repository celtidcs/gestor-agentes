//! Oráculo de pruebas: abre una carpeta con el inicializador real, con el reloj fijado en
//! 2026-11-10 12:00:00, para comparar byte a byte el árbol resultante con el que genera otra
//! versión de la aplicación.
//!
//! Uso: oraculo <carpeta-del-proyecto> [Claude|Codex|Gemini] [instancia]. Escribe el informe en
//! la salida estándar, con el salto de línea del sistema como Console.WriteLine. Devuelve 0 si el
//! informe no tiene errores, 1 si los tiene y 2 si los argumentos no son válidos.

use std::path::PathBuf;
use std::process::ExitCode;

use nucleo::agentes::{IdentidadAgente, TipoAgente};
use nucleo::git::{EjecutorDelSistema, ServicioGit};
use nucleo::proyecto::inicializador::{ConfiguracionDeInstrucciones, InicializadorDeProyecto};
use nucleo::proyecto::localizador::LocalizadorDeInstrucciones;
use nucleo::proyecto::momento::{Momento, RelojFijo};

/// Salto de línea de Console.WriteLine en .NET (Environment.NewLine).
const SALTO: &str = if cfg!(windows) { "\r\n" } else { "\n" };

/// Configuración sin instrucciones personalizadas antiguas, como en el oráculo de la 2.3.2.
struct SinInstruccionesAntiguas;

impl ConfiguracionDeInstrucciones for SinInstruccionesAntiguas {
    fn leer_instrucciones_antiguas(&self) -> Option<String> {
        None
    }
}

fn main() -> ExitCode {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    let (carpeta, lanzada) = match argumentos.as_slice() {
        [carpeta] => (carpeta, None),
        [carpeta, tipo, instancia] => match identidad(tipo, instancia) {
            Some(identidad) => (carpeta, Some(identidad)),
            None => {
                eprintln!("Agente no válido: {tipo} {instancia}");
                return ExitCode::from(2);
            }
        },
        _ => {
            eprintln!("Uso: oraculo <carpeta-del-proyecto> [Claude|Codex|Gemini] [instancia]");
            return ExitCode::from(2);
        }
    };

    let Ok(momento) = Momento::new(2026, 11, 10, 12, 0, 0) else {
        return ExitCode::from(2);
    };
    let raiz = std::path::absolute(carpeta).unwrap_or_else(|_| PathBuf::from(carpeta));
    let ejecutor = EjecutorDelSistema::default();
    let reloj = RelojFijo(momento);
    let informe = InicializadorDeProyecto::new(
        &SinInstruccionesAntiguas,
        &LocalizadorDeInstrucciones,
        ServicioGit::new(&ejecutor),
        &reloj,
    )
    .inicializar(&raiz, lanzada);

    print!("{}{SALTO}", informe.componer_texto());
    if informe.errores().is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Identidad a partir del tipo exacto y la instancia, o nada si no son válidos.
fn identidad(tipo: &str, instancia: &str) -> Option<IdentidadAgente> {
    let tipo = TipoAgente::TODOS.into_iter().find(|t| t.nombre() == tipo)?;
    IdentidadAgente::new(tipo, instancia.trim().parse().ok()?).ok()
}
