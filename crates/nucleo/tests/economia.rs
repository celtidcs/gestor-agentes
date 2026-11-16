//! Barreras de la economía de mensajes y de los archivos ligeros: el núcleo común manda escribir en
//! el canal solo lo que hace falta, el diario y el estado se quedan con lo vigente, y la preparación
//! avisa de los diarios y del estado que se han hecho demasiado grandes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use comun::{CarpetaTemporal, Entorno};
use nucleo::agentes::{IdentidadAgente, TipoAgente};
use nucleo::plantillas::{self, EsqueletoGestor};
use nucleo::proyecto::inicializador;

#[test]
fn el_nucleo_pide_mensajes_solo_con_resultado_pregunta_o_bloqueo() {
    let nucleo = plantillas::nucleo_comun();
    for frase in [
        "Escribe en el canal solo con un resultado, una pregunta o un bloqueo: sin acuses ni avisos de arranque.",
        "Cita rutas, no copies contenido.",
        "Copia al Director solo si decide él.",
        "No repitas en la terminal lo del canal.",
        "Quien recibe una orden directa del Director la traslada",
        "Las que llegan por el canal no se recopian.",
    ] {
        assert!(nucleo.contains(frase), "{frase}");
    }
    assert!(!nucleo.contains("Toda orden del Director se traslada"));
}

#[test]
fn el_diario_y_el_estado_solo_guardan_lo_vigente() {
    let claude_2 = IdentidadAgente::new(TipoAgente::Claude, 2).unwrap();
    let nucleo = plantillas::nucleo_comun();
    assert!(nucleo.contains("Lo cerrado va a `historico-diario.md`."));
    assert!(nucleo.contains("`estado.md` solo recoge hechos comprobables y vigentes"));
    assert!(
        plantillas::esqueleto_gestor(EsqueletoGestor::TareaEnCurso, Some(&claude_2))
            .contains("historico-diario.md")
    );
    assert!(plantillas::esqueleto_gestor(EsqueletoGestor::Estado, None)
        .contains("lo superado pasa a `.gestor/archivo/`"));
}

#[test]
fn un_archivo_de_arranque_avisa_solo_si_pasa_del_limite() {
    let entradas = vec![
        (String::from("b"), 101),
        (String::from("igual"), 100),
        (String::from("a"), 102),
        (String::from("menor"), 99),
    ];
    assert_eq!(
        inicializador::archivos_de_arranque_grandes(&entradas, 100),
        vec![(String::from("a"), 102), (String::from("b"), 101)]
    );
}

#[test]
fn la_preparacion_avisa_de_los_diarios_y_del_estado_grandes() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let limite =
        usize::try_from(nucleo::ajustes::LIMITE_DE_BANDEJA_POR_DEFECTO).unwrap_or(usize::MAX);
    let de_arranque =
        |informe: &nucleo::proyecto::informe::InformeDeInicializacion| -> Vec<String> {
            informe
                .avisos()
                .iter()
                .filter(|a| a.contains("que conviene leer en cada arranque"))
                .cloned()
                .collect()
        };

    let informe = entorno.inicializar(carpeta.ruta(), None, None);
    assert!(de_arranque(&informe).is_empty(), "{:?}", informe.avisos());

    carpeta.escribir(".gestor/claude2/tarea-en-curso.md", &"x".repeat(limite + 1));
    carpeta.escribir(".gestor/claude/tarea-en-curso.md", &"x".repeat(limite));
    let estado = carpeta.leer(".gestor/canal/estado.md");
    carpeta.escribir(
        ".gestor/canal/estado.md",
        &format!("{estado}{}", "x".repeat(limite)),
    );
    let informe = entorno.inicializar(carpeta.ruta(), None, None);
    let avisos = de_arranque(&informe);
    assert_eq!(avisos.len(), 2, "{avisos:?}");
    assert!(avisos[0].starts_with(".gestor/canal/estado.md mide"));
    assert!(avisos[1].starts_with(".gestor/claude2/tarea-en-curso.md mide"));
}
