//! Consulta real a la API de GitHub, para comprobar en una máquina de pruebas que el aviso de
//! versión funciona contra el servicio de verdad, con el módulo aviso_de_version y la biblioteca
//! aviso-version-github. No es una prueba automática, porque necesita red.
//!
//! Uso: cargo run -p gestor-agentes --example ultima_publicacion [versión instalada]
//! Sin argumento, la instalada es la de esta compilación. Con una antigua, como 0.0.1, tiene que
//! salir un aviso con la última publicada; con la última o una posterior, ninguno. La última de
//! Programator sale siempre. Si una comprobación falla, se dice junto a su resultado, y la línea
//! completa del registro va a la salida de errores.

#![allow(missing_docs)]

use std::fmt;
use std::sync::{Arc, Mutex};

use gestor_agentes::aviso_de_version::{self, Registro, VERSION_INSTALADA};
use nucleo::ajustes::Ajustes;

/// Un registro que escribe como el real y además guarda los fallos, para decirlos en el resultado.
fn registro_que_cuenta() -> (Registro, Arc<Mutex<Vec<String>>>) {
    let fallos = Arc::new(Mutex::new(Vec::new()));
    let destino = Arc::clone(&fallos);
    let real = aviso_de_version::registro_real();
    let registro: Registro = Arc::new(move |fallo: &dyn fmt::Display| {
        real(fallo);
        if let Ok(mut fallos) = destino.lock() {
            fallos.push(fallo.to_string());
        }
    });
    (registro, fallos)
}

/// « (fallo: …)» si la comprobación registró alguno, o nada.
fn fallos_de(fallos: &Mutex<Vec<String>>) -> String {
    fallos
        .lock()
        .map(|fallos| fallos.join("; "))
        .ok()
        .filter(|texto| !texto.is_empty())
        .map_or_else(String::new, |texto| format!(" (fallo: {texto})"))
}

fn main() {
    let instalada = std::env::args()
        .nth(1)
        .unwrap_or_else(|| VERSION_INSTALADA.to_owned());
    let ajustes = Ajustes::default();
    println!("REPOSITORIO: {}", ajustes.repositorio_de_actualizaciones());
    let (registro, fallos) = registro_que_cuenta();
    let avisos = aviso_de_version::arrancar(
        &ajustes,
        instalada.clone(),
        aviso_de_version::consulta_real,
        registro,
        Box::new(|| {}),
    );
    match avisos.recv() {
        Ok(aviso) => println!(
            "AVISO: versión nueva {} para la instalada {instalada}: {}",
            aviso.version, aviso.enlace
        ),
        Err(_) => println!(
            "AVISO: ninguno para la instalada {instalada}{}",
            fallos_de(&fallos)
        ),
    }
    let (registro, fallos) = registro_que_cuenta();
    let programator = aviso_de_version::arrancar_programator(
        &ajustes,
        aviso_de_version::consulta_real,
        registro,
        Box::new(|| {}),
    );
    println!(
        "PROGRAMATOR PUBLICADA: {}",
        programator.recv().map_or_else(
            |_| format!("ninguna{}", fallos_de(&fallos)),
            |v| v.to_string()
        )
    );
}
