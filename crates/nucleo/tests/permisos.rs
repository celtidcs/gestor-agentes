//! Barreras de los permisos del arranque: la preparación deja en el archivo de permisos de Claude
//! Code las reglas del arranque, solo esas, sin quitar nada de lo que el usuario tenga, y una
//! segunda preparación no cambia nada.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use comun::{CarpetaTemporal, Entorno};
use nucleo::proyecto::informe::TipoDeCambio;
use nucleo::proyecto::permisos::{self, Fusion, ARCHIVO_DE_CLAUDE, REGLAS_DE_CLAUDE};
use serde_json::Value;

fn permitidas(texto: &str) -> Vec<String> {
    let valor: Value = serde_json::from_str(texto).unwrap();
    valor["permissions"]["allow"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

fn escrito(fusion: Fusion) -> String {
    match fusion {
        Fusion::Escribir(texto) => texto,
        otra => panic!("se esperaba texto: {otra:?}"),
    }
}

#[test]
fn sin_archivo_se_crea_con_las_reglas_del_arranque_y_nada_mas() {
    let texto = escrito(permisos::fusionar(None, &REGLAS_DE_CLAUDE));
    assert_eq!(
        permitidas(&texto),
        REGLAS_DE_CLAUDE.map(String::from).to_vec()
    );
    let valor: Value = serde_json::from_str(&texto).unwrap();
    assert_eq!(valor.as_object().unwrap().len(), 1);
    assert_eq!(valor["permissions"].as_object().unwrap().len(), 1);
}

#[test]
fn las_reglas_del_arranque_no_dan_permiso_amplio() {
    for regla in REGLAS_DE_CLAUDE {
        let (herramienta, resto) = regla.split_once('(').unwrap();
        assert!(resto.ends_with(')'), "{regla}");
        let patron = &resto[..resto.len() - 1];
        assert!(!patron.trim().is_empty(), "{regla}");
        assert!(!patron.trim_start().starts_with('*'), "{regla}");
        if herramienta == "Edit" {
            assert_eq!(patron, "/.gestor/**", "{regla}");
        } else {
            assert!(matches!(herramienta, "Bash" | "PowerShell"), "{regla}");
            for prohibida in [
                "push",
                "rm ",
                "curl",
                "wget",
                "Remove-Item",
                "Invoke-WebRequest",
            ] {
                assert!(!patron.contains(prohibida), "{regla}");
            }
        }
    }
}

#[test]
fn conserva_lo_del_usuario_y_su_orden_y_solo_anade_lo_que_falta() {
    let actual = r#"{
  "model": "opus",
  "permissions": {
    "deny": ["Bash(git push *)"],
    "allow": ["Bash(cargo test *)", "Bash(date *)"]
  },
  "env": {"A": "1"}
}"#;
    let texto = escrito(permisos::fusionar(Some(actual), &REGLAS_DE_CLAUDE));
    let valor: Value = serde_json::from_str(&texto).unwrap();
    let claves: Vec<&String> = valor.as_object().unwrap().keys().collect();
    assert_eq!(claves, ["model", "permissions", "env"]);
    assert_eq!(valor["env"]["A"], "1");
    assert_eq!(valor["permissions"]["deny"][0], "Bash(git push *)");
    let lista = permitidas(&texto);
    assert_eq!(lista[0], "Bash(cargo test *)");
    assert_eq!(lista[1], "Bash(date *)");
    assert_eq!(lista.iter().filter(|r| *r == "Bash(date *)").count(), 1);
    for regla in REGLAS_DE_CLAUDE {
        assert!(lista.iter().any(|r| r == regla), "{regla}");
    }
}

#[test]
fn con_todas_las_reglas_no_cambia_nada() {
    let texto = escrito(permisos::fusionar(None, &REGLAS_DE_CLAUDE));
    assert_eq!(
        permisos::fusionar(Some(&texto), &REGLAS_DE_CLAUDE),
        Fusion::SinCambios
    );
}

#[test]
fn un_archivo_que_no_se_entiende_no_se_toca() {
    for actual in [
        "{roto",
        "[1, 2]",
        r#"{"permissions": 3}"#,
        r#"{"permissions": {"allow": "x"}}"#,
    ] {
        assert!(
            matches!(
                permisos::fusionar(Some(actual), &REGLAS_DE_CLAUDE),
                Fusion::Ilegible(_)
            ),
            "{actual}"
        );
    }
}

#[test]
fn la_preparacion_escribe_los_permisos_una_vez_y_respeta_un_archivo_roto() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();

    let informe = entorno.inicializar(carpeta.ruta(), None, None);
    assert!(informe
        .cambios()
        .iter()
        .any(|c| c.archivo == ARCHIVO_DE_CLAUDE && c.tipo == TipoDeCambio::Creado));
    let primero = carpeta.leer(ARCHIVO_DE_CLAUDE);
    assert_eq!(
        permitidas(&primero),
        REGLAS_DE_CLAUDE.map(String::from).to_vec()
    );

    let informe = entorno.inicializar(carpeta.ruta(), None, None);
    assert!(!informe
        .cambios()
        .iter()
        .any(|c| c.archivo == ARCHIVO_DE_CLAUDE));
    assert_eq!(carpeta.leer(ARCHIVO_DE_CLAUDE), primero);

    carpeta.escribir(ARCHIVO_DE_CLAUDE, "{roto");
    let informe = entorno.inicializar(carpeta.ruta(), None, None);
    assert_eq!(carpeta.leer(ARCHIVO_DE_CLAUDE), "{roto");
    assert!(informe
        .avisos()
        .iter()
        .any(|a| a.starts_with(ARCHIVO_DE_CLAUDE)));
}
