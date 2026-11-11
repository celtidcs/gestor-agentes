//! Escapes de texto para los guiones de terminal.
//!
//! Los guiones que reciben PowerShell y bash son el único sitio en que los datos (rutas, títulos y
//! valores de variables) se meten dentro de un texto que el intérprete analiza. Se meten siempre
//! entre comillas simples, que en los dos intérpretes no expanden variables ni sustituciones de
//! órdenes, y solo hay que neutralizar los caracteres que el intérprete toma por comilla simple.

/// Caracteres que PowerShell trata como comilla simple: la recta y las cuatro tipográficas
/// U+2018, U+2019, U+201A y U+201B. Dentro de una cadena entre comillas simples, cualquiera de
/// ellos se escribe literalmente duplicándolo.
///
/// La 2.3.2 solo duplicaba la recta; las otras cuatro permitían cerrar la cadena e inyectar
/// órdenes (reproducido con powershell -EncodedCommand y Set-Location -LiteralPath, como lo
/// hace la aplicación). Es una diferencia declarada frente a la paridad.
const COMILLAS_SIMPLES_POWERSHELL: [char; 5] =
    ['\'', '\u{2018}', '\u{2019}', '\u{201A}', '\u{201B}'];

/// Escapa un texto para ponerlo entre comillas simples en PowerShell, duplicando cada carácter
/// que PowerShell toma por comilla simple.
pub(crate) fn comilla_simple_powershell(valor: &str) -> String {
    let mut salida = String::with_capacity(valor.len());
    for caracter in valor.chars() {
        if COMILLAS_SIMPLES_POWERSHELL.contains(&caracter) {
            salida.push(caracter);
        }
        salida.push(caracter);
    }
    salida
}

/// Escapa un texto para ponerlo entre comillas simples en bash. Cada comilla se sustituye por
/// `'\''`, que cierra la cadena, escribe una comilla escapada y la vuelve a abrir. Igual que
/// EscapeBashSingleQuote de la 2.3.2.
pub(crate) fn comilla_simple_bash(valor: &str) -> String {
    valor.replace('\'', "'\\''")
}
