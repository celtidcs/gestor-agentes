//! Barreras de A3 (3.7.0): una instalación nueva no trae las cuentas de nadie. Parte de una cuenta
//! por herramienta, la detección añade las instaladas y el alta a mano cubre las que viven fuera
//! de la carpeta personal. Una lista ya guardada no cambia.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::path::Path;

use comun::CarpetaTemporal;
use nucleo::configuracion::{self, Configuracion, HerramientaDelPerfil, PerfilDeAgente};
use nucleo::cuentas::{self, AltaRechazada, CarpetaPersonal};

const PERSONAL: &str = "/home/u";

fn nombres(perfiles: &[PerfilDeAgente]) -> Vec<String> {
    perfiles.iter().map(PerfilDeAgente::nombre).collect()
}

fn cuenta_de_claude(nombre: &str) -> CarpetaPersonal {
    CarpetaPersonal {
        nombre: nombre.to_owned(),
        archivos: vec![".credentials.json".to_owned()],
    }
}

#[test]
fn una_instalacion_nueva_trae_una_cuenta_de_cada_herramienta() {
    assert_eq!(
        nombres(&PerfilDeAgente::por_defecto()),
        ["Claude 1", "Codex 1", "Gemini 1", "Programator 1"]
    );
}

#[test]
fn la_deteccion_anade_claude_2_solo_si_su_cuenta_existe() {
    let mut con = PerfilDeAgente::por_defecto();
    let anadidos = cuentas::incorporar(
        &mut con,
        cuentas::perfiles_de_cuentas(&[cuenta_de_claude(".claude"), cuenta_de_claude(".claude2")]),
    );
    assert_eq!(anadidos, 1);
    assert_eq!(
        nombres(&con),
        [
            "Claude 1",
            "Claude 2",
            "Codex 1",
            "Gemini 1",
            "Programator 1"
        ]
    );

    let mut sin = PerfilDeAgente::por_defecto();
    let carpeta_sin_sesion = CarpetaPersonal {
        nombre: ".claude2".to_owned(),
        archivos: Vec::new(),
    };
    let anadidos = cuentas::incorporar(
        &mut sin,
        cuentas::perfiles_de_cuentas(&[cuenta_de_claude(".claude"), carpeta_sin_sesion]),
    );
    assert_eq!(anadidos, 0);
    assert_eq!(nombres(&sin), nombres(&PerfilDeAgente::por_defecto()));
}

#[test]
fn una_lista_guardada_con_claude_2_se_conserva() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        configuracion::ARCHIVO_PERFILES,
        r#"[{"Tipo":"Claude","Instancia":1,"DirectorioConfiguracion":"~/.claude"},{"Tipo":"Claude","Instancia":2,"DirectorioConfiguracion":"~/.claude2"},{"Tipo":"Codex","Instancia":1},{"Tipo":"Gemini","Instancia":1},{"Tipo":"Programator","Instancia":1}]"#,
    );
    carpeta.escribir(configuracion::ARCHIVO_VERSION_PERFILES, "4");

    let perfiles = Configuracion::new(carpeta.ruta())
        .cargar_perfiles()
        .unwrap();

    assert_eq!(
        nombres(&perfiles),
        [
            "Claude 1",
            "Claude 2",
            "Codex 1",
            "Gemini 1",
            "Programator 1"
        ]
    );
}

#[test]
fn el_alta_a_mano_crea_la_instancia_siguiente_detras_de_su_herramienta() {
    let mut perfiles = PerfilDeAgente::por_defecto();

    let alta = cuentas::dar_de_alta(
        &mut perfiles,
        HerramientaDelPerfil::Codex,
        Path::new("D:/cuentas/codex-trabajo"),
        Path::new(PERSONAL),
    );

    assert_eq!(alta, Ok("Codex 2".to_owned()));
    assert_eq!(
        nombres(&perfiles),
        [
            "Claude 1",
            "Codex 1",
            "Codex 2",
            "Gemini 1",
            "Programator 1"
        ]
    );
    assert_eq!(
        perfiles[2].directorio_configuracion(),
        Some("D:/cuentas/codex-trabajo")
    );
}

#[test]
fn el_alta_a_mano_rechaza_una_carpeta_que_ya_usa_otro_perfil() {
    let mut perfiles = PerfilDeAgente::por_defecto();
    let antes = perfiles.clone();
    let personal = Path::new(PERSONAL);

    // Claude 1 fija «~/.claude»; da igual la barra final o la barra invertida.
    assert_eq!(
        cuentas::dar_de_alta(
            &mut perfiles,
            HerramientaDelPerfil::Claude,
            &personal.join(".claude/"),
            personal,
        ),
        Err(AltaRechazada::CarpetaEnUso("Claude 1".to_owned()))
    );
    // Codex 1 no fija carpeta: usa ~/.codex.
    assert_eq!(
        cuentas::dar_de_alta(
            &mut perfiles,
            HerramientaDelPerfil::Codex,
            &personal.join(".codex"),
            personal,
        ),
        Err(AltaRechazada::CarpetaEnUso("Codex 1".to_owned()))
    );
    assert_eq!(perfiles, antes);
}

#[test]
fn gemini_y_programator_no_tienen_alta_de_cuentas() {
    let mut perfiles = PerfilDeAgente::por_defecto();
    for herramienta in [
        HerramientaDelPerfil::Gemini,
        HerramientaDelPerfil::Programator,
    ] {
        assert_eq!(
            cuentas::dar_de_alta(
                &mut perfiles,
                herramienta,
                Path::new("D:/x"),
                Path::new(PERSONAL)
            ),
            Err(AltaRechazada::SinCuentas)
        );
    }
}
