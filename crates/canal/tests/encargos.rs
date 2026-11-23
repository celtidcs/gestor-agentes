//! Barreras de la lectura de encargos (criterios V1 y V3 de la 3.6.0). Los casos salen de los
//! fallos vistos en la 3.5.1.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use canal::encargos::{destinatarios, encargos_para, es_para};
use canal::identidad::Identidad;

fn id(texto: &str) -> Identidad {
    Identidad::leer(texto).unwrap()
}

#[test]
fn un_destinatario_con_numero() {
    assert!(es_para(
        "2026-11-19 19:01:12. Para Codex 1, con copia al Director.",
        &id("Codex 1")
    ));
}

#[test]
fn la_instancia_1_vale_sin_numero() {
    assert!(es_para("Para Gemini: empieza F3", &id("Gemini 1")));
    assert!(es_para("## Para Codex — revisión", &id("Codex 1")));
}

#[test]
fn sin_numero_no_vale_para_otra_instancia() {
    assert!(!es_para("Para Claude: revisa esto", &id("Claude 2")));
}

#[test]
fn otro_numero_no_vale() {
    assert!(!es_para("Para Claude 1, revisa esto", &id("Claude 2")));
    assert!(!es_para("Para Claude 2, revisa esto", &id("Claude 1")));
}

#[test]
fn varios_destinatarios_despiertan_a_todos() {
    let linea = "## Para Claude 2 y Claude 1 — reparto";
    assert!(es_para(linea, &id("Claude 2")));
    assert!(es_para(linea, &id("Claude 1")));
    let otra = "Para Gemini 1, Codex 1 y Claude 1: empezad";
    assert!(es_para(otra, &id("Gemini 1")));
    assert!(es_para(otra, &id("Codex 1")));
    assert!(es_para(otra, &id("Claude 1")));
    assert!(!es_para(otra, &id("Claude 2")));
}

#[test]
fn lo_que_va_tras_los_dos_puntos_no_es_destinatario() {
    let linea = "Para Claude 2: pregunta a Codex 1 si puede";
    assert!(es_para(linea, &id("Claude 2")));
    assert!(!es_para(linea, &id("Codex 1")));
}

#[test]
fn los_parentesis_no_cortan_la_lista() {
    let linea = "Para Claude 2 (Jefe de proyecto) y Codex 1, con copia al Director.";
    assert!(es_para(linea, &id("Claude 2")));
    assert!(es_para(linea, &id("Codex 1")));
}

#[test]
fn las_negritas_no_estorban() {
    assert!(es_para("**Para Gemini 1**: plan aprobado", &id("Gemini 1")));
}

#[test]
fn para_en_minuscula_o_dentro_de_otra_palabra_no_es_encargo() {
    assert!(!es_para(
        "Esto es para Codex 1 cuando pueda",
        &id("Codex 1")
    ));
    assert!(!es_para("Comparar Codex 1 con otro", &id("Codex 1")));
    assert!(!es_para("SeparaPara Codex 1", &id("Codex 1")));
}

#[test]
fn la_copia_al_director_no_es_un_destinatario_agente() {
    assert_eq!(
        destinatarios("Para Codex 1, con copia al Director."),
        vec!["Codex 1".to_owned(), "con copia al Director".to_owned()]
    );
}

#[test]
fn el_bloque_acaba_en_la_linea_en_blanco() {
    let texto = "Para Codex 1. Haz esto:\n- uno\n- dos\n\nOtra cosa";
    let encargos = encargos_para(texto, &id("Codex 1"));
    assert_eq!(encargos.len(), 1);
    assert_eq!(encargos[0].primera_linea, "Para Codex 1. Haz esto:");
    let solo_bloque = encargos_para("Para Codex 1. Haz esto:\n- uno\n- dos", &id("Codex 1"));
    assert_eq!(encargos[0].huella, solo_bloque[0].huella);
}

#[test]
fn dos_encargos_seguidos_son_dos() {
    let texto = "Para Codex 1: primero\nPara Codex 1: segundo";
    assert_eq!(encargos_para(texto, &id("Codex 1")).len(), 2);
}

#[test]
fn el_mismo_bloque_en_otro_sitio_tiene_la_misma_huella() {
    let bloque = "2026-11-19 10:00:00. Para Codex 1: haz F5\n- detalle";
    let antes = format!("Cabecera\n\n{bloque}\n");
    let despues = format!("Cabecera\n\nPara Gemini 1: otra\n\n{bloque}\n\nfinal");
    let a = encargos_para(&antes, &id("Codex 1"));
    let b = encargos_para(&despues, &id("Codex 1"));
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(a[0].huella, b[0].huella);
}

#[test]
fn el_mismo_titulo_con_otra_hora_es_otro_encargo() {
    let a = encargos_para("2026-11-19 10:00:00. Para Codex 1: revisa", &id("Codex 1"));
    let b = encargos_para("2026-11-19 11:00:00. Para Codex 1: revisa", &id("Codex 1"));
    assert_ne!(a[0].huella, b[0].huella);
}

#[test]
fn los_finales_crlf_no_cambian_la_huella() {
    let a = encargos_para("Para Codex 1: x\n- y\n", &id("Codex 1"));
    let b = encargos_para("Para Codex 1: x\r\n- y\r\n", &id("Codex 1"));
    assert_eq!(a[0].huella, b[0].huella);
}
