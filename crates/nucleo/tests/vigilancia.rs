//! Barreras de la lectura de lo que deja el vigilante para el panel del equipo: vivo.json, el
//! último acuse RECIBIDO de la bandeja y los encargos de avisos.log posteriores a él.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use chrono::NaiveDateTime;
use nucleo::proyecto::vigilancia::{self, VigilanteAnotado};

#[test]
fn lee_vivo_json_tal_como_lo_escribe_el_vigilante() {
    let texto = "{\"identidad\":\"Claude 2\",\"pid\":4242,\"arranque\":1763570000,\"modo\":\"hasta-cambio\",\"desde\":1763570001}\n";
    assert_eq!(
        vigilancia::leer_vivo(texto),
        Some(VigilanteAnotado {
            pid: 4242,
            arranque: 1_763_570_000,
            modo: "hasta-cambio".to_owned(),
        })
    );
    assert_eq!(vigilancia::leer_vivo("roto"), None);
    assert_eq!(vigilancia::leer_vivo("{\"pid\":1}"), None);
}

#[test]
fn el_ultimo_acuse_es_el_mas_reciente_legible() {
    let bandeja = "LATIDO: 2026-11-19 10:00:00\n\
                   RECIBIDO 2026-11-19 10:05:00: encargo A\n\
                   RECIBIDO 2026-11-19 09:00:00: encargo B, anotado después\n\
                   RECIBIDO ayer: sin hora\n\
                   Texto que menciona RECIBIDO 2026-11-20 00:00:00 a mitad de línea\n";
    assert_eq!(
        vigilancia::ultimo_acuse(bandeja),
        Some(NaiveDateTime::parse_from_str("2026-11-19 10:05:00", "%Y-%m-%d %H:%M:%S").unwrap())
    );
    assert_eq!(
        vigilancia::ultimo_acuse("LATIDO: 2026-11-19 10:00:00\n"),
        None
    );
}

fn momento(texto: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(texto, "%Y-%m-%d %H:%M:%S").unwrap()
}

#[test]
fn un_acuse_con_solo_la_hora_toma_la_fecha_del_latido_anterior() {
    // Así escriben el acuse los agentes en el canal (caso P1 de la verificación de Codex 1).
    let bandeja = "LATIDO: 2026-11-19 22:13:20\n\
                   RECIBIDO 22:13:20: encargo viejo\n\
                   \n\
                   LATIDO: 2026-11-20 07:17:33\n\
                   \n\
                   RECIBIDO 07:17:33: verificación de la 3.6.0\n";
    assert_eq!(
        vigilancia::ultimo_acuse(bandeja),
        Some(momento("2026-11-20 07:17:33"))
    );
    // Una hora más temprana bajo un LATIDO más reciente sigue siendo posterior.
    let bandeja = "LATIDO: 2026-11-19 23:50:00\n\
                   RECIBIDO 2026-11-19 23:50:00: con fecha\n\
                   LATIDO: 2026-11-20 00:05:00\n\
                   RECIBIDO 00:05:00: tras medianoche\n";
    assert_eq!(
        vigilancia::ultimo_acuse(bandeja),
        Some(momento("2026-11-20 00:05:00"))
    );
}

#[test]
fn un_acuse_con_solo_la_hora_sin_latido_antes_no_cuenta() {
    let sin_latido = "RECIBIDO 07:17:33: sin latido\n";
    let latido_despues = "RECIBIDO 07:17:33: sin latido\nLATIDO: 2026-11-20 07:18:00\n";
    assert_eq!(vigilancia::ultimo_acuse(sin_latido), None);
    assert_eq!(vigilancia::ultimo_acuse(latido_despues), None);
}

#[test]
fn cuenta_los_encargos_avisados_despues_del_acuse() {
    let avisos = "100\tencargo\tcodex.md\t\n\
                  200\tmutacion\testado.md\t\n\
                  300\tencargo,mutacion\tclaude2.md,estado.md\tentregado con codex queue\n\
                  linea rota\n";
    assert_eq!(vigilancia::encargos_sin_acuse(avisos, None), 2);
    assert_eq!(vigilancia::encargos_sin_acuse(avisos, Some(100)), 1);
    assert_eq!(vigilancia::encargos_sin_acuse(avisos, Some(300)), 0);
}
