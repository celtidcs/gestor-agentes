//! Barreras de las mutaciones vivas (criterio V2 de la 3.6.0), de las identidades y de las
//! huellas.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use canal::huella;
use canal::identidad::{Identidad, IdentidadInvalida};
use canal::mutaciones::{es_vacio, hay_vivas, huella_de_las_vivas};

const BASE: &str = "# Estado\n\n## Reservas activas\n\nNinguna.\n\n## Mutaciones vivas\n\n";

#[test]
fn ninguna_es_vacio() {
    assert!(!hay_vivas(&format!(
        "{BASE}Ninguna.\n\n## Prohibiciones\n\nalgo\n"
    )));
}

#[test]
fn una_nota_entre_parentesis_antes_de_ninguna_sigue_siendo_vacio() {
    let estado = format!("{BASE}- (Añadido por Claude 1, 2026-11-18 13:18.) Ninguna.\n");
    assert!(!hay_vivas(&estado));
    assert!(es_vacio("- Ninguna"));
    assert!(es_vacio("Ninguna."));
}

#[test]
fn un_titulo_con_espacio_delante_cierra_el_apartado() {
    let estado = format!("{BASE}Ninguna.\n ## Raíz reorganizada\n\nTexto que no es una mutación\n");
    assert!(!hay_vivas(&estado));
}

#[test]
fn una_mutacion_real_es_viva() {
    let estado = format!("{BASE}- Codex 1, 2026-11-18 19:00: M1 en src/a.rs\n");
    assert!(hay_vivas(&estado));
}

#[test]
fn sin_apartado_no_hay_vivas() {
    assert!(!hay_vivas("# Estado\n\n## Reservas activas\n\nNinguna.\n"));
}

#[test]
fn la_huella_solo_cambia_con_las_mutaciones() {
    let viva = "- Codex 1, 2026-11-18 19:00: M1 en src/a.rs\n";
    let a = format!("{BASE}{viva}\n## Otro\n\nuno\n");
    let b = format!("{BASE}{viva}\n## Otro\n\ndos\n");
    assert_eq!(huella_de_las_vivas(&a), huella_de_las_vivas(&b));
    let c = format!("{BASE}{viva}- Codex 1, 2026-11-18 19:05: M2 en src/b.rs\n");
    assert_ne!(huella_de_las_vivas(&a), huella_de_las_vivas(&c));
    assert_eq!(huella_de_las_vivas(&format!("{BASE}Ninguna.\n")), None);
}

#[test]
fn identidad_con_y_sin_numero() {
    let codex = Identidad::leer("Codex").unwrap();
    assert_eq!(codex.instancia(), 1);
    assert_eq!(codex.to_string(), "Codex 1");
    assert_eq!(codex.nombre_de_archivos(), "codex");
    let claude = Identidad::leer(" Claude 2 ").unwrap();
    assert_eq!(claude.nombre_de_archivos(), "claude2");
    assert_eq!(claude.to_string(), "Claude 2");
}

#[test]
fn identidades_invalidas() {
    assert_eq!(Identidad::leer("  "), Err(IdentidadInvalida::Vacia));
    assert!(Identidad::leer("Claude 0").is_err());
    assert!(Identidad::leer("Claude dos").is_err());
    assert!(Identidad::leer("../x 1").is_err());
    assert!(Identidad::leer("Claude 2 3").is_err());
    assert!(Identidad::leer("2Claude").is_err());
}

#[test]
fn huella_fnv_conocida() {
    // Valores de referencia de FNV-1a de 64 bits: "" y "a"
    // (http://www.isthe.com/chongo/tech/comp/fnv/).
    assert_eq!(huella::de_bytes(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(huella::de_bytes(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(huella::en_hexadecimal(1), "0000000000000001");
}
