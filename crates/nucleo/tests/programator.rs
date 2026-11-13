//! Barreras de la guía de Programator: la aplicación escribe en cada proyecto la guía
//! para trabajar con Programator, el núcleo dice cuándo se lee, y ni su guía publicada en el canal
//! ni los históricos cuentan como bandejas. Y lo que Programator escribe en el canal, una reserva en
//! estado.md y una intervención en su bandeja, pasa la revisión del canal sin avisos.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use chrono::NaiveDateTime;
use nucleo::plantillas::{self, DocumentoDeConsulta, EsqueletoGestor};
use nucleo::proyecto::catalogo;
use nucleo::proyecto::inicializador::{bandejas_sin_podar, es_bandeja};
use nucleo::proyecto::revision_del_canal::{self, Bandeja};

/// Reserva tal como la escribe Programator 0.12.0 (`linea_de_reserva` de su src/protocolo/estado.rs).
const RESERVA_DE_PROGRAMATOR: &str =
    "- Programator, 2026-11-13 11:40: src/main.rs — encargo de Claude 1";

/// Intervención tal como la publica Programator 0.12.0 (`Canal::publicar` de su
/// src/protocolo/canal.rs): latido, resumen, `LEÍDO:` y el cuerpo tras una raya.
const INTERVENCION_DE_PROGRAMATOR: &str = "\nLATIDO: 2026-11-13 11:40:12\nPropuesta entregada.\n\
LEÍDO: `claude.md` 2026-11-13 11:35:00\n\n---\n\nPropuesta entregada.\n";

/// estado.md de la plantilla con `reserva` anotada como lo hace Programator: tras el título y su
/// línea en blanco, quitando el «Ninguna.» del apartado.
fn estado_con_reserva_de_programator(reserva: &str) -> String {
    let plantilla = plantillas::esqueleto_gestor(EsqueletoGestor::Estado, None);
    let (antes, apartado) = plantilla.split_once("## Reservas activas\n\n").unwrap();
    let (apartado, despues) = apartado.split_once("## Mutaciones vivas").unwrap();
    assert_eq!(apartado.matches("Ninguna.\n").count(), 1);
    let apartado = apartado.replace("Ninguna.\n", "");
    format!("{antes}## Reservas activas\n\n{reserva}\n{apartado}## Mutaciones vivas{despues}")
}

fn bandeja_de_programator(intervencion: &str) -> Bandeja {
    Bandeja {
        nombre: "programator.md".to_owned(),
        texto: format!("# Programator\n{intervencion}"),
    }
}

fn hora(texto: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(texto, "%Y-%m-%d %H:%M:%S").unwrap()
}

#[test]
fn la_guia_de_programator_se_escribe_en_consulta() {
    let gestionados: Vec<&str> = catalogo::archivos_gestionados(None)
        .iter()
        .map(|a| a.ruta_relativa)
        .collect();

    assert!(gestionados.contains(&".gestor/consulta/programator.md"));
    assert_eq!(
        DocumentoDeConsulta::Programator.ruta_relativa(),
        ".gestor/consulta/programator.md"
    );
}

#[test]
fn el_nucleo_manda_leerla_antes_de_encargarle_algo() {
    let nucleo = plantillas::nucleo_comun();

    assert!(nucleo.contains(
        "| Antes de encargarle algo a Programator | `.gestor/consulta/programator.md` |"
    ));
}

#[test]
fn la_guia_dice_como_se_encarga_donde_entrega_y_quien_verifica() {
    let guia = DocumentoDeConsulta::Programator.texto();

    for imprescindible in [
        "empiece exactamente por `Para Programator: <nombre corto>`",
        "**tu propia bandeja**",
        "`escribir_propuesta`",
        "`.gestor/canal/programator.md`",
        "`.gestor/candidatos/programator/`",
        "**Quien encarga, verifica**",
        "`.gestor/canal/COMO-ENCARGAR-A-PROGRAMATOR.md`",
        "latido.json",
        "No los pidas desde otro proyecto.",
    ] {
        assert!(guia.contains(imprescindible), "falta {imprescindible}");
    }
}

#[test]
fn la_guia_publicada_y_los_historicos_no_son_bandejas() {
    for nombre in [
        "claude.md",
        "claude2.md",
        "codex.md",
        "gemini.md",
        "programator.md",
    ] {
        assert!(es_bandeja(nombre), "{nombre} debería ser una bandeja");
    }
    for nombre in [
        "estado.md",
        "COMO-ENCARGAR-A-PROGRAMATOR.md",
        "historico-programator.md",
        "Histórico-claude.md",
        "notas.txt",
    ] {
        assert!(!es_bandeja(nombre), "{nombre} no es una bandeja");
    }
}

#[test]
fn el_aviso_de_tamano_no_salta_con_la_guia_ni_con_un_historico() {
    let limite = 100;
    let entradas = vec![
        ("COMO-ENCARGAR-A-PROGRAMATOR.md".to_owned(), 14_000),
        ("historico-programator.md".to_owned(), 12_000),
        ("codex.md".to_owned(), 44_000),
    ];

    assert_eq!(
        bandejas_sin_podar(&entradas, limite),
        vec![("codex.md".to_owned(), 44_000)]
    );
}

#[test]
fn lo_que_escribe_programator_en_el_canal_no_da_avisos() {
    let estado = estado_con_reserva_de_programator(RESERVA_DE_PROGRAMATOR);
    let bandejas = [bandeja_de_programator(INTERVENCION_DE_PROGRAMATOR)];

    let avisos = revision_del_canal::revisar(&estado, &bandejas, hora("2026-11-13 12:00:00"), 12);
    let resumen = revision_del_canal::resumir(&estado, &bandejas);

    assert_eq!(avisos, Vec::new());
    assert_eq!(resumen.reservas, 1);
    assert_eq!(resumen.latidos.len(), 1);
    assert_eq!(resumen.latidos[0].identificador, "programator");
    assert_eq!(resumen.latidos[0].momento, hora("2026-11-13 11:40:12"));
}
