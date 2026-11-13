//! Barreras del aviso de versión: versión SemVer, lectura de la respuesta de GitHub,
//! validación del enlace, decisión de avisar y ajustes de la consulta. Sin red.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::time::Duration;

use comun::CarpetaTemporal;
use nucleo::actualizaciones::{self, Publicacion, Version};
use nucleo::ajustes::Ajustes;
use nucleo::configuracion::{Configuracion, ErrorDeConfiguracion};

const REPOSITORIO: &str = "celtidcs/gestor-agentes";

/// Respuesta como la de la API de GitHub, reducida a lo que se lee y con un campo de más.
fn respuesta(etiqueta: &str, enlace: &str) -> String {
    format!(
        "{{\"tag_name\": \"{etiqueta}\", \"html_url\": \"{enlace}\", \"name\": \"Gestor Agentes\", \"draft\": false}}"
    )
}

fn enlace(etiqueta: &str) -> String {
    format!("https://github.com/{REPOSITORIO}/releases/tag/{etiqueta}")
}

fn publicacion(etiqueta: &str) -> Option<Publicacion> {
    actualizaciones::leer_publicacion(&respuesta(etiqueta, &enlace(etiqueta)), REPOSITORIO)
}

#[test]
fn la_version_se_lee_con_o_sin_v_y_se_ordena_como_semver() {
    assert_eq!(Version::leer("v3.1.0"), Version::leer("3.1.0"));
    assert_eq!(
        Version::leer("3.10.0").map(|v| v.to_string()).as_deref(),
        Some("3.10.0")
    );
    assert!(Version::leer("3.10.0") > Version::leer("3.9.9"));
    assert!(Version::leer("4.0.0") > Version::leer("3.99.99"));
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
        assert_eq!(Version::leer(mala), None, "{mala:?}");
    }
}

#[test]
fn se_avisa_solo_de_una_version_mas_nueva() {
    let nueva = actualizaciones::novedad("3.1.0", publicacion("v3.2.0"));
    assert_eq!(
        nueva.map(|p| (p.version.to_string(), p.enlace)),
        Some(("3.2.0".to_owned(), enlace("v3.2.0")))
    );
    for igual_o_vieja in ["v3.1.0", "v3.0.0", "v2.9.9"] {
        assert_eq!(
            actualizaciones::novedad("3.1.0", publicacion(igual_o_vieja)),
            None,
            "{igual_o_vieja}"
        );
    }
    assert_eq!(
        actualizaciones::novedad("no es versión", publicacion("v9.0.0")),
        None
    );
    assert_eq!(actualizaciones::novedad("3.1.0", None), None);
}

#[test]
fn una_respuesta_rara_no_da_publicacion() {
    for json in [
        String::new(),
        "no es JSON".to_owned(),
        "{\"message\": \"Not Found\"}".to_owned(),
        respuesta("v3.2.0-beta", &enlace("v3.2.0-beta")),
    ] {
        assert_eq!(
            actualizaciones::leer_publicacion(&json, REPOSITORIO),
            None,
            "{json}"
        );
    }
}

#[test]
fn solo_se_acepta_un_enlace_del_propio_repositorio_en_github() {
    for ajeno in [
        "https://ejemplo.com/celtidcs/gestor-agentes/releases/tag/v9.0.0",
        "http://github.com/celtidcs/gestor-agentes/releases/tag/v9.0.0",
        "https://github.com/otro/gestor-agentes/releases/tag/v9.0.0",
        "https://github.com/celtidcs/gestor-agentes-falso/releases/tag/v9.0.0",
    ] {
        assert_eq!(
            actualizaciones::leer_publicacion(&respuesta("v9.0.0", ajeno), REPOSITORIO),
            None,
            "{ajeno}"
        );
    }
}

#[test]
fn la_consulta_va_a_la_api_de_releases_con_un_agente_de_usuario_generico() {
    assert_eq!(
        actualizaciones::direccion_de_la_consulta(REPOSITORIO).as_deref(),
        Some("https://api.github.com/repos/celtidcs/gestor-agentes/releases/latest")
    );
    assert_eq!(
        actualizaciones::agente_de_usuario("3.1.0"),
        "gestor-agentes/3.1.0"
    );
    for malo in [
        "",
        "celtidcs",
        "celtidcs/",
        "/gestor",
        "a/b/c",
        "a b/c",
        "a/../c?x",
    ] {
        assert!(!actualizaciones::repositorio_valido(malo), "{malo:?}");
        assert_eq!(actualizaciones::direccion_de_la_consulta(malo), None);
    }
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
