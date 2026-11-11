//! Crea los archivos de .gestor/ que define el núcleo común.
//! Solo crea lo que falta: nunca sobrescribe un archivo de .gestor/, porque cada uno pertenece a
//! un agente o recoge hechos que escriben los agentes.

use std::io;
use std::path::Path;

use super::archivos;
use super::informe::{InformeDeInicializacion, TipoDeCambio};
use crate::agentes::{IdentidadAgente, CARPETA_CANAL, CARPETA_GESTOR};
use crate::plantillas::{self, EsqueletoGestor};

/// Asegura la estructura de .gestor/ para las identidades indicadas, en este orden: crea
/// estado.md y ordenes-del-director.md y, por cada identidad, su bandeja y los archivos de su
/// carpeta propia.
///
/// # Errors
///
/// Devuelve el primer error de disco; lo hecho hasta entonces queda en el informe.
pub fn asegurar(
    raiz: &Path,
    identidades: &[IdentidadAgente],
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    crear(
        raiz,
        &format!("{CARPETA_CANAL}/estado.md"),
        &plantillas::esqueleto_gestor(EsqueletoGestor::Estado, None),
        informe,
    )?;
    crear(
        raiz,
        &format!("{CARPETA_GESTOR}/ordenes-del-director.md"),
        &plantillas::esqueleto_gestor(EsqueletoGestor::OrdenesDelDirector, None),
        informe,
    )?;

    // Una identidad repetida no hace nada la segunda vez, porque crear() no toca lo que existe.
    for identidad in identidades {
        crear(
            raiz,
            &identidad.ruta_bandeja(),
            &plantillas::esqueleto_gestor(EsqueletoGestor::Bandeja, Some(identidad)),
            informe,
        )?;
        for esqueleto in EsqueletoGestor::DE_LA_CARPETA_PROPIA {
            crear(
                raiz,
                &format!(
                    "{}/{}",
                    identidad.ruta_carpeta_propia(),
                    esqueleto.nombre_archivo()
                ),
                &plantillas::esqueleto_gestor(esqueleto, Some(identidad)),
                informe,
            )?;
        }
    }
    Ok(())
}

/// Crea un archivo si falta y lo anota.
fn crear(
    raiz: &Path,
    ruta_relativa: &str,
    contenido: &str,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    if archivos::crear_si_no_existe(&archivos::ruta(raiz, ruta_relativa), contenido)? {
        informe.anotar_cambio(ruta_relativa, TipoDeCambio::Creado, None);
    }
    Ok(())
}
