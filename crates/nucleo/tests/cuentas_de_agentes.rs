//! Las seis pruebas mínimas que la norma de integración de deteccion-agentes (N10) exige a todo
//! programa que la usa, sobre el módulo cuentas_de_agentes y la traducción de Gestor. Trabajan en
//! carpetas temporales propias, con una función de variables propia, sin leer la carpeta personal
//! ni el entorno reales.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::ffi::OsString;
use std::path::PathBuf;

use comun::CarpetaTemporal;
use nucleo::configuracion::{HerramientaDelPerfil, PerfilDeAgente};
use nucleo::cuentas::{self, AjusteDeEntorno, Herramienta, OrigenDeLaCuenta, RechazoDeAlta};
use nucleo::cuentas_de_agentes::{
    carpeta_personal, cuentas_del_equipo, validar_alta, CuentaManual,
};

/// Ninguna variable puesta.
fn sin_variables(_: &str) -> Option<OsString> {
    None
}

/// N10.1: la cuenta por defecto se encuentra y su consola quita la variable.
#[test]
fn la_cuenta_por_defecto_se_encuentra_y_quita_la_variable() {
    let personal = CarpetaTemporal::new();
    personal.escribir(".claude/settings.json", "{}");

    let encontradas = cuentas_del_equipo(personal.ruta(), sin_variables, &[]);

    assert_eq!(encontradas.len(), 1, "{encontradas:?}");
    assert_eq!(encontradas[0].origen, OrigenDeLaCuenta::PorDefecto);
    let claude_1 = PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1);
    assert_eq!(
        cuentas::ajuste_de_lanzamiento(&claude_1, personal.ruta()),
        Some(AjusteDeEntorno::Quitar {
            variable: "CLAUDE_CONFIG_DIR"
        })
    );
}

/// N10.2: una segunda cuenta se encuentra por su nombre y su consola pone la variable con su
/// carpeta.
#[test]
fn una_segunda_cuenta_se_encuentra_y_pone_la_variable() {
    let personal = CarpetaTemporal::new();
    personal.escribir(".codex-trabajo/config.toml", "");
    let segunda = personal.ruta().join(".codex-trabajo");

    let encontradas = cuentas_del_equipo(personal.ruta(), sin_variables, &[]);
    let cuenta = encontradas
        .iter()
        .find(|cuenta| cuenta.carpeta == segunda)
        .expect("la segunda cuenta tiene que encontrarse");
    assert_eq!(cuenta.origen, OrigenDeLaCuenta::Busqueda);

    let mut perfiles = Vec::new();
    cuentas::incorporar_cuentas(&mut perfiles, &encontradas, personal.ruta());
    assert_eq!(perfiles.len(), 1);
    assert_eq!(
        cuentas::ajuste_de_lanzamiento(&perfiles[0], personal.ruta()),
        Some(AjusteDeEntorno::Poner {
            variable: "CODEX_HOME",
            valor: segunda,
        })
    );
}

/// N10.3: una carpeta con el nombre pero sin archivos de cuenta no es una cuenta.
#[test]
fn una_carpeta_sin_archivos_de_cuenta_no_cuenta() {
    let personal = CarpetaTemporal::new();
    personal.crear_carpeta(".claude-vacia");

    assert!(cuentas_del_equipo(personal.ruta(), sin_variables, &[]).is_empty());
}

/// N10.4: la carpeta de la variable de entorno cuenta aunque esté fuera de la carpeta personal.
#[test]
fn la_carpeta_de_la_variable_cuenta_aunque_este_fuera() {
    let personal = CarpetaTemporal::new();
    let fuera = CarpetaTemporal::new();
    let leer =
        |nombre: &str| (nombre == "CLAUDE_CONFIG_DIR").then(|| fuera.ruta().as_os_str().to_owned());

    let encontradas = cuentas_del_equipo(personal.ruta(), leer, &[]);

    assert!(
        encontradas
            .iter()
            .any(|c| c.carpeta == fuera.ruta() && c.origen == OrigenDeLaCuenta::Variable),
        "{encontradas:?}"
    );
}

/// N10.5: una cuenta manual se añade y se conserva aunque su carpeta no exista; una de agy no se
/// admite, y el alta de una carpeta ya usada se rechaza con su motivo.
#[test]
fn las_cuentas_manuales_se_anaden_y_las_de_agy_no() {
    let personal = CarpetaTemporal::new();
    let manuales = [
        CuentaManual {
            herramienta: Herramienta::ClaudeCode,
            carpeta: personal.archivo("ya-no-existe"),
        },
        CuentaManual {
            herramienta: Herramienta::Agy,
            carpeta: personal.archivo("otra-de-agy"),
        },
    ];

    let encontradas = cuentas_del_equipo(personal.ruta(), sin_variables, &manuales);

    assert_eq!(encontradas.len(), 1, "{encontradas:?}");
    assert_eq!(encontradas[0].origen, OrigenDeLaCuenta::Manual);
    assert_eq!(encontradas[0].herramienta, Herramienta::ClaudeCode);
    assert_eq!(
        validar_alta(&manuales[1], &encontradas),
        Err(RechazoDeAlta::SinVariableDeCuenta)
    );
    assert!(matches!(
        validar_alta(&manuales[0], &encontradas),
        Err(RechazoDeAlta::CarpetaYaUsada(_))
    ));
}

/// N10.6: la carpeta personal sale de la variable del sistema, y vacía no vale.
#[test]
fn la_carpeta_personal_sale_de_la_variable_del_sistema() {
    let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let leer = |nombre: &str| (nombre == variable).then(|| OsString::from("/casa/persona"));

    assert_eq!(carpeta_personal(leer), Some(PathBuf::from("/casa/persona")));
    assert_eq!(carpeta_personal(|_| Some(OsString::new())), None);
    assert_eq!(carpeta_personal(sin_variables), None);
}
