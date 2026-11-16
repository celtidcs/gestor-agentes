//! Barreras de los avisos de la preparación para los agentes: los avisos no abren ventana, se
//! guardan en .gestor/avisos-de-preparacion.md, que el arranque manda leer, y el archivo
//! desaparece cuando ya no queda ningún aviso.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use comun::{momento_fijo, CarpetaTemporal, Entorno};
use nucleo::plantillas;
use nucleo::proyecto::avisos_de_preparacion::{self, RUTA_RELATIVA};
use nucleo::proyecto::informe::{InformeDeInicializacion, TipoDeCambio};

#[test]
fn sin_avisos_no_hay_texto() {
    assert_eq!(avisos_de_preparacion::componer(&[], momento_fijo()), None);
}

#[test]
fn cada_aviso_va_en_su_linea_con_la_fecha_de_la_preparacion() {
    let avisos = vec![String::from("primero"), String::from("segundo")];
    let texto = avisos_de_preparacion::componer(&avisos, momento_fijo()).unwrap();
    assert!(texto.contains(&momento_fijo().to_string()), "{texto}");
    assert!(texto.ends_with("\n- primero\n- segundo\n"), "{texto}");
}

#[test]
fn la_preparacion_guarda_los_avisos_y_borra_el_archivo_cuando_se_resuelven() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let limite =
        usize::try_from(nucleo::ajustes::LIMITE_DE_BANDEJA_POR_DEFECTO).unwrap_or(usize::MAX);
    carpeta.escribir(".gestor/canal/codex.md", &"x".repeat(limite + 1));

    let informe = entorno.inicializar(carpeta.ruta(), None, None);
    assert!(!informe.avisos().is_empty());
    let guardado = carpeta.leer(RUTA_RELATIVA);
    for aviso in informe.avisos() {
        assert!(guardado.contains(&format!("- {aviso}\n")), "{guardado}");
    }

    carpeta.escribir(".gestor/canal/codex.md", "podada\n");
    let informe = entorno.inicializar(carpeta.ruta(), None, None);
    assert!(informe.avisos().is_empty(), "{:?}", informe.avisos());
    assert!(!carpeta.existe(RUTA_RELATIVA));
}

#[test]
fn si_no_se_puede_borrar_el_archivo_de_avisos_se_dice_como_error() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    carpeta.crear_carpeta(RUTA_RELATIVA);

    let informe = entorno.inicializar(carpeta.ruta(), None, None);
    assert!(informe.avisos().is_empty(), "{:?}", informe.avisos());
    assert!(
        informe
            .errores()
            .iter()
            .any(|e| e.starts_with(RUTA_RELATIVA)),
        "{:?}",
        informe.errores()
    );
}

#[test]
fn los_avisos_solos_no_piden_ventana_y_los_errores_y_respaldos_si() {
    let mut informe = InformeDeInicializacion::new();
    informe.anotar_cambio("AGENTS.md", TipoDeCambio::Actualizado, None);
    informe.anotar_aviso("una bandeja sin podar");
    assert!(!informe.requiere_ventana());

    let mut con_error = informe.clone();
    con_error.anotar_error("CLAUDE.md: acceso denegado");
    assert!(con_error.requiere_ventana());

    for tipo in [
        TipoDeCambio::Migrado,
        TipoDeCambio::Respaldado,
        TipoDeCambio::Archivado,
    ] {
        let mut con_cambio = InformeDeInicializacion::new();
        con_cambio.anotar_cambio("CLAUDE.md", tipo, None);
        assert!(con_cambio.requiere_ventana(), "{tipo:?}");
    }
}

#[test]
fn el_arranque_manda_leer_los_avisos() {
    let cabecera = plantillas::cabecera();
    assert!(cabecera.contains(&format!(
        "Si existe `{RUTA_RELATIVA}`, léelo y atiende lo tuyo."
    )));
}

#[test]
fn el_vigilante_de_agy_es_continuo_y_la_pasada_unica_es_el_ultimo_recurso() {
    let cabecera = plantillas::cabecera();
    let linea = cabecera
        .lines()
        .find(|l| l.contains("**agy y los demás:**"))
        .expect("línea de agy");
    assert!(linea.contains("monta una vigilancia continua de `.gestor/canal/`"));
    assert!(linea.contains("aunque estés esperando; una sola y que puedas parar."));
    assert!(linea.contains("Solo si no puedes, pasada única"));
    assert!(linea.contains("dilo en tu bandeja"));
}
