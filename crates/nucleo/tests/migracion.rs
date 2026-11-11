//! Barreras de la migración al archivo único de instrucciones: qué texto de los
//! archivos anteriores pasa al archivo propio del proyecto, cómo se compone ese archivo y cuándo
//! se respalda un archivo que la aplicación escribe entero.
//!
//! Los casos reproducen las familias de archivos que dejan las versiones anteriores en proyectos
//! reales: regiones de la 2.3.2
//! con la zona del proyecto debajo, archivos sin marcas escritos a mano, la misma regla repetida
//! en los tres archivos, marcas dañadas, plantillas completas de la 1.x y bloques copiados de las
//! normas de la carpeta proyecto. La condición que fijan todas es la misma: ningún párrafo propio
//! se pierde y ninguno sale dos veces.

use nucleo::instrucciones::marcas;
use nucleo::instrucciones::migracion::{
    self, Archivado, ArchivoAnterior, TextoPropio, TextosDeLaAplicacion,
};
use nucleo::instrucciones::sustitucion::{self, Sustitucion};
use nucleo::instrucciones::{formato, huella};
use nucleo::plantillas;
use nucleo::proyecto::catalogo;

const REGLA_DE_COMPILACION: &str = "Se compila siempre en la VM de Linux con la caché propia.";
const REGLA_SOLO_GEMINI: &str = "Gemini 1 no toca la carpeta de datos.";

fn region(id: &str, contenido: &str) -> String {
    format!("{}\n{contenido}\n{}", marcas::inicio(id), marcas::fin(id))
}

fn conocidos() -> TextosDeLaAplicacion {
    catalogo::textos_de_la_aplicacion(None)
}

fn recoger(archivos: &[(&str, &str)]) -> TextoPropio {
    let anteriores: Vec<ArchivoAnterior<'_>> = archivos
        .iter()
        .map(|(ruta_relativa, texto)| ArchivoAnterior {
            ruta_relativa,
            texto,
        })
        .collect();
    migracion::recoger(&anteriores, &conocidos())
}

/// Archivo como lo dejaba la 2.3.2: regiones gestionadas arriba, zona del proyecto debajo y lo
/// que el proyecto escribió en ella.
fn archivo_de_la_2_3_2(propio: &str) -> String {
    let zona = plantillas::textos_anteriores_de_la_aplicacion()
        .into_iter()
        .find(|t| {
            t.starts_with("Instrucciones propias de este proyecto") && t.contains("Gestor Claude")
        })
        .unwrap_or_default();
    format!(
        "{}\n\n{}\n\n{}\n\n{zona}\n\n{propio}\n",
        region(marcas::CABECERA, "Cabecera de la 2.3.2"),
        region(
            marcas::NUCLEO_COMUN,
            "Núcleo de la 2.3.2\n\nCon dos párrafos."
        ),
        region(marcas::ENTORNO, "Normas de la carpeta proyecto copiadas"),
    )
}

// Recogida del texto propio.

#[test]
fn de_un_archivo_de_la_2_3_2_solo_sale_lo_escrito_por_el_proyecto() {
    let claude = archivo_de_la_2_3_2(REGLA_DE_COMPILACION);

    let propio = recoger(&[("CLAUDE.md", &claude)]);

    assert_eq!(propio.parrafos, vec![REGLA_DE_COMPILACION]);
    assert!(propio.archivos_con_marcas_danadas.is_empty());
}

#[test]
fn la_misma_regla_en_los_tres_archivos_sale_una_sola_vez_y_no_se_pierde_ninguna() {
    let claude = archivo_de_la_2_3_2(REGLA_DE_COMPILACION);
    let agents = archivo_de_la_2_3_2(&format!("{REGLA_DE_COMPILACION}  \r\n"));
    let gemini = archivo_de_la_2_3_2(&format!("{REGLA_DE_COMPILACION}\n\n{REGLA_SOLO_GEMINI}"));

    let propio = recoger(&[
        ("CLAUDE.md", &claude),
        ("AGENTS.md", &agents),
        ("GEMINI.md", &gemini),
    ]);

    assert_eq!(
        propio.parrafos,
        vec![REGLA_DE_COMPILACION, REGLA_SOLO_GEMINI]
    );
}

#[test]
fn un_archivo_sin_marcas_escrito_a_mano_pasa_entero_por_parrafos() {
    let a_mano = "Instrucciones de Gestión de Proyecto\n\nRegla propia de Delta: archivar canales.\nSegunda línea de la misma regla.\n\n\nOtra regla.\n";

    let propio = recoger(&[("CLAUDE.md", a_mano)]);

    assert_eq!(
        propio.parrafos,
        vec![
            "Instrucciones de Gestión de Proyecto",
            "Regla propia de Delta: archivar canales.\nSegunda línea de la misma regla.",
            "Otra regla.",
        ]
    );
}

#[test]
fn con_marcas_danadas_se_conserva_el_texto_entero_y_se_anota_el_archivo() {
    let danado = format!(
        "{}\nnúcleo sin cierre\n\n{REGLA_DE_COMPILACION}\n",
        marcas::inicio(marcas::NUCLEO_COMUN)
    );

    let propio = recoger(&[("AGENTS.md", &danado)]);

    assert_eq!(propio.archivos_con_marcas_danadas, vec!["AGENTS.md"]);
    assert!(propio
        .parrafos
        .iter()
        .any(|p| p.contains("núcleo sin cierre")));
    assert!(propio.parrafos.iter().any(|p| p == REGLA_DE_COMPILACION));
    for parrafo in &propio.parrafos {
        assert!(!marcas::contiene_marcas(parrafo), "{parrafo}");
    }
}

#[test]
fn una_plantilla_completa_de_la_1x_o_un_archivo_vacio_no_aportan_nada() {
    let plantillas_1x = plantillas::plantillas_antiguas();
    assert!(!plantillas_1x.is_empty());

    for plantilla in &plantillas_1x {
        let con_crlf = plantilla.replace('\n', "\r\n");
        let propio = recoger(&[("CLAUDE.md", &con_crlf), ("AGENTS.md", " \n\n")]);
        assert!(propio.esta_vacio(), "{:?}", propio.parrafos);
    }
}

#[test]
fn la_plantilla_personalizada_que_guardo_el_usuario_tampoco_aporta_nada() {
    let guardada = "Plantilla guardada en AppData.\n\nCon una regla general.\n";
    let conocidos = catalogo::textos_de_la_aplicacion(Some(guardada));
    let anteriores = [ArchivoAnterior {
        ruta_relativa: "CLAUDE.md",
        texto: &guardada.replace('\n', "\r\n"),
    }];

    assert!(migracion::recoger(&anteriores, &conocidos).esta_vacio());
}

#[test]
fn los_bloques_copiados_de_la_carpeta_proyecto_y_los_legados_ya_vistos_no_se_repiten() {
    let encabezado_viejo = plantillas::textos_anteriores_de_la_aplicacion()
        .into_iter()
        .find(|t| t.starts_with("Instrucciones anteriores pendientes de revisión"))
        .unwrap_or_default()
        .replace("{{FECHA}}", "2026-10-26")
        .replace("{{RESPALDO}}", ".gestor/respaldos/CLAUDE.md.bak");
    let texto = format!(
        "{}\n\n{}\n",
        region(
            marcas::UNIDAD_DE_TRABAJO,
            "La unidad de trabajo es la de los proyectos."
        ),
        region(
            marcas::LEGADO,
            &format!("{encabezado_viejo}\n\n{REGLA_SOLO_GEMINI}")
        ),
    );

    let propio = recoger(&[("CLAUDE.md", &texto)]);

    assert_eq!(propio.parrafos, vec![REGLA_SOLO_GEMINI]);
}

#[test]
fn ningun_parrafo_propio_se_pierde_aunque_se_parezca_a_un_texto_de_la_aplicacion() {
    // Empieza igual que el título de la zona del proyecto pero sigue con texto propio en el mismo
    // párrafo, así que no es el título y tiene que llegar.
    let parecido = "Instrucciones propias de este proyecto: compilar en la VM.";
    let claude = archivo_de_la_2_3_2(parecido);

    let propio = recoger(&[("CLAUDE.md", &claude)]);

    assert_eq!(propio.parrafos, vec![parecido]);
}

// Composición del archivo propio.

#[test]
fn sin_nada_que_conservar_el_archivo_propio_es_la_plantilla_inicial() {
    let texto = migracion::componer_instrucciones_del_proyecto(
        &plantillas::instrucciones_del_proyecto(),
        "encabezado",
        "aviso",
        &TextoPropio::default(),
        &[],
    );

    assert_eq!(texto, plantillas::instrucciones_del_proyecto() + "\n");
}

#[test]
fn el_bloque_pendiente_lleva_lo_recogido_y_la_lista_de_lo_archivado() {
    let propio = TextoPropio {
        parrafos: vec![REGLA_DE_COMPILACION.to_owned()],
        archivos_con_marcas_danadas: Vec::new(),
    };
    let archivados = [Archivado {
        origen: String::from("GEMINI.md"),
        destino: String::from(".gestor/archivo/sistema-anterior/GEMINI.md"),
    }];

    let texto = migracion::componer_instrucciones_del_proyecto(
        "Inicial",
        "Encabezado pendiente",
        "Aviso de archivados:",
        &propio,
        &archivados,
    );

    assert_eq!(
        texto,
        format!(
            "Inicial\n\n{}\nEncabezado pendiente\n\n{REGLA_DE_COMPILACION}\n\nAviso de archivados:\n\n- GEMINI.md, ahora en .gestor/archivo/sistema-anterior/GEMINI.md\n{}\n",
            marcas::inicio(marcas::LEGADO),
            marcas::fin(marcas::LEGADO)
        )
    );
}

#[test]
fn solo_con_archivados_tambien_hay_bloque_pendiente_con_su_lista() {
    let archivados = [Archivado {
        origen: String::from(".gestor/memoria-compartida.md"),
        destino: String::from(".gestor/archivo/sistema-anterior/memoria-compartida.md"),
    }];

    let texto = migracion::componer_instrucciones_del_proyecto(
        "Inicial",
        "Encabezado",
        "Aviso:",
        &TextoPropio::default(),
        &archivados,
    );

    assert!(texto.contains(&marcas::inicio(marcas::LEGADO)));
    assert!(texto.contains(
        "- .gestor/memoria-compartida.md, ahora en .gestor/archivo/sistema-anterior/memoria-compartida.md"
    ));
}

// Sustitución de los archivos que la aplicación escribe enteros.

#[test]
fn sustitucion_crea_lo_que_falta_y_no_toca_lo_que_esta_al_dia() {
    let vigente = "Texto vigente\n";

    assert_eq!(
        sustitucion::decidir(None, vigente, None),
        Sustitucion::Crear
    );
    assert_eq!(
        sustitucion::decidir(Some(vigente), vigente, None),
        Sustitucion::SinCambios
    );
}

#[test]
fn sustitucion_sin_respaldo_solo_si_lo_existente_es_lo_ultimo_que_escribio_la_aplicacion() {
    let anterior = "Texto que escribió la aplicación\n";
    let vigente = "Texto vigente\n";
    let escrito = huella::calcular(&formato::normalizar(anterior));

    assert_eq!(
        sustitucion::decidir(Some(anterior), vigente, Some(escrito.as_str())),
        Sustitucion::Sustituir { respaldar: false }
    );
    assert_eq!(
        sustitucion::decidir(Some("Editado a mano\n"), vigente, Some(escrito.as_str())),
        Sustitucion::Sustituir { respaldar: true }
    );
    assert_eq!(
        sustitucion::decidir(Some(anterior), vigente, None),
        Sustitucion::Sustituir { respaldar: true }
    );
}
