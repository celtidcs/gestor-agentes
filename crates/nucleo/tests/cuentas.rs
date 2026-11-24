//! Barreras de la puesta al día de los perfiles con las cuentas que encuentra la biblioteca
//! deteccion-agentes (3.9.0). Cada prueba crea las cuentas en una carpeta personal temporal y no
//! lee la carpeta personal ni las variables reales.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::ffi::OsString;
use std::path::Path;

use comun::CarpetaTemporal;
use nucleo::configuracion::{HerramientaDelPerfil, PerfilDeAgente};
use nucleo::cuentas::{self, Incorporacion};
use nucleo::cuentas_de_agentes;

/// Ninguna variable puesta.
fn sin_variables(_: &str) -> Option<OsString> {
    None
}

/// Busca las cuentas de `personal` con las manuales de `perfiles` y pone los perfiles al día, como
/// hace Gestor al buscar los agentes. Devuelve cuántos cambios ha hecho.
fn buscar_con(
    perfiles: &mut Vec<PerfilDeAgente>,
    personal: &Path,
    leer_variable: impl Fn(&str) -> Option<OsString>,
) -> Incorporacion {
    let manuales = cuentas::manuales_de(perfiles, personal);
    let encontradas = cuentas_de_agentes::cuentas_del_equipo(personal, leer_variable, &manuales);
    cuentas::incorporar_cuentas(perfiles, &encontradas, personal)
}

/// Cuántos perfiles nuevos añade una búsqueda sin variables de cuenta.
fn buscar(perfiles: &mut Vec<PerfilDeAgente>, personal: &Path) -> usize {
    buscar_con(perfiles, personal, sin_variables).anadidos
}

/// Una cuenta de Claude Code en `carpeta` de la personal.
fn claude(personal: &CarpetaTemporal, carpeta: &str) {
    personal.escribir(&format!("{carpeta}/.credentials.json"), "{}");
}

/// Una cuenta de Codex CLI en `carpeta` de la personal.
fn codex(personal: &CarpetaTemporal, carpeta: &str) {
    personal.escribir(&format!("{carpeta}/auth.json"), "{}");
}

fn nombres(perfiles: &[PerfilDeAgente]) -> Vec<String> {
    perfiles.iter().map(PerfilDeAgente::nombre).collect()
}

fn carpetas_de(perfiles: &[PerfilDeAgente]) -> Vec<Option<&str>> {
    perfiles
        .iter()
        .map(PerfilDeAgente::directorio_configuracion)
        .collect()
}

#[test]
fn sin_cuentas_no_hay_perfiles() {
    let personal = CarpetaTemporal::new();
    let mut perfiles = Vec::new();

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 0);
    assert!(perfiles.is_empty());
}

/// Desde la 3.9.0 la cuenta por defecto no fija carpeta; se abre quitando la variable.
#[test]
fn una_cuenta_de_claude_da_claude_1_sin_carpeta() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude");
    let mut perfiles = Vec::new();

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 1);
    assert_eq!(nombres(&perfiles), ["Claude 1"]);
    assert_eq!(carpetas_de(&perfiles), [None]);
}

#[test]
fn dos_cuentas_de_claude_dan_dos_perfiles_en_orden() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude2");
    claude(&personal, ".claude");
    let mut perfiles = Vec::new();

    buscar(&mut perfiles, personal.ruta());

    assert_eq!(nombres(&perfiles), ["Claude 1", "Claude 2"]);
    assert_eq!(carpetas_de(&perfiles), [None, Some("~/.claude2")]);
}

#[test]
fn las_cuentas_de_codex_fijan_carpeta_solo_desde_la_segunda() {
    let personal = CarpetaTemporal::new();
    codex(&personal, ".codex3");
    codex(&personal, ".codex");
    let mut perfiles = Vec::new();

    buscar(&mut perfiles, personal.ruta());

    assert_eq!(nombres(&perfiles), ["Codex 1", "Codex 3"]);
    assert_eq!(carpetas_de(&perfiles), [None, Some("~/.codex3")]);
}

#[test]
fn claude_va_antes_que_codex() {
    let personal = CarpetaTemporal::new();
    codex(&personal, ".codex");
    claude(&personal, ".claude");
    let mut perfiles = Vec::new();

    buscar(&mut perfiles, personal.ruta());

    assert_eq!(nombres(&perfiles), ["Claude 1", "Codex 1"]);
}

/// Cambio visible de la 3.9.0: la biblioteca decide qué es una cuenta. Una ~/.claude vacía
/// ya es la cuenta por defecto y una carpeta con sufijo vale con settings.json; una carpeta con
/// sufijo sin ningún archivo de cuenta no vale.
#[test]
fn la_biblioteca_decide_que_carpeta_es_una_cuenta() {
    let personal = CarpetaTemporal::new();
    personal.crear_carpeta(".claude");
    personal.escribir(".claude3/settings.json", "{}");
    personal.crear_carpeta(".claude2");
    personal.escribir(".codex4/otro.txt", "x");
    let mut perfiles = Vec::new();

    buscar(&mut perfiles, personal.ruta());

    assert_eq!(nombres(&perfiles), ["Claude 1", "Claude 3"]);
}

/// Una carpeta con un sufijo que no es un número es una cuenta con la instancia libre
/// siguiente.
#[test]
fn un_sufijo_no_numerico_es_una_cuenta_con_instancia_nueva() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude");
    claude(&personal, ".claude-trabajo");
    let mut perfiles = PerfilDeAgente::por_defecto();

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 1);

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
    assert_eq!(
        perfiles[1].directorio_configuracion(),
        Some("~/.claude-trabajo")
    );
}

/// Una cuenta ya guardada no cambia de identidad aunque aparezca otra que va antes en el
/// orden de la biblioteca; y una carpeta con número pide el suyo aunque vaya detrás.
#[test]
fn la_identidad_de_una_cuenta_guardada_es_estable() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude");
    claude(&personal, ".claude-trabajo");
    let mut perfiles = PerfilDeAgente::por_defecto();
    buscar(&mut perfiles, personal.ruta());
    assert_eq!(perfiles[1].nombre(), "Claude 2");

    claude(&personal, ".claude-a");
    claude(&personal, ".claude5");
    assert_eq!(buscar(&mut perfiles, personal.ruta()), 2);

    let claude_de = |carpeta: &str| {
        perfiles
            .iter()
            .find(|perfil| perfil.directorio_configuracion() == Some(carpeta))
            .map(PerfilDeAgente::nombre)
    };
    assert_eq!(claude_de("~/.claude-trabajo").as_deref(), Some("Claude 2"));
    assert_eq!(claude_de("~/.claude5").as_deref(), Some("Claude 5"));
    assert_eq!(claude_de("~/.claude-a").as_deref(), Some("Claude 6"));
}

/// Entre dos cuentas nuevas, la que tiene número lo pide antes que la otra tome el
/// siguiente libre.
#[test]
fn una_carpeta_con_numero_conserva_su_numero_frente_a_otra_nueva() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude");
    claude(&personal, ".claude-a");
    claude(&personal, ".claude2");
    let mut perfiles = PerfilDeAgente::por_defecto();

    buscar(&mut perfiles, personal.ruta());

    assert_eq!(
        carpetas_de(&perfiles[..3]),
        [None, Some("~/.claude2"), Some("~/.claude-a")]
    );
    assert_eq!(
        nombres(&perfiles[..3]),
        ["Claude 1", "Claude 2", "Claude 3"]
    );
}

/// El perfil guardado se casa con su cuenta aunque su carpeta lleve barra final y, en
/// Windows, otras mayúsculas y la otra barra.
#[test]
fn el_perfil_guardado_se_casa_con_su_cuenta_aunque_se_escriba_distinto() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude2");
    let escrita = if cfg!(windows) {
        format!("{}\\", personal.ruta().join(".CLAUDE2").display()).replace('/', "\\")
    } else {
        format!("{}/", personal.ruta().join(".claude2").display())
    };
    let mut perfiles =
        vec![PerfilDeAgente::new(HerramientaDelPerfil::Claude, 2).con_directorio(&escrita)];
    let antes = perfiles.clone();

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 0);
    assert_eq!(perfiles, antes);
}

/// La variable de cuenta del proceso de Gestor apuntando a una cuenta ya encontrada no la
/// duplica; apuntando a otra carpeta añade esa cuenta con su ruta completa.
#[test]
fn la_variable_del_proceso_no_duplica_y_anade_la_de_fuera() {
    let personal = CarpetaTemporal::new();
    let fuera = CarpetaTemporal::new();
    claude(&personal, ".claude");
    claude(&personal, ".claude2");
    let dentro = personal.ruta().join(".claude2");
    let mut perfiles = PerfilDeAgente::por_defecto();
    let a_la_segunda =
        |nombre: &str| (nombre == "CLAUDE_CONFIG_DIR").then(|| dentro.clone().into_os_string());

    assert_eq!(
        buscar_con(&mut perfiles, personal.ruta(), a_la_segunda).anadidos,
        1
    );
    assert_eq!(nombres(&perfiles[..2]), ["Claude 1", "Claude 2"]);

    let a_fuera =
        |nombre: &str| (nombre == "CLAUDE_CONFIG_DIR").then(|| fuera.ruta().as_os_str().to_owned());
    assert_eq!(
        buscar_con(&mut perfiles, personal.ruta(), a_fuera).anadidos,
        1
    );
    let fuera_texto = fuera.ruta().to_string_lossy().into_owned();
    assert_eq!(perfiles[2].nombre(), "Claude 3");
    assert_eq!(
        perfiles[2].directorio_configuracion(),
        Some(fuera_texto.as_str())
    );
}

/// Los nombres que se parecen: «claude» sin punto no es una cuenta, el archivo ~/.claude.json
/// basta para la cuenta por defecto y «.claude1» no pide el 1, que es el de la por defecto.
#[test]
fn las_carpetas_con_nombre_parecido() {
    let personal = CarpetaTemporal::new();
    personal.escribir(".claude.json", "{}");
    claude(&personal, "claude");
    claude(&personal, ".claude1");
    let mut perfiles = Vec::new();

    buscar(&mut perfiles, personal.ruta());

    assert_eq!(nombres(&perfiles), ["Claude 1", "Claude 2"]);
    assert_eq!(carpetas_de(&perfiles), [None, Some("~/.claude1")]);
}

/// agy no tiene cuentas separadas: encontrarlo no añade perfiles; su perfil de base lo crea la
/// configuración.
#[test]
fn agy_no_anade_perfiles() {
    let personal = CarpetaTemporal::new();
    personal.crear_carpeta(".gemini/antigravity-cli");
    let mut perfiles = Vec::new();

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 0);
    assert!(perfiles.is_empty());
}

/// Lista guardada de quien tiene dos cuentas de Claude: la inicial más Claude 2.
fn guardados() -> Vec<PerfilDeAgente> {
    let mut perfiles = PerfilDeAgente::por_defecto();
    perfiles.insert(
        1,
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 2).con_directorio("~/.claude2"),
    );
    perfiles
}

#[test]
fn incorporar_no_duplica_lo_que_ya_esta() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude");
    claude(&personal, ".claude2");
    codex(&personal, ".codex");
    let mut perfiles = guardados();
    let antes = perfiles.clone();

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 0);
    assert_eq!(perfiles, antes);
}

#[test]
fn incorporar_anade_la_cuenta_nueva_detras_de_las_de_su_herramienta() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude3");
    codex(&personal, ".codex2");
    let mut perfiles = guardados();

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 2);
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

/// Un perfil guardado no se quita ni cambia nunca. Si la cuenta por defecto no es la de ningún
/// perfil (Claude 1 fija otra carpeta), se añade con la siguiente identidad libre.
#[test]
fn incorporar_nunca_cambia_ni_quita_un_perfil_guardado() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude");
    let mut perfiles = vec![
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1)
            .con_directorio("D:/cuentas/claude")
            .con_ejecutable("D:/bin/claude.exe"),
        PerfilDeAgente::new(HerramientaDelPerfil::Codex, 5),
    ];
    let antes = perfiles.clone();

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 1);
    assert_eq!(perfiles[0], antes[0]);
    assert_eq!(perfiles[2], antes[1]);
    assert_eq!(perfiles[1].nombre(), "Claude 2");
    assert_eq!(perfiles[1].directorio_configuracion(), None);
}

/// El Claude 1 guardado hasta la 3.8 con «~/.claude» deja de fijarla (es la cuenta por
/// defecto), y el cambio cuenta para guardar, pero no como cuenta nueva. Igual con la ruta completa o la barra invertida.
#[test]
fn el_claude_1_de_la_3_8_deja_de_fijar_su_carpeta() {
    let personal = CarpetaTemporal::new();
    let completa = personal
        .ruta()
        .join(".claude")
        .to_string_lossy()
        .into_owned();
    for carpeta in ["~/.claude", "~\\.claude", completa.as_str()] {
        let mut perfiles =
            vec![PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1).con_directorio(carpeta)];

        let primera = buscar_con(&mut perfiles, personal.ruta(), sin_variables);
        assert_eq!(
            primera,
            Incorporacion {
                normalizados: 1,
                anadidos: 0
            },
            "{carpeta}"
        );
        assert!(primera.hay_cambios());
        assert_eq!(carpetas_de(&perfiles), [None], "{carpeta}");
        let segunda = buscar_con(&mut perfiles, personal.ruta(), sin_variables);
        assert!(!segunda.hay_cambios(), "{carpeta}");
    }
}

#[test]
fn incorporar_sin_perfiles_de_esa_herramienta_anade_al_final() {
    let personal = CarpetaTemporal::new();
    codex(&personal, ".codex");
    let mut perfiles = vec![PerfilDeAgente::new(HerramientaDelPerfil::Gemini, 1)];

    buscar(&mut perfiles, personal.ruta());

    assert_eq!(nombres(&perfiles), ["Gemini 1", "Codex 1"]);
}

/// La carpeta de la cuenta de cada perfil sale de la biblioteca: la que fija o la por defecto de su
/// herramienta; agy siempre la suya y Programator ninguna.
#[test]
fn la_carpeta_de_la_cuenta_de_cada_perfil() {
    let casa = Path::new("/h");
    let carpeta = |p: &PerfilDeAgente| cuentas::carpeta_de_la_cuenta(p, casa);
    assert_eq!(
        carpeta(&PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1)),
        Some(casa.join(".claude"))
    );
    assert_eq!(
        carpeta(&PerfilDeAgente::new(HerramientaDelPerfil::Codex, 2).con_directorio("~/.codex2")),
        Some(casa.join(".codex2"))
    );
    assert_eq!(
        carpeta(&PerfilDeAgente::new(HerramientaDelPerfil::Gemini, 2).con_directorio("~/otra")),
        Some(casa.join(".gemini"))
    );
    assert_eq!(
        carpeta(&PerfilDeAgente::new(HerramientaDelPerfil::Programator, 1)),
        None
    );
}

/// Una carpeta con cero delante, como «.claude02», no pide su número; toma el
/// siguiente al mayor.
#[test]
fn un_numero_con_cero_delante_no_pide_su_instancia() {
    let personal = CarpetaTemporal::new();
    claude(&personal, ".claude");
    claude(&personal, ".claude02");
    let mut perfiles = vec![
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1),
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 5).con_directorio("D:/cuentas/cinco"),
    ];

    buscar(&mut perfiles, personal.ruta());

    assert_eq!(nombres(&perfiles), ["Claude 1", "Claude 5", "Claude 6"]);
    assert_eq!(perfiles[2].directorio_configuracion(), Some("~/.claude02"));
}

/// Un perfil de otra herramienta que fija la misma carpeta no se toma por el de
/// la cuenta; la cuenta recibe su propio perfil.
#[test]
fn un_perfil_de_otra_herramienta_con_la_misma_carpeta_no_es_el_de_la_cuenta() {
    let personal = CarpetaTemporal::new();
    codex(&personal, ".codex");
    claude(&personal, ".claude2");
    let mut perfiles = vec![
        PerfilDeAgente::new(HerramientaDelPerfil::Codex, 1),
        PerfilDeAgente::new(HerramientaDelPerfil::Codex, 2).con_directorio("~/.claude2"),
    ];

    assert_eq!(buscar(&mut perfiles, personal.ruta()), 1);

    assert_eq!(nombres(&perfiles), ["Codex 1", "Codex 2", "Claude 2"]);
}

/// Una cuenta cuya carpeta no es texto válido no añade perfil,
/// porque el perfil no podría guardar su ruta tal cual y no volvería a casar con ella.
#[cfg(unix)]
#[test]
fn una_carpeta_que_no_es_texto_valido_no_anade_perfil() {
    use std::os::unix::ffi::OsStrExt;
    let personal = CarpetaTemporal::new();
    let fuera = CarpetaTemporal::new();
    let rara = fuera
        .ruta()
        .join(std::ffi::OsStr::from_bytes(b"cuenta-\xff"));
    std::fs::create_dir_all(&rara).unwrap();
    std::fs::write(rara.join(".credentials.json"), "{}").unwrap();
    let mut perfiles = Vec::new();
    let variable =
        |nombre: &str| (nombre == "CLAUDE_CONFIG_DIR").then(|| rara.clone().into_os_string());

    let cambios = buscar_con(&mut perfiles, personal.ruta(), variable);

    assert_eq!(cambios.anadidos, 0);
    assert!(perfiles.is_empty());
}

/// La barra final no distingue carpetas, tampoco si no existen y no
/// se pueden resolver a su forma canónica. Una cuenta bajo una carpeta personal escrita con barra
/// final sigue pidiendo su número y guardándose con «~/».
#[test]
fn la_barra_final_no_distingue_carpetas_que_no_existen() {
    let personal = Path::new("/no-existe/casa/");
    let cuenta = cuentas::Cuenta {
        herramienta: cuentas::Herramienta::ClaudeCode,
        carpeta: Path::new("/no-existe/casa").join(".claude7"),
        origen: cuentas::OrigenDeLaCuenta::Busqueda,
    };
    let mut perfiles = Vec::new();

    cuentas::incorporar_cuentas(&mut perfiles, &[cuenta], personal);

    assert_eq!(nombres(&perfiles), ["Claude 7"]);
    assert_eq!(perfiles[0].directorio_configuracion(), Some("~/.claude7"));
}
