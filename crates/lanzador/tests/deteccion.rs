//! Barreras de la detección de ejecutables.
//!
//! Se comprueba, por herramienta y sistema, que se encuentra el programa donde lo instala cada
//! una y que no se encuentra cuando no está; que una ruta fijada manda si existe y da «no
//! encontrado» si no existe, aunque haya otro programa en las ubicaciones por defecto; que
//! Programator no se busca; y que lo detectado es exactamente lo que abre el plan.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default,
    missing_docs
)]

mod comun;

use std::path::{Path, PathBuf};

use comun::{guion_bash, guion_powershell, peticion, plan, EntornoDeEnsayo};
use lanzador::{detectar_ejecutable, Herramienta, Sistema};

/// Carpeta personal de ejemplo en Windows.
const PERSONAL_WINDOWS: &str = r"C:\Users\u";
/// Carpeta personal de ejemplo en Linux.
const PERSONAL_LINUX: &str = "/home/u";
/// Carpeta de datos locales de ejemplo en Windows.
const DATOS_LOCALES: &str = r"C:\Users\u\AppData\Local";
/// Carpeta del PATH de ejemplo en Windows, donde npm deja sus lanzadores .cmd.
const NPM: &str = r"C:\npm";

fn detectar(
    herramienta: Herramienta,
    sistema: Sistema,
    entorno: &EntornoDeEnsayo,
) -> Option<PathBuf> {
    detectar_ejecutable(herramienta, None, sistema, entorno)
}

fn local_bin(personal: &str, programa: &str) -> PathBuf {
    PathBuf::from(personal)
        .join(".local")
        .join("bin")
        .join(programa)
}

fn entorno_windows() -> EntornoDeEnsayo {
    let mut entorno = EntornoDeEnsayo::default().con_path(NPM);
    entorno.personal = Some(PathBuf::from(PERSONAL_WINDOWS));
    entorno.datos_locales = Some(PathBuf::from(DATOS_LOCALES));
    entorno
}

fn entorno_linux() -> EntornoDeEnsayo {
    let mut entorno = EntornoDeEnsayo::default().con_path("/opt/bin");
    entorno.personal = Some(PathBuf::from(PERSONAL_LINUX));
    entorno
}

fn codex_del_instalador() -> PathBuf {
    PathBuf::from(DATOS_LOCALES)
        .join("Programs")
        .join("OpenAI")
        .join("Codex")
        .join("bin")
        .join("codex.exe")
}

fn agy_del_instalador() -> PathBuf {
    PathBuf::from(DATOS_LOCALES)
        .join("agy")
        .join("bin")
        .join("agy.exe")
}

// ── Sin nada instalado ────────────────────────────────────

#[test]
fn sin_nada_instalado_no_se_detecta_ninguna_herramienta() {
    for herramienta in [
        Herramienta::Claude,
        Herramienta::Codex,
        Herramienta::Gemini,
        Herramienta::Programator,
    ] {
        assert_eq!(
            detectar(herramienta, Sistema::Windows, &entorno_windows()),
            None,
            "{herramienta:?}"
        );
        assert_eq!(
            detectar(herramienta, Sistema::Linux, &entorno_linux()),
            None,
            "{herramienta:?}"
        );
    }
}

// ── Claude ────────────────────────────────────────────────

#[test]
fn claude_en_windows_se_detecta_en_local_bin_y_si_no_el_cmd_de_npm_del_path() {
    let local = local_bin(PERSONAL_WINDOWS, "claude.exe");
    let npm = PathBuf::from(NPM).join("claude.cmd");
    let mut entorno = entorno_windows()
        .con_archivo(npm.clone())
        .con_archivo(local.clone());
    assert_eq!(
        detectar(Herramienta::Claude, Sistema::Windows, &entorno),
        Some(local.clone())
    );
    entorno.archivos.remove(&local);
    assert_eq!(
        detectar(Herramienta::Claude, Sistema::Windows, &entorno),
        Some(npm)
    );
}

#[test]
fn claude_en_linux_se_detecta_en_local_bin_en_las_carpetas_del_sistema_y_en_el_path() {
    let en_path = PathBuf::from("/opt/bin").join("claude");
    let sistema = PathBuf::from("/usr/bin").join("claude");
    let local = local_bin(PERSONAL_LINUX, "claude");
    let mut entorno = entorno_linux().con_archivo(en_path.clone());
    assert_eq!(
        detectar(Herramienta::Claude, Sistema::Linux, &entorno),
        Some(en_path)
    );
    entorno = entorno.con_archivo(sistema.clone());
    assert_eq!(
        detectar(Herramienta::Claude, Sistema::Linux, &entorno),
        Some(sistema)
    );
    entorno = entorno.con_archivo(local.clone());
    assert_eq!(
        detectar(Herramienta::Claude, Sistema::Linux, &entorno),
        Some(local)
    );
}

// ── Codex ─────────────────────────────────────────────────

#[test]
fn codex_en_windows_se_detecta_tambien_como_cmd_de_npm() {
    let cmd = PathBuf::from(NPM).join("codex.cmd");
    let entorno = entorno_windows().con_archivo(cmd.clone());
    assert_eq!(
        detectar(Herramienta::Codex, Sistema::Windows, &entorno),
        Some(cmd)
    );
    let exe = PathBuf::from(NPM).join("codex.exe");
    let entorno = entorno.con_archivo(exe.clone());
    assert_eq!(
        detectar(Herramienta::Codex, Sistema::Windows, &entorno),
        Some(exe)
    );
}

#[test]
fn codex_en_windows_prefiere_local_bin_al_path() {
    let local = local_bin(PERSONAL_WINDOWS, "codex.exe");
    let entorno = entorno_windows()
        .con_archivo(PathBuf::from(NPM).join("codex.exe"))
        .con_archivo(local.clone());
    assert_eq!(
        detectar(Herramienta::Codex, Sistema::Windows, &entorno),
        Some(local)
    );
}

#[test]
fn codex_en_linux_se_detecta_en_local_bin_y_en_el_path() {
    let en_path = PathBuf::from("/opt/bin").join("codex");
    let local = local_bin(PERSONAL_LINUX, "codex");
    let entorno = entorno_linux().con_archivo(en_path.clone());
    assert_eq!(
        detectar(Herramienta::Codex, Sistema::Linux, &entorno),
        Some(en_path)
    );
    let entorno = entorno.con_archivo(local.clone());
    assert_eq!(
        detectar(Herramienta::Codex, Sistema::Linux, &entorno),
        Some(local)
    );
}

/// El instalador oficial de Codex CLI deja codex.exe en
/// %LOCALAPPDATA%\Programs\OpenAI\Codex\bin (https://releases.openai.com/codex/install.ps1).
#[test]
fn cx_codex_en_windows_se_detecta_donde_lo_deja_su_instalador() {
    let instalador = codex_del_instalador();
    let entorno = entorno_windows().con_archivo(instalador.clone());
    assert_eq!(
        detectar(Herramienta::Codex, Sistema::Windows, &entorno),
        Some(instalador)
    );
}

/// El de la carpeta del instalador va antes que ~/.local/bin y el PATH, como deja el propio
/// instalador, que pone su carpeta delante en el PATH si encuentra otra instalación.
#[test]
fn cx_el_codex_del_instalador_gana_a_local_bin_y_al_path() {
    let instalador = codex_del_instalador();
    let entorno = entorno_windows()
        .con_archivo(PathBuf::from(NPM).join("codex.cmd"))
        .con_archivo(local_bin(PERSONAL_WINDOWS, "codex.exe"))
        .con_archivo(instalador.clone());
    assert_eq!(
        detectar(Herramienta::Codex, Sistema::Windows, &entorno),
        Some(instalador)
    );
}

/// Sin %LOCALAPPDATA% no se rompe nada: se busca como antes.
#[test]
fn cx_sin_datos_locales_codex_se_busca_como_antes() {
    let cmd = PathBuf::from(NPM).join("codex.cmd");
    let mut entorno = entorno_windows().con_archivo(cmd.clone());
    entorno.datos_locales = None;
    assert_eq!(
        detectar(Herramienta::Codex, Sistema::Windows, &entorno),
        Some(cmd)
    );
}

// ── Gemini (agy) ──────────────────────────────────────────

#[test]
fn gemini_en_windows_se_detecta_donde_lo_deja_su_instalador_antes_que_en_el_path() {
    let instalador = agy_del_instalador();
    let en_path = PathBuf::from(NPM).join("agy.exe");
    let mut entorno = entorno_windows()
        .con_archivo(en_path.clone())
        .con_archivo(instalador.clone());
    assert_eq!(
        detectar(Herramienta::Gemini, Sistema::Windows, &entorno),
        Some(instalador.clone())
    );
    entorno.archivos.remove(&instalador);
    assert_eq!(
        detectar(Herramienta::Gemini, Sistema::Windows, &entorno),
        Some(en_path)
    );
}

#[test]
fn gemini_en_linux_se_detecta_en_local_bin() {
    let local = local_bin(PERSONAL_LINUX, "agy");
    let entorno = entorno_linux().con_archivo(local.clone());
    assert_eq!(
        detectar(Herramienta::Gemini, Sistema::Linux, &entorno),
        Some(local)
    );
}

// ── Programator ───────────────────────────────────────────

#[test]
fn programator_no_se_busca_aunque_este_en_el_path() {
    let windows = entorno_windows().con_archivo(PathBuf::from(NPM).join("programator.exe"));
    assert_eq!(
        detectar(Herramienta::Programator, Sistema::Windows, &windows),
        None
    );
    let linux = entorno_linux().con_archivo(PathBuf::from("/opt/bin").join("programator"));
    assert_eq!(
        detectar(Herramienta::Programator, Sistema::Linux, &linux),
        None
    );
}

// ── Ruta fijada ───────────────────────────────────────────

#[test]
fn una_ruta_fijada_que_existe_manda_sobre_las_ubicaciones_por_defecto() {
    let fijada = PathBuf::from("/mis/programas/agente");
    for herramienta in [
        Herramienta::Claude,
        Herramienta::Codex,
        Herramienta::Gemini,
        Herramienta::Programator,
    ] {
        let entorno = entorno_linux()
            .con_archivo(local_bin(PERSONAL_LINUX, "claude"))
            .con_archivo(local_bin(PERSONAL_LINUX, "codex"))
            .con_archivo(local_bin(PERSONAL_LINUX, "agy"))
            .con_archivo(fijada.clone());
        assert_eq!(
            detectar_ejecutable(herramienta, Some(&fijada), Sistema::Linux, &entorno),
            Some(fijada.clone()),
            "{herramienta:?}"
        );
    }
}

/// Una ruta mal puesta se ve como «no encontrado» y no queda tapada por otro
/// programa que esté en las ubicaciones por defecto.
#[test]
fn una_ruta_fijada_que_no_existe_no_se_detecta_aunque_haya_otro_programa() {
    let fijada = Path::new("/no/existe/agente");
    let windows = entorno_windows()
        .con_archivo(PathBuf::from(NPM).join("claude.cmd"))
        .con_archivo(PathBuf::from(NPM).join("codex.cmd"))
        .con_archivo(agy_del_instalador());
    let linux = entorno_linux()
        .con_archivo(local_bin(PERSONAL_LINUX, "claude"))
        .con_archivo(local_bin(PERSONAL_LINUX, "codex"))
        .con_archivo(local_bin(PERSONAL_LINUX, "agy"));
    for herramienta in [Herramienta::Claude, Herramienta::Codex, Herramienta::Gemini] {
        assert!(
            detectar(herramienta, Sistema::Windows, &windows).is_some(),
            "{herramienta:?}"
        );
        assert!(
            detectar(herramienta, Sistema::Linux, &linux).is_some(),
            "{herramienta:?}"
        );
        assert_eq!(
            detectar_ejecutable(herramienta, Some(fijada), Sistema::Windows, &windows),
            None,
            "{herramienta:?}"
        );
        assert_eq!(
            detectar_ejecutable(herramienta, Some(fijada), Sistema::Linux, &linux),
            None,
            "{herramienta:?}"
        );
    }
}

// ── La detección y el lanzamiento coinciden ───────────────

/// Programa que abre el guion de la terminal del plan: lo que va tras «& » en PowerShell o el
/// primer literal tras «cd '<carpeta>'; » en bash.
fn programa_del_plan(
    herramienta: Herramienta,
    sistema: Sistema,
    entorno: &EntornoDeEnsayo,
) -> String {
    let plan = plan(&peticion(herramienta), sistema, entorno);
    match sistema {
        Sistema::Windows => {
            let guion = guion_powershell(&plan);
            let resto = guion
                .split("& '")
                .nth(1)
                .expect("el guion abre un programa");
            resto.split('\'').next().unwrap().to_owned()
        }
        Sistema::Linux => {
            let guion = guion_bash(&plan);
            let resto = guion
                .split("'; '")
                .nth(1)
                .expect("el guion abre un programa");
            resto.split('\'').next().unwrap().to_owned()
        }
    }
}

/// La detección del arranque y el lanzamiento usan la misma búsqueda: para cada herramienta,
/// con varios candidatos instalados a la vez, el programa del plan es el detectado.
#[test]
fn lo_detectado_es_lo_que_abre_el_plan() {
    let windows = entorno_windows()
        .con_archivo(PathBuf::from(NPM).join("claude.cmd"))
        .con_archivo(local_bin(PERSONAL_WINDOWS, "claude.exe"))
        .con_archivo(PathBuf::from(NPM).join("codex.cmd"))
        .con_archivo(local_bin(PERSONAL_WINDOWS, "codex.exe"))
        .con_archivo(codex_del_instalador())
        .con_archivo(PathBuf::from(NPM).join("agy.exe"))
        .con_archivo(agy_del_instalador());
    let linux = entorno_linux()
        .con_archivo(PathBuf::from("/opt/bin").join("claude"))
        .con_archivo(local_bin(PERSONAL_LINUX, "claude"))
        .con_archivo(PathBuf::from("/opt/bin").join("codex"))
        .con_archivo(local_bin(PERSONAL_LINUX, "codex"))
        .con_archivo(PathBuf::from("/opt/bin").join("agy"))
        .con_archivo(local_bin(PERSONAL_LINUX, "agy"));
    for herramienta in [Herramienta::Claude, Herramienta::Codex, Herramienta::Gemini] {
        for (sistema, entorno) in [(Sistema::Windows, &windows), (Sistema::Linux, &linux)] {
            let detectado = detectar(herramienta, sistema, entorno).expect("está instalado");
            assert_eq!(
                programa_del_plan(herramienta, sistema, entorno),
                detectado.to_str().unwrap(),
                "{herramienta:?} en {sistema:?}"
            );
        }
    }
}
