//! Codificación de la orden de PowerShell para el parámetro -EncodedCommand.
//!
//! PowerShell espera en -EncodedCommand el texto de la orden en UTF-16LE y codificado en Base64
//! estándar (RFC 4648, apartado 4, con relleno). Así ninguna comilla de una ruta puede cerrar el
//! argumento y añadir opciones a powershell.exe. Se implementa aquí, en unas líneas, para no añadir
//! una dependencia por una sola función.

/// Alfabeto Base64 estándar del RFC 4648, tabla 1.
const ALFABETO: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Codifica un texto como lo espera -EncodedCommand: UTF-16LE y Base64 con relleno.
pub(crate) fn orden_codificada_powershell(orden: &str) -> String {
    let bytes: Vec<u8> = orden.encode_utf16().flat_map(u16::to_le_bytes).collect();
    codificar(&bytes)
}

/// Codifica bytes en Base64 estándar con relleno «=».
fn codificar(bytes: &[u8]) -> String {
    let mut salida = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for grupo in bytes.chunks(3) {
        let b0 = grupo.first().copied().unwrap_or(0);
        let b1 = grupo.get(1).copied().unwrap_or(0);
        let b2 = grupo.get(2).copied().unwrap_or(0);
        let n = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);
        // Cada grupo de 3 bytes da 4 símbolos de 6 bits; los que no tienen datos son relleno.
        let simbolos = grupo.len() + 1;
        for posicion in 0..4 {
            if posicion < simbolos {
                let indice = (n >> (18 - 6 * posicion)) & 0x3F;
                salida.push(char::from(ALFABETO[indice as usize]));
            } else {
                salida.push('=');
            }
        }
    }
    salida
}

#[cfg(test)]
#[path = "../tests/unitarias/base64.rs"]
mod pruebas;
