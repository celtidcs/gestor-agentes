//! Barreras de los agentes en terminal: Codex CLI y agy, los argumentos del perfil y
//! Programator dentro de un emulador de terminal en Linux. Desde la 3.0.0,
//! Codex y Gemini se abren siempre así: las variantes CodexTerminal y GeminiTerminal pasaron a
//! ser Codex y Gemini.
//!
//! Se comprueba el plan completo: programa, argumentos (y, dentro de ellos, el guion exacto),
//! variables, carpeta y carpetas que se crean, además del orden de búsqueda de cada programa y
//! los errores cuando no se encuentra.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default,
    missing_docs
)]

mod comun;

use std::path::{Path, PathBuf};

use comun::{guion_bash, guion_powershell, peticion, plan, textos, EntornoDeEnsayo};
use lanzador::{ErrorDeLanzamiento, Herramienta, PlanDeLanzamiento, Sistema};

/// Parte fija del guion de PowerShell, igual que en el de Claude: título y temporizador que lo
/// restaura.
fn cabecera_powershell(titulo: &str) -> String {
    format!(
        "$t = '{titulo}'; [Console]::Title = $t; \
         $cb = [System.Threading.TimerCallback]{{param($s) [Console]::Title = $s}}; \
         $tmr = New-Object System.Threading.Timer($cb, $t, 0, 1000)"
    )
}

/// Final fijo del guion de PowerShell.
const FINAL_POWERSHELL: &str = "$tmr.Dispose(); [Console]::Title = $t";

fn texto(ruta: &Path) -> &str {
    ruta.to_str().unwrap()
}

fn no_encontrada(resultado: Result<PlanDeLanzamiento, ErrorDeLanzamiento>) -> (String, String) {
    match resultado {
        Err(ErrorDeLanzamiento::NoEncontrada {
            herramienta,
            detalle,
        }) => (herramienta, detalle),
        otro => panic!("se esperaba NoEncontrada y se obtuvo {otro:?}"),
    }
}

/// Comprobaciones comunes a todo plan de terminal: sin variables en el proceso (van en el
/// guion) y la carpeta indicada.
fn es_de_terminal(plan: &PlanDeLanzamiento, carpeta: &Path) {
    assert!(plan.variables.is_empty(), "{:?}", plan.variables);
    assert_eq!(plan.carpeta, carpeta);
}

// ── Codex CLI ─────────────────────────────────────────────

fn peticion_codex() -> lanzador::PeticionDeLanzamiento {
    let mut p = peticion(Herramienta::Codex);
    p.identidad = "Codex 2".to_owned();
    p.directorio_configuracion = Some(PathBuf::from("/cfg/codex2"));
    p.argumentos_extra = vec!["-c".to_owned(), "project_doc_max_bytes=65536".to_owned()];
    p
}

#[test]
fn codex_terminal_en_windows_abre_powershell_con_identidad_codex_home_y_argumentos() {
    let codex = PathBuf::from(r"C:\bin").join("codex.exe");
    let entorno = EntornoDeEnsayo::default()
        .con_path(r"C:\bin")
        .con_archivo(codex.clone());
    let p = peticion_codex();
    let plan = plan(&p, Sistema::Windows, &entorno);
    let esperado = format!(
        "{}; $env:GESTOR_AGENTE = 'Codex 2'; $env:CODEX_HOME = '/cfg/codex2'; \
         Set-Location -LiteralPath '/proyectos/Demo'; \
         & '{}' '-c' 'project_doc_max_bytes=65536'; {FINAL_POWERSHELL}",
        cabecera_powershell("Codex 2 · Demo"),
        texto(&codex)
    );
    assert_eq!(guion_powershell(&plan), esperado);
    es_de_terminal(&plan, &p.carpeta_proyecto);
    assert_eq!(plan.carpetas_a_crear, [PathBuf::from("/cfg/codex2")]);
}

#[test]
fn codex_terminal_en_linux_va_en_el_emulador_con_identidad_codex_home_y_argumentos() {
    let codex = PathBuf::from("/home/u")
        .join(".local")
        .join("bin")
        .join("codex");
    let mut entorno = EntornoDeEnsayo::default().con_archivo(codex.clone());
    entorno.personal = Some(PathBuf::from("/home/u"));
    let p = peticion_codex();
    let plan = plan(&p, Sistema::Linux, &entorno);
    assert_eq!(plan.programa, PathBuf::from("xterm"));
    assert_eq!(&textos(&plan.argumentos)[..2], ["-T", "Codex 2 · Demo"]);
    assert_eq!(
        guion_bash(&plan),
        format!(
            "export GESTOR_AGENTE='Codex 2'; export CODEX_HOME='/cfg/codex2'; \
             cd '/proyectos/Demo'; '{}' '-c' 'project_doc_max_bytes=65536'; exec bash",
            texto(&codex)
        )
    );
    es_de_terminal(&plan, &p.carpeta_proyecto);
    assert_eq!(plan.carpetas_a_crear, [PathBuf::from("/cfg/codex2")]);
}

#[test]
fn codex_terminal_sin_carpeta_de_configuracion_no_fija_codex_home() {
    let mut p = peticion_codex();
    p.directorio_configuracion = None;
    p.ejecutable = Some(PathBuf::from("/bin/codex"));
    let entorno = EntornoDeEnsayo::default().con_archivo("/bin/codex");
    let windows = plan(&p, Sistema::Windows, &entorno);
    let linux = plan(&p, Sistema::Linux, &entorno);
    assert!(!guion_powershell(&windows).contains("CODEX_HOME"));
    assert!(!guion_bash(&linux).contains("CODEX_HOME"));
    assert!(windows.carpetas_a_crear.is_empty());
}

#[test]
fn codex_terminal_en_windows_acepta_un_cmd_y_en_la_misma_carpeta_prefiere_el_exe() {
    let p = peticion(Herramienta::Codex);
    let cmd = PathBuf::from(r"C:\npm").join("codex.cmd");
    let solo_cmd = EntornoDeEnsayo::default()
        .con_path(r"C:\npm")
        .con_archivo(cmd.clone());
    assert!(guion_powershell(&plan(&p, Sistema::Windows, &solo_cmd))
        .contains(&format!("& '{}'", texto(&cmd))));

    let exe = PathBuf::from(r"C:\npm").join("codex.exe");
    let los_dos = solo_cmd.con_archivo(exe.clone());
    assert!(guion_powershell(&plan(&p, Sistema::Windows, &los_dos))
        .contains(&format!("& '{}'", texto(&exe))));
}

#[test]
fn codex_terminal_en_windows_busca_primero_en_local_bin() {
    let local = PathBuf::from(r"C:\Users\u")
        .join(".local")
        .join("bin")
        .join("codex.exe");
    let en_path = PathBuf::from(r"C:\bin").join("codex.exe");
    let mut entorno = EntornoDeEnsayo::default()
        .con_path(r"C:\bin")
        .con_archivo(en_path)
        .con_archivo(local.clone());
    entorno.personal = Some(PathBuf::from(r"C:\Users\u"));
    let p = peticion(Herramienta::Codex);
    assert!(guion_powershell(&plan(&p, Sistema::Windows, &entorno))
        .contains(&format!("& '{}'", texto(&local))));
}

#[test]
fn codex_terminal_sin_programa_no_se_encuentra() {
    let p = peticion(Herramienta::Codex);
    for sistema in [Sistema::Windows, Sistema::Linux] {
        let (herramienta, detalle) = no_encontrada(lanzador::planificar(
            &p,
            sistema,
            &EntornoDeEnsayo::default(),
        ));
        assert_eq!(herramienta, "Codex CLI");
        assert!(detalle.contains("PATH"), "{detalle}");
    }
}

// ── agy (Gemini en terminal) ─────────────────────────────

fn peticion_agy() -> lanzador::PeticionDeLanzamiento {
    let mut p = peticion(Herramienta::Gemini);
    p.identidad = "Gemini 1".to_owned();
    p.directorio_configuracion = Some(PathBuf::from("/cfg/no-se-usa"));
    p.argumentos_extra = vec![
        "--model".to_owned(),
        "modelo-de-prueba".to_owned(),
        "--effort".to_owned(),
        "high".to_owned(),
    ];
    p
}

fn agy_de_localappdata() -> PathBuf {
    PathBuf::from(r"C:\Users\u\AppData\Local")
        .join("agy")
        .join("bin")
        .join("agy.exe")
}

#[test]
fn gemini_terminal_en_windows_abre_agy_con_identidad_y_argumentos_sin_otras_variables() {
    let agy = agy_de_localappdata();
    let mut entorno = EntornoDeEnsayo::default().con_archivo(agy.clone());
    entorno.datos_locales = Some(PathBuf::from(r"C:\Users\u\AppData\Local"));
    let p = peticion_agy();
    let plan = plan(&p, Sistema::Windows, &entorno);
    let esperado = format!(
        "{}; $env:GESTOR_AGENTE = 'Gemini 1'; Set-Location -LiteralPath '/proyectos/Demo'; \
         & '{}' '--model' 'modelo-de-prueba' '--effort' 'high'; {FINAL_POWERSHELL}",
        cabecera_powershell("Gemini 1 · Demo"),
        texto(&agy)
    );
    assert_eq!(guion_powershell(&plan), esperado);
    es_de_terminal(&plan, &p.carpeta_proyecto);
    assert!(plan.carpetas_a_crear.is_empty());
}

#[test]
fn gemini_terminal_en_windows_prefiere_la_carpeta_del_instalador_al_path() {
    let agy = agy_de_localappdata();
    let en_path = PathBuf::from(r"C:\bin").join("agy.exe");
    let mut entorno = EntornoDeEnsayo::default()
        .con_path(r"C:\bin")
        .con_archivo(en_path.clone())
        .con_archivo(agy.clone());
    entorno.datos_locales = Some(PathBuf::from(r"C:\Users\u\AppData\Local"));
    let p = peticion_agy();
    assert!(guion_powershell(&plan(&p, Sistema::Windows, &entorno))
        .contains(&format!("& '{}'", texto(&agy))));

    entorno.archivos.remove(&agy);
    assert!(guion_powershell(&plan(&p, Sistema::Windows, &entorno))
        .contains(&format!("& '{}'", texto(&en_path))));
}

#[test]
fn gemini_terminal_en_linux_busca_agy_en_local_bin() {
    let agy = PathBuf::from("/home/u")
        .join(".local")
        .join("bin")
        .join("agy");
    let mut entorno = EntornoDeEnsayo::default().con_archivo(agy.clone());
    entorno.personal = Some(PathBuf::from("/home/u"));
    let p = peticion_agy();
    let plan = plan(&p, Sistema::Linux, &entorno);
    assert_eq!(
        guion_bash(&plan),
        format!(
            "export GESTOR_AGENTE='Gemini 1'; cd '/proyectos/Demo'; \
             '{}' '--model' 'modelo-de-prueba' '--effort' 'high'; exec bash",
            texto(&agy)
        )
    );
    es_de_terminal(&plan, &p.carpeta_proyecto);
}

#[test]
fn gemini_terminal_sin_agy_no_se_encuentra() {
    let p = peticion(Herramienta::Gemini);
    for sistema in [Sistema::Windows, Sistema::Linux] {
        let mut entorno = EntornoDeEnsayo::default();
        entorno.datos_locales = Some(PathBuf::from(r"C:\Users\u\AppData\Local"));
        let (herramienta, detalle) = no_encontrada(lanzador::planificar(&p, sistema, &entorno));
        assert_eq!(herramienta, "agy (Antigravity CLI)");
        assert!(detalle.contains("PATH"), "{detalle}");
    }
}

#[test]
fn el_ejecutable_del_perfil_manda_y_si_no_existe_no_se_encuentra() {
    for (herramienta, nombre) in [
        (Herramienta::Codex, "Codex CLI"),
        (Herramienta::Gemini, "agy (Antigravity CLI)"),
    ] {
        let mut p = peticion(herramienta);
        p.ejecutable = Some(PathBuf::from("/opt/fijado/programa"));
        let entorno = EntornoDeEnsayo::default().con_archivo("/opt/fijado/programa");
        assert!(guion_bash(&plan(&p, Sistema::Linux, &entorno)).contains("'/opt/fijado/programa'"));
        let resultado = lanzador::planificar(&p, Sistema::Linux, &EntornoDeEnsayo::default());
        assert_eq!(no_encontrada(resultado).0, nombre);
    }
}

// ── Claude y Programator ignoran los argumentos del perfil ───

/// Codex y Gemini sí pasan los argumentos, y eso lo prueban las de arriba.
#[test]
fn las_demas_variantes_ignoran_los_argumentos_del_perfil() {
    let programator = PathBuf::from("/opt/Programator/programator");
    let entorno = EntornoDeEnsayo::default()
        .con_archivo(PathBuf::from("/usr/local/bin").join("claude"))
        .con_path(r"C:\bin")
        .con_archivo(PathBuf::from(r"C:\bin").join("claude.exe"))
        .con_archivo(programator.clone());
    for herramienta in [Herramienta::Claude, Herramienta::Programator] {
        let mut sin = peticion(herramienta);
        if herramienta == Herramienta::Programator {
            sin.ejecutable = Some(programator.clone());
        }
        let mut con = sin.clone();
        con.argumentos_extra = vec!["--extra".to_owned(), "; touch PWNED".to_owned()];
        for sistema in [Sistema::Windows, Sistema::Linux] {
            assert_eq!(
                plan(&con, sistema, &entorno),
                plan(&sin, sistema, &entorno),
                "{herramienta:?} en {sistema:?}"
            );
        }
    }
}

// ── Programator en Linux ───────────────────────────

#[test]
fn programator_en_linux_se_abre_en_un_emulador_de_terminal_desde_la_carpeta_de_su_ejecutable() {
    let ejecutable = PathBuf::from("/opt/Programator/programator");
    let mut p = peticion(Herramienta::Programator);
    p.identidad = "Programator 1".to_owned();
    p.ejecutable = Some(ejecutable.clone());
    p.argumentos_extra = vec!["--no-se-usa".to_owned()];
    let entorno = EntornoDeEnsayo::default()
        .con_archivo(ejecutable)
        .con_path("/usr/bin")
        .con_archivo(PathBuf::from("/usr/bin").join("gnome-terminal"));
    let plan = plan(&p, Sistema::Linux, &entorno);
    assert_eq!(plan.programa, PathBuf::from("gnome-terminal"));
    assert_eq!(
        textos(&plan.argumentos),
        [
            "--title=Programator 1 · Demo",
            "--",
            "bash",
            "-c",
            "export GESTOR_AGENTE='Programator 1'; cd '/opt/Programator'; \
             '/opt/Programator/programator' '--ruta' '/proyectos/Demo'; exec bash",
        ]
    );
    es_de_terminal(&plan, Path::new("/opt/Programator"));
    assert!(plan.carpetas_a_crear.is_empty());
}

#[test]
fn programator_en_linux_sigue_exigiendo_su_ejecutable() {
    let mut p = peticion(Herramienta::Programator);
    p.identidad = "Programator 1".to_owned();
    let resultado = lanzador::planificar(&p, Sistema::Linux, &EntornoDeEnsayo::default());
    assert!(
        matches!(resultado, Err(ErrorDeLanzamiento::PerfilIncompleto { .. })),
        "{resultado:?}"
    );
}
