//! Punto de entrada de gestor-vigilante: lee las opciones, la identidad y la carpeta .gestor/, y
//! pasa el trabajo a [`vigilante::ejecucion`].

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use canal::identidad::Identidad;
use vigilante::ejecucion::{self, Contexto, CODIGO_DE_USO};
use vigilante::opciones;
use vigilante::procesos::ProcesosDelSistema;

/// Variable con la identidad que pone Gestor Agentes al lanzar cada agente.
const VARIABLE_DE_IDENTIDAD: &str = "GESTOR_AGENTE";

fn main() -> ExitCode {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    let codigo = match preparar(&argumentos) {
        Ok((contexto, opciones)) => {
            let mut salida = io::stdout();
            ejecucion::ejecutar(
                &contexto,
                &opciones,
                &mut ProcesosDelSistema::new(),
                &mut salida,
            )
        }
        Err(motivo) => {
            let _ = writeln!(io::stderr(), "VIGILANTE: {motivo}");
            CODIGO_DE_USO
        }
    };
    ExitCode::from(u8::try_from(codigo).unwrap_or(1))
}

/// Opciones, identidad y carpeta .gestor/ comprobadas.
fn preparar(argumentos: &[String]) -> Result<(Contexto, opciones::Opciones), String> {
    let opciones = opciones::interpretar(argumentos).map_err(|error| error.to_string())?;
    let texto = opciones
        .agente
        .clone()
        .or_else(|| std::env::var(VARIABLE_DE_IDENTIDAD).ok())
        .unwrap_or_default();
    if texto.trim().is_empty() {
        return Err(format!(
            "{VARIABLE_DE_IDENTIDAD} está vacía; pasa tu identidad con --agente \"Claude 2\""
        ));
    }
    let identidad = Identidad::leer(&texto).map_err(|error| error.to_string())?;
    let gestor = match opciones.gestor.clone() {
        Some(gestor) => gestor,
        None => carpeta_gestor_del_ejecutable()?,
    };
    if !gestor.join("canal").is_dir() {
        return Err(format!("no existe {}", gestor.join("canal").display()));
    }
    Ok((Contexto { gestor, identidad }, opciones))
}

/// .gestor/ a partir de la ubicación del ejecutable, que Gestor Agentes deja en
/// .gestor/vigilante/.
fn carpeta_gestor_del_ejecutable() -> Result<PathBuf, String> {
    let ejecutable = std::env::current_exe().map_err(|error| error.to_string())?;
    ejecutable
        .parent()
        .and_then(|vigilante| vigilante.parent())
        .map(PathBuf::from)
        .ok_or_else(|| String::from("no sé en qué proyecto estoy; pasa --gestor"))
}
