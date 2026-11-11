//! Lectura y escritura de los archivos de configuración de la carpeta datos/.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::{disco, ErrorDeConfiguracion};
use super::esquema::{self, PerfilGuardado};
use super::perfiles::{self, PerfilDeAgente, PerfilesInvalidos};
use super::proyectos::ProyectoRegistrado;
use super::{
    ARCHIVO_INSTRUCCIONES_ANTIGUAS, ARCHIVO_PERFILES, ARCHIVO_PROYECTOS, ARCHIVO_VERSION_PERFILES,
};
use crate::ajustes::{Ajustes, ARCHIVO_AJUSTES};
use crate::proyecto::inicializador::ConfiguracionDeInstrucciones;

/// Configuración guardada en una carpeta de datos.
#[derive(Debug, Clone)]
pub struct Configuracion {
    carpeta: PathBuf,
}

impl Configuracion {
    /// Configuración en la carpeta indicada, normalmente datos/ junto al ejecutable.
    #[must_use]
    pub fn new(carpeta: impl Into<PathBuf>) -> Self {
        Self {
            carpeta: carpeta.into(),
        }
    }

    /// Carpeta de la configuración.
    #[must_use]
    pub fn carpeta(&self) -> &Path {
        &self.carpeta
    }

    /// Ruta del archivo de perfiles, para mostrarla al usuario.
    #[must_use]
    pub fn ruta_perfiles(&self) -> PathBuf {
        self.carpeta.join(ARCHIVO_PERFILES)
    }

    /// Ajustes de funcionamiento. Si no hay archivo, lo escribe con los valores por defecto, para
    /// que el usuario vea qué puede cambiar.
    ///
    /// # Errors
    ///
    /// Devuelve [`ErrorDeConfiguracion`] si el archivo no se puede leer o escribir, no es JSON
    /// válido o tiene un valor rechazado por [`Ajustes::validar`].
    pub fn cargar_ajustes(&self) -> Result<Ajustes, ErrorDeConfiguracion> {
        let Some(ajustes) = self.leer_json::<Ajustes>(ARCHIVO_AJUSTES)? else {
            let ajustes = Ajustes::default();
            self.escribir_json(ARCHIVO_AJUSTES, &ajustes)?;
            return Ok(ajustes);
        };
        ajustes
            .validar()
            .map_err(|motivo| ErrorDeConfiguracion::AjusteInvalido {
                ruta: self.carpeta.join(ARCHIVO_AJUSTES),
                motivo,
            })
    }

    /// Proyectos registrados; vacío si todavía no hay archivo.
    ///
    /// # Errors
    ///
    /// Devuelve [`ErrorDeConfiguracion`] si el archivo no se puede leer o no es válido.
    pub fn cargar_proyectos(&self) -> Result<Vec<ProyectoRegistrado>, ErrorDeConfiguracion> {
        Ok(self.leer_json(ARCHIVO_PROYECTOS)?.unwrap_or_default())
    }

    /// Guarda la lista de proyectos.
    ///
    /// # Errors
    ///
    /// Devuelve [`ErrorDeConfiguracion::Disco`] si no se puede escribir.
    pub fn guardar_proyectos(
        &self,
        proyectos: &[ProyectoRegistrado],
    ) -> Result<(), ErrorDeConfiguracion> {
        self.escribir_json(ARCHIVO_PROYECTOS, &proyectos)
    }

    /// Carga los perfiles. Si no hay archivo, escribe los de por defecto; si el archivo es de un
    /// esquema anterior, lo pone al día (ver el módulo `esquema`) y lo reescribe una sola vez, de
    /// modo que un perfil que el usuario quite después no vuelve a aparecer.
    ///
    /// # Errors
    ///
    /// Devuelve [`ErrorDeConfiguracion`] si el archivo no es válido, un perfil tiene instancia 0,
    /// hay dos perfiles con la misma identidad o no se puede escribir.
    pub fn cargar_perfiles(&self) -> Result<Vec<PerfilDeAgente>, ErrorDeConfiguracion> {
        let Some(guardados) = self.leer_json::<Vec<PerfilGuardado>>(ARCHIVO_PERFILES)? else {
            let perfiles = PerfilDeAgente::por_defecto();
            self.escribir_json(ARCHIVO_PERFILES, &perfiles)?;
            self.guardar_version_perfiles()?;
            return Ok(perfiles);
        };

        let version = self.leer_version_perfiles();
        let perfiles = esquema::poner_al_dia(guardados, version);
        if !esquema::esta_al_dia(version) {
            self.validar_perfiles(&perfiles)?;
            self.escribir_json(ARCHIVO_PERFILES, &perfiles)?;
            self.guardar_version_perfiles()?;
        }

        self.validar_perfiles(&perfiles)?;
        Ok(perfiles)
    }

    /// Guarda los perfiles, por ejemplo tras cambiar la ruta de un ejecutable o añadir las cuentas
    /// detectadas. Valida antes de escribir, para no dejar en el disco un archivo que el siguiente
    /// arranque rechazaría.
    ///
    /// # Errors
    ///
    /// Devuelve [`ErrorDeConfiguracion`] si un perfil no es válido, hay dos con la misma
    /// identidad o no se puede escribir.
    pub fn guardar_perfiles(
        &self,
        perfiles: &[PerfilDeAgente],
    ) -> Result<(), ErrorDeConfiguracion> {
        self.validar_perfiles(perfiles)?;
        self.escribir_json(ARCHIVO_PERFILES, perfiles)?;
        self.guardar_version_perfiles()
    }

    /// Valida los perfiles y, si fallan, pone la ruta del archivo de perfiles en el error.
    fn validar_perfiles(&self, perfiles: &[PerfilDeAgente]) -> Result<(), ErrorDeConfiguracion> {
        perfiles::validar(perfiles).map_err(|fallo| match fallo {
            PerfilesInvalidos::Perfil(motivo) => ErrorDeConfiguracion::PerfilInvalido {
                ruta: self.ruta_perfiles(),
                motivo,
            },
            PerfilesInvalidos::Repetido(nombre) => ErrorDeConfiguracion::PerfilRepetido {
                ruta: self.ruta_perfiles(),
                nombre,
            },
        })
    }

    fn leer_version_perfiles(&self) -> u32 {
        fs::read_to_string(self.carpeta.join(ARCHIVO_VERSION_PERFILES))
            .ok()
            .and_then(|texto| texto.trim().parse().ok())
            .unwrap_or(esquema::VERSION_SIN_ARCHIVO)
    }

    fn guardar_version_perfiles(&self) -> Result<(), ErrorDeConfiguracion> {
        let ruta = self.carpeta.join(ARCHIVO_VERSION_PERFILES);
        fs::write(&ruta, esquema::VERSION_PERFILES.to_string())
            .map_err(|origen| disco(&ruta, origen))
    }

    fn leer_json<T: for<'de> Deserialize<'de>>(
        &self,
        archivo: &str,
    ) -> Result<Option<T>, ErrorDeConfiguracion> {
        let ruta = self.carpeta.join(archivo);
        let texto = match fs::read_to_string(&ruta) {
            Ok(texto) => texto,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(origen) => return Err(disco(&ruta, origen)),
        };
        serde_json::from_str::<Option<T>>(texto.trim_start_matches('\u{FEFF}')).map_err(|error| {
            ErrorDeConfiguracion::FormatoInvalido {
                ruta,
                motivo: error.to_string(),
            }
        })
    }

    fn escribir_json<T: Serialize + ?Sized>(
        &self,
        archivo: &str,
        datos: &T,
    ) -> Result<(), ErrorDeConfiguracion> {
        fs::create_dir_all(&self.carpeta).map_err(|origen| disco(&self.carpeta, origen))?;
        let ruta = self.carpeta.join(archivo);
        let texto = serde_json::to_string_pretty(datos).map_err(|error| {
            ErrorDeConfiguracion::FormatoInvalido {
                ruta: ruta.clone(),
                motivo: error.to_string(),
            }
        })?;
        fs::write(&ruta, texto).map_err(|origen| disco(&ruta, origen))
    }
}

impl ConfiguracionDeInstrucciones for Configuracion {
    fn leer_instrucciones_antiguas(&self) -> Option<String> {
        fs::read_to_string(self.carpeta.join(ARCHIVO_INSTRUCCIONES_ANTIGUAS)).ok()
    }
}
