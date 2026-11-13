//! Barrera del vigilante de Codex: la cabecera de AGENTS.md pide a Codex que monte él mismo, al
//! arrancar, un sistema de vigilancia del canal que lo avise aunque esté esperando. Solo le dice
//! el objetivo; cómo hacerlo lo decide él.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use nucleo::plantillas;

#[test]
fn codex_monta_su_propia_vigilancia_del_canal() {
    let cabecera = plantillas::cabecera();

    assert!(cabecera.contains("- **Codex:** monta tú mismo, al arrancar, un sistema de vigilancia"));
    assert!(cabecera.contains("aunque estés esperando. Tú decides cómo."));
    assert!(cabecera.contains("usa la pasada única"));
}
