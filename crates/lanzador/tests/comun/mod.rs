//! Utilidades compartidas por las pruebas del lanzador: un doble de EntornoDelSistema y la
//! decodificación de -EncodedCommand para leer el guion de PowerShell.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default,
    dead_code,
    missing_docs
)]

use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use lanzador::{EntornoDelSistema, Herramienta, PeticionDeLanzamiento, PlanDeLanzamiento, Sistema};

/// Doble del entorno: un sistema de archivos imaginario (la lista de archivos que existen) y
/// las carpetas del PATH, con la misma regla de búsqueda que el entorno real.
#[derive(Debug, Default, Clone)]
pub struct EntornoDeEnsayo {
    pub archivos: HashSet<PathBuf>,
    pub path: Vec<PathBuf>,
    pub personal: Option<PathBuf>,
    pub datos_locales: Option<PathBuf>,
}

impl EntornoDeEnsayo {
    pub fn con_archivo(mut self, ruta: impl Into<PathBuf>) -> Self {
        self.archivos.insert(ruta.into());
        self
    }

    pub fn con_path(mut self, carpeta: impl Into<PathBuf>) -> Self {
        self.path.push(carpeta.into());
        self
    }
}

impl EntornoDelSistema for EntornoDeEnsayo {
    fn buscar_en_path(&self, nombre: &str, extensiones: &[&str]) -> Option<PathBuf> {
        self.path.iter().find_map(|carpeta| {
            if extensiones.is_empty() {
                Some(carpeta.join(nombre)).filter(|c| self.archivos.contains(c))
            } else {
                extensiones
                    .iter()
                    .map(|ext| carpeta.join(format!("{nombre}{ext}")))
                    .find(|c| self.archivos.contains(c))
            }
        })
    }

    fn carpeta_personal(&self) -> Option<PathBuf> {
        self.personal.clone()
    }

    fn carpeta_de_datos_locales(&self) -> Option<PathBuf> {
        self.datos_locales.clone()
    }

    fn existe(&self, ruta: &Path) -> bool {
        self.archivos.contains(ruta)
    }
}

/// Carpeta en el temporal del sistema para una prueba que ejecuta un guion de verdad. Se borra al
/// soltarla, también cuando la prueba falla: un pánico deshace la pila y llama a `Drop`, mientras
/// que un `remove_dir_all` al final de la prueba no llegaría a ejecutarse.
pub struct CarpetaTemporal(PathBuf);

impl CarpetaTemporal {
    /// Crea `<temporal>/<prefijo>-<pid>`. Cada prefijo lo usa una sola prueba por sistema, así que
    /// dos pruebas del mismo proceso nunca comparten carpeta.
    pub fn nueva(prefijo: &str) -> Self {
        let ruta = std::env::temp_dir().join(format!("{prefijo}-{}", std::process::id()));
        std::fs::create_dir_all(&ruta).unwrap();
        Self(ruta)
    }

    /// Ruta de la carpeta.
    pub fn ruta(&self) -> &Path {
        &self.0
    }
}

impl Drop for CarpetaTemporal {
    fn drop(&mut self) {
        // Si no se puede borrar, no se hace nada: un pánico dentro de Drop durante otro pánico
        // abortaría el proceso de pruebas y taparía el fallo de verdad.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Entorno vacío salvo el ejecutable fijado en la petición, que existe. Desde la 3.0.0, una ruta
/// fijada tiene que existir para que el plan salga.
pub fn con_su_ejecutable(p: &PeticionDeLanzamiento) -> EntornoDeEnsayo {
    let mut entorno = EntornoDeEnsayo::default();
    if let Some(ejecutable) = &p.ejecutable {
        entorno = entorno.con_archivo(ejecutable.clone());
    }
    entorno
}

/// Petición con valores de ejemplo.
pub fn peticion(herramienta: Herramienta) -> PeticionDeLanzamiento {
    PeticionDeLanzamiento {
        herramienta,
        identidad: "Claude 2".to_owned(),
        carpeta_proyecto: PathBuf::from("/proyectos/MMCelt"),
        nombre_proyecto: "MMCelt".to_owned(),
        directorio_configuracion: None,
        ejecutable: None,
        argumentos_extra: Vec::new(),
    }
}

/// Planifica y exige que salga bien.
pub fn plan(
    p: &PeticionDeLanzamiento,
    sistema: Sistema,
    entorno: &EntornoDeEnsayo,
) -> PlanDeLanzamiento {
    lanzador::planificar(p, sistema, entorno).expect("la planificación debía salir bien")
}

/// Argumentos como texto, para comparar.
pub fn textos(argumentos: &[OsString]) -> Vec<String> {
    argumentos
        .iter()
        .map(|a| a.to_str().unwrap().to_owned())
        .collect()
}

/// Variables como pares de texto, para comparar.
pub fn variables(plan: &PlanDeLanzamiento) -> Vec<(String, String)> {
    plan.variables
        .iter()
        .map(|(n, v)| (n.clone(), v.to_str().unwrap().to_owned()))
        .collect()
}

/// Guion de PowerShell de un plan de Claude en Windows: comprueba la forma de los argumentos
/// (-NoExit -EncodedCommand <base64>) y decodifica el tercero (Base64 de UTF-16LE).
pub fn guion_powershell(plan: &PlanDeLanzamiento) -> String {
    let args = textos(&plan.argumentos);
    assert_eq!(plan.programa, PathBuf::from("powershell.exe"));
    assert_eq!(args.len(), 3);
    assert_eq!(args[0], "-NoExit");
    assert_eq!(args[1], "-EncodedCommand");
    let bytes = decodificar_base64(&args[2]);
    assert_eq!(bytes.len() % 2, 0, "UTF-16LE ocupa un número par de bytes");
    let unidades: Vec<u16> = bytes
        .chunks(2)
        .map(|par| u16::from_le_bytes([par[0], par[1]]))
        .collect();
    String::from_utf16(&unidades).unwrap()
}

/// Guion de bash de un plan de Claude en Linux: el último argumento, tras «bash -c».
pub fn guion_bash(plan: &PlanDeLanzamiento) -> String {
    let args = textos(&plan.argumentos);
    let n = args.len();
    assert!(n >= 3);
    assert_eq!(args[n - 3], "bash");
    assert_eq!(args[n - 2], "-c");
    args[n - 1].clone()
}

/// Decodificador Base64 estándar, escrito aparte del codificador del crate para no compartir sus
/// posibles errores.
pub fn decodificar_base64(texto: &str) -> Vec<u8> {
    let valor = |c: u8| -> u32 {
        match c {
            b'A'..=b'Z' => u32::from(c - b'A'),
            b'a'..=b'z' => u32::from(c - b'a') + 26,
            b'0'..=b'9' => u32::from(c - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            otro => panic!("carácter no Base64: {otro}"),
        }
    };
    let limpio = texto.trim_end_matches('=').as_bytes();
    let mut salida = Vec::new();
    let mut acumulado = 0u32;
    let mut bits = 0;
    for &c in limpio {
        acumulado = (acumulado << 6) | valor(c);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            salida.push(((acumulado >> bits) & 0xFF) as u8);
        }
    }
    salida
}
