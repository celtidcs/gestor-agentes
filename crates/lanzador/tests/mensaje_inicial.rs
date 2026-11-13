//! Barreras del mensaje inicial: Claude Code, codex y agy abren su sesión
//! interactiva con un primer mensaje, para que el agente ejecute el arranque de AGENTS.md sin
//! esperar a que el usuario escriba. Programator no lo recibe. Sin mensaje, o con un mensaje en
//! blanco, la orden es la misma que en la 3.0.0.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::path::PathBuf;

use comun::{con_su_ejecutable, guion_bash, guion_powershell, peticion, plan};
use lanzador::{Herramienta, PeticionDeLanzamiento, Sistema};

/// Mensaje de ejemplo, con tildes, para comprobar que llega como un solo argumento.
const MENSAJE: &str = "Ejecuta ahora el arranque obligatorio de AGENTS.md, paso a paso.";

/// Ejecutable fijado de ejemplo en Linux.
const PROGRAMA: &str = "/opt/agente/bin/programa";

fn peticion_con(herramienta: Herramienta, mensaje: Option<&str>) -> PeticionDeLanzamiento {
    let mut p = peticion(herramienta);
    p.ejecutable = Some(PathBuf::from(PROGRAMA));
    p.mensaje_inicial = mensaje.map(str::to_owned);
    p
}

fn con_argumentos_del_perfil(mut p: PeticionDeLanzamiento) -> PeticionDeLanzamiento {
    p.argumentos_extra = vec!["--model".to_owned(), "m1".to_owned()];
    p
}

/// La parte del guion de bash que lanza el programa, sin el `exec bash` final.
fn orden_bash(p: &PeticionDeLanzamiento) -> String {
    let guion = guion_bash(&plan(p, Sistema::Linux, &con_su_ejecutable(p)));
    let inicio = guion
        .find(&format!("'{PROGRAMA}'"))
        .expect("el guion lanza el programa");
    guion[inicio..].trim_end_matches("; exec bash").to_owned()
}

#[test]
fn claude_recibe_el_mensaje_como_unico_argumento() {
    let p = peticion_con(Herramienta::Claude, Some(MENSAJE));
    assert_eq!(orden_bash(&p), format!("'{PROGRAMA}' '{MENSAJE}'"));
}

#[test]
fn codex_recibe_el_mensaje_despues_de_los_argumentos_del_perfil() {
    let p = con_argumentos_del_perfil(peticion_con(Herramienta::Codex, Some(MENSAJE)));
    assert_eq!(
        orden_bash(&p),
        format!("'{PROGRAMA}' '--model' 'm1' '{MENSAJE}'")
    );
}

#[test]
fn agy_recibe_el_mensaje_con_la_opcion_interactiva() {
    let p = con_argumentos_del_perfil(peticion_con(Herramienta::Gemini, Some(MENSAJE)));
    assert_eq!(
        orden_bash(&p),
        format!("'{PROGRAMA}' '--model' 'm1' '-i' '{MENSAJE}'")
    );
}

#[test]
fn sin_mensaje_o_en_blanco_la_orden_es_la_de_antes() {
    for mensaje in [None, Some(""), Some("   ")] {
        assert_eq!(
            orden_bash(&peticion_con(Herramienta::Claude, mensaje)),
            format!("'{PROGRAMA}'")
        );
        assert_eq!(
            orden_bash(&con_argumentos_del_perfil(peticion_con(
                Herramienta::Gemini,
                mensaje
            ))),
            format!("'{PROGRAMA}' '--model' 'm1'")
        );
    }
}

#[test]
fn programator_no_recibe_el_mensaje() {
    let mut p = peticion_con(Herramienta::Programator, Some(MENSAJE));
    p.ejecutable = Some(PathBuf::from("/opt/programator/programator"));
    let plan = plan(&p, Sistema::Linux, &con_su_ejecutable(&p));
    assert!(
        !guion_bash(&plan).contains(MENSAJE),
        "{}",
        guion_bash(&plan)
    );
}

#[test]
fn en_windows_el_mensaje_va_escapado_como_un_literal_de_powershell() {
    let mut p = peticion_con(Herramienta::Claude, Some("Di 'hola' y sigue."));
    p.ejecutable = Some(PathBuf::from(r"C:\bin\claude.exe"));
    let guion = guion_powershell(&plan(&p, Sistema::Windows, &con_su_ejecutable(&p)));
    assert!(
        guion.contains(r"& 'C:\bin\claude.exe' 'Di ''hola'' y sigue.'"),
        "{guion}"
    );
}
