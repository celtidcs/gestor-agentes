//! Barreras de las reglas antiguas de agy: las reglas de proyecto que agy carga desde .agents/
//! (https://antigravity.google/docs/rules/) se tratan como GEMINI.md. Su texto propio pasa al
//! bloque pendiente y el archivo se aparta a .gestor/archivo/sistema-anterior/agy/. Las
//! habilidades de .agents/skills no se tocan. Las instrucciones prohíben seguir lo archivado y
//! mandan borrarlo al terminar de consolidar.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use comun::{CarpetaTemporal, Entorno};
use nucleo::plantillas::{self, DocumentoDeConsulta};
use nucleo::proyecto::catalogo;
use nucleo::proyecto::sistema_anterior;

const PROPIO: &str = catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO;

/// Regla propia de un proyecto escrita para agy en .agents/AGENTS.md.
const REGLA_DE_AGY: &str = "Los assets se buscan en todos los repositorios de la comunidad.";

/// Regla modular de agy en .agents/rules/.
const REGLA_MODULAR: &str = "Las texturas van en formato PNG.";

fn proyecto_con_reglas_de_agy() -> CarpetaTemporal {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        ".agents/AGENTS.md",
        &format!("# Reglas\n\n{REGLA_DE_AGY}\n"),
    );
    carpeta.escribir(".agents/rules/texturas.md", &format!("{REGLA_MODULAR}\n"));
    carpeta.escribir(".agents/rules/notas.txt", "No es una regla.\n");
    carpeta.escribir(".agents/skills/diseno/SKILL.md", "Habilidad.\n");
    carpeta
}

#[test]
fn las_reglas_de_agy_pasan_al_bloque_pendiente_y_se_archivan() {
    let carpeta = proyecto_con_reglas_de_agy();

    let _ = Entorno::default().inicializar(carpeta.ruta(), None, None);

    let propio = carpeta.leer(PROPIO);
    assert!(propio.contains(REGLA_DE_AGY), "{propio}");
    assert!(propio.contains(REGLA_MODULAR), "{propio}");
    let archivo = sistema_anterior::CARPETA_ARCHIVO_SISTEMA_ANTERIOR;
    for (origen, destino) in [
        (".agents/AGENTS.md", format!("{archivo}/agy/AGENTS.md")),
        (
            ".agents/rules/texturas.md",
            format!("{archivo}/agy/rules/texturas.md"),
        ),
    ] {
        assert!(!carpeta.existe(origen), "{origen} sigue en su sitio");
        assert!(carpeta.existe(&destino), "falta {destino}");
        assert!(
            propio.contains(&format!("- {origen}, ahora en {destino}")),
            "{propio}"
        );
    }
}

#[test]
fn lo_que_no_es_regla_de_agy_se_queda_donde_estaba() {
    let carpeta = proyecto_con_reglas_de_agy();

    let _ = Entorno::default().inicializar(carpeta.ruta(), None, None);

    assert!(carpeta.existe(".agents/skills/diseno/SKILL.md"));
    assert!(carpeta.existe(".agents/rules/notas.txt"));
}

#[test]
fn la_ruta_antigua_de_reglas_de_agy_tambien_se_archiva() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(".agent/rules/antigua.md", "Regla de la carpeta antigua.\n");

    let _ = Entorno::default().inicializar(carpeta.ruta(), None, None);

    assert!(!carpeta.existe(".agent/rules/antigua.md"));
    assert!(carpeta.existe(&format!(
        "{}/agy/rules/antigua.md",
        sistema_anterior::CARPETA_ARCHIVO_SISTEMA_ANTERIOR
    )));
}

#[test]
fn las_instrucciones_prohiben_seguir_lo_archivado_y_mandan_borrarlo() {
    let nucleo = plantillas::nucleo_comun();
    assert!(
        nucleo.contains("No leas ni sigas nada de `.gestor/archivo/`"),
        "el núcleo no prohíbe seguir lo archivado"
    );

    let metodo = DocumentoDeConsulta::Metodo.texto();
    for carpeta in [
        sistema_anterior::CARPETA_ARCHIVO_SISTEMA_ANTERIOR,
        sistema_anterior::CARPETA_ARCHIVO_GEMINI,
    ] {
        assert!(
            metodo.contains(&format!("borra `{carpeta}/`")),
            "metodo.md no manda borrar {carpeta}"
        );
    }
}
