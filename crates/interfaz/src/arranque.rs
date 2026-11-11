//! Arranque: carpeta de datos, migración desde la 2.x y estado inicial.

use std::ffi::OsString;
use std::path::PathBuf;

use lanzador::Sistema;
use nucleo::configuracion::{self, Configuracion};

use crate::aplicacion::{Aplicacion, Servicios};
use crate::rutas;
use crate::textos;

/// No se sabe dónde está el ejecutable, así que no hay carpeta de datos.
#[derive(Debug, thiserror::Error)]
#[error("no se puede saber la carpeta del ejecutable: {0}")]
pub struct SinCarpetaDeDatos(String);

/// Carpeta de datos del ejecutable en curso.
///
/// # Errors
///
/// [`SinCarpetaDeDatos`] si el sistema no dice dónde está el ejecutable.
pub fn carpeta_de_datos() -> Result<PathBuf, SinCarpetaDeDatos> {
    let ejecutable =
        std::env::current_exe().map_err(|error| SinCarpetaDeDatos(error.to_string()))?;
    rutas::carpeta_de_datos(&ejecutable)
        .ok_or_else(|| SinCarpetaDeDatos(ejecutable.display().to_string()))
}

/// Estado inicial: copia la configuración de la 2.x si es el primer arranque, carga proyectos y
/// perfiles y deja preparado el aviso de la migración o su error.
#[must_use]
pub fn estado_inicial(
    configuracion: &Configuracion,
    servicios: Box<dyn Servicios>,
    sistema: Sistema,
    variable: impl Fn(&str) -> Option<OsString>,
) -> Aplicacion {
    let migracion = rutas::datos_de_aplicacion(sistema, variable).map(|datos| {
        let origen = configuracion::carpeta_de_la_2x(&datos);
        let resultado = configuracion.migrar_desde(&origen);
        (origen, resultado)
    });

    let mut aplicacion = Aplicacion::new(
        servicios,
        configuracion.cargar_proyectos(),
        configuracion.cargar_perfiles(),
    );

    match migracion {
        Some((origen, Ok(copiados))) if !copiados.is_empty() => aplicacion.informar(
            textos::TITULO_MIGRACION,
            textos::migracion(&origen, configuracion.carpeta(), &copiados),
        ),
        Some((_, Err(error))) => aplicacion.anunciar(textos::migracion_fallida(&error)),
        _ => {}
    }
    aplicacion
}
