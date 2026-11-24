//! Barreras del aviso de versión en el núcleo: lo que Gestor Agentes toma de la biblioteca común
//! aviso-version-github (versión SemVer y repositorio válido) y los ajustes de la consulta. La
//! decisión de avisar, el enlace y la última versión de Programator se prueban en la interfaz,
//! sobre el módulo aviso_de_version y con una consulta falsa. Sin red.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::time::Duration;

use comun::CarpetaTemporal;
use nucleo::actualizaciones::{self, Version};
use nucleo::ajustes::Ajustes;
use nucleo::configuracion::{Configuracion, ErrorDeConfiguracion};

const REPOSITORIO: &str = "celtidcs/gestor-agentes";

#[test]
fn la_version_se_lee_con_o_sin_v_y_se_ordena_como_semver() {
    assert_eq!(
        Version::desde_etiqueta("v3.1.0"),
        Version::desde_etiqueta("3.1.0")
    );
    assert_eq!(
        Version::desde_etiqueta("3.10.0")
            .map(|v| v.to_string())
            .as_deref(),
        Some("3.10.0")
    );
    assert!(Version::desde_etiqueta("3.10.0") > Version::desde_etiqueta("3.9.9"));
    assert!(Version::desde_etiqueta("4.0.0") > Version::desde_etiqueta("3.99.99"));
    for mala in [
        "",
        "3.1",
        "3.1.0.1",
        "3.1.x",
        "3.2.0-rc.1",
        "v",
        "3..1",
        " 3.1.0 x",
    ] {
        assert_eq!(Version::desde_etiqueta(mala), None, "{mala:?}");
    }
}

#[test]
fn solo_vale_un_repositorio_propietario_nombre() {
    assert!(actualizaciones::repositorio_valido(REPOSITORIO));
    assert!(actualizaciones::repositorio_valido("celtidcs/Programator"));
    for malo in [
        "",
        "celtidcs",
        "celtidcs/",
        "/gestor",
        "a/b/c",
        "a b/c",
        "a/../c?x",
        "a/.oculto",
    ] {
        assert!(!actualizaciones::repositorio_valido(malo), "{malo:?}");
    }
}

#[test]
fn los_ajustes_de_la_biblioteca_llevan_el_repositorio_y_el_tiempo_de_gestor() {
    let ajustes = actualizaciones::ajustes_de_version(REPOSITORIO, 7);

    assert_eq!(ajustes.repositorio, REPOSITORIO);
    assert_eq!(ajustes.segundos_de_espera, 7);
    assert_eq!(ajustes.url_api, "https://api.github.com");
    assert_eq!(ajustes.url_web, "https://github.com");
}

#[test]
fn los_ajustes_de_la_consulta_tienen_sus_valores_por_defecto() {
    let ajustes = Ajustes::default();

    assert!(ajustes.comprobar_actualizaciones());
    assert_eq!(ajustes.repositorio_de_actualizaciones(), REPOSITORIO);
    assert_eq!(
        ajustes.tiempo_maximo_actualizacion(),
        Duration::from_secs(5)
    );
}

#[test]
fn la_consulta_se_puede_desactivar_desde_ajustes_json() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("ajustes.json", "{\"ComprobarActualizaciones\": false}");

    let ajustes = Configuracion::new(carpeta.ruta()).cargar_ajustes();

    assert_eq!(
        ajustes.map(|a| a.comprobar_actualizaciones()).ok(),
        Some(false)
    );
}

#[test]
fn un_repositorio_mal_escrito_o_un_tiempo_cero_se_rechazan() {
    for (json, clave) in [
        (
            "{\"RepositorioDeActualizaciones\": \"no-vale\"}",
            "RepositorioDeActualizaciones",
        ),
        (
            "{\"RepositorioDeProgramator\": \"no-vale\"}",
            "RepositorioDeProgramator",
        ),
        (
            "{\"TiempoMaximoActualizacionSegundos\": 0}",
            "TiempoMaximoActualizacionSegundos",
        ),
    ] {
        let carpeta = CarpetaTemporal::new();
        carpeta.escribir("ajustes.json", json);

        let error = Configuracion::new(carpeta.ruta()).cargar_ajustes();

        assert!(
            matches!(&error, Err(ErrorDeConfiguracion::AjusteInvalido { .. })),
            "{error:?}"
        );
        assert!(
            error.err().is_some_and(|e| e.to_string().contains(clave)),
            "{clave}"
        );
    }
}
