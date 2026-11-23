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

use comun::{guion_bash, guion_powershell, peticion, plan, textos, EntornoDeEnsayo, PROYECTO_AGY};
use lanzador::{ErrorDeLanzamiento, Herramienta, PlanDeLanzamiento, Sistema};

/// Parte fija del guion de PowerShell, igual que en el de Claude: título y temporizador que lo
/// restaura.
fn cabecera_powershell(titulo: &str) -> String {
    format!(
        "$t = '{titulo}'; [Console]::Title = $t; Add-Type -TypeDefinition 'public static class GestorAgentesTitulo {{ static System.Threading.Timer reloj; public static void Fijar(string titulo) {{ reloj = new System.Threading.Timer(estado => System.Console.Title = titulo, null, 0, 1000); }} public static void Parar() {{ if (reloj != null) {{ reloj.Dispose(); }} }} }}'; [GestorAgentesTitulo]::Fijar($t)"
    )
}

/// Final fijo del guion de PowerShell.
const FINAL_POWERSHELL: &str = "[GestorAgentesTitulo]::Parar(); [Console]::Title = $t";

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

/// Opciones `-c` con las variables de las órdenes de Codex 2 en /proyectos/Demo, tal como quedan
/// en el guion de PowerShell: el valor TOML va entre comillas simples, que PowerShell duplica.
const VARIABLES_CODEX_POWERSHELL: &str =
    "'-c' 'shell_environment_policy.set.GESTOR_AGENTE=''Codex 2''' \
     '-c' 'shell_environment_policy.set.GESTOR_PROYECTO=''/proyectos/Demo'''";

/// Las mismas opciones en el guion de bash, donde cada comilla simple se escribe `'\''`.
const VARIABLES_CODEX_BASH: &str =
    "'-c' 'shell_environment_policy.set.GESTOR_AGENTE='\\''Codex 2'\\''' \
     '-c' 'shell_environment_policy.set.GESTOR_PROYECTO='\\''/proyectos/Demo'\\'''";

#[test]
fn codex_terminal_en_windows_abre_powershell_con_identidad_codex_home_y_argumentos() {
    let codex = PathBuf::from(r"C:\bin").join("codex.exe");
    let entorno = EntornoDeEnsayo::default()
        .con_path(r"C:\bin")
        .con_archivo(codex.clone());
    let p = peticion_codex();
    let plan = plan(&p, Sistema::Windows, &entorno);
    let esperado = format!(
        "{}; $env:GESTOR_AGENTE = 'Codex 2'; $env:GESTOR_PROYECTO = '/proyectos/Demo'; $env:CODEX_HOME = '/cfg/codex2'; \
         Set-Location -LiteralPath '/proyectos/Demo'; \
         if (Test-Path -LiteralPath '.gestor\\vigilante\\gestor-vigilante.exe') {{ Start-Process -FilePath '.gestor\\vigilante\\gestor-vigilante.exe' -ArgumentList \"--persistente --entrega codex --desde-terminal $PID\" -WindowStyle Hidden }}; \
         & '{}' {VARIABLES_CODEX_POWERSHELL} '-c' 'project_doc_max_bytes=65536' '-s' 'workspace-write' '-a' 'never' '-c' 'sandbox_workspace_write.network_access=true' '-c' 'features.prefer_mxc=true'; \
         {FINAL_POWERSHELL}",
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
            "export GESTOR_AGENTE='Codex 2'; export GESTOR_PROYECTO='/proyectos/Demo'; export CODEX_HOME='/cfg/codex2'; \
             cd '/proyectos/Demo'; if [ -x '.gestor/vigilante/gestor-vigilante' ]; then '.gestor/vigilante/gestor-vigilante' --persistente --entrega codex --desde-terminal $$ >/dev/null 2>&1 & fi; \
             '{}' {VARIABLES_CODEX_BASH} '-c' 'project_doc_max_bytes=65536' \
             '-s' 'workspace-write' '-a' 'never' '-c' 'sandbox_workspace_write.network_access=true'; exec bash",
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
        "{}; $env:GESTOR_AGENTE = 'Gemini 1'; $env:GESTOR_PROYECTO = '/proyectos/Demo'; Set-Location -LiteralPath '/proyectos/Demo'; \
         & '{}' '--project' '{PROYECTO_AGY}' '--model' 'modelo-de-prueba' '--effort' 'high'; \
         {FINAL_POWERSHELL}",
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
            "export GESTOR_AGENTE='Gemini 1'; export GESTOR_PROYECTO='/proyectos/Demo'; cd '/proyectos/Demo'; \
             '{}' '--project' '{PROYECTO_AGY}' '--model' 'modelo-de-prueba' '--effort' 'high'; \
             exec bash",
            texto(&agy)
        )
    );
    es_de_terminal(&plan, &p.carpeta_proyecto);
}

#[test]
fn codex_no_recibe_permisos_si_el_perfil_ya_fija_sandbox_o_aprobacion() {
    let codex = PathBuf::from("/bin").join("codex");
    let entorno = EntornoDeEnsayo::default()
        .con_path("/bin")
        .con_archivo(codex);
    for fijado in [
        vec!["-s", "read-only"],
        vec!["--sandbox=danger-full-access"],
        vec!["-a", "never"],
        vec!["--ask-for-approval", "on-request"],
        vec!["--approve-for-me"],
        vec!["--dangerously-bypass-approvals-and-sandbox"],
        vec!["-c", "approval_policy=\"on-request\""],
        vec!["-c", "sandbox_mode=\"read-only\""],
        vec!["--config", "sandbox_workspace_write.network_access=false"],
        vec!["-c", "features.prefer_mxc=false"],
        vec!["-c", "windows.sandbox=\"elevated\""],
    ] {
        let mut p = peticion_codex();
        p.argumentos_extra = fijado.iter().map(|a| (*a).to_owned()).collect();
        let guion = guion_bash(&plan(&p, Sistema::Linux, &entorno));
        assert!(!guion.contains("'workspace-write'"), "{fijado:?}: {guion}");
        assert!(
            !guion.contains("'never'") || fijado.contains(&"never"),
            "{fijado:?}: {guion}"
        );
        assert!(
            !guion.contains("network_access=true"),
            "{fijado:?}: {guion}"
        );
    }
    // Un -c que no toca los permisos no los impide.
    let p = peticion_codex();
    assert!(guion_bash(&plan(&p, Sistema::Linux, &entorno)).contains(
        "'-s' 'workspace-write' '-a' 'never' '-c' 'sandbox_workspace_write.network_access=true'"
    ));
}

#[test]
fn gemini_terminal_sin_proyecto_de_agy_para_la_carpeta_lo_crea() {
    let agy = agy_de_localappdata();
    let mut entorno = EntornoDeEnsayo::default().con_archivo(agy);
    entorno.datos_locales = Some(PathBuf::from(r"C:\Users\u\AppData\Local"));
    entorno.sin_proyecto_agy = true;
    let p = peticion_agy();
    let guion = guion_powershell(&plan(&p, Sistema::Windows, &entorno));
    assert!(guion.contains("'--new-project'"), "{guion}");
    assert!(!guion.contains("'--project'"), "{guion}");
}

#[test]
fn gemini_terminal_con_los_proyectos_de_agy_ilegibles_no_se_lanza() {
    let agy = agy_de_localappdata();
    let mut entorno = EntornoDeEnsayo::default().con_archivo(agy);
    entorno.datos_locales = Some(PathBuf::from(r"C:\Users\u\AppData\Local"));
    entorno.proyectos_agy_ilegibles = true;
    let p = peticion_agy();
    match lanzador::planificar(&p, Sistema::Windows, &entorno) {
        Err(ErrorDeLanzamiento::SinProyectoDeAgy { carpeta }) => {
            assert_eq!(carpeta, p.carpeta_proyecto);
        }
        otro => panic!("se esperaba SinProyectoDeAgy y se obtuvo {otro:?}"),
    }
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

#[test]
fn solo_la_terminal_de_codex_lanza_el_vigilante_persistente() {
    let entorno = EntornoDeEnsayo::default().con_archivo(PathBuf::from("/bin/codex"));
    let mut codex = peticion_codex();
    codex.ejecutable = Some(PathBuf::from("/bin/codex"));
    for sistema in [Sistema::Windows, Sistema::Linux] {
        let plan_codex = plan(&codex, sistema, &entorno);
        let guion = match sistema {
            Sistema::Windows => guion_powershell(&plan_codex),
            Sistema::Linux => guion_bash(&plan_codex),
        };
        assert_eq!(
            guion
                .matches("--persistente --entrega codex --desde-terminal")
                .count(),
            1,
            "{guion}"
        );
        // El vigilante va antes de abrir Codex, para que lo encuentre al arrancar.
        assert!(
            guion.find("--desde-terminal") < guion.find("/bin/codex"),
            "{guion}"
        );
    }
    for herramienta in [Herramienta::Claude] {
        let mut otra = peticion_codex();
        otra.herramienta = herramienta;
        otra.ejecutable = Some(PathBuf::from("/bin/codex"));
        for sistema in [Sistema::Windows, Sistema::Linux] {
            let plan_otra = plan(&otra, sistema, &entorno);
            let guion = match sistema {
                Sistema::Windows => guion_powershell(&plan_otra),
                Sistema::Linux => guion_bash(&plan_otra),
            };
            assert!(
                !guion.contains("gestor-vigilante"),
                "{herramienta:?}: {guion}"
            );
        }
    }
}

#[test]
fn el_titulo_no_se_restaura_con_un_bloque_de_powershell_en_otro_hilo() {
    // Un bloque de PowerShell como TimerCallback tumbaba powershell.exe a los dos segundos
    // (PSInvalidOperation en ScriptBlock.GetContextFromTLS).
    let entorno = EntornoDeEnsayo::default().con_archivo(PathBuf::from("/bin/codex"));
    let mut p = peticion_codex();
    p.ejecutable = Some(PathBuf::from("/bin/codex"));
    let guion = guion_powershell(&plan(&p, Sistema::Windows, &entorno));
    assert!(!guion.contains("TimerCallback"), "{guion}");
    assert!(!guion.contains("]{"), "{guion}");
    assert!(guion.contains("Add-Type -TypeDefinition 'public static class GestorAgentesTitulo"));
    let fijar = guion
        .find("[GestorAgentesTitulo]::Fijar($t)")
        .expect("fija el título");
    let parar = guion
        .find("[GestorAgentesTitulo]::Parar()")
        .expect("para el temporizador");
    let programa = guion.find("/bin/codex").expect("abre el programa");
    assert!(fijar < programa && programa < parar, "{guion}");
}
