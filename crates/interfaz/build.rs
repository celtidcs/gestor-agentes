//! Script de compilación de gestor-agentes. En Windows incrusta gestor-claude.ico como
//! recurso del ejecutable, para que el Explorador y la barra de tareas muestren el icono de la
//! aplicación. En cualquier otro sistema no hace nada.
//!
//! El destino se lee de CARGO_CFG_TARGET_OS y no con #[cfg(windows)], porque en un script de
//! compilación cfg describe la máquina que compila, no aquella para la que se compila.

use std::env;

/// Icono del ejecutable, el mismo de la 2.3.2 (siete tamaños, de 16 a 256 píxeles).
const ICONO_DEL_EJECUTABLE: &str = "assets/gestor-claude.ico";

fn main() {
    println!("cargo:rerun-if-changed={ICONO_DEL_EJECUTABLE}");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut recurso = winresource::WindowsResource::new();
    recurso.set_icon(ICONO_DEL_EJECUTABLE);
    // Sin rc.exe (el del Windows SDK, que winresource busca en el registro) el ejecutable sale
    // sin icono, pero sale: el icono no justifica dejar a nadie sin compilar. Cargo muestra el
    // aviso al final de la compilación.
    if let Err(error) = recurso.compile() {
        println!(
            "cargo:warning=el ejecutable queda sin icono: no se pudo compilar el recurso ({error})"
        );
    }
}
