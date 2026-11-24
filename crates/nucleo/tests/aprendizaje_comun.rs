//! Barreras del aprendizaje común: las fichas de aprendizaje-comun/ viajan todas en el ejecutable
//! con sus campos fijos, el documento de consulta solo lleva las comprobadas, y las reglas
//! candidatas de un proyecto se juntan en la carpeta de datos sin repetirse.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::fs;
use std::path::Path;

use comun::{CarpetaTemporal, Entorno};
use nucleo::aprendizaje::{self, EstadoDeFicha};
use nucleo::proyecto::catalogo;

const CARPETA_DE_FICHAS: &str = "../../aprendizaje-comun";

#[test]
fn todas_las_fichas_de_la_carpeta_van_en_el_ejecutable_y_se_leen() {
    let mut en_carpeta: Vec<String> =
        fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(CARPETA_DE_FICHAS))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".md"))
            .collect();
    en_carpeta.sort();
    let mut incluidas: Vec<String> = aprendizaje::FICHAS
        .iter()
        .map(|(n, _)| (*n).to_owned())
        .collect();
    incluidas.sort();
    assert_eq!(en_carpeta, incluidas);
    for (nombre, contenido) in aprendizaje::FICHAS {
        assert!(
            aprendizaje::leer_ficha(contenido).is_some(),
            "{nombre} no tiene la forma de ficha"
        );
        for prohibida in ["M:\\", "C:\\Users"] {
            assert!(!contenido.contains(prohibida), "{nombre} cita {prohibida}");
        }
    }
}

#[test]
fn una_ficha_sin_un_campo_o_con_estado_desconocido_no_se_lee() {
    let buena = "# T\n\nEstado: comprobada\nProblema: p\nCómo se reconoce: c\nSolución: s\nEvidencia: e\nHerramientas y versiones: h\nAplica a: a\n";
    assert_eq!(
        aprendizaje::leer_ficha(buena).unwrap().estado,
        EstadoDeFicha::Comprobada
    );
    assert!(aprendizaje::leer_ficha(&buena.replace("Evidencia: e\n", "")).is_none());
    assert!(aprendizaje::leer_ficha(&buena.replace("comprobada", "quizá")).is_none());
    assert!(aprendizaje::leer_ficha(&buena.replace("# T", "T")).is_none());
}

#[test]
fn el_documento_lleva_solo_las_comprobadas_y_se_escribe_al_preparar() {
    let documento = aprendizaje::documento();
    for (nombre, contenido) in aprendizaje::FICHAS {
        let ficha = aprendizaje::leer_ficha(contenido).unwrap();
        assert_eq!(
            documento.contains(&format!("## {}\n", ficha.titulo)),
            ficha.estado.se_publica(),
            "{nombre}"
        );
    }
    assert!(catalogo::archivos_gestionados(None)
        .iter()
        .any(|a| a.ruta_relativa == aprendizaje::RUTA_DEL_DOCUMENTO));

    let proyecto = CarpetaTemporal::new();
    Entorno::default().inicializar(proyecto.ruta(), None, None);
    assert_eq!(proyecto.leer(aprendizaje::RUTA_DEL_DOCUMENTO), documento);
}

#[test]
fn el_nucleo_dice_cuando_se_lee_el_aprendizaje_comun() {
    let nucleo = nucleo::plantillas::nucleo_comun();
    assert!(nucleo
        .lines()
        .any(|l| l.starts_with("| ") && l.contains(aprendizaje::RUTA_DEL_DOCUMENTO)));
}

#[test]
fn las_candidatas_se_leen_de_las_reglas_aprendidas() {
    let instrucciones = "Texto.\n\n## Reglas aprendidas\n\n\
        - Regla local. (2026-11-18, Claude 1, acta)\n\
        - Regla para todos. (2026-11-18, Claude 2, acta) (candidata a común)\n\
        \n## Otro apartado\n\n- Fuera. (candidata a común)\n";
    assert_eq!(
        aprendizaje::candidatas(instrucciones),
        ["Regla para todos. (2026-11-18, Claude 2, acta)"]
    );
    assert!(aprendizaje::candidatas("Sin apartado. (candidata a común)\n").is_empty());
}

#[test]
fn las_candidatas_se_anaden_sin_repetirse() {
    let candidatas = vec!["Regla A".to_owned(), "Regla B".to_owned()];
    let primero = aprendizaje::anadir_candidatas(None, &candidatas, "Uno", "2026-11-19").unwrap();
    assert!(primero.contains("- Regla A. (Uno, 2026-11-19)\n"));
    assert!(primero.contains("- Regla B. (Uno, 2026-11-19)\n"));
    assert_eq!(
        aprendizaje::anadir_candidatas(Some(&primero), &candidatas, "Dos", "2026-11-20"),
        None
    );
    let mas = aprendizaje::anadir_candidatas(
        Some(&primero),
        &["Regla C".to_owned()],
        "Dos",
        "2026-11-20",
    )
    .unwrap();
    assert!(mas.starts_with(&primero));
    assert!(mas.ends_with("- Regla C. (Dos, 2026-11-20)\n"));
}

#[test]
fn una_candidata_repetida_en_la_misma_recogida_se_escribe_una_vez() {
    // La misma regla dos veces en una recogida.
    let candidatas = vec!["Regla A".to_owned(), "Regla A".to_owned()];
    let texto = aprendizaje::anadir_candidatas(None, &candidatas, "Uno", "2026-11-20").unwrap();
    assert_eq!(
        texto.matches("- Regla A. (Uno, 2026-11-20)").count(),
        1,
        "{texto}"
    );
}

#[test]
fn una_candidata_que_es_prefijo_de_otra_ya_anotada_se_anade() {
    // Se compara la regla entera, no una subcadena.
    let previo = aprendizaje::anadir_candidatas(
        None,
        &[
            "Regla larga con más texto".to_owned(),
            "Con (paréntesis) propios".to_owned(),
        ],
        "Uno",
        "2026-11-19",
    )
    .unwrap();
    let corta = vec!["Regla larga".to_owned()];
    let texto = aprendizaje::anadir_candidatas(Some(&previo), &corta, "Dos", "2026-11-20")
        .expect("la regla corta se descarta");
    assert!(
        texto.ends_with("- Regla larga. (Dos, 2026-11-20)\n"),
        "{texto}"
    );
    let repetida = vec!["Con (paréntesis) propios".to_owned()];
    assert_eq!(
        aprendizaje::anadir_candidatas(Some(&texto), &repetida, "Dos", "2026-11-20"),
        None
    );
}

#[test]
fn un_proyecto_con_punto_y_parentesis_en_el_nombre_no_duplica_la_candidata() {
    // El separador no puede aparecer en el nombre.
    let candidatas = vec!["Regla A".to_owned()];
    let primera =
        aprendizaje::anadir_candidatas(None, &candidatas, "Gestor. (piloto)", "2026-11-20")
            .unwrap();
    assert!(
        primera.ends_with("- Regla A. (Gestor. [piloto], 2026-11-20)\n"),
        "{primera}"
    );
    assert_eq!(
        aprendizaje::anadir_candidatas(
            Some(&primera),
            &candidatas,
            "Gestor. (piloto)",
            "2026-11-21"
        ),
        None
    );
}

#[test]
fn al_preparar_se_juntan_las_candidatas_en_la_carpeta_de_datos() {
    let datos = CarpetaTemporal::new();
    let proyecto = CarpetaTemporal::new();
    proyecto.escribir(
        ".gestor/instrucciones-del-proyecto.md",
        "## Reglas aprendidas

- Una regla. (candidata a común)
",
    );
    let entorno = Entorno::default();
    *entorno.configuracion.datos.borrow_mut() = Some(datos.ruta().to_path_buf());

    let informe = entorno.inicializar(proyecto.ruta(), None, None);
    let candidatas = datos.leer(aprendizaje::ARCHIVO_DE_CANDIDATAS);
    assert!(candidatas.contains("- Una regla. ("), "{candidatas}");
    assert!(informe
        .avisos()
        .iter()
        .any(|a| a.contains(aprendizaje::ARCHIVO_DE_CANDIDATAS)));

    let informe = entorno.inicializar(proyecto.ruta(), None, None);
    assert_eq!(datos.leer(aprendizaje::ARCHIVO_DE_CANDIDATAS), candidatas);
    assert!(!informe
        .avisos()
        .iter()
        .any(|a| a.contains(aprendizaje::ARCHIVO_DE_CANDIDATAS)));
}
