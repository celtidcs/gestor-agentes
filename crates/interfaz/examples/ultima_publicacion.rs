//! Consulta real a la API de GitHub, para comprobar en una máquina de pruebas que la consulta de
//! versión funciona contra el servicio de verdad. No es una prueba automática, porque necesita red.
//!
//! Uso: cargo run -p gestor-agentes --example ultima_publicacion [versión instalada]
//! Con una versión instalada antigua, como 0.0.1, tiene que salir la última publicada.

#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use gestor_agentes::actualizaciones::{self, Consulta, ConsultorDePublicaciones};
use gestor_agentes::sistema::ConsultorGithub;
use nucleo::actualizaciones as decision;
use nucleo::ajustes::Ajustes;

fn main() {
    let instalada = std::env::args()
        .nth(1)
        .unwrap_or_else(|| actualizaciones::VERSION_INSTALADA.to_owned());
    let ajustes = Ajustes::default();
    let consulta = Consulta::desde(&ajustes).expect("activada por defecto");
    let direccion =
        decision::direccion_de_la_consulta(ajustes.repositorio_de_actualizaciones()).unwrap();
    let cuerpo = ConsultorGithub.consultar(
        &direccion,
        &decision::agente_de_usuario(&instalada),
        ajustes.tiempo_maximo_actualizacion(),
    );
    println!("DIRECCION: {direccion}");
    println!(
        "RESPUESTA: {}",
        if cuerpo.is_some() {
            "recibida"
        } else {
            "ninguna"
        }
    );
    let leida = cuerpo
        .as_deref()
        .and_then(|c| decision::leer_publicacion(c, ajustes.repositorio_de_actualizaciones()));
    println!(
        "PUBLICADA: {}",
        leida.map_or_else(
            || "ninguna".to_owned(),
            |p| format!("{} {}", p.version, p.enlace)
        )
    );
    match actualizaciones::comprobar(&consulta, &instalada, &ConsultorGithub) {
        Some(p) => println!(
            "AVISO: versión nueva {} para la instalada {instalada}",
            p.version
        ),
        None => println!("AVISO: ninguno para la instalada {instalada}"),
    }
}
