//! Huella estable de un texto, para reconocer lo ya visto entre dos ejecuciones y entre versiones.
//!
//! Es FNV-1a de 64 bits, escrita aquí para que no dependa de la versión de Rust (el hash de la
//! biblioteca estándar no promete ser estable) ni de ninguna dependencia. No es criptográfica: solo
//! distingue textos del propio proyecto, que nadie fabrica para colisionar.

/// Base de desplazamiento de FNV-1a de 64 bits (http://www.isthe.com/chongo/tech/comp/fnv/).
const BASE_FNV: u64 = 0xcbf2_9ce4_8422_2325;
/// Primo de FNV de 64 bits, de la misma fuente.
const PRIMO_FNV: u64 = 0x0000_0100_0000_01b3;

/// Huella de unos bytes.
#[must_use]
pub fn de_bytes(bytes: &[u8]) -> u64 {
    bytes.iter().fold(BASE_FNV, |acumulada, byte| {
        (acumulada ^ u64::from(*byte)).wrapping_mul(PRIMO_FNV)
    })
}

/// Huella de un texto sin tener en cuenta los finales de línea (CRLF o LF) ni los espacios al
/// final de cada línea, que cambian al guardar con otro editor sin cambiar lo dicho.
#[must_use]
pub fn de_texto(texto: &str) -> u64 {
    let normalizado: Vec<&str> = texto.lines().map(str::trim_end).collect();
    de_bytes(normalizado.join("\n").as_bytes())
}

/// Huella escrita como 16 cifras hexadecimales, la forma en que se guarda en disco.
#[must_use]
pub fn en_hexadecimal(huella: u64) -> String {
    format!("{huella:016x}")
}
