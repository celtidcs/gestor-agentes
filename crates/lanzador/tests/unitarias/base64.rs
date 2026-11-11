//! Pruebas del codificador de -EncodedCommand. Viven aquí, fuera de src/, y se enlazan desde
//! src/base64.rs con #[path] porque prueban una función privada.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{codificar, orden_codificada_powershell};

/// Vectores de prueba del RFC 4648, apartado 10.
#[test]
fn codifica_los_vectores_del_rfc_4648() {
    let casos = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];
    for (entrada, esperado) in casos {
        assert_eq!(
            codificar(entrada.as_bytes()),
            esperado,
            "entrada {entrada:?}"
        );
    }
}

/// «dir» en UTF-16LE (64 00 69 00 72 00) es ZABpAHIA en Base64, el ejemplo de la documentación de
/// powershell.exe para -EncodedCommand.
#[test]
fn codifica_en_utf16le() {
    assert_eq!(orden_codificada_powershell("dir"), "ZABpAHIA");
}

/// Los caracteres fuera del plano básico van como pares sustitutos de UTF-16.
#[test]
fn codifica_caracteres_fuera_del_plano_basico() {
    // U+1F600 = D83D DE00 → 3D D8 00 DE.
    assert_eq!(orden_codificada_powershell("\u{1F600}"), "PdgA3g==");
}
