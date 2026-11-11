//! Formato físico de un archivo de texto: si empieza con marca de orden de bytes UTF-8 y qué
//! salto de línea usa. Se conserva al reescribir un archivo para que el diff de git muestre solo
//! los cambios reales de contenido.
//!
//! Si el BOM desapareciera, git mostraría el archivo entero como reescrito. Conservar el formato
//! evita ese ruido.

/// Bytes de la marca de orden de bytes de UTF-8.
const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// Marca de orden de bytes ya decodificada como carácter.
const BOM_DECODIFICADO: char = '\u{FEFF}';

/// Formato de un archivo. El valor por defecto, UTF-8 sin BOM con saltos \n, es el de los
/// archivos nuevos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormatoDeTexto {
    /// El archivo empieza con los bytes EF BB BF.
    pub tiene_bom: bool,
    /// El archivo usa \r\n como salto de línea dominante.
    pub usa_crlf: bool,
}

impl FormatoDeTexto {
    /// Decodifica los bytes de un archivo, detecta su formato y normaliza los saltos a \n.
    ///
    /// Las secuencias UTF-8 inválidas se sustituyen por U+FFFD, como hace el decodificador de
    /// .NET que usaba la 2.3.2. El salto dominante es \r\n cuando hay alguno y al menos la mitad
    /// de los \n van precedidos de \r.
    #[must_use]
    pub fn leer(bytes: &[u8]) -> (String, Self) {
        let (tiene_bom, cuerpo) = match bytes.strip_prefix(&BOM[..]) {
            Some(resto) => (true, resto),
            None => (false, bytes),
        };
        let texto = String::from_utf8_lossy(cuerpo);
        let saltos_crlf = texto.matches("\r\n").count();
        let saltos_totales = texto.matches('\n').count();
        let usa_crlf = saltos_crlf > 0 && saltos_crlf * 2 >= saltos_totales;
        (
            normalizar(&texto),
            Self {
                tiene_bom,
                usa_crlf,
            },
        )
    }

    /// Convierte un texto normalizado con \n en los bytes que corresponden a este formato.
    #[must_use]
    pub fn codificar(&self, texto_normalizado: &str) -> Vec<u8> {
        let texto = if self.usa_crlf {
            texto_normalizado.replace('\n', "\r\n")
        } else {
            texto_normalizado.to_owned()
        };
        let mut bytes = Vec::with_capacity(texto.len() + BOM.len());
        if self.tiene_bom {
            bytes.extend_from_slice(&BOM);
        }
        bytes.extend_from_slice(texto.as_bytes());
        bytes
    }
}

/// Normaliza los saltos de línea a \n y elimina los BOM decodificados del principio.
#[must_use]
pub fn normalizar(texto: &str) -> String {
    texto
        .trim_start_matches(BOM_DECODIFICADO)
        .replace("\r\n", "\n")
}

/// Normaliza un texto para compararlo con otro ignorando el formato y los espacios y saltos de
/// los extremos. Es el criterio con el que se reconoce una plantilla antigua.
#[must_use]
pub fn normalizar_para_comparar(texto: &str) -> String {
    normalizar(texto).trim().to_owned()
}
