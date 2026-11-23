//! Barreras de F7 (3.7.0): la versión del Programator de la ruta se lee del recurso de versión del
//! ejecutable y se compara con la última publicada.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use nucleo::actualizaciones::{Publicacion, Version};
use nucleo::version_de_programa::{mas_nueva, version_en_recurso};

/// Trozo de un recurso de versión como el de programator.exe 0.12.0: la clave en UTF-16LE, su
/// terminador, relleno y el valor terminado en cero, entre bytes cualesquiera.
fn ejecutable_con(valor: &str) -> Vec<u8> {
    let utf16 =
        |texto: &str| -> Vec<u8> { texto.encode_utf16().flat_map(u16::to_le_bytes).collect() };
    let mut bytes = b"MZ\x90\x00 basura del ejecutable ".to_vec();
    bytes.extend(utf16("FileVersion"));
    bytes.extend([0, 0]);
    bytes.extend(utf16("9.9.9"));
    bytes.extend([0, 0]);
    bytes.extend(utf16("ProductVersion"));
    bytes.extend([0, 0, 0, 0]);
    bytes.extend(utf16(valor));
    bytes.extend([0, 0, 7, 7]);
    bytes
}

fn publicada(version: &str) -> Publicacion {
    Publicacion {
        version: Version::leer(version).unwrap(),
        enlace: String::from("https://github.com/celtidcs/Programator/releases/tag/v0.13.0"),
    }
}

#[test]
fn lee_la_version_del_producto_y_no_la_del_archivo() {
    assert_eq!(
        version_en_recurso(&ejecutable_con("0.12.0")),
        Version::leer("0.12.0")
    );
    assert_eq!(
        version_en_recurso(&ejecutable_con("0.12.0.0")),
        Version::leer("0.12.0")
    );
}

#[test]
fn sin_recurso_o_con_un_valor_raro_no_hay_version() {
    assert_eq!(version_en_recurso(b"\x7fELF ejecutable de Linux"), None);
    assert_eq!(version_en_recurso(&ejecutable_con("beta")), None);
}

#[test]
fn solo_hay_version_nueva_si_la_publicada_es_mayor() {
    let instalada = Version::leer("0.12.0");
    assert_eq!(
        mas_nueva(instalada, Some(&publicada("0.13.0"))),
        Version::leer("0.13.0")
    );
    assert_eq!(mas_nueva(instalada, Some(&publicada("0.12.0"))), None);
    assert_eq!(mas_nueva(None, Some(&publicada("0.13.0"))), None);
    assert_eq!(mas_nueva(instalada, None), None);
}
