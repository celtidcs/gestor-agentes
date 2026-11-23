//! Cadenas TOML para las opciones `-c clave=valor` de Codex.
//!
//! Codex analiza el valor de `-c` como TOML: «`--config` values are parsed as TOML»
//! (https://learn.chatgpt.com/docs/config-file/config-advanced). Un valor que ya es una cadena
//! TOML válida llega intacto; uno mal escrito haría que Codex lo interpretase de otra forma. Las
//! formas de cadena son las de la especificación TOML 1.0 (https://toml.io/es/v1.0.0#cadena).
//!
//! Se prefieren las cadenas literales, sin comillas dobles, porque PowerShell 5.1 no escapa las
//! comillas dobles de un argumento al pasarlo a un programa nativo: una ruta de Windows nunca
//! lleva comillas dobles, así que con las literales la ruta llega bien siempre.

/// Delimitador de la cadena literal de una línea.
const LITERAL: &str = "'";
/// Delimitador de la cadena literal multilínea, que admite comillas simples sueltas o en pareja
/// dentro, y hasta dos más pegadas al delimitador de cierre (TOML 1.0, «Multi-line literal
/// strings»).
const LITERAL_MULTILINEA: &str = "'''";
/// Delimitador de la cadena básica, la única que admite cualquier texto escapándolo.
const BASICA: &str = "\"";

/// Escribe `valor` como cadena TOML que, analizada, devuelve exactamente `valor`.
///
/// Usa la forma más simple que lo representa sin escapes:
/// - literal (`'…'`) si no tiene comillas simples ni caracteres de control;
/// - literal multilínea (`'''…'''`) si tiene comillas simples pero no tres seguidas, ni empieza
///   por un salto de línea, ni tiene caracteres de control;
/// - básica (`"…"`) con escapes en cualquier otro caso.
///
/// No falla: todo texto Unicode tiene una cadena básica que lo representa.
pub(crate) fn cadena_toml(valor: &str) -> String {
    let con_control = valor.chars().any(es_control_no_literal);
    if con_control {
        return basica(valor);
    }
    if !valor.contains(LITERAL) {
        return format!("{LITERAL}{valor}{LITERAL}");
    }
    if !valor.contains(LITERAL_MULTILINEA) {
        return format!("{LITERAL_MULTILINEA}{valor}{LITERAL_MULTILINEA}");
    }
    basica(valor)
}

/// Caracteres que una cadena literal no admite: los de control salvo el tabulador (TOML 1.0,
/// «Literal strings»: «Control characters other than tab are not permitted»).
fn es_control_no_literal(caracter: char) -> bool {
    caracter != '\t' && caracter.is_control()
}

/// Cadena básica: escapa la barra invertida, la comilla doble y los caracteres de control, estos
/// con `\uXXXX` (TOML 1.0, «Basic strings»).
fn basica(valor: &str) -> String {
    let mut salida = String::with_capacity(valor.len() + 2);
    salida.push_str(BASICA);
    for caracter in valor.chars() {
        match caracter {
            '\\' => salida.push_str("\\\\"),
            '"' => salida.push_str("\\\""),
            c if c.is_control() => salida.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => salida.push(c),
        }
    }
    salida.push_str(BASICA);
    salida
}

#[cfg(test)]
#[path = "../tests/unitarias/toml.rs"]
mod pruebas;
