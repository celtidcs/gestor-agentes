//! Huella SHA-256 del contenido de una región. La aplicación guarda la huella de lo último que
//! escribió en cada región para distinguir, en la siguiente actualización, entre un contenido que
//! salió de una versión anterior de la plantilla y uno que alguien editó a mano.
//!
//! SHA-256 se implementa aquí según FIPS 180-4
//! (https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf), sin dependencias, porque solo se
//! usa para reconocer contenido propio, no para nada criptográfico. Las barreras la contrastan con
//! los vectores de prueba del NIST y con las huellas que guardaba la 2.3.2.

/// Constantes de ronda: los 32 primeros bits de la parte fraccionaria de la raíz cúbica de los 64
/// primeros números primos (FIPS 180-4, apartado 4.2.2).
const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// Valor inicial: los 32 primeros bits de la parte fraccionaria de la raíz cuadrada de los ocho
/// primeros números primos (FIPS 180-4, apartado 5.3.3).
const VALOR_INICIAL: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// Tamaño de bloque de SHA-256 en bytes.
const BYTES_POR_BLOQUE: usize = 64;

/// Bytes que ocupa la longitud del mensaje al final del relleno.
const BYTES_DE_LONGITUD: usize = 8;

/// Calcula la huella de un texto normalizado, codificado en UTF-8.
///
/// Devuelve los 32 bytes de SHA-256 en hexadecimal en minúsculas, igual que
/// Convert.ToHexStringLower en la 2.3.2.
#[must_use]
pub fn calcular(contenido: &str) -> String {
    sha256(contenido.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Huella SHA-256 de una secuencia de bytes.
fn sha256(datos: &[u8]) -> [u8; 32] {
    let mut estado = VALOR_INICIAL;
    let mensaje = rellenar(datos);
    let (bloques, _) = mensaje.as_chunks::<BYTES_POR_BLOQUE>();
    for bloque in bloques {
        comprimir(&mut estado, bloque);
    }
    let mut huella = [0u8; 32];
    for (destino, palabra) in huella.as_chunks_mut::<4>().0.iter_mut().zip(estado) {
        *destino = palabra.to_be_bytes();
    }
    huella
}

/// Añade el relleno de FIPS 180-4 (apartado 5.1.1): un bit 1, ceros hasta dejar sitio para la
/// longitud y la longitud del mensaje en bits como entero de 64 bits big-endian.
fn rellenar(datos: &[u8]) -> Vec<u8> {
    let longitud_en_bits = u64::try_from(datos.len())
        .unwrap_or(u64::MAX)
        .wrapping_mul(8);
    let mut mensaje = datos.to_vec();
    mensaje.push(0x80);
    while mensaje.len() % BYTES_POR_BLOQUE != BYTES_POR_BLOQUE - BYTES_DE_LONGITUD {
        mensaje.push(0);
    }
    mensaje.extend_from_slice(&longitud_en_bits.to_be_bytes());
    mensaje
}

/// Procesa un bloque de 64 bytes (FIPS 180-4, apartado 6.2.2).
fn comprimir(estado: &mut [u32; 8], bloque: &[u8; BYTES_POR_BLOQUE]) {
    let mut w = [0u32; 64];
    for (palabra, bytes) in w.iter_mut().zip(bloque.as_chunks::<4>().0) {
        *palabra = u32::from_be_bytes(*bytes);
    }
    for t in 16..64 {
        let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
        let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
        w[t] = w[t - 16]
            .wrapping_add(s0)
            .wrapping_add(w[t - 7])
            .wrapping_add(s1);
    }

    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *estado;
    for (k, palabra) in K.iter().zip(w) {
        let suma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let eleccion = (e & f) ^ (!e & g);
        let temporal1 = h
            .wrapping_add(suma1)
            .wrapping_add(eleccion)
            .wrapping_add(*k)
            .wrapping_add(palabra);
        let suma0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let mayoria = (a & b) ^ (a & c) ^ (b & c);
        let temporal2 = suma0.wrapping_add(mayoria);
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temporal1);
        d = c;
        c = b;
        b = a;
        a = temporal1.wrapping_add(temporal2);
    }

    for (acumulado, nuevo) in estado.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *acumulado = acumulado.wrapping_add(nuevo);
    }
}

/// Huella por región de lo último que escribió la aplicación en un archivo.
///
/// Conserva el orden de inserción, que es el de las regiones en la definición del archivo, para
/// que el registro en disco salga igual que en la 2.3.2. Un identificador aparece una sola vez:
/// insertar uno que ya está sustituye su huella sin moverlo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HuellasPorRegion {
    entradas: Vec<(String, String)>,
}

impl HuellasPorRegion {
    /// Crea un registro vacío.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entradas: Vec::new(),
        }
    }

    /// Guarda la huella de una región, sustituyendo la anterior si ya la tenía.
    pub fn insertar(&mut self, id: impl Into<String>, huella: impl Into<String>) {
        let id = id.into();
        let huella = huella.into();
        match self.entradas.iter_mut().find(|(actual, _)| *actual == id) {
            Some((_, existente)) => *existente = huella,
            None => self.entradas.push((id, huella)),
        }
    }

    /// Huella guardada de una región, si la hay.
    #[must_use]
    pub fn obtener(&self, id: &str) -> Option<&str> {
        self.entradas
            .iter()
            .find(|(actual, _)| actual == id)
            .map(|(_, huella)| huella.as_str())
    }

    /// Recorre las huellas en orden de inserción.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entradas
            .iter()
            .map(|(id, huella)| (id.as_str(), huella.as_str()))
    }

    /// Número de regiones con huella.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entradas.len()
    }

    /// Indica si no hay ninguna huella.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entradas.is_empty()
    }
}

impl<I: Into<String>, H: Into<String>> FromIterator<(I, H)> for HuellasPorRegion {
    fn from_iter<T: IntoIterator<Item = (I, H)>>(iter: T) -> Self {
        let mut huellas = Self::new();
        for (id, huella) in iter {
            huellas.insertar(id, huella);
        }
        huellas
    }
}
