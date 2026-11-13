//! Barreras de la identidad de los agentes y del contenido de las plantillas canónicas.
//!
//! Traducen las pruebas de identidad y de plantillas de la
//! 2.3.2: ninguna marca de región en línea completa y ninguna regla de un proyecto concreto en el
//! núcleo. Desde la 3.1.0, los textos que leen los agentes
//! se escriben para un agente, en markdown: la barrera de prosa sin asteriscos ni almohadillas se
//! sustituye por la que exige que cada uno empiece con un título. Desde la 3.0.0 fijan además
//! que la cabecera es una sola para todas las herramientas, que CLAUDE.md solo apunta a AGENTS.md
//! y al archivo propio, y que el núcleo remite a los documentos de consulta que se escriben.

use nucleo::agentes::{ErrorDeIdentidad, IdentidadAgente, TipoAgente};
use nucleo::instrucciones::{huella, marcas};
use nucleo::plantillas::{self, DocumentoDeConsulta, EsqueletoGestor};

/// Textos que leen los agentes, con un nombre para el mensaje de error.
fn textos_escritos_para_los_agentes() -> Vec<(String, String)> {
    let codex = IdentidadAgente::primera(TipoAgente::Codex);
    let mut textos = vec![
        (String::from("núcleo común"), plantillas::nucleo_comun()),
        (String::from("cabecera"), plantillas::cabecera()),
        (
            String::from("encabezado de entorno"),
            plantillas::encabezado_entorno(),
        ),
        (
            String::from("encabezado pendiente"),
            plantillas::encabezado_pendiente(),
        ),
        (
            String::from("aviso de archivados"),
            plantillas::aviso_archivados(),
        ),
        (
            String::from("instrucciones del proyecto"),
            plantillas::instrucciones_del_proyecto(),
        ),
    ];
    for documento in DocumentoDeConsulta::TODOS {
        textos.push((documento.ruta_relativa().to_owned(), documento.texto()));
    }
    for esqueleto in EsqueletoGestor::TODOS {
        textos.push((
            esqueleto.nombre_archivo().to_owned(),
            plantillas::esqueleto_gestor(esqueleto, Some(&codex)),
        ));
    }
    textos
}

// Identidad.

#[test]
fn identidad_determina_nombre_y_archivos_propios() {
    let casos = [
        (
            TipoAgente::Claude,
            1,
            "Claude 1",
            ".gestor/canal/claude.md",
            ".gestor/claude",
            "CLAUDE.md",
        ),
        (
            TipoAgente::Claude,
            2,
            "Claude 2",
            ".gestor/canal/claude2.md",
            ".gestor/claude2",
            "CLAUDE.md",
        ),
        (
            TipoAgente::Codex,
            3,
            "Codex 3",
            ".gestor/canal/codex3.md",
            ".gestor/codex3",
            "AGENTS.md",
        ),
        (
            TipoAgente::Gemini,
            1,
            "Gemini 1",
            ".gestor/canal/gemini.md",
            ".gestor/gemini",
            "AGENTS.md",
        ),
    ];
    for (tipo, instancia, nombre, bandeja, carpeta, instrucciones) in casos {
        let identidad = IdentidadAgente::new(tipo, instancia);

        let identidad = identidad.as_ref().ok();
        assert_eq!(
            identidad.map(IdentidadAgente::nombre).as_deref(),
            Some(nombre)
        );
        assert_eq!(
            identidad.map(IdentidadAgente::ruta_bandeja).as_deref(),
            Some(bandeja)
        );
        assert_eq!(
            identidad
                .map(IdentidadAgente::ruta_carpeta_propia)
                .as_deref(),
            Some(carpeta)
        );
        assert_eq!(
            identidad.map(IdentidadAgente::archivo_instrucciones),
            Some(instrucciones)
        );
    }
}

#[test]
fn identidad_rechaza_instancia_cero() {
    assert_eq!(
        IdentidadAgente::new(TipoAgente::Claude, 0),
        Err(ErrorDeIdentidad::InstanciaCero { tipo: "Claude" })
    );
}

#[test]
fn primera_instancia_es_la_uno() {
    assert_eq!(
        Ok(IdentidadAgente::primera(TipoAgente::Gemini)),
        IdentidadAgente::new(TipoAgente::Gemini, 1)
    );
}

// Plantillas.

/// Textos que se insertan dentro de otro y no son un documento propio: el aviso de archivados
/// es la frase que presenta una lista dentro del bloque pendiente.
const FRAGMENTOS_SIN_TITULO: [&str; 1] = ["aviso de archivados"];

#[test]
fn plantillas_para_los_agentes_empiezan_con_un_titulo_markdown() {
    for (nombre, texto) in textos_escritos_para_los_agentes() {
        if FRAGMENTOS_SIN_TITULO.contains(&nombre.as_str()) {
            continue;
        }
        assert!(
            texto.starts_with("# ") || texto.starts_with("## "),
            "{nombre} no empieza con un título: {:?}",
            texto.lines().next()
        );
    }
}

#[test]
fn plantillas_no_contienen_marcas_de_region_que_rompan_el_archivo() {
    for (nombre, texto) in textos_escritos_para_los_agentes() {
        assert_eq!(
            marcas::neutralizar(&texto),
            texto,
            "{nombre} contiene una marca de región en línea completa"
        );
    }
}

#[test]
fn nucleo_no_arrastra_reglas_de_un_proyecto_concreto_ni_del_sistema_anterior() {
    let nucleo = plantillas::nucleo_comun();
    for fragmento in [
        "usuario_nobel",
        "material_secundario",
        "P3-B",
        "claudemarketplaces",
        "Agente 1",
        "GEMINI.md",
    ] {
        assert!(!nucleo.contains(fragmento), "{fragmento}");
    }
}

#[test]
fn la_cabecera_es_una_sola_y_nombra_agents_md() {
    let cabecera = plantillas::cabecera();

    assert!(cabecera.contains(
        "\"He leído AGENTS.md y las instrucciones del proyecto. Soy Agente N. Canal Agente N disponible y listo.\""
    ));
    assert!(cabecera.contains("GESTOR_AGENTE"));
}

#[test]
fn el_puntero_de_claude_carga_agents_md_y_el_archivo_propio() {
    let puntero = plantillas::puntero_claude();
    let importaciones: Vec<&str> = puntero.lines().filter(|l| l.starts_with('@')).collect();

    assert_eq!(
        importaciones,
        vec!["@AGENTS.md", "@.gestor/instrucciones-del-proyecto.md"]
    );
}

#[test]
fn el_nucleo_remite_a_cada_documento_de_consulta_y_al_archivo_propio() {
    let nucleo = plantillas::nucleo_comun();

    for documento in DocumentoDeConsulta::TODOS {
        assert!(
            nucleo.contains(documento.ruta_relativa()),
            "{}",
            documento.ruta_relativa()
        );
    }
    assert!(nucleo.contains(".gestor/instrucciones-del-proyecto.md"));
}

#[test]
fn el_inicio_de_sesion_obliga_a_consolidar_el_bloque_pendiente() {
    let nucleo = plantillas::nucleo_comun();

    assert!(nucleo.contains(
        "Si .gestor/instrucciones-del-proyecto.md contiene un bloque pendiente de consolidar"
    ));
    assert!(nucleo.contains("se consolida el bloque pendiente si lo hay"));
    assert!(plantillas::encabezado_pendiente()
        .contains("El primer agente que arranque consolida este bloque"));
    assert!(plantillas::encabezado_pendiente().contains("{{FECHA}}"));
}

#[test]
fn el_nucleo_advierte_que_claude_ve_dos_veces_las_normas_de_la_carpeta() {
    let nucleo = plantillas::nucleo_comun();

    assert!(nucleo.contains("Claude las ve dos veces"));
    assert!(nucleo.contains("manda la original"));
}

#[test]
fn las_plantillas_vigentes_llaman_gestor_agentes_a_la_aplicacion() {
    for (nombre, texto) in textos_escritos_para_los_agentes() {
        assert!(!texto.contains("Gestor Claude"), "{nombre}");
    }
    assert!(plantillas::puntero_claude().contains("Gestor Agentes"));
    assert!(plantillas::nucleo_comun().contains("Gestor Agentes"));
}

#[test]
fn los_textos_anteriores_se_reconocen_con_los_dos_nombres_de_la_aplicacion() {
    let anteriores = plantillas::textos_anteriores_de_la_aplicacion();

    assert_eq!(anteriores.len(), 6);
    for par in anteriores.chunks(2) {
        assert_eq!(par[0], plantillas::version_de_la_2x(&par[1]));
    }
}

#[test]
fn las_plantillas_antiguas_se_conservan_tal_como_las_genero_la_2_3_2() {
    let huellas: Vec<String> = plantillas::plantillas_antiguas()
        .iter()
        .map(|t| huella::calcular(t))
        .collect();

    assert_eq!(
        huellas,
        vec![
            "3eace34d47be4b73388cd69ea4f3319209b69b2bbde7aac2da0700055dc8f363",
            "35e34b6840f63f1efefe8f3aaf6ae902b01aaa9f7e6bdb47ca75954be31142f6",
        ]
    );
}

#[test]
fn los_esqueletos_sustituyen_el_agente_y_terminan_en_salto_de_linea() {
    let claude2 = IdentidadAgente::new(TipoAgente::Claude, 2).ok();

    let bandeja = plantillas::esqueleto_gestor(EsqueletoGestor::Bandeja, claude2.as_ref());
    let estado = plantillas::esqueleto_gestor(EsqueletoGestor::Estado, None);

    assert!(bandeja.starts_with("# Bandeja de Claude 2\n"));
    assert!(!bandeja.contains(plantillas::MARCADOR_AGENTE));
    assert!(bandeja.ends_with('\n') && !bandeja.ends_with("\n\n"));
    assert!(estado.ends_with('\n') && !estado.ends_with("\n\n"));
}
