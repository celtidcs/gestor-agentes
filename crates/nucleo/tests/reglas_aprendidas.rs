//! Barreras de las reglas aprendidas: una trampa reproducible deja una regla de una línea en
//! «## Reglas aprendidas» del archivo propio del proyecto, sin repetirla, y el Jefe las agrupa
//! cuando pasan de treinta. Lo manda metodo.md; verificacion.md y el núcleo remiten a él.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use nucleo::plantillas::{self, DocumentoDeConsulta};

/// Encabezado del apartado en el archivo propio del proyecto.
const APARTADO: &str = "## Reglas aprendidas";

/// Remisión exacta al apartado de metodo.md que lo explica.
const REMISION: &str = "`.gestor/consulta/metodo.md`, apartado 3";

#[test]
fn metodo_explica_cuando_y_como_se_anota_una_regla_aprendida() {
    let metodo = DocumentoDeConsulta::Metodo.texto();
    let apartado = metodo
        .split("\n## ")
        .find(|a| a.starts_with("3. Reglas aprendidas"))
        .expect("metodo.md tiene el apartado 3, Reglas aprendidas");

    for imprescindible in [
        "**trampa reproducible**",
        "si ya está. Si está, no la repitas",
        "**una sola línea**",
        "pasa de 30 reglas, el Jefe las agrupa",
        APARTADO,
    ] {
        assert!(apartado.contains(imprescindible), "falta {imprescindible}");
    }
}

#[test]
fn verificacion_y_el_nucleo_remiten_al_apartado() {
    assert!(DocumentoDeConsulta::Verificacion.texto().contains(REMISION));
    assert!(plantillas::nucleo_comun().contains(REMISION));
}

#[test]
fn el_archivo_propio_nace_con_el_apartado_de_reglas_aprendidas() {
    let propio = plantillas::instrucciones_del_proyecto();

    assert!(propio.contains(&format!("\n{APARTADO}\n")), "{propio}");
}
