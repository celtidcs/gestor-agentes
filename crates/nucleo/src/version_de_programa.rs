//! Versión de un programa instalado y si hay otra más nueva publicada (F7, 3.7.0).
//!
//! Programator se instala como carpeta portable: una versión nueva suele ir a otra carpeta, y la
//! ruta del perfil sigue apuntando a la antigua. En Windows, su ejecutable lleva la versión en el
//! recurso de versión (VS_VERSIONINFO), en la cadena ProductVersion codificada en UTF-16LE
//! (https://learn.microsoft.com/windows/win32/menurc/stringfileinfo-block); comprobado en
//! programator.exe 0.12.0. Los ejecutables de Linux no llevan ese recurso. Lógica pura: los bytes
//! llegan ya leídos.

use crate::actualizaciones::{Publicacion, Version};

/// Nombre de la cadena del recurso con la versión del producto.
const CLAVE_DE_VERSION: &str = "ProductVersion";

/// Caracteres que se leen como mucho del valor, para no recorrer el archivo entero si el recurso
/// está dañado: «65535.65535.65535.65535» tiene 23.
const LARGO_MAXIMO_DEL_VALOR: usize = 32;

/// Versión que declara el recurso de versión del ejecutable, o nada si no lo lleva o no es una
/// versión de tres números. Acepta también la forma de cuatro números de Windows si el cuarto es
/// 0 («0.12.0.0»).
#[must_use]
pub fn version_en_recurso(bytes: &[u8]) -> Option<Version> {
    let clave = utf16(CLAVE_DE_VERSION);
    let inicio = bytes
        .windows(clave.len())
        .position(|ventana| ventana == clave)?
        + clave.len();
    let unidades: Vec<u16> = bytes[inicio..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|par| u16::from_le_bytes(*par))
        .skip_while(|&unidad| unidad == 0)
        .take_while(|&unidad| unidad != 0)
        .take(LARGO_MAXIMO_DEL_VALOR)
        .collect();
    let texto = String::from_utf16(&unidades).ok()?;
    let texto = texto.trim();
    Version::leer(texto).or_else(|| Version::leer(texto.strip_suffix(".0")?))
}

/// La versión publicada si es más nueva que la instalada; nada si no se sabe alguna de las dos.
#[must_use]
pub fn mas_nueva(instalada: Option<Version>, publicada: Option<&Publicacion>) -> Option<Version> {
    let instalada = instalada?;
    publicada
        .map(|publicacion| publicacion.version)
        .filter(|version| *version > instalada)
}

/// Bytes UTF-16LE de un texto ASCII.
fn utf16(texto: &str) -> Vec<u8> {
    texto.encode_utf16().flat_map(u16::to_le_bytes).collect()
}
