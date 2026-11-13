//! Barreras de la revisión del canal: avisa de los apartados que faltan en
//! estado.md, de las mutaciones vivas, de las reservas mal escritas, de dos agentes con la misma
//! ruta y de las reservas de un agente sin latido reciente; y el inicializador pone esos avisos en
//! su informe. Con la plantilla recién creada no avisa de nada.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use chrono::NaiveDateTime;
use comun::{CarpetaTemporal, Entorno};
use nucleo::plantillas::{self, EsqueletoGestor};
use nucleo::proyecto::revision_del_canal::{self, AvisoDelCanal, Bandeja};

const LIMITE: u64 = 12;

fn hora(texto: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(texto, "%Y-%m-%d %H:%M:%S").unwrap()
}

fn bandeja(nombre: &str, latido: &str) -> Bandeja {
    Bandeja {
        nombre: nombre.to_owned(),
        texto: format!(
            "# Bandeja\n\nLATIDO: 2026-11-01 08:00\n\nViejo.\n\nLATIDO: {latido}\n\nNuevo.\n"
        ),
    }
}

/// estado.md de la plantilla con estas líneas en sus reservas y en sus mutaciones vivas.
fn estado(reservas: &[&str], mutaciones: &str) -> String {
    let plantilla = plantillas::esqueleto_gestor(EsqueletoGestor::Estado, None);
    let reservas = if reservas.is_empty() {
        "Ninguna.".to_owned()
    } else {
        reservas.join("\n")
    };
    plantilla
        .replace(
            "## Reservas activas\n\n",
            &format!("## Reservas activas\n\n{reservas}\n\n"),
        )
        .replace(
            "## Mutaciones vivas\n\nNinguna.",
            &format!("## Mutaciones vivas\n\n{mutaciones}"),
        )
}

fn revisar(estado: &str, bandejas: &[Bandeja]) -> Vec<AvisoDelCanal> {
    revision_del_canal::revisar(estado, bandejas, hora("2026-11-13 12:00:00"), LIMITE)
}

#[test]
fn la_plantilla_recien_creada_no_da_ningun_aviso() {
    let plantilla = plantillas::esqueleto_gestor(EsqueletoGestor::Estado, None);

    assert_eq!(revisar(&plantilla, &[]), Vec::new());
}

#[test]
fn avisa_de_cada_apartado_obligatorio_que_falta() {
    let antiguo = "Estado del proyecto\n\nReservas activas\nNinguna.\n";

    assert_eq!(
        revisar(antiguo, &[]),
        vec![
            AvisoDelCanal::FaltaApartado("## Papeles"),
            AvisoDelCanal::FaltaApartado("## Reservas activas"),
            AvisoDelCanal::FaltaApartado("## Mutaciones vivas"),
        ]
    );
}

#[test]
fn avisa_de_las_mutaciones_vivas() {
    let con_mutacion = estado(&[], "- Codex 1: crates/nucleo/src/git.rs, M3 aplicada.");

    assert!(revisar(&con_mutacion, &[]).contains(&AvisoDelCanal::MutacionesVivas));
}

#[test]
fn una_reserva_vigente_y_bien_escrita_no_avisa() {
    let texto = estado(
        &["- Claude 2, 2026-11-13 11:00: crates/nucleo/src/git.rs, documentation/ — M3"],
        "Ninguna.",
    );

    assert_eq!(
        revisar(&texto, &[bandeja("claude2.md", "2026-11-13 11:30:05")]),
        Vec::new()
    );
}

#[test]
fn avisa_de_dos_agentes_con_la_misma_ruta_o_una_carpeta_que_la_contiene() {
    let texto = estado(
        &[
            "- Claude 2, 2026-11-13 11:00: crates/nucleo/ — M3",
            "- Codex 1, 2026-11-13 11:05: crates/nucleo/src/git.rs — arreglo",
            "- Gemini 1, 2026-11-13 11:06: README.md — repaso",
        ],
        "Ninguna.",
    );
    let bandejas = [
        bandeja("claude2.md", "2026-11-13 11:30"),
        bandeja("codex.md", "2026-11-13 11:31"),
        bandeja("gemini.md", "2026-11-13 11:32"),
    ];

    assert_eq!(
        revisar(&texto, &bandejas),
        vec![AvisoDelCanal::ReservaRepetida {
            ruta: "crates/nucleo/src/git.rs".to_owned(),
            agentes: ("Claude 2".to_owned(), "Codex 1".to_owned()),
        }]
    );
}

#[test]
fn avisa_de_la_reserva_de_un_agente_sin_latido_reciente_o_sin_bandeja() {
    let texto = estado(
        &[
            "- Claude 2, 2026-11-12 08:00: crates/nucleo/ — antigua",
            "- Codex 1, 2026-11-13 11:00: README.md — sin bandeja",
        ],
        "Ninguna.",
    );

    assert_eq!(
        revisar(&texto, &[bandeja("claude2.md", "2026-11-12 23:00:00")]),
        vec![
            AvisoDelCanal::ReservaSinLatido {
                agente: "Claude 2".to_owned(),
                horas: Some(13),
            },
            AvisoDelCanal::ReservaSinLatido {
                agente: "Codex 1".to_owned(),
                horas: None,
            },
        ]
    );
}

#[test]
fn avisa_de_una_reserva_que_no_sigue_el_formato() {
    let texto = estado(&["- Claude 2: crates/nucleo/, por si acaso"], "Ninguna.");

    assert_eq!(
        revisar(&texto, &[]),
        vec![AvisoDelCanal::ReservaIlegible(
            "- Claude 2: crates/nucleo/, por si acaso".to_owned()
        )]
    );
}

#[test]
fn el_resumen_del_equipo_lee_jefe_latidos_reservas_y_alertas() {
    let texto = estado(
        &["- Gemini 1, 2026-11-13 11:00: README.md — repaso"],
        "- Gemini 1: README.md, mutación de prueba.",
    )
    .replace(
        "- Jefe de proyecto: sin asignar",
        "- Jefe de proyecto: Claude 2 (Director, 2026-11-13 09:00)",
    );
    let bandejas = [
        bandeja("gemini.md", "2026-11-13 11:32:10"),
        bandeja("claude2.md", "2026-11-13 11:30"),
    ];

    let resumen = revision_del_canal::resumir(&texto, &bandejas);

    assert_eq!(
        resumen.jefe.as_deref(),
        Some("Claude 2 (Director, 2026-11-13 09:00)")
    );
    assert_eq!(
        resumen
            .latidos
            .iter()
            .map(|l| (l.identificador.as_str(), l.momento))
            .collect::<Vec<_>>(),
        vec![
            ("claude2", hora("2026-11-13 11:30:00")),
            ("gemini", hora("2026-11-13 11:32:10")),
        ]
    );
    assert_eq!(resumen.reservas, 1);
    assert!(resumen.mutaciones_vivas);
    assert!(!resumen.autoexclusiones);
    assert_eq!(
        revision_del_canal::resumir(
            &plantillas::esqueleto_gestor(EsqueletoGestor::Estado, None),
            &[]
        )
        .jefe,
        None
    );
}

#[test]
fn el_identificador_de_un_agente_es_el_nombre_de_su_bandeja() {
    for (nombre, esperado) in [
        ("Claude 1", Some("claude")),
        ("Claude 2", Some("claude2")),
        ("Codex", Some("codex")),
        ("Programator 1", Some("programator")),
        ("Claude 0", None),
        ("Claude dos", None),
        ("", None),
    ] {
        assert_eq!(
            revision_del_canal::identificador_de(nombre).as_deref(),
            esperado,
            "{nombre:?}"
        );
    }
}

#[test]
fn el_inicializador_pone_los_avisos_del_canal_en_su_informe() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let limpio = entorno.inicializar(carpeta.ruta(), None, None);
    assert!(
        !limpio.avisos().iter().any(|a| a.contains("estado.md")),
        "{:?}",
        limpio.avisos()
    );

    carpeta.escribir(
        ".gestor/canal/estado.md",
        "Estado del proyecto\n\nReservas activas\nNinguna.\n",
    );
    let informe = entorno.inicializar(carpeta.ruta(), None, None);

    assert!(
        informe
            .avisos()
            .iter()
            .any(|a| a.contains(".gestor/canal/estado.md") && a.contains("## Papeles")),
        "{:?}",
        informe.avisos()
    );
}
