//! Barreras del plan de Claude Code.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default,
    missing_docs
)]

mod comun;

use std::path::PathBuf;

use comun::{guion_bash, guion_powershell, peticion, plan, textos, EntornoDeEnsayo};
use lanzador::{ErrorDeLanzamiento, Herramienta, Sistema};

/// Parte fija del principio del guion de PowerShell, tras el título.
const TEMPORIZADOR: &str = "[Console]::Title = $t; \
    $cb = [System.Threading.TimerCallback]{param($s) [Console]::Title = $s}; \
    $tmr = New-Object System.Threading.Timer($cb, $t, 0, 1000); ";

/// Parte fija del final del guion de PowerShell.
const FINAL_POWERSHELL: &str = "; $tmr.Dispose(); [Console]::Title = $t";

#[test]
fn windows_abre_powershell_con_el_guion_completo() {
    let mut p = peticion(Herramienta::Claude);
    p.carpeta_proyecto = PathBuf::from(r"C:\proyectos\Demo");
    p.directorio_configuracion = Some(PathBuf::from(r"C:\Users\usuario\.claude2"));
    p.ejecutable = Some(PathBuf::from(r"C:\herramientas\claude.exe"));
    let entorno = EntornoDeEnsayo::default().con_archivo(r"C:\herramientas\claude.exe");
    let plan = plan(&p, Sistema::Windows, &entorno);

    let esperado = format!(
        "$t = 'Claude 2 · Demo'; {TEMPORIZADOR}\
         $env:GESTOR_AGENTE = 'Claude 2'; $env:CLAUDECODE = ''; \
         $env:CLAUDE_CONFIG_DIR = 'C:\\Users\\usuario\\.claude2'; \
         Set-Location -LiteralPath 'C:\\proyectos\\Demo'; \
         & 'C:\\herramientas\\claude.exe'{FINAL_POWERSHELL}"
    );
    assert_eq!(guion_powershell(&plan), esperado);
    assert_eq!(plan.carpeta, PathBuf::from(r"C:\proyectos\Demo"));
    assert_eq!(
        plan.carpetas_a_crear,
        vec![PathBuf::from(r"C:\Users\usuario\.claude2")]
    );
    assert!(
        plan.variables.is_empty(),
        "las variables van dentro del guion"
    );
}

#[test]
fn sin_carpeta_de_configuracion_no_se_fija_claude_config_dir_ni_se_crea_nada() {
    let mut p = peticion(Herramienta::Claude);
    p.ejecutable = Some(PathBuf::from("claude"));
    let entorno = EntornoDeEnsayo::default().con_archivo("claude");
    let windows = plan(&p, Sistema::Windows, &entorno);
    assert!(!guion_powershell(&windows).contains("CLAUDE_CONFIG_DIR"));
    assert!(windows.carpetas_a_crear.is_empty());
    let linux = plan(&p, Sistema::Linux, &entorno);
    assert!(!guion_bash(&linux).contains("CLAUDE_CONFIG_DIR"));
    assert!(linux.carpetas_a_crear.is_empty());
}

#[test]
fn linux_abre_gnome_terminal_si_esta_con_el_guion_completo() {
    let mut p = peticion(Herramienta::Claude);
    p.directorio_configuracion = Some(PathBuf::from("/home/usuario/.claude2"));
    p.ejecutable = Some(PathBuf::from("/home/usuario/.local/bin/claude"));
    let entorno = EntornoDeEnsayo::default()
        .con_path("/usr/bin")
        .con_archivo("/usr/bin/gnome-terminal")
        .con_archivo("/usr/bin/xterm")
        .con_archivo("/home/usuario/.local/bin/claude");
    let plan = plan(&p, Sistema::Linux, &entorno);

    let guion = "export GESTOR_AGENTE='Claude 2'; export CLAUDECODE=''; \
                 export CLAUDE_CONFIG_DIR='/home/usuario/.claude2'; \
                 cd '/proyectos/Demo'; '/home/usuario/.local/bin/claude'; exec bash";
    assert_eq!(plan.programa, PathBuf::from("gnome-terminal"));
    assert_eq!(
        textos(&plan.argumentos),
        ["--title=Claude 2 · Demo", "--", "bash", "-c", guion]
    );
}

#[test]
fn linux_elige_el_emulador_por_orden_de_preferencia() {
    let mut p = peticion(Herramienta::Claude);
    p.ejecutable = Some(PathBuf::from("claude"));
    let titulo = "Claude 2 · Demo";
    let casos: [(&[&str], &str, Vec<&str>); 4] = [
        (
            &["konsole", "xfce4-terminal", "xterm"],
            "konsole",
            vec!["--title", titulo, "-e"],
        ),
        (
            &["xfce4-terminal", "xterm"],
            "xfce4-terminal",
            vec!["--title=Claude 2 · Demo", "-x"],
        ),
        (&["xterm"], "xterm", vec!["-T", titulo, "-e"]),
        // Sin ninguno instalado se recurre a xterm, como la 2.3.2.
        (&[], "xterm", vec!["-T", titulo, "-e"]),
    ];
    for (instalados, programa, principio) in casos {
        let mut entorno = EntornoDeEnsayo::default()
            .con_path("/usr/bin")
            .con_archivo("claude");
        for nombre in instalados {
            entorno = entorno.con_archivo(format!("/usr/bin/{nombre}"));
        }
        let plan = plan(&p, Sistema::Linux, &entorno);
        assert_eq!(
            plan.programa,
            PathBuf::from(programa),
            "instalados: {instalados:?}"
        );
        let args = textos(&plan.argumentos);
        assert_eq!(&args[..principio.len()], principio.as_slice(), "{programa}");
        assert_eq!(
            &args[principio.len()..principio.len() + 2],
            ["bash", "-c"],
            "{programa}"
        );
        assert_eq!(args.len(), principio.len() + 3, "{programa}");
    }
}

#[test]
fn windows_busca_claude_en_local_bin_antes_que_en_el_path() {
    let personal = PathBuf::from(r"C:\Users\usuario");
    let local = personal.join(".local").join("bin").join("claude.exe");
    let en_path = PathBuf::from(r"C:\npm").join("claude.cmd");
    let mut entorno = EntornoDeEnsayo::default()
        .con_path(r"C:\npm")
        .con_archivo(en_path.clone())
        .con_archivo(local.clone());
    entorno.personal = Some(personal);
    let p = peticion(Herramienta::Claude);
    let guion = guion_powershell(&plan(&p, Sistema::Windows, &entorno));
    assert!(
        guion.contains(&format!("& '{}'", local.display())),
        "{guion}"
    );

    // Sin el de ~/.local/bin, vale el lanzador .cmd de npm del PATH.
    entorno.archivos.remove(&local);
    let guion = guion_powershell(&plan(&p, Sistema::Windows, &entorno));
    assert!(
        guion.contains(&format!("& '{}'", en_path.display())),
        "{guion}"
    );
}

/// En la 2.3.2, si no se encontraba claude se confiaba en que el intérprete lo encontrase por
/// su nombre. Ahora es un error, para que el lanzamiento no discrepe de la detección que
/// decide si el botón se muestra.
#[test]
fn si_no_encuentra_claude_no_lanza_y_dice_donde_lo_ha_buscado() {
    let p = peticion(Herramienta::Claude);
    for (sistema, busqueda) in [
        (Sistema::Windows, "claude.cmd en ~/.local/bin ni en el PATH"),
        (Sistema::Linux, "/usr/bin ni en el PATH"),
    ] {
        let resultado = lanzador::planificar(&p, sistema, &EntornoDeEnsayo::default());
        match resultado {
            Err(ErrorDeLanzamiento::NoEncontrada {
                herramienta,
                detalle,
            }) => {
                assert_eq!(herramienta, "Claude Code");
                assert!(detalle.contains(busqueda), "{detalle}");
            }
            otro => panic!("se esperaba NoEncontrada y se obtuvo {otro:?}"),
        }
    }
}

#[test]
fn una_ruta_de_claude_fijada_que_no_existe_no_se_lanza() {
    let mut p = peticion(Herramienta::Claude);
    p.ejecutable = Some(PathBuf::from("/no/existe/claude"));
    let entorno = EntornoDeEnsayo::default()
        .con_path("/usr/bin")
        .con_archivo("/usr/bin/claude");
    let resultado = lanzador::planificar(&p, Sistema::Linux, &entorno);
    assert!(
        matches!(resultado, Err(ErrorDeLanzamiento::NoEncontrada { ref detalle, .. }) if detalle.contains("/no/existe/claude")),
        "{resultado:?}"
    );
}

#[test]
fn linux_busca_claude_en_local_bin_y_carpetas_del_sistema_por_orden() {
    let mut entorno = EntornoDeEnsayo::default()
        .con_archivo("/usr/bin/claude")
        .con_archivo("/usr/local/bin/claude");
    entorno.personal = Some(PathBuf::from("/home/usuario"));
    let p = peticion(Herramienta::Claude);
    // La ruta esperada se compone con join, como en el código: al planificar para Linux desde
    // Windows (solo pasa en las pruebas), el separador es el del sistema que ejecuta.
    let sistema = format!(
        "'{}'; exec bash",
        PathBuf::from("/usr/local/bin").join("claude").display()
    );
    assert!(guion_bash(&plan(&p, Sistema::Linux, &entorno)).contains(&sistema));
    entorno = entorno.con_archivo(
        PathBuf::from("/home/usuario")
            .join(".local")
            .join("bin")
            .join("claude"),
    );
    let esperado = format!(
        "'{}'; exec bash",
        PathBuf::from("/home/usuario")
            .join(".local")
            .join("bin")
            .join("claude")
            .display()
    );
    assert!(guion_bash(&plan(&p, Sistema::Linux, &entorno)).contains(&esperado));
}
