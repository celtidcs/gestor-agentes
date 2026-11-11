//! Registro, por archivo, de la huella de lo último que la aplicación escribió en cada archivo
//! que mantiene. Se guarda en .gestor/regiones-gestionadas.json. Hasta la 3.0.0 la huella era por
//! región; desde entonces los archivos se escriben enteros y cada uno lleva una sola huella, con
//! la clave [`CLAVE_ARCHIVO_COMPLETO`].
//!
//! Si el registro falta o está dañado se empieza vacío. La consecuencia es solo conservadora: un
//! archivo que difiera del texto vigente se tratará como editado a mano y se guardará una copia
//! antes de sustituirlo.
//!
//! El archivo se escribe a mano para que salga byte a byte como lo escribía System.Text.Json en la
//! 2.3.2: sangría de dos espacios, el orden de inserción, el escape por defecto de .NET y el salto
//! de línea del sistema dentro del objeto (Environment.NewLine, que en .NET 9 es el NewLine por
//! defecto de JsonWriterOptions), con un \n al final. Se lee con serde_json, en orden.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::Deserialize;

use super::archivos;
use super::informe::InformeDeInicializacion;
use crate::instrucciones::formato::FormatoDeTexto;
use crate::instrucciones::huella::HuellasPorRegion;
use crate::textos;

/// Ruta del registro, relativa a la raíz del proyecto.
pub const RUTA_RELATIVA: &str = ".gestor/regiones-gestionadas.json";

/// Clave con la que se registra la huella de un archivo escrito entero.
pub const CLAVE_ARCHIVO_COMPLETO: &str = "ARCHIVO";

/// Salto de línea dentro del JSON, el de Environment.NewLine en la 2.3.2.
const SALTO_JSON: &str = if cfg!(windows) { "\r\n" } else { "\n" };

/// Sangría de cada nivel del JSON.
const SANGRIA: &str = "  ";

/// Huellas registradas de todos los archivos de un proyecto, en orden de inserción.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistroDeHuellas {
    archivos: Vec<(String, HuellasPorRegion)>,
}

impl RegistroDeHuellas {
    /// Carga el registro de un proyecto. Si no existe empieza vacío; si no se puede leer o está
    /// dañado, también, y lo anota como aviso en el informe.
    #[must_use]
    pub fn cargar(raiz: &Path, informe: &mut InformeDeInicializacion) -> Self {
        let ruta = archivos::ruta(raiz, RUTA_RELATIVA);
        let texto = match fs::read_to_string(&ruta) {
            Ok(texto) => texto,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Self::default(),
            Err(error) => return Self::vacio_con_aviso(informe, &error.to_string()),
        };
        match serde_json::from_str::<Option<RegistroDeHuellas>>(
            texto.trim_start_matches('\u{FEFF}'),
        ) {
            Ok(registro) => registro.unwrap_or_default(),
            Err(error) => Self::vacio_con_aviso(informe, &error.to_string()),
        }
    }

    /// Huellas registradas para un archivo; vacío si no hay registro.
    #[must_use]
    pub fn huellas_de(&self, archivo: &str) -> HuellasPorRegion {
        self.archivos
            .iter()
            .find(|(nombre, _)| nombre == archivo)
            .map(|(_, huellas)| huellas.clone())
            .unwrap_or_default()
    }

    /// Sustituye las huellas registradas de un archivo, sin moverlo de su sitio si ya estaba.
    pub fn registrar(&mut self, archivo: &str, huellas: HuellasPorRegion) {
        match self
            .archivos
            .iter_mut()
            .find(|(nombre, _)| nombre == archivo)
        {
            Some((_, existentes)) => *existentes = huellas,
            None => self.archivos.push((archivo.to_owned(), huellas)),
        }
    }

    /// Huella de lo último que la aplicación escribió en un archivo escrito entero, si consta.
    #[must_use]
    pub fn huella_del_archivo(&self, archivo: &str) -> Option<String> {
        self.huellas_de(archivo)
            .obtener(CLAVE_ARCHIVO_COMPLETO)
            .map(str::to_owned)
    }

    /// Registra la huella de un archivo escrito entero, sustituyendo lo que hubiera de él.
    pub fn registrar_archivo(&mut self, archivo: &str, huella: impl Into<String>) {
        let mut huellas = HuellasPorRegion::new();
        huellas.insertar(CLAVE_ARCHIVO_COMPLETO, huella);
        self.registrar(archivo, huellas);
    }

    /// Quita del registro un archivo que la aplicación ya no mantiene.
    pub fn olvidar(&mut self, archivo: &str) {
        self.archivos.retain(|(nombre, _)| nombre != archivo);
    }

    /// Guarda el registro en el proyecto.
    ///
    /// # Errors
    ///
    /// Devuelve el error de disco si no se puede escribir.
    pub fn guardar(&self, raiz: &Path) -> io::Result<()> {
        archivos::escribir(
            &archivos::ruta(raiz, RUTA_RELATIVA),
            &(self.componer_json() + "\n"),
            FormatoDeTexto::default(),
        )
    }

    /// Texto JSON del registro, sin salto final, como lo escribía System.Text.Json indentado.
    #[must_use]
    pub fn componer_json(&self) -> String {
        if self.archivos.is_empty() {
            return String::from("{}");
        }
        let entradas: Vec<String> = self
            .archivos
            .iter()
            .map(|(archivo, huellas)| {
                format!(
                    "{SANGRIA}{}: {}",
                    cadena_json(archivo),
                    objeto_de_huellas(huellas)
                )
            })
            .collect();
        format!(
            "{{{SALTO_JSON}{}{SALTO_JSON}}}",
            entradas.join(&format!(",{SALTO_JSON}"))
        )
    }

    fn vacio_con_aviso(informe: &mut InformeDeInicializacion, motivo: &str) -> Self {
        informe.anotar_aviso(textos::registro_ilegible(RUTA_RELATIVA, motivo));
        Self::default()
    }
}

/// Objeto anidado de un archivo, con un nivel más de sangría.
fn objeto_de_huellas(huellas: &HuellasPorRegion) -> String {
    if huellas.is_empty() {
        return String::from("{}");
    }
    let entradas: Vec<String> = huellas
        .iter()
        .map(|(id, huella)| {
            format!(
                "{SANGRIA}{SANGRIA}{}: {}",
                cadena_json(id),
                cadena_json(huella)
            )
        })
        .collect();
    format!(
        "{{{SALTO_JSON}{}{SALTO_JSON}{SANGRIA}}}",
        entradas.join(&format!(",{SALTO_JSON}"))
    )
}

/// Cadena JSON con el escape por defecto de System.Text.Json (JavaScriptEncoder.Default): se
/// escapan las comillas, la barra invertida, los controles, todo lo que no es ASCII y los
/// caracteres sensibles en HTML (< > & ' + `), estos con \uXXXX en mayúsculas.
fn cadena_json(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len() + 2);
    salida.push('"');
    for caracter in texto.chars() {
        match caracter {
            '"' => salida.push_str("\\\""),
            '\\' => salida.push_str("\\\\"),
            '\n' => salida.push_str("\\n"),
            '\r' => salida.push_str("\\r"),
            '\t' => salida.push_str("\\t"),
            '\u{8}' => salida.push_str("\\b"),
            '\u{c}' => salida.push_str("\\f"),
            ' '..='~' if !matches!(caracter, '<' | '>' | '&' | '\'' | '+' | '`') => {
                salida.push(caracter);
            }
            _ => {
                let mut unidades = [0u16; 2];
                for unidad in caracter.encode_utf16(&mut unidades) {
                    salida.push_str(&format!("\\u{unidad:04X}"));
                }
            }
        }
    }
    salida.push('"');
    salida
}

impl<'de> Deserialize<'de> for RegistroDeHuellas {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(VisitanteDeRegistro)
    }
}

/// Lee el objeto exterior conservando el orden del archivo.
struct VisitanteDeRegistro;

impl<'de> Visitor<'de> for VisitanteDeRegistro {
    type Value = RegistroDeHuellas;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("un objeto de archivos con sus huellas por región")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut mapa: A) -> Result<Self::Value, A::Error> {
        let mut registro = RegistroDeHuellas::default();
        while let Some((archivo, huellas)) = mapa.next_entry::<String, HuellasLeidas>()? {
            registro.registrar(&archivo, huellas.0);
        }
        Ok(registro)
    }
}

/// Huellas de un archivo leídas en el orden del JSON.
struct HuellasLeidas(HuellasPorRegion);

impl<'de> Deserialize<'de> for HuellasLeidas {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(VisitanteDeHuellas)
    }
}

/// Lee el objeto de un archivo conservando el orden.
struct VisitanteDeHuellas;

impl<'de> Visitor<'de> for VisitanteDeHuellas {
    type Value = HuellasLeidas;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("un objeto de huellas por región")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut mapa: A) -> Result<Self::Value, A::Error> {
        let mut huellas = HuellasPorRegion::new();
        while let Some((id, huella)) = mapa.next_entry::<String, String>()? {
            huellas.insertar(id, huella);
        }
        Ok(HuellasLeidas(huellas))
    }
}
