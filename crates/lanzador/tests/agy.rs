//! Barreras de la búsqueda del proyecto de agy por carpeta, con JSON del formato observado en
//! ~/.gemini/config/projects/.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::path::Path;

use lanzador::agy::{buscar_proyecto, proyecto_de_la_carpeta, ProyectoDeAgy};

fn carpeta_suelta(id: &str, direccion: &str) -> String {
    format!(
        r#"{{"id": "{id}", "name": "x", "projectResources": {{"resources": [{{"folderUri": "{direccion}"}}]}}}}"#
    )
}

fn carpeta_git(id: &str, direccion: &str) -> String {
    format!(
        r#"{{"id": "{id}", "projectResources": {{"resources": [{{"gitFolder": {{"folderUri": "{direccion}"}}}}]}}}}"#
    )
}

#[test]
fn encuentra_la_carpeta_codificada_con_la_letra_en_minuscula() {
    let proyectos = [
        carpeta_suelta("otro", "file:///d%3A/Mis%20proyectos/Uno"),
        carpeta_suelta("buscado", "file:///d%3A/Mis%20proyectos/Dos"),
    ];
    assert_eq!(
        proyecto_de_la_carpeta(&proyectos, Path::new(r"D:\Mis proyectos\Dos")),
        Some("buscado".to_owned())
    );
}

#[test]
fn encuentra_la_carpeta_de_git_y_la_escrita_sin_codificar() {
    let proyectos = [
        carpeta_git("git", "file:///d%3A/Mis%20proyectos/Tres"),
        carpeta_suelta("crudo", "file://D:/Mis proyectos/Cuatro"),
    ];
    assert_eq!(
        proyecto_de_la_carpeta(&proyectos, Path::new(r"D:\Mis proyectos\Tres\")),
        Some("git".to_owned())
    );
    assert_eq!(
        proyecto_de_la_carpeta(&proyectos, Path::new("D:/Mis proyectos/Cuatro")),
        Some("crudo".to_owned())
    );
}

#[test]
fn una_subcarpeta_o_una_carpeta_parecida_no_valen() {
    let proyectos = [carpeta_suelta(
        "padre",
        "file:///d%3A/Mis%20proyectos/Cuatro",
    )];
    for carpeta in [
        r"D:\Mis proyectos\Cuatro\.gestor",
        r"D:\Mis proyectos\Cuatro2",
        r"E:\Mis proyectos\Cuatro",
    ] {
        assert_eq!(
            proyecto_de_la_carpeta(&proyectos, Path::new(carpeta)),
            None,
            "{carpeta}"
        );
    }
}

#[test]
fn en_linux_se_distinguen_mayusculas() {
    let proyectos = [carpeta_suelta("linux", "file:///home/u/Demo")];
    assert_eq!(
        proyecto_de_la_carpeta(&proyectos, Path::new("/home/u/Demo")),
        Some("linux".to_owned())
    );
    assert_eq!(
        proyecto_de_la_carpeta(&proyectos, Path::new("/home/u/demo")),
        None
    );
}

#[test]
fn los_json_ilegibles_o_sin_id_se_ignoran() {
    let proyectos = [
        "no es json".to_owned(),
        r#"{"projectResources": {"resources": [{"folderUri": "file:///home/u/Demo"}]}}"#.to_owned(),
        carpeta_suelta("bueno", "file:///home/u/Demo"),
    ];
    assert_eq!(
        proyecto_de_la_carpeta(&proyectos, Path::new("/home/u/Demo")),
        Some("bueno".to_owned())
    );
    assert_eq!(proyecto_de_la_carpeta(&[], Path::new("/home/u/Demo")), None);
}

#[test]
fn sin_proyecto_para_la_carpeta_falta_y_sin_ninguno_legible_es_ilegible() {
    let proyectos = [
        carpeta_suelta("uno", "file:///d%3A/Mis%20proyectos/Uno"),
        String::from("esto no es json"),
    ];
    assert_eq!(
        buscar_proyecto(&proyectos, Path::new(r"D:\Mis proyectos\Uno")),
        ProyectoDeAgy::Encontrado("uno".to_owned())
    );
    assert_eq!(
        buscar_proyecto(&proyectos, Path::new(r"D:\Mis proyectos\Otro")),
        ProyectoDeAgy::Falta
    );
    // Ningún proyecto, o ninguno con la forma conocida: no se sabe si la carpeta tiene el suyo.
    for ilegibles in [
        vec![],
        vec![String::from("esto no es json")],
        vec![String::from(r#"{"id": "x"}"#)],
        vec![String::from(
            r#"{"id": 7, "projectResources": {"resources": []}}"#,
        )],
        vec![String::from(r#"{"projectResources": {"resources": []}}"#)],
        vec![String::from(
            r#"{"id": "x", "projectResources": {"resources": {}}}"#,
        )],
    ] {
        assert_eq!(
            buscar_proyecto(&ilegibles, Path::new(r"D:\Mis proyectos\Uno")),
            ProyectoDeAgy::Ilegible,
            "{ilegibles:?}"
        );
    }
}
