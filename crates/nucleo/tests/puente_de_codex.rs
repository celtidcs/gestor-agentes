//! Barrera del vigilante de Codex. Hasta la 3.5.1 la cabecera pedía a Codex que montara él mismo su
//! vigilancia; desde la 3.6.0 Codex lanza una sola vez
//! gestor-vigilante en modo persistente, que le entrega los avisos en su sesión con codex queue.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use nucleo::plantillas;

#[test]
fn codex_lanza_una_vez_el_vigilante_persistente() {
    let cabecera = plantillas::cabecera();
    let codex = cabecera
        .lines()
        .find(|l| l.contains("- **Codex:**"))
        .unwrap();

    assert!(codex.contains("`--anotar-hilo`"), "{codex}");
    assert!(codex.contains("no lo rearmes"), "{codex}");
    assert!(!cabecera.contains("monta tú mismo"));
    assert!(cabecera.contains("`--una-vez`"));
}
