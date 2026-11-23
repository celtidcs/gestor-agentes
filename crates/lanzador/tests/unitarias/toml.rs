//! Pruebas de las cadenas TOML de `-c` de Codex. Viven aquí, fuera de src/, y se enlazan desde
//! src/toml.rs con #[path] porque prueban una función privada.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::cadena_toml;

/// Valor que devuelve un analizador TOML real al leer `clave = <cadena>`.
fn leido(cadena: &str) -> String {
    let documento: toml_edit::DocumentMut = format!("v = {cadena}\n")
        .parse()
        .unwrap_or_else(|error| panic!("{cadena} no es TOML válido: {error}"));
    documento["v"]
        .as_str()
        .unwrap_or_else(|| panic!("{cadena} no es una cadena"))
        .to_owned()
}

#[test]
fn una_ruta_de_windows_va_como_literal_sin_escapes() {
    let ruta = r"D:\Mis proyectos\Gestor Agentes";
    assert_eq!(cadena_toml(ruta), format!("'{ruta}'"));
    assert_eq!(leido(&cadena_toml(ruta)), ruta);
}

#[test]
fn una_comilla_simple_pasa_a_literal_multilinea() {
    let ruta = r"C:\Users\O'Brien\proyecto";
    assert_eq!(cadena_toml(ruta), format!("'''{ruta}'''"));
    assert_eq!(leido(&cadena_toml(ruta)), ruta);
}

#[test]
fn tres_comillas_o_caracteres_de_control_pasan_a_basica_con_escapes() {
    for valor in ["f'''; & touch x", "a\nb", "c\u{7}d", "e'''\\\"x\""] {
        let cadena = cadena_toml(valor);
        assert!(cadena.starts_with('"'), "{valor:?} → {cadena}");
        assert_eq!(leido(&cadena), valor, "{valor:?} → {cadena}");
    }
}

#[test]
fn cualquier_valor_hostil_se_lee_igual_que_el_original() {
    let valores = [
        "",
        "'",
        "''",
        "'a'",
        "a'",
        "a''",
        "'''",
        "''''",
        "tab\taquí",
        "\"",
        "\\\"x\"",
        "b\\\"; touch t; echo \"",
        "d$env:PATH ${t} \"$t\" '+'",
        "é ñ ü — Proyecto ‘con espacios’",
        "\u{1b}[31m",
        "\r\n",
        "\u{7f}",
    ];
    for valor in valores {
        let cadena = cadena_toml(valor);
        assert_eq!(leido(&cadena), valor, "{valor:?} → {cadena}");
    }
}
