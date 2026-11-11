//! Informe de lo que ha hecho la inicialización de un proyecto: cambios, avisos y errores. Lo lee
//! el usuario, así que el texto es el mismo que mostraba la 2.3.2. Los textos están en crate::textos.

use crate::textos;

/// Clase de cambio hecho en un archivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDeCambio {
    /// El archivo no existía y se ha creado.
    Creado,
    /// Se han reescrito regiones gestionadas o se ha añadido una entrada.
    Actualizado,
    /// El archivo se ha pasado al formato por regiones.
    Migrado,
    /// Un archivo del sistema anterior se ha movido a .gestor/archivo/.
    Archivado,
    /// Se ha guardado una copia exacta antes de modificarlo.
    Respaldado,
    /// Se ha creado el repositorio git.
    RepositorioCreado,
}

impl TipoDeCambio {
    /// Texto con que se describe el cambio en el informe.
    #[must_use]
    pub const fn descripcion(self) -> &'static str {
        match self {
            Self::Creado => textos::CAMBIO_CREADO,
            Self::Actualizado => textos::CAMBIO_ACTUALIZADO,
            Self::Migrado => textos::CAMBIO_MIGRADO,
            Self::Archivado => textos::CAMBIO_ARCHIVADO,
            Self::Respaldado => textos::CAMBIO_RESPALDADO,
            Self::RepositorioCreado => textos::CAMBIO_REPOSITORIO_CREADO,
        }
    }

    /// Los cambios que el usuario debe revisar aunque todo haya ido bien.
    const fn requiere_atencion(self) -> bool {
        matches!(self, Self::Migrado | Self::Respaldado | Self::Archivado)
    }
}

/// Un cambio anotado en el informe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CambioRealizado {
    /// Ruta relativa del archivo, con barras normales.
    pub archivo: String,
    /// Clase de cambio.
    pub tipo: TipoDeCambio,
    /// Explicación adicional, si la hay.
    pub detalle: Option<String>,
}

/// Informe de una inicialización. Solo crece: se anota en el orden en que pasan las cosas.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InformeDeInicializacion {
    cambios: Vec<CambioRealizado>,
    avisos: Vec<String>,
    errores: Vec<String>,
}

impl InformeDeInicializacion {
    /// Crea un informe vacío.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Cambios anotados, en orden.
    #[must_use]
    pub fn cambios(&self) -> &[CambioRealizado] {
        &self.cambios
    }

    /// Avisos anotados, en orden.
    #[must_use]
    pub fn avisos(&self) -> &[String] {
        &self.avisos
    }

    /// Errores anotados, en orden.
    #[must_use]
    pub fn errores(&self) -> &[String] {
        &self.errores
    }

    /// Indica si se ha cambiado algo.
    #[must_use]
    pub fn hay_cambios(&self) -> bool {
        !self.cambios.is_empty()
    }

    /// Indica si el usuario debe leer el informe: hay errores o avisos, o se ha migrado,
    /// respaldado o archivado algo.
    #[must_use]
    pub fn requiere_atencion(&self) -> bool {
        !self.errores.is_empty()
            || !self.avisos.is_empty()
            || self.cambios.iter().any(|c| c.tipo.requiere_atencion())
    }

    /// Anota un cambio.
    pub fn anotar_cambio(
        &mut self,
        archivo: impl Into<String>,
        tipo: TipoDeCambio,
        detalle: Option<String>,
    ) {
        self.cambios.push(CambioRealizado {
            archivo: archivo.into(),
            tipo,
            detalle,
        });
    }

    /// Anota un aviso.
    pub fn anotar_aviso(&mut self, aviso: impl Into<String>) {
        self.avisos.push(aviso.into());
    }

    /// Anota un error.
    pub fn anotar_error(&mut self, error: impl Into<String>) {
        self.errores.push(error.into());
    }

    /// Texto del informe para el usuario: apartados de errores, avisos y cambios, cada uno con sus
    /// líneas precedidas de guion, o una frase si no ha hecho falta nada.
    #[must_use]
    pub fn componer_texto(&self) -> String {
        let mut texto = String::new();
        anadir_apartado(
            &mut texto,
            textos::APARTADO_ERRORES,
            self.errores.iter().cloned(),
        );
        anadir_apartado(
            &mut texto,
            textos::APARTADO_AVISOS,
            self.avisos.iter().cloned(),
        );
        anadir_apartado(
            &mut texto,
            textos::APARTADO_CAMBIOS,
            self.cambios.iter().map(describir_cambio),
        );
        if texto.is_empty() {
            String::from(textos::INFORME_SIN_CAMBIOS)
        } else {
            texto.trim_end().to_owned()
        }
    }
}

/// Línea del informe para un cambio.
fn describir_cambio(cambio: &CambioRealizado) -> String {
    textos::linea_de_cambio(
        &cambio.archivo,
        cambio.tipo.descripcion(),
        cambio.detalle.as_deref(),
    )
}

/// Añade un apartado con título y líneas, o nada si no hay líneas.
fn anadir_apartado(texto: &mut String, titulo: &str, lineas: impl Iterator<Item = String>) {
    let lineas: Vec<String> = lineas.collect();
    if lineas.is_empty() {
        return;
    }
    texto.push_str(titulo);
    texto.push('\n');
    for linea in lineas {
        texto.push_str("- ");
        texto.push_str(&linea);
        texto.push('\n');
    }
    texto.push('\n');
}
