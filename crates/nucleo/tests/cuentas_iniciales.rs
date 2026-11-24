//! Barreras de A3 (3.7.0): una instalación nueva no trae las cuentas de nadie. Parte de una cuenta
//! por herramienta, la detección añade las instaladas y el alta a mano cubre las que viven fuera
//! de la carpeta personal. Una lista ya guardada no cambia.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::path::Path;

use comun::CarpetaTemporal;
use nucleo::configuracion::{self, Configuracion, HerramientaDelPerfil, PerfilDeAgente};
use nucleo::cuentas::{self, AltaRechazada, Cuenta, Herramienta, OrigenDeLaCuenta};
use nucleo::cuentas_de_agentes;

const PERSONAL: &str = "/home/u";

fn nombres(perfiles: &[PerfilDeAgente]) -> Vec<String> {
    perfiles.iter().map(PerfilDeAgente::nombre).collect()
}

/// Busca las cuentas de `personal` y pone los perfiles al día, como Gestor al buscar los agentes.
/// Cuántos perfiles nuevos añade una búsqueda sin variables de cuenta.
fn incorporar(perfiles: &mut Vec<PerfilDeAgente>, personal: &Path) -> usize {
    let manuales = cuentas::manuales_de(perfiles, personal);
    let encontradas = cuentas_de_agentes::cuentas_del_equipo(personal, |_| None, &manuales);
    cuentas::incorporar_cuentas(perfiles, &encontradas, personal).anadidos
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
    let personal = CarpetaTemporal::new();
    personal.escribir(".claude/.credentials.json", "{}");
    personal.escribir(".claude2/.credentials.json", "{}");
    let mut con = PerfilDeAgente::por_defecto();
    let anadidos = incorporar(&mut con, personal.ruta());
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

    let personal = CarpetaTemporal::new();
    personal.escribir(".claude/.credentials.json", "{}");
    personal.crear_carpeta(".claude2");
    let mut sin = PerfilDeAgente::por_defecto();
    let anadidos = incorporar(&mut sin, personal.ruta());
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
        &[],
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

    // Claude 1 es la cuenta por defecto, ~/.claude; da igual la barra final.
    assert_eq!(
        cuentas::dar_de_alta(
            &mut perfiles,
            HerramientaDelPerfil::Claude,
            &personal.join(".claude/"),
            personal,
            &[],
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
            &[],
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
                Path::new(PERSONAL),
                &[],
            ),
            Err(AltaRechazada::SinCuentas)
        );
    }
}

/// El alta se valida también contra las cuentas encontradas. Si la carpeta es la de una
/// cuenta que ya tiene perfil, el rechazo lleva su identidad; si no lo tiene, la carpeta.
#[test]
fn el_alta_a_mano_rechaza_la_carpeta_de_una_cuenta_encontrada() {
    let personal = Path::new(PERSONAL);
    let trabajo = personal.join(".claude-trabajo");
    let encontrada = Cuenta {
        herramienta: Herramienta::ClaudeCode,
        carpeta: trabajo.clone(),
        origen: OrigenDeLaCuenta::Busqueda,
    };
    let mut perfiles = PerfilDeAgente::por_defecto();

    assert_eq!(
        cuentas::dar_de_alta(
            &mut perfiles,
            HerramientaDelPerfil::Claude,
            &trabajo,
            personal,
            std::slice::from_ref(&encontrada),
        ),
        Err(AltaRechazada::CarpetaEnUso(trabajo.display().to_string()))
    );

    cuentas::incorporar_cuentas(&mut perfiles, std::slice::from_ref(&encontrada), personal);
    assert_eq!(
        cuentas::dar_de_alta(
            &mut perfiles,
            HerramientaDelPerfil::Claude,
            &trabajo,
            personal,
            std::slice::from_ref(&encontrada),
        ),
        Err(AltaRechazada::CarpetaEnUso("Claude 2".to_owned()))
    );
}

/// Una carpeta usada por otra herramienta no impide el alta.
#[test]
fn el_alta_a_mano_admite_la_carpeta_de_otra_herramienta() {
    let personal = Path::new(PERSONAL);
    let mut perfiles = PerfilDeAgente::por_defecto();

    assert_eq!(
        cuentas::dar_de_alta(
            &mut perfiles,
            HerramientaDelPerfil::Codex,
            &personal.join(".claude"),
            personal,
            &[],
        ),
        Ok("Codex 2".to_owned())
    );
}

/// El alta a mano compone el perfil como la búsqueda: una carpeta
/// de la personal se guarda con «~/» y pide el número de su nombre si está libre.
#[test]
fn el_alta_a_mano_de_una_carpeta_de_la_personal_se_guarda_como_la_busqueda() {
    let personal = Path::new(PERSONAL);
    let mut perfiles = PerfilDeAgente::por_defecto();

    let alta = cuentas::dar_de_alta(
        &mut perfiles,
        HerramientaDelPerfil::Claude,
        &personal.join(".claude5"),
        personal,
        &[],
    );

    assert_eq!(alta, Ok("Claude 5".to_owned()));
    assert_eq!(perfiles[1].directorio_configuracion(), Some("~/.claude5"));
}
