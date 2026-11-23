//! Barreras de MH3 (3.7.0): un encargo ya acusado en la bandeja del agente no vuelve a avisarse.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use canal::acuses;
use canal::encargos;
use canal::identidad::Identidad;

/// Bandeja de Codex 1 del 2026-11-22, tal como acusó PING-1.
const BANDEJA_DE_CODEX: &str = "# Bandeja de Codex 1\n\n\
    LATIDO: 2026-11-22 13:27:10\n\n\
    PING-1 recibido 13:27:10\n";

#[test]
fn la_etiqueta_es_la_primera_palabra_en_mayusculas_tras_los_destinatarios() {
    assert_eq!(
        acuses::etiqueta("Para Codex 1: PING-1").as_deref(),
        Some("PING-1")
    );
    assert_eq!(
        acuses::etiqueta("Para Gemini 1. Petición VERIF-360D: verifica F17").as_deref(),
        Some("VERIF-360D")
    );
    assert_eq!(
        acuses::etiqueta("Para Codex 1: VC6-1").as_deref(),
        Some("VC6-1")
    );
    assert_eq!(acuses::etiqueta("Para Codex 1: revisa esto"), None);
    assert_eq!(acuses::etiqueta("Para Codex 1: T4"), None);
}

#[test]
fn el_acuse_vale_en_cualquier_forma_y_despues_del_latido_del_encargo() {
    assert!(acuses::acusado(
        "PING-1",
        Some("2026-11-22 13:17:48"),
        BANDEJA_DE_CODEX
    ));
    let mayusculas = "LATIDO: 2026-11-22 14:50:26\n\nRECIBIDO 14:50:22: VC6-1 recibido.\n";
    assert!(acuses::acusado(
        "VC6-1",
        Some("2026-11-22 14:48:25"),
        mayusculas
    ));
}

#[test]
fn un_acuse_anterior_al_encargo_u_otra_etiqueta_no_valen() {
    assert!(!acuses::acusado(
        "PING-1",
        Some("2026-11-22 13:30:00"),
        BANDEJA_DE_CODEX
    ));
    assert!(!acuses::acusado(
        "PING-2",
        Some("2026-11-22 13:17:48"),
        BANDEJA_DE_CODEX
    ));
    let sin_palabra = "LATIDO: 2026-11-22 14:00:00\n\nPING-1 lo miro luego\n";
    assert!(!acuses::acusado(
        "PING-1",
        Some("2026-11-22 13:17:48"),
        sin_palabra
    ));
}

#[test]
fn cada_encargo_lleva_el_latido_de_su_intervencion() {
    let yo = Identidad::leer("Codex 1").unwrap();
    let bandeja = "LATIDO: 2026-11-22 13:17:48\n\nPara Codex 1: PING-1\n- Responde.\n";
    let leidos = encargos::encargos_para(bandeja, &yo);
    assert_eq!(leidos.len(), 1);
    assert_eq!(leidos[0].latido.as_deref(), Some("2026-11-22 13:17:48"));
}
