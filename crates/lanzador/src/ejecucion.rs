//! Ejecución de un plan de lanzamiento ya decidido.

use std::process::Command;

use crate::{ErrorDeLanzamiento, PlanDeLanzamiento};

/// Véase [`crate::ejecutar`].
pub(crate) fn ejecutar(plan: &PlanDeLanzamiento) -> Result<(), ErrorDeLanzamiento> {
    for carpeta in &plan.carpetas_a_crear {
        std::fs::create_dir_all(carpeta).map_err(|origen| ErrorDeLanzamiento::Carpeta {
            carpeta: carpeta.clone(),
            origen,
        })?;
    }
    let mut orden = Command::new(&plan.programa);
    orden
        .args(&plan.argumentos)
        .current_dir(&plan.carpeta)
        .envs(plan.variables.iter().map(|(nombre, valor)| (nombre, valor)));
    con_consola_nueva(&mut orden);
    let mut hijo = orden
        .spawn()
        .map_err(|origen| ErrorDeLanzamiento::Proceso {
            programa: plan.programa.clone(),
            origen,
        })?;
    // Se recoge el proceso cuando termine, para que no quede como zombi; su resultado no importa,
    // porque la aplicación no espera a los agentes. Si wait falla, se ignora a propósito: este hilo
    // solo existe para recogerlo, nadie espera su respuesta y no hay ninguna acción posible.
    std::thread::spawn(move || {
        let _ = hijo.wait();
    });
    Ok(())
}

/// En Windows, el proceso se crea con una consola nueva y visible: todo lo que se abre es un
/// programa de consola que maneja el usuario (la terminal de PowerShell o Programator).
#[cfg(windows)]
fn con_consola_nueva(orden: &mut Command) {
    use std::os::windows::process::CommandExt;
    /// CREATE_NEW_CONSOLE, de la documentación de Process Creation Flags de Microsoft.
    const CONSOLA_NUEVA: u32 = 0x0000_0010;
    orden.creation_flags(CONSOLA_NUEVA);
}

/// En Linux no hay consola que crear: la ventana la abre el propio emulador de terminal.
#[cfg(not(windows))]
const fn con_consola_nueva(_orden: &mut Command) {}
