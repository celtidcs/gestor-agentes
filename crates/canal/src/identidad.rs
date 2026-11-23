//! Identidad de un agente tal como la pone Gestor Agentes en GESTOR_AGENTE: el nombre de la
//! herramienta y un número de instancia, como «Claude 2» o «Codex 1».

use std::fmt;

/// Identidad de un agente.
///
/// Invariantes: la herramienta solo lleva letras y cifras ASCII y empieza por letra; la instancia
/// es 1 o más.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identidad {
    /// Herramienta como se escribe en el canal, por ejemplo «Claude».
    herramienta: String,
    /// Número de instancia, desde 1.
    instancia: u32,
}

/// Motivo por el que un texto no es una identidad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentidadInvalida {
    /// El texto está vacío o solo tiene espacios.
    Vacia,
    /// El texto no sigue la forma «Herramienta N».
    Mal(String),
}

impl fmt::Display for IdentidadInvalida {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vacia => write!(f, "la identidad está vacía"),
            Self::Mal(texto) => write!(f, "identidad no válida: {texto}"),
        }
    }
}

impl Identidad {
    /// Lee «Herramienta» o «Herramienta N». Sin número es la instancia 1.
    ///
    /// # Errors
    ///
    /// [`IdentidadInvalida`] si el texto está vacío, la herramienta no es una palabra ASCII que
    /// empiece por letra o el número no es un entero de 1 en adelante.
    pub fn leer(texto: &str) -> Result<Self, IdentidadInvalida> {
        let partes: Vec<&str> = texto.split_whitespace().collect();
        let (herramienta, instancia) = match partes.as_slice() {
            [] => return Err(IdentidadInvalida::Vacia),
            [herramienta] => (*herramienta, 1),
            [herramienta, numero] => (
                *herramienta,
                numero
                    .parse::<u32>()
                    .ok()
                    .filter(|n| *n >= 1)
                    .ok_or_else(|| IdentidadInvalida::Mal(texto.to_owned()))?,
            ),
            _ => return Err(IdentidadInvalida::Mal(texto.to_owned())),
        };
        let valida = herramienta
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
            && herramienta.chars().all(|c| c.is_ascii_alphanumeric());
        if !valida {
            return Err(IdentidadInvalida::Mal(texto.to_owned()));
        }
        Ok(Self {
            herramienta: herramienta.to_owned(),
            instancia,
        })
    }

    /// Herramienta como se escribió, por ejemplo «Claude».
    #[must_use]
    pub fn herramienta(&self) -> &str {
        &self.herramienta
    }

    /// Número de instancia, desde 1.
    #[must_use]
    pub const fn instancia(&self) -> u32 {
        self.instancia
    }

    /// Nombre de sus archivos: la herramienta en minúsculas y el número si no es 1, como
    /// «claude2» o «codex».
    #[must_use]
    pub fn nombre_de_archivos(&self) -> String {
        let base = self.herramienta.to_ascii_lowercase();
        if self.instancia == 1 {
            base
        } else {
            format!("{base}{}", self.instancia)
        }
    }
}

impl fmt::Display for Identidad {
    /// La identidad con su número siempre, como «Codex 1».
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.herramienta, self.instancia)
    }
}
