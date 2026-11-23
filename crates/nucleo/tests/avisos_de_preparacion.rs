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
fn claude_code_y_agy_esperan_al_cambio_y_la_pasada_unica_es_el_ultimo_recurso() {
    // Desde la 3.5.0, agy espera al cambio, que termina con el primer encargo y lo despierta; desde
    // la 3.5.1, Claude Code también, en lugar del Monitor continuo que dejaba procesos huérfanos
    // (orden del Director del 2026-11-18). Desde la 3.6.0 lo hace el ejecutable gestor-vigilante.
    let cabecera = plantillas::cabecera();
    let linea = cabecera
        .lines()
        .find(|l| l.contains("**Claude Code, agy y los demás:**"))
        .expect("línea de Claude Code y agy");
    assert!(linea.contains("lánzalo en segundo plano con `--hasta-cambio`"));
    assert!(linea.contains("en Claude Code, con `run_in_background`, nunca con Monitor."));
    assert!(linea.contains("atiéndelo y vuelve a lanzarlo"));
    assert!(linea.contains("también si la herramienta lo detiene"));
    let ultimo_recurso = cabecera
        .lines()
        .find(|l| l.contains("Si no puede vigilar o no te despierta"))
        .expect("línea de la pasada única");
    assert!(ultimo_recurso.contains("`--una-vez`"));
    assert!(ultimo_recurso.contains("dilo en tu bandeja"));
    // Sin código en la línea de órdenes: la envoltura de agy expande las variables de PowerShell.
    assert!(!linea.contains('$'), "{linea}");
}

#[test]
fn ningun_vigilante_se_rearma_sin_fin_y_el_de_codex_se_entrega_en_su_sesion() {
    // El «Rearma cuando caduque» del Monitor acumulaba vigilantes vivos (incidente del
    // 2026-11-18): ya no se pide a nadie. Codex no monta su propio sistema desde la 3.6.0: usa el
    // modo persistente, que le entrega los avisos en su sesión.
    let cabecera = plantillas::cabecera();
    assert!(!cabecera.contains("Rearma cuando caduque"));
    assert!(!cabecera.contains("herramienta Monitor con"));
    assert!(!cabecera.contains("monta tú mismo"));
    let codex = cabecera
        .lines()
        .find(|l| l.contains("**Codex:**"))
        .expect("línea de Codex");
    assert!(codex.contains("`--anotar-hilo`"));
    assert!(codex.contains("no lo rearmes"));
}
