//! Barreras de lo que la preparación hace desde la 3.6.0: copiar el ejecutable del vigilante y
//! retirar los guiones anteriores, completar los apartados de estado.md, anotar el papel elegido en
//! la ventana, avisar del código que cita los archivos de reglas y borrar el archivo del legado una
//! vez consolidado.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use comun::{momento_fijo, CarpetaTemporal, Entorno};
use nucleo::instrucciones::marcas;
use nucleo::proyecto::informe::TipoDeCambio;
use nucleo::proyecto::papeles::{self, Papel};
use nucleo::proyecto::{citas_de_reglas, vigilante};

const ESTADO: &str = ".gestor/canal/estado.md";

fn hay_cambio(
    informe: &nucleo::proyecto::informe::InformeDeInicializacion,
    archivo: &str,
    tipo: TipoDeCambio,
) -> bool {
    informe
        .cambios()
        .iter()
        .any(|c| c.archivo == archivo && c.tipo == tipo)
}

// ── Vigilante ──────────────────────────────────────────────

#[test]
fn copia_el_ejecutable_del_vigilante_y_solo_lo_reescribe_si_cambia() {
    let origen = CarpetaTemporal::new();
    origen.escribir_bytes("gestor-vigilante", b"version 1");
    let proyecto = CarpetaTemporal::new();
    let entorno = Entorno::default();
    *entorno.configuracion.vigilante.borrow_mut() = Some(origen.archivo("gestor-vigilante"));
    let relativa = vigilante::ruta_relativa();

    let informe = entorno.inicializar(proyecto.ruta(), None, None);
    assert!(
        hay_cambio(&informe, &relativa, TipoDeCambio::Creado),
        "{:?}",
        informe.cambios()
    );
    assert_eq!(proyecto.leer_bytes(&relativa), b"version 1");

    let informe = entorno.inicializar(proyecto.ruta(), None, None);
    assert!(!informe.cambios().iter().any(|c| c.archivo == relativa));

    origen.escribir_bytes("gestor-vigilante", b"version 2");
    let informe = entorno.inicializar(proyecto.ruta(), None, None);
    assert!(hay_cambio(&informe, &relativa, TipoDeCambio::Actualizado));
    assert_eq!(proyecto.leer_bytes(&relativa), b"version 2");
}

#[test]
fn sin_el_ejecutable_junto_a_la_aplicacion_avisa_y_no_copia() {
    let proyecto = CarpetaTemporal::new();
    let entorno = Entorno::default();
    *entorno.configuracion.vigilante.borrow_mut() = Some(proyecto.archivo("no-existe"));

    let informe = entorno.inicializar(proyecto.ruta(), None, None);
    assert!(
        informe
            .avisos()
            .iter()
            .any(|a| a.contains("gestor-vigilante")),
        "{:?}",
        informe.avisos()
    );
    assert!(!proyecto.existe(&vigilante::ruta_relativa()));
}

#[test]
fn retira_los_guiones_y_las_referencias_del_vigilante_anterior() {
    let proyecto = CarpetaTemporal::new();
    for guion in vigilante::GUIONES_ANTERIORES {
        proyecto.escribir(guion, "viejo");
    }
    proyecto.escribir(".gestor/claude2/canal-visto.txt", "huellas");
    proyecto.escribir(".gestor/codex/canal-visto/estado.md", "copia");
    proyecto.escribir(".gestor/codex/tarea-en-curso.md", "se queda");

    let informe = Entorno::default().inicializar(proyecto.ruta(), None, None);
    for guion in vigilante::GUIONES_ANTERIORES {
        assert!(!proyecto.existe(guion), "{guion}");
        assert!(
            hay_cambio(&informe, guion, TipoDeCambio::Borrado),
            "{guion}"
        );
    }
    assert!(!proyecto.existe(".gestor/claude2/canal-visto.txt"));
    assert!(!proyecto.archivo(".gestor/codex/canal-visto").exists());
    assert_eq!(proyecto.leer(".gestor/codex/tarea-en-curso.md"), "se queda");
}

// ── estado.md: apartados y papeles ──────────────────────────

#[test]
fn completa_los_apartados_que_faltan_sin_tocar_lo_demas() {
    let proyecto = CarpetaTemporal::new();
    let previo = "# Estado del proyecto\n\n## Reservas activas\n\n- Claude 2, 2026-11-19 10:00: src/ — algo\n";
    proyecto.escribir(ESTADO, previo);

    let informe = Entorno::default().inicializar(proyecto.ruta(), None, None);
    let estado = proyecto.leer(ESTADO);
    assert!(estado.starts_with(previo.trim_end()), "{estado}");
    assert_eq!(estado.matches("\n## Papeles\n").count(), 1);
    assert_eq!(
        estado.matches("\n## Mutaciones vivas\n\nNinguna.").count(),
        1
    );
    assert_eq!(estado.matches("## Reservas activas").count(), 1);
    assert!(hay_cambio(&informe, ESTADO, TipoDeCambio::Actualizado));

    // La segunda vez no hay nada que añadir.
    let informe = Entorno::default().inicializar(proyecto.ruta(), None, None);
    assert!(!informe.cambios().iter().any(|c| c.archivo == ESTADO));
}

#[test]
fn el_papel_se_anota_al_final_de_su_apartado_y_no_se_repite() {
    let estado = "# Estado\n\n## Papeles\n\n- Jefe de proyecto: Claude 2\n\n## Reservas activas\n\nNinguna.\n\n## Mutaciones vivas\n\nNinguna.\n";
    let nuevo =
        papeles::anotar_papel(estado, "Codex 1", Papel::Ayudante, "2026-11-19 20:00:00").unwrap();
    assert!(nuevo.contains(
        "- Jefe de proyecto: Claude 2\n- Codex 1, 2026-11-19 20:00:00: **ayudante** (elegido por el Director en Gestor Agentes al lanzarlo).\n\n## Reservas activas"
    ), "{nuevo}");
    assert_eq!(
        papeles::papel_anotado_por_gestor(&nuevo, "Codex 1"),
        Some(Papel::Ayudante)
    );
    assert_eq!(
        papeles::anotar_papel(&nuevo, "Codex 1", Papel::Ayudante, "2026-11-19 21:00:00"),
        None
    );

    let jefe =
        papeles::anotar_papel(&nuevo, "Codex 1", Papel::Jefe, "2026-11-19 21:00:00").unwrap();
    assert!(jefe.contains("**Jefe de proyecto**"));
    assert!(jefe.contains("Sustituye a cualquier Jefe anotado antes."));
    assert_eq!(
        papeles::papel_anotado_por_gestor(&jefe, "Codex 1"),
        Some(Papel::Jefe)
    );
    // No confunde a Codex 1 con Codex 12.
    assert_eq!(papeles::papel_anotado_por_gestor(&jefe, "Codex 12"), None);
}

#[test]
fn volver_a_elegir_jefe_tras_un_relevo_se_anota() {
    // Caso E2 de la verificación de Codex 1: Claude 1 Jefe, Claude 2 Jefe y otra vez Claude 1.
    let estado = "# Estado\n\n## Papeles\n\n## Reservas activas\n\nNinguna.\n";
    let uno =
        papeles::anotar_papel(estado, "Claude 1", Papel::Jefe, "2026-11-20 08:00:00").unwrap();
    let dos = papeles::anotar_papel(&uno, "Claude 2", Papel::Jefe, "2026-11-20 08:10:00").unwrap();
    let tres = papeles::anotar_papel(&dos, "Claude 1", Papel::Jefe, "2026-11-20 08:20:00")
        .expect("la tercera elección se pierde");
    let ultima_de_papeles = tres
        .lines()
        .rfind(|linea| linea.contains("**Jefe de proyecto**"))
        .unwrap();
    assert!(
        ultima_de_papeles.starts_with("- Claude 1, 2026-11-20 08:20:00"),
        "{tres}"
    );
    // Sin relevo en medio no se repite.
    assert_eq!(
        papeles::anotar_papel(&tres, "Claude 1", Papel::Jefe, "2026-11-20 08:30:00"),
        None
    );
}

#[test]
fn una_reserva_del_agente_en_otro_apartado_no_releva_su_papel() {
    // Caso E2 de la segunda verificación de Codex 1.
    let estado = "## Papeles\n\n## Reservas activas\n- Codex 1, 2026-11-20 08:05:00: crates/ (verificar)\n\n## Mutaciones vivas\n\nNinguna.\n";
    let primera =
        papeles::anotar_papel(estado, "Codex 1", Papel::Ayudante, "2026-11-20 08:00:00").unwrap();
    assert_eq!(
        papeles::anotar_papel(&primera, "Codex 1", Papel::Ayudante, "2026-11-20 08:10:00"),
        None,
        "{primera}"
    );
    // Tampoco una línea de otro apartado que nombre un Jefe releva al Jefe.
    let jefe =
        papeles::anotar_papel(estado, "Claude 1", Papel::Jefe, "2026-11-20 08:00:00").unwrap();
    let con_nota = jefe.replace(
        "## Mutaciones vivas\n",
        "## Mutaciones vivas\n- Claude 2, 2026-11-20 08:05:00: Jefe de proyecto revisa M1.\n",
    );
    assert_eq!(
        papeles::anotar_papel(&con_nota, "Claude 1", Papel::Jefe, "2026-11-20 08:10:00"),
        None,
        "{con_nota}"
    );
}

#[test]
fn una_linea_posterior_del_propio_agente_deja_sin_vigor_la_de_gestor() {
    // El agente anota su papel a su manera después de la línea de Gestor: la suya es la más
    // reciente, así que volver a elegir el papel de Gestor se anota otra vez.
    let estado = "# Estado\n\n## Papeles\n\n## Reservas activas\n\nNinguna.\n";
    let gestor =
        papeles::anotar_papel(estado, "Codex 1", Papel::Ayudante, "2026-11-20 08:00:00").unwrap();
    let propia = gestor.replace(
        "## Reservas activas",
        "- Codex 1, 2026-11-20 08:05:00: Jefe de proyecto por orden del Director.\n\n## Reservas activas",
    );
    assert!(
        papeles::anotar_papel(&propia, "Codex 1", Papel::Ayudante, "2026-11-20 08:10:00").is_some()
    );
}

#[test]
fn el_papel_crea_su_apartado_si_falta() {
    let nuevo = papeles::anotar_papel(
        "# Estado\n",
        "Gemini 1",
        Papel::Ayudante,
        "2026-11-19 20:00:00",
    )
    .unwrap();
    let papeles_ =
        &nuevo[nuevo.find("## Papeles").unwrap()..nuevo.find("## Reservas activas").unwrap()];
    assert!(
        papeles_.contains("- Gemini 1, 2026-11-19 20:00:00: **ayudante**"),
        "{nuevo}"
    );
}

#[test]
fn el_papel_se_anota_en_disco() {
    let proyecto = CarpetaTemporal::new();
    Entorno::default().inicializar(proyecto.ruta(), None, None);
    assert!(
        papeles::anotar_en_disco(proyecto.ruta(), "Claude 3", Papel::Ayudante, momento_fijo())
            .unwrap()
    );
    assert!(proyecto
        .leer(ESTADO)
        .contains("- Claude 3, 2026-10-26 15:30:00: **ayudante**"));
    assert!(!papeles::anotar_en_disco(
        proyecto.ruta(),
        "Claude 3",
        Papel::Ayudante,
        momento_fijo()
    )
    .unwrap());
}

// ── Código que cita los archivos de reglas ─────────────────

#[test]
fn avisa_del_codigo_que_cita_los_archivos_de_reglas() {
    let proyecto = CarpetaTemporal::new();
    proyecto.escribir(
        "src/prueba.rs",
        "let t = std::fs::read_to_string(\"AGENTS.md\");\n",
    );
    proyecto.escribir("docs/guia.md", "Lee AGENTS.md.\n");
    proyecto.escribir("target/debug/generado.rs", "CLAUDE.md\n");
    proyecto.escribir(".oculta/x.py", "GEMINI.md\n");

    assert_eq!(
        citas_de_reglas::buscar(proyecto.ruta()).unwrap(),
        ["src/prueba.rs"]
    );
    let informe = Entorno::default().inicializar(proyecto.ruta(), None, None);
    assert!(
        informe.avisos().iter().any(|a| a.contains("src/prueba.rs")),
        "{:?}",
        informe.avisos()
    );
}

#[test]
fn sin_citas_no_hay_aviso() {
    let proyecto = CarpetaTemporal::new();
    proyecto.escribir("src/main.rs", "fn main() {}\n");
    assert!(citas_de_reglas::buscar(proyecto.ruta()).unwrap().is_empty());
}

// ── Legado consolidado ─────────────────────────────────────

#[test]
fn borra_el_archivo_del_legado_solo_cuando_el_bloque_que_dejo_se_ha_consolidado() {
    let proyecto = CarpetaTemporal::new();
    proyecto.escribir(".gestor/memoria-compartida.md", "tarea antigua\n");
    let entorno = Entorno::default();

    entorno.inicializar(proyecto.ruta(), None, None);
    let propio = proyecto.leer(".gestor/instrucciones-del-proyecto.md");
    assert!(propio.contains(&marcas::inicio(marcas::LEGADO)), "{propio}");
    assert!(proyecto.existe(".gestor/archivo/sistema-anterior/memoria-compartida.md"));

    // Con el bloque todavía pendiente no se borra nada.
    entorno.inicializar(proyecto.ruta(), None, None);
    assert!(proyecto.existe(".gestor/archivo/sistema-anterior/memoria-compartida.md"));

    // Un agente consolida: borra el bloque con sus marcas.
    let inicio = propio.find(&marcas::inicio(marcas::LEGADO)).unwrap();
    let fin_marca = marcas::fin(marcas::LEGADO);
    let fin = propio.find(&fin_marca).unwrap() + fin_marca.len();
    proyecto.escribir(
        ".gestor/instrucciones-del-proyecto.md",
        &format!("{}{}", &propio[..inicio], &propio[fin..]),
    );
    let informe = entorno.inicializar(proyecto.ruta(), None, None);
    assert!(!proyecto
        .archivo(".gestor/archivo/sistema-anterior")
        .exists());
    assert!(hay_cambio(
        &informe,
        ".gestor/archivo/sistema-anterior",
        TipoDeCambio::Borrado
    ));
}

#[test]
fn no_borra_un_archivo_del_legado_que_no_anoto() {
    let proyecto = CarpetaTemporal::new();
    proyecto.escribir(
        ".gestor/instrucciones-del-proyecto.md",
        "Instrucciones propias.\n",
    );
    proyecto.escribir(
        ".gestor/archivo/sistema-anterior/viejo.md",
        "de otra versión\n",
    );

    Entorno::default().inicializar(proyecto.ruta(), None, None);
    assert!(proyecto.existe(".gestor/archivo/sistema-anterior/viejo.md"));
}
