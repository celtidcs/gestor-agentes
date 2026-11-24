//! Barrera de lo que consta en cada verificación: la consulta de verificación que Gestor escribe
//! en cada proyecto pide la orden literal con sus horas y su código, todos los criterios, ningún
//! valor copiado de otro agente, las puertas por manifiesto, las VM en lugar del equipo del
//! Director y las señales de alarma para quien acepta.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use nucleo::plantillas::DocumentoDeConsulta;

/// Título del apartado, tal como empieza tras «## ».
const TITULO: &str = "7. Qué consta en cada verificación";

/// Una frase de cada uno de los seis puntos del apartado.
const PUNTOS: [&str; 6] = [
    "la orden literal, la hora de inicio y de fin, el código de salida",
    "Se ejecutan todos los criterios publicados, no solo las cuatro puertas",
    "Ningún valor esperado se copia de la salida de otro agente",
    "entrando en la carpeta de cada uno: una orden en la raíz no los cubre",
    "nunca el equipo del Director",
    "Señales de alarma para quien acepta",
];

fn apartado() -> String {
    DocumentoDeConsulta::Verificacion
        .texto()
        .split("\n## ")
        .find(|a| a.starts_with(TITULO))
        .map(str::to_owned)
        .expect("verificacion.md tiene el apartado «Qué consta en cada verificación»")
}

#[test]
fn verificacion_pide_lo_que_debe_constar() {
    let apartado = apartado();
    for punto in PUNTOS {
        assert!(apartado.contains(punto), "falta «{punto}»");
    }
}

#[test]
fn el_apartado_no_nombra_a_ningun_agente() {
    let apartado = apartado();
    for nombre in ["Claude", "Codex", "Gemini", "agy", "Programator"] {
        assert!(!apartado.contains(nombre), "nombra a {nombre}");
    }
}
