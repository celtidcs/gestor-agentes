//! Barreras de la detección de cuentas y de su incorporación a los perfiles guardados.

mod comun;

use comun::CarpetaTemporal;
use nucleo::configuracion::{HerramientaDelPerfil, PerfilDeAgente};
use nucleo::cuentas::{self, CarpetaPersonal};

fn carpeta(nombre: &str, archivos: &[&str]) -> CarpetaPersonal {
    CarpetaPersonal {
        nombre: nombre.to_owned(),
        archivos: archivos.iter().map(|&archivo| archivo.to_owned()).collect(),
    }
}

fn claude(nombre: &str) -> CarpetaPersonal {
    carpeta(nombre, &[".credentials.json", "settings.json"])
}

fn codex(nombre: &str) -> CarpetaPersonal {
    carpeta(nombre, &["auth.json", "config.toml"])
}

fn nombres(perfiles: &[PerfilDeAgente]) -> Vec<String> {
    perfiles.iter().map(PerfilDeAgente::nombre).collect()
}

fn carpetas_de(perfiles: &[PerfilDeAgente]) -> Vec<Option<&str>> {
    perfiles
        .iter()
        .map(|perfil| perfil.directorio_configuracion())
        .collect()
}

#[test]
fn sin_cuentas_no_hay_perfiles() {
    assert!(cuentas::perfiles_de_cuentas(&[]).is_empty());
}

#[test]
fn una_cuenta_de_claude_da_claude_1_con_su_carpeta() {
    let perfiles = cuentas::perfiles_de_cuentas(&[claude(".claude")]);

    assert_eq!(nombres(&perfiles), ["Claude 1"]);
    assert_eq!(carpetas_de(&perfiles), [Some("~/.claude")]);
}

#[test]
fn dos_cuentas_de_claude_dan_dos_perfiles_en_orden() {
    let perfiles = cuentas::perfiles_de_cuentas(&[claude(".claude2"), claude(".claude")]);

    assert_eq!(nombres(&perfiles), ["Claude 1", "Claude 2"]);
    assert_eq!(
        carpetas_de(&perfiles),
        [Some("~/.claude"), Some("~/.claude2")]
    );
}

#[test]
fn las_cuentas_de_codex_fijan_carpeta_solo_desde_la_segunda() {
    let perfiles = cuentas::perfiles_de_cuentas(&[codex(".codex3"), codex(".codex")]);

    assert_eq!(nombres(&perfiles), ["Codex 1", "Codex 3"]);
    assert_eq!(carpetas_de(&perfiles), [None, Some("~/.codex3")]);
}

#[test]
fn claude_va_antes_que_codex() {
    let perfiles = cuentas::perfiles_de_cuentas(&[codex(".codex"), claude(".claude")]);

    assert_eq!(nombres(&perfiles), ["Claude 1", "Codex 1"]);
}

#[test]
fn una_carpeta_sin_archivo_de_sesion_no_es_una_cuenta() {
    let perfiles = cuentas::perfiles_de_cuentas(&[
        carpeta(".claude", &["settings.json"]),
        carpeta(".codex", &["config.toml"]),
        carpeta(".claude2", &[]),
    ]);

    assert!(perfiles.is_empty(), "{perfiles:?}");
}

#[test]
fn el_archivo_de_sesion_de_otra_herramienta_no_vale() {
    let perfiles = cuentas::perfiles_de_cuentas(&[
        carpeta(".claude", &["auth.json"]),
        carpeta(".codex", &[".credentials.json"]),
    ]);

    assert!(perfiles.is_empty(), "{perfiles:?}");
}

#[test]
fn las_carpetas_con_nombre_parecido_no_cuentan() {
    let perfiles = cuentas::perfiles_de_cuentas(&[
        claude(".claude-antigua"),
        claude(".claude1"),
        claude(".claude02"),
        claude(".claude.json"),
        claude("claude"),
        claude(".claudeX2"),
        codex(".codex-global"),
    ]);

    assert!(perfiles.is_empty(), "{perfiles:?}");
}

fn guardados() -> Vec<PerfilDeAgente> {
    PerfilDeAgente::por_defecto()
}

#[test]
fn incorporar_no_duplica_lo_que_ya_esta() {
    let mut perfiles = guardados();
    let antes = perfiles.clone();

    let anadidos = cuentas::incorporar(
        &mut perfiles,
        cuentas::perfiles_de_cuentas(&[claude(".claude"), claude(".claude2"), codex(".codex")]),
    );

    assert_eq!(anadidos, 0);
    assert_eq!(perfiles, antes);
}

#[test]
fn incorporar_anade_la_cuenta_nueva_detras_de_las_de_su_herramienta() {
    let mut perfiles = guardados();

    let anadidos = cuentas::incorporar(
        &mut perfiles,
        cuentas::perfiles_de_cuentas(&[claude(".claude3"), codex(".codex2")]),
    );

    assert_eq!(anadidos, 2);
    assert_eq!(
        nombres(&perfiles),
        [
            "Claude 1",
            "Claude 2",
            "Claude 3",
            "Codex 1",
            "Codex 2",
            "Gemini 1",
            "Programator 1"
        ]
    );
}

#[test]
fn incorporar_nunca_cambia_ni_quita_un_perfil_guardado() {
    let mut perfiles = vec![
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1)
            .con_directorio("D:/cuentas/claude")
            .con_ejecutable("D:/bin/claude.exe"),
        PerfilDeAgente::new(HerramientaDelPerfil::Codex, 5),
    ];
    let antes = perfiles.clone();

    let anadidos = cuentas::incorporar(
        &mut perfiles,
        cuentas::perfiles_de_cuentas(&[claude(".claude")]),
    );

    assert_eq!(anadidos, 0);
    assert_eq!(perfiles, antes);
}

#[test]
fn incorporar_sin_perfiles_de_esa_herramienta_anade_al_final() {
    let mut perfiles = vec![PerfilDeAgente::new(HerramientaDelPerfil::Gemini, 1)];

    cuentas::incorporar(
        &mut perfiles,
        cuentas::perfiles_de_cuentas(&[codex(".codex")]),
    );

    assert_eq!(nombres(&perfiles), ["Gemini 1", "Codex 1"]);
}

#[test]
fn explorar_lee_las_carpetas_de_cuentas_y_sus_archivos() {
    let personal = CarpetaTemporal::new();
    personal.escribir(".claude/.credentials.json", "{}");
    personal.escribir(".claude2/.credentials.json", "{}");
    personal.escribir(".claude2/sub/otro.json", "{}");
    personal.escribir(".codex/auth.json", "{}");
    personal.escribir(".claude.json", "{}");
    personal.escribir("Documentos/nota.txt", "x");

    let mut carpetas = cuentas::explorar(personal.ruta());
    carpetas.sort_by(|a, b| a.nombre.cmp(&b.nombre));

    assert_eq!(
        carpetas,
        [
            carpeta(".claude", &[".credentials.json"]),
            carpeta(".claude2", &[".credentials.json"]),
            carpeta(".codex", &["auth.json"]),
        ]
    );
    assert_eq!(
        nombres(&cuentas::perfiles_de_cuentas(&carpetas)),
        ["Claude 1", "Claude 2", "Codex 1"]
    );
}

#[test]
fn explorar_una_carpeta_inexistente_no_falla() {
    let personal = CarpetaTemporal::new();

    assert!(cuentas::explorar(&personal.archivo("no-existe")).is_empty());
}
