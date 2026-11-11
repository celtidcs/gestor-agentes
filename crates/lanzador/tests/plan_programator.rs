//! Barreras del plan de Programator en Windows. Las aplicaciones de escritorio de Codex y de
//! Google Antigravity ya no se abren, así que no tienen pruebas.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default,
    missing_docs
)]

mod comun;

use std::path::PathBuf;

use comun::{peticion, plan, textos, variables, EntornoDeEnsayo};
use lanzador::{ErrorDeLanzamiento, Herramienta, Sistema};

fn par(n: &str, v: &str) -> (String, String) {
    (n.to_owned(), v.to_owned())
}

// ── Programator ────────────────────────────────────────────

/// En Windows, Programator se abre directamente en una consola nueva. En Linux va dentro de un
/// emulador de terminal: su prueba está en tests/plan_terminal.rs.
#[test]
fn programator_recibe_ruta_del_proyecto_y_trabaja_en_la_carpeta_de_su_ejecutable() {
    let carpeta = PathBuf::from("App_Portables").join("Programator");
    let ejecutable = carpeta.join("programator.exe");
    let mut p = peticion(Herramienta::Programator);
    p.identidad = "Programator 1".to_owned();
    p.ejecutable = Some(ejecutable.clone());
    let entorno = EntornoDeEnsayo::default().con_archivo(ejecutable.clone());
    {
        let plan = plan(&p, Sistema::Windows, &entorno);
        assert_eq!(plan.programa, ejecutable);
        assert_eq!(textos(&plan.argumentos), ["--ruta", "/proyectos/MMCelt"]);
        assert_eq!(plan.carpeta, carpeta);
        assert_eq!(variables(&plan), [par("GESTOR_AGENTE", "Programator 1")]);
    }
}

#[test]
fn programator_sin_ejecutable_es_perfil_incompleto() {
    let resultado = lanzador::planificar(
        &peticion(Herramienta::Programator),
        Sistema::Windows,
        &EntornoDeEnsayo::default(),
    );
    assert!(
        matches!(resultado, Err(ErrorDeLanzamiento::PerfilIncompleto { ref identidad, .. }) if identidad == "Claude 2"),
        "{resultado:?}"
    );
}

#[test]
fn programator_con_ejecutable_inexistente_no_se_encuentra() {
    let mut p = peticion(Herramienta::Programator);
    p.ejecutable = Some(PathBuf::from("no").join("programator.exe"));
    let resultado = lanzador::planificar(&p, Sistema::Windows, &EntornoDeEnsayo::default());
    assert!(
        matches!(resultado, Err(ErrorDeLanzamiento::NoEncontrada { ref herramienta, .. }) if herramienta == "Programator"),
        "{resultado:?}"
    );
}

#[test]
fn programator_con_ejecutable_sin_carpeta_es_perfil_incompleto() {
    let mut p = peticion(Herramienta::Programator);
    p.ejecutable = Some(PathBuf::from("programator.exe"));
    let entorno = EntornoDeEnsayo::default().con_archivo("programator.exe");
    let resultado = lanzador::planificar(&p, Sistema::Windows, &entorno);
    assert!(
        matches!(resultado, Err(ErrorDeLanzamiento::PerfilIncompleto { .. })),
        "{resultado:?}"
    );
}

#[test]
fn los_errores_se_explican_al_usuario() {
    let error = ErrorDeLanzamiento::PerfilIncompleto {
        identidad: "Programator 1".to_owned(),
        detalle: "falta la ruta del ejecutable de Programator".to_owned(),
    };
    assert_eq!(
        error.to_string(),
        "el perfil de Programator 1 no está completo: falta la ruta del ejecutable de Programator"
    );
}
