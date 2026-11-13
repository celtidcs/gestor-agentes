//! Barreras de la preparación de proyectos: inicializador, localizador del entorno, servicio de
//! git, registro de huellas, informe y momento. Traducen
//! las del localizador y del servicio de git de la 2.3.2 y añaden las del formato exacto del
//! registro y del informe, que la paridad con la versión anterior exige.
//! Las del inicializador fijan desde la 3.0.0 el archivo único de reglas: AGENTS.md
//! y CLAUDE.md se escriben enteros con copia de lo ajeno, el archivo propio del proyecto se crea
//! una vez con lo recogido de los archivos anteriores y no se vuelve a tocar, y lo del sistema
//! anterior se archiva sin borrarse.

mod comun;

use comun::{CarpetaTemporal, EjecutorFalso, Entorno};
use nucleo::agentes::{IdentidadAgente, TipoAgente};
use nucleo::git::{self, ServicioGit};
use nucleo::instrucciones::huella::HuellasPorRegion;
use nucleo::instrucciones::{analizador, formato, marcas};
use nucleo::plantillas::{self, DocumentoDeConsulta};
use nucleo::proyecto::archivos;
use nucleo::proyecto::catalogo;
use nucleo::proyecto::informe::{InformeDeInicializacion, TipoDeCambio};
use nucleo::proyecto::inicializador;
use nucleo::proyecto::localizador::{
    FuenteDeEntorno, InstruccionesDelEspacioDeTrabajo, LocalizadorDeInstrucciones,
};
use nucleo::proyecto::momento::Momento;
use nucleo::proyecto::registro_de_huellas::{self, RegistroDeHuellas};
use nucleo::proyecto::sistema_anterior;
use std::path::PathBuf;

const NORMAS_DEL_ESPACIO: &str =
    "Normas del espacio de trabajo\n\nSolo puede haber una VM encendida a la vez.\n";

const NORMAS_CON_BLOQUE_AJENO: &str = "<!-- UNIDAD-DE-TRABAJO:INICIO -->\nLa unidad de trabajo es la de los proyectos.\n<!-- UNIDAD-DE-TRABAJO:FIN -->\n\nSolo puede haber una VM encendida a la vez.\n";

fn claude2() -> Option<IdentidadAgente> {
    IdentidadAgente::new(TipoAgente::Claude, 2).ok()
}

// Inicializador.

/// Límite de AGENTS.md que lee Antigravity, el más bajo de las tres herramientas: 24.000 bytes
/// (Codex lee 32 KiB). Fuente: https://antigravity.google/docs/rules/.
const LIMITE_DE_ANTIGRAVITY: usize = 24_000;

/// Tamaño de unas normas de la carpeta proyecto realistas: las vigentes miden 2.991 bytes y se
/// deja margen.
const TAMANO_DE_NORMAS_REALISTAS: usize = 3_500;

/// Archivos que la aplicación escribe enteros en cada lanzamiento.
const ARCHIVOS_ENTEROS: [&str; 4] = [
    "AGENTS.md",
    "CLAUDE.md",
    ".gestor/consulta/codigo.md",
    ".gestor/consulta/metodo.md",
];

const PROPIO: &str = ".gestor/instrucciones-del-proyecto.md";

const REGLA_PROPIA: &str = "Regla propia de Delta: archivar canales.";

/// Contenido de una región en un texto con marcas, o nada si no está o las marcas no valen.
fn region_de(texto: &str, id: &str) -> Option<String> {
    let ids = [marcas::CABECERA, marcas::NUCLEO_COMUN, marcas::ENTORNO];
    analizador::analizar(texto, &ids)
        .ok()
        .and_then(|segmentos| {
            segmentos.into_iter().find_map(|s| match s {
                analizador::Segmento::Region(r) if r.id == id => Some(r.contenido()),
                _ => None,
            })
        })
}

fn hay_cambio(informe: &InformeDeInicializacion, archivo: &str, tipo: TipoDeCambio) -> bool {
    informe
        .cambios()
        .iter()
        .any(|c| c.archivo == archivo && c.tipo == tipo)
}

#[test]
fn proyecto_vacio_recibe_el_archivo_unico_y_lo_demas_canonico() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();

    let informe = entorno.inicializar(carpeta.ruta(), claude2(), None);

    assert!(informe.errores().is_empty(), "{}", informe.componer_texto());
    for relativa in ARCHIVOS_ENTEROS.into_iter().chain([
        PROPIO,
        ".gitignore",
        ".gestor/canal/estado.md",
        ".gestor/canal/claude.md",
        ".gestor/canal/claude2.md",
        ".gestor/canal/codex.md",
        ".gestor/canal/gemini.md",
        ".gestor/claude2/tarea-en-curso.md",
        ".gestor/claude2/verificaciones.md",
        ".gestor/claude2/funciones.md",
        ".gestor/gemini/funciones.md",
        ".gestor/ordenes-del-director.md",
        registro_de_huellas::RUTA_RELATIVA,
    ]) {
        assert!(carpeta.existe(relativa), "Falta {relativa}");
    }
    for ausente in ["GEMINI.md", ".gestor/memoria-compartida.md"] {
        assert!(!carpeta.existe(ausente), "Sobra {ausente}");
    }
    assert!(carpeta
        .leer(".gestor/canal/claude2.md")
        .contains("Bandeja de Claude 2"));
}

#[test]
fn los_archivos_enteros_tienen_exactamente_el_texto_vigente() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();

    let _ = entorno.inicializar(carpeta.ruta(), None, None);

    assert_eq!(carpeta.leer("AGENTS.md"), catalogo::reglas(None));
    assert_eq!(carpeta.leer("CLAUDE.md"), catalogo::puntero_claude());
    for documento in DocumentoDeConsulta::TODOS {
        assert_eq!(
            carpeta.leer(documento.ruta_relativa()),
            documento.texto() + "\n"
        );
    }
    assert_eq!(
        carpeta.leer(PROPIO),
        plantillas::instrucciones_del_proyecto() + "\n",
        "un proyecto nuevo no lleva bloque pendiente"
    );
    for tipo in TipoAgente::TODOS {
        assert!(carpeta.existe(tipo.archivo_instrucciones()), "{tipo:?}");
    }
}

#[test]
fn el_archivo_unico_cabe_en_el_limite_de_antigravity() {
    let normas = InstruccionesDelEspacioDeTrabajo {
        ruta: PathBuf::from("/espacio/CLAUDE.md"),
        texto: "Norma de la carpeta proyecto. ".repeat(TAMANO_DE_NORMAS_REALISTAS / 30),
    };

    let reglas = catalogo::reglas(Some(&normas));

    assert!(
        reglas.len() <= LIMITE_DE_ANTIGRAVITY,
        "AGENTS.md mide {} bytes",
        reglas.len()
    );
}

#[test]
fn segunda_ejecucion_no_cambia_nada() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    *entorno.fuente.texto.borrow_mut() = NORMAS_DEL_ESPACIO.to_owned();
    let _ = entorno.inicializar(carpeta.ruta(), claude2(), None);
    let primera: Vec<Vec<u8>> = ARCHIVOS_ENTEROS
        .into_iter()
        .chain([PROPIO])
        .map(|r| carpeta.leer_bytes(r))
        .collect();

    let segunda = entorno.inicializar(carpeta.ruta(), claude2(), None);

    assert!(!segunda.hay_cambios(), "{}", segunda.componer_texto());
    assert!(!segunda.requiere_atencion(), "{}", segunda.componer_texto());
    let despues: Vec<Vec<u8>> = ARCHIVOS_ENTEROS
        .into_iter()
        .chain([PROPIO])
        .map(|r| carpeta.leer_bytes(r))
        .collect();
    assert_eq!(primera, despues);
}

#[test]
fn el_archivo_propio_no_se_toca_nunca_una_vez_creado() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let mut propio = vec![0xEF, 0xBB, 0xBF];
    propio.extend_from_slice(b"Reglas del proyecto\r\n\r\nSe compila con cargo.\r\n");
    carpeta.escribir_bytes(PROPIO, &propio);
    carpeta.escribir("CLAUDE.md", &format!("Algo a mano\n\n{REGLA_PROPIA}\n"));
    carpeta.escribir("GEMINI.md", "Reglas de Gemini\n");

    for normas in ["Normas nuevas", "Normas más nuevas"] {
        *entorno.fuente.texto.borrow_mut() = normas.to_owned();
        let informe = entorno.inicializar(carpeta.ruta(), None, None);
        assert!(!informe.cambios().iter().any(|c| c.archivo == PROPIO));
    }

    assert_eq!(carpeta.leer_bytes(PROPIO), propio);
}

#[test]
fn un_cambio_del_entorno_actualiza_agents_md_sin_respaldo() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    *entorno.fuente.texto.borrow_mut() = String::from("Máquinas de prueba en TrueNAS");

    let informe = entorno.inicializar(carpeta.ruta(), None, None);

    assert!(carpeta
        .leer("AGENTS.md")
        .contains("Máquinas de prueba en TrueNAS"));
    assert!(hay_cambio(&informe, "AGENTS.md", TipoDeCambio::Actualizado));
    assert!(!informe
        .cambios()
        .iter()
        .any(|c| c.tipo == TipoDeCambio::Respaldado));
}

#[test]
fn una_edicion_a_mano_de_agents_md_se_respalda_exacta_y_se_sustituye() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let editado = format!("{}\n{REGLA_PROPIA}\n", carpeta.leer("AGENTS.md"));
    carpeta.escribir("AGENTS.md", &editado);

    let informe = entorno.inicializar(carpeta.ruta(), None, None);

    let respaldo = archivos::ruta_de_respaldo("AGENTS.md", comun::momento_fijo());
    assert_eq!(carpeta.leer(&respaldo), editado);
    assert_eq!(carpeta.leer("AGENTS.md"), catalogo::reglas(None));
    assert!(hay_cambio(&informe, "AGENTS.md", TipoDeCambio::Respaldado));
    assert!(!carpeta.leer(PROPIO).contains(REGLA_PROPIA));
}

#[test]
fn archivo_con_bom_y_crlf_conserva_su_formato_al_actualizarse() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let con_crlf = carpeta.leer("AGENTS.md").replace('\n', "\r\n");
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(con_crlf.as_bytes());
    carpeta.escribir_bytes("AGENTS.md", &bytes);
    *entorno.fuente.texto.borrow_mut() = String::from("Entorno nuevo");

    let informe = entorno.inicializar(carpeta.ruta(), None, None);

    let bytes = carpeta.leer_bytes("AGENTS.md");
    assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
    let texto = String::from_utf8_lossy(&bytes);
    assert!(texto.contains("Entorno nuevo\r\n"));
    assert!(!texto.contains("\r\r"));
    assert_eq!(texto.matches('\n').count(), texto.matches("\r\n").count());
    assert!(!hay_cambio(&informe, "AGENTS.md", TipoDeCambio::Respaldado));
}

#[test]
fn proyecto_con_tres_archivos_anteriores_pasa_lo_propio_al_archivo_del_proyecto() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let claude = format!("Instrucciones de Gestión de Proyecto\n\n{REGLA_PROPIA}\n");
    let agents = format!("{REGLA_PROPIA}\r\n\r\nCodex no toca la interfaz.\r\n");
    let gemini = "Gemini 1 no toca la carpeta de datos.\n";
    carpeta.escribir("CLAUDE.md", &claude);
    carpeta.escribir("AGENTS.md", &agents);
    carpeta.escribir("GEMINI.md", gemini);

    let informe = entorno.inicializar(carpeta.ruta(), None, None);

    let propio = carpeta.leer(PROPIO);
    assert!(propio.starts_with(&plantillas::instrucciones_del_proyecto()));
    assert!(propio.contains(&marcas::inicio(marcas::LEGADO)));
    assert!(propio.contains("2026-10-26"));
    assert_eq!(propio.matches(REGLA_PROPIA).count(), 1, "{propio}");
    for parrafo in [
        "Instrucciones de Gestión de Proyecto",
        "Codex no toca la interfaz.",
        "Gemini 1 no toca la carpeta de datos.",
    ] {
        assert!(propio.contains(parrafo), "Falta {parrafo}");
    }
    assert!(propio.contains("- GEMINI.md, ahora en .gestor/archivo/sistema-anterior/GEMINI.md"));

    for (relativa, original) in [
        ("CLAUDE.md", claude.as_str()),
        ("AGENTS.md", agents.as_str()),
    ] {
        let respaldo = archivos::ruta_de_respaldo(relativa, comun::momento_fijo());
        assert_eq!(
            carpeta.leer_bytes(&respaldo),
            original.as_bytes(),
            "{relativa}"
        );
    }
    assert!(!carpeta.existe("GEMINI.md"));
    assert_eq!(
        carpeta.leer(&format!(
            "{}/GEMINI.md",
            sistema_anterior::CARPETA_ARCHIVO_SISTEMA_ANTERIOR
        )),
        gemini
    );
    assert_eq!(
        formato::normalizar(&carpeta.leer("AGENTS.md")),
        catalogo::reglas(None)
    );
    assert!(informe.requiere_atencion());
}

#[test]
fn claude_md_igual_a_la_plantilla_personalizada_antigua_no_deja_bloque_pendiente() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let guardada = "# Instrucciones de Gestión de Proyecto\n\nPlantilla guardada en AppData.\n";
    *entorno.configuracion.instrucciones_antiguas.borrow_mut() = Some(guardada.to_owned());
    carpeta.escribir("CLAUDE.md", &guardada.replace('\n', "\r\n"));

    let _ = entorno.inicializar(carpeta.ruta(), None, None);

    assert_eq!(
        carpeta.leer(PROPIO),
        plantillas::instrucciones_del_proyecto() + "\n"
    );
    assert_eq!(
        formato::normalizar(&carpeta.leer("CLAUDE.md")),
        catalogo::puntero_claude()
    );
}

#[test]
fn marcas_danadas_pasan_el_texto_entero_al_bloque_pendiente_y_se_avisa() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let danado = format!(
        "{}\nnúcleo sin cierre\n\n{REGLA_PROPIA}\n",
        marcas::inicio(marcas::NUCLEO_COMUN)
    );
    carpeta.escribir("AGENTS.md", &danado);

    let informe = entorno.inicializar(carpeta.ruta(), None, None);

    let propio = carpeta.leer(PROPIO);
    assert!(propio.contains("núcleo sin cierre"));
    assert!(propio.contains(REGLA_PROPIA));
    assert!(informe
        .avisos()
        .iter()
        .any(|a| a.starts_with("AGENTS.md tenía las marcas de región dañadas")));
    let respaldo = archivos::ruta_de_respaldo("AGENTS.md", comun::momento_fijo());
    assert_eq!(carpeta.leer(&respaldo), danado);
    assert_eq!(region_de(&propio, marcas::NUCLEO_COMUN), None);
}

#[test]
fn canal_gemini_antiguo_se_archiva_sin_borrarse() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    carpeta.escribir(".gestor/gemini/canal.md", "Canal Claude ↔ Gemini");

    let _ = entorno.inicializar(carpeta.ruta(), None, None);

    assert!(!carpeta.existe(".gestor/gemini/canal.md"));
    assert_eq!(
        carpeta.leer(&format!(
            "{}/canal.md",
            sistema_anterior::CARPETA_ARCHIVO_GEMINI
        )),
        "Canal Claude ↔ Gemini"
    );
    assert!(carpeta.existe(".gestor/gemini/tarea-en-curso.md"));
}

#[test]
fn canal_gemini_antiguo_ya_archivado_se_guarda_con_marca_de_tiempo() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let archivado = format!("{}/canal.md", sistema_anterior::CARPETA_ARCHIVO_GEMINI);
    carpeta.escribir(&archivado, "primero");
    carpeta.escribir(".gestor/gemini/canal.md", "segundo");

    let _ = entorno.inicializar(carpeta.ruta(), None, None);

    assert_eq!(carpeta.leer(&archivado), "primero");
    assert_eq!(
        carpeta.leer(&format!("{archivado}.20261026-153000")),
        "segundo"
    );
}

#[test]
fn memoria_compartida_con_tarea_viva_se_archiva_y_el_archivo_propio_la_cita() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let tarea = "Tarea en curso: sí\nPróximo paso exacto: decidir la vía.\n";
    carpeta.escribir(".gestor/memoria-compartida.md", tarea);

    let _ = entorno.inicializar(carpeta.ruta(), None, None);

    let destino = format!(
        "{}/memoria-compartida.md",
        sistema_anterior::CARPETA_ARCHIVO_SISTEMA_ANTERIOR
    );
    assert!(!carpeta.existe(".gestor/memoria-compartida.md"));
    assert_eq!(carpeta.leer(&destino), tarea);
    assert!(carpeta.leer(PROPIO).contains(&format!(
        "- .gestor/memoria-compartida.md, ahora en {destino}"
    )));
}

#[test]
fn lo_archivado_con_archivo_propio_existente_se_avisa_sin_tocarlo() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let propio = carpeta.leer_bytes(PROPIO);
    carpeta.escribir("GEMINI.md", "Reglas de Gemini\n");

    let informe = entorno.inicializar(carpeta.ruta(), None, None);

    assert_eq!(carpeta.leer_bytes(PROPIO), propio);
    assert!(informe.avisos().iter().any(|a| a
        .contains("GEMINI.md (ahora en .gestor/archivo/sistema-anterior/GEMINI.md)")
        && a.contains(PROPIO)));
}

#[test]
fn archivo_de_estado_de_un_agente_nunca_se_sobrescribe() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let viva = "Bandeja de Claude 1\n\nLATIDO: 2026-10-26 09:00\nMensaje vivo.\n";
    carpeta.escribir(".gestor/canal/claude.md", viva);

    let _ = entorno.inicializar(carpeta.ruta(), None, None);

    assert_eq!(carpeta.leer(".gestor/canal/claude.md"), viva);
}

#[test]
fn una_bandeja_sin_podar_se_avisa_y_estado_md_no_cuenta() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let _ = entorno.inicializar(carpeta.ruta(), None, None);
    let limite =
        usize::try_from(nucleo::ajustes::LIMITE_DE_BANDEJA_POR_DEFECTO).unwrap_or(usize::MAX);
    let grande = "x".repeat(limite + 1);
    carpeta.escribir(".gestor/canal/codex.md", &grande);
    carpeta.escribir(".gestor/canal/estado.md", &grande);
    carpeta.escribir(".gestor/canal/claude.md", &"x".repeat(limite));

    let informe = entorno.inicializar(carpeta.ruta(), None, None);

    let avisos: Vec<&String> = informe
        .avisos()
        .iter()
        // Solo los avisos de tamaño: desde la 3.2.0, un estado.md sin sus apartados da además los
        // de la revisión del canal, que vigila tests/revision_del_canal.rs.
        .filter(|a| a.starts_with(".gestor/canal/") && a.contains(" mide "))
        .collect();
    assert_eq!(avisos.len(), 1, "{avisos:?}");
    assert!(avisos[0].starts_with(".gestor/canal/codex.md mide"));
}

fn entradas(lista: &[(&str, u64)]) -> Vec<(String, u64)> {
    lista
        .iter()
        .map(|(nombre, tamano)| ((*nombre).to_owned(), *tamano))
        .collect()
}

#[test]
fn n1_una_bandeja_avisa_solo_si_pasa_del_limite() {
    let canal = entradas(&[("igual.md", 100), ("pasa.md", 101), ("menor.md", 99)]);

    assert_eq!(
        inicializador::bandejas_sin_podar(&canal, 100),
        entradas(&[("pasa.md", 101)])
    );
}

#[test]
fn n1_estado_md_y_lo_que_no_es_md_nunca_avisan() {
    let canal = entradas(&[("estado.md", 500), ("notas.txt", 500), ("gemini.md", 500)]);

    assert_eq!(
        inicializador::bandejas_sin_podar(&canal, 100),
        entradas(&[("gemini.md", 500)])
    );
}

#[test]
fn n1_las_bandejas_salen_ordenadas_por_nombre() {
    let canal = entradas(&[("codex.md", 200), ("claude2.md", 300), ("claude.md", 400)]);

    assert_eq!(
        inicializador::bandejas_sin_podar(&canal, 100),
        entradas(&[("claude.md", 400), ("claude2.md", 300), ("codex.md", 200)])
    );
}

#[test]
fn carpeta_inexistente_se_anota_como_error_sin_crear_nada() {
    let carpeta = CarpetaTemporal::new();
    let entorno = Entorno::default();
    let inexistente = carpeta.archivo("no-existe");

    let informe = entorno.inicializar(&inexistente, None, None);

    assert_eq!(informe.errores().len(), 1);
    assert!(informe.errores()[0].starts_with("La carpeta del proyecto no existe: "));
    assert!(!inexistente.exists());
}

// Localizador.

#[test]
fn archivo_de_la_carpeta_superior_es_el_entorno_del_proyecto() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("CLAUDE.md", NORMAS_DEL_ESPACIO);
    let proyecto = carpeta.crear_carpeta("proyecto");

    let encontradas = LocalizadorDeInstrucciones.buscar(&proyecto);

    assert_eq!(
        encontradas,
        Some(InstruccionesDelEspacioDeTrabajo {
            ruta: carpeta.archivo("CLAUDE.md"),
            texto: NORMAS_DEL_ESPACIO.to_owned(),
        })
    );
}

#[test]
fn archivo_del_propio_proyecto_no_cuenta_como_espacio_de_trabajo() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("CLAUDE.md", NORMAS_DEL_ESPACIO);
    carpeta.escribir("proyecto/CLAUDE.md", "Instrucciones del propio proyecto\n");

    let encontradas = LocalizadorDeInstrucciones.buscar(&carpeta.archivo("proyecto"));

    assert_eq!(
        encontradas.map(|e| e.texto).as_deref(),
        Some(NORMAS_DEL_ESPACIO)
    );
}

#[test]
fn archivo_gestionado_de_un_proyecto_padre_se_salta_y_se_sigue_subiendo() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("CLAUDE.md", NORMAS_DEL_ESPACIO);
    carpeta.escribir(
        "padre/CLAUDE.md",
        &format!(
            "{}\nnúcleo\n{}\n",
            marcas::inicio(marcas::NUCLEO_COMUN),
            marcas::fin(marcas::NUCLEO_COMUN)
        ),
    );
    let proyecto = carpeta.crear_carpeta("padre/hijo");

    let encontradas = LocalizadorDeInstrucciones.buscar(&proyecto);

    assert_eq!(
        encontradas.map(|e| e.ruta),
        Some(carpeta.archivo("CLAUDE.md"))
    );
}

#[test]
fn sin_claude_md_se_usa_agents_md_de_la_misma_carpeta() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("AGENTS.md", NORMAS_DEL_ESPACIO);
    let proyecto = carpeta.crear_carpeta("proyecto");

    let encontradas = LocalizadorDeInstrucciones.buscar(&proyecto);

    assert_eq!(
        encontradas.map(|e| e.ruta),
        Some(carpeta.archivo("AGENTS.md"))
    );
}

#[test]
fn con_el_localizador_real_las_normas_llegan_a_agents_md_con_su_origen() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        "CLAUDE.md",
        "Normas comunes\nsin marcas\nfin de las normas\n",
    );
    let proyecto = carpeta.crear_carpeta("proyecto");
    let entorno = Entorno::default();

    let _ = entorno.inicializar(&proyecto, None, Some(&LocalizadorDeInstrucciones));

    let origen = carpeta.archivo("CLAUDE.md");
    let texto = carpeta.leer("proyecto/AGENTS.md");
    assert!(texto.contains("Normas comunes\nsin marcas\nfin de las normas"));
    assert!(texto.contains(&*origen.to_string_lossy()));
    assert!(!carpeta
        .leer("proyecto/CLAUDE.md")
        .contains("Normas comunes"));
}

#[test]
fn bloque_con_marcas_ajenas_no_impide_usar_el_archivo_como_entorno() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("CLAUDE.md", NORMAS_CON_BLOQUE_AJENO);
    let proyecto = carpeta.crear_carpeta("proyecto");

    let encontradas = LocalizadorDeInstrucciones.buscar(&proyecto);

    assert_eq!(
        encontradas.map(|e| e.ruta),
        Some(carpeta.archivo("CLAUDE.md"))
    );
}

#[test]
fn cada_marca_propia_de_la_app_hace_que_el_archivo_se_salte() {
    for id in [
        marcas::CABECERA,
        marcas::NUCLEO_COMUN,
        marcas::ENTORNO,
        marcas::PUNTERO,
        marcas::LEGADO,
    ] {
        let carpeta = CarpetaTemporal::new();
        carpeta.escribir("CLAUDE.md", NORMAS_DEL_ESPACIO);
        carpeta.escribir(
            "padre/CLAUDE.md",
            &format!(
                "texto\n{}\nregión\n{}\n",
                marcas::inicio(id),
                marcas::fin(id)
            ),
        );
        let proyecto = carpeta.crear_carpeta("padre/hijo");

        let encontradas = LocalizadorDeInstrucciones.buscar(&proyecto);

        assert_eq!(
            encontradas.map(|e| e.ruta),
            Some(carpeta.archivo("CLAUDE.md")),
            "{id}"
        );
    }
}

#[test]
fn bloque_con_marcas_ajenas_llega_neutralizado_y_una_segunda_apertura_no_cambia_nada() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("CLAUDE.md", NORMAS_CON_BLOQUE_AJENO);
    let proyecto = carpeta.crear_carpeta("proyecto");
    let entorno = Entorno::default();
    let rutas: Vec<String> = ARCHIVOS_ENTEROS
        .iter()
        .chain([&PROPIO])
        .map(|r| format!("proyecto/{r}"))
        .collect();

    let _ = entorno.inicializar(&proyecto, None, Some(&LocalizadorDeInstrucciones));
    let primera: Vec<String> = rutas.iter().map(|r| carpeta.leer(r)).collect();

    let reglas = &primera[0];
    assert!(reglas.contains("La unidad de trabajo es la de los proyectos."));
    assert!(!reglas.contains(catalogo::ENTORNO_SIN_CONFIGURAR));
    for texto in &primera {
        assert!(!texto
            .split('\n')
            .any(|linea| linea.trim() == "<!-- UNIDAD-DE-TRABAJO:INICIO -->"));
    }

    let informe = entorno.inicializar(&proyecto, None, Some(&LocalizadorDeInstrucciones));

    let segunda: Vec<String> = rutas.iter().map(|r| carpeta.leer(r)).collect();
    assert_eq!(primera, segunda);
    assert!(!informe.hay_cambios(), "{}", informe.componer_texto());
    assert!(!informe.requiere_atencion(), "{}", informe.componer_texto());
}

#[test]
fn marcas_dentro_de_las_normas_se_neutralizan_para_no_romper_el_archivo() {
    let normas = InstruccionesDelEspacioDeTrabajo {
        ruta: PathBuf::from("/espacio/CLAUDE.md"),
        texto: format!("Normas\n{}\nmás normas", marcas::fin(marcas::ENTORNO)),
    };

    let entorno = region_de(&catalogo::reglas(Some(&normas)), marcas::ENTORNO).unwrap_or_default();

    assert!(!marcas::contiene_marcas(&entorno));
    assert!(entorno.contains("más normas"));
}

#[test]
fn sin_entorno_o_con_entorno_en_blanco_se_usa_el_texto_sin_configurar() {
    let en_blanco = InstruccionesDelEspacioDeTrabajo {
        ruta: PathBuf::from("/espacio/CLAUDE.md"),
        texto: String::from("\n \n"),
    };
    for espacio in [None, Some(&en_blanco)] {
        assert_eq!(
            region_de(&catalogo::reglas(espacio), marcas::ENTORNO).as_deref(),
            Some(catalogo::ENTORNO_SIN_CONFIGURAR)
        );
    }
}

// Git.

#[test]
fn proyecto_sin_repositorio_se_inicializa() {
    let ejecutor = EjecutorFalso::default();
    let mut informe = InformeDeInicializacion::new();

    ServicioGit::new(&ejecutor)
        .asegurar_repositorio(std::path::Path::new("/proyectos/demo"), &mut informe);

    assert!(ejecutor
        .llamadas
        .borrow()
        .iter()
        .any(|l| l.iter().any(|a| a == "init")));
    assert!(informe
        .cambios()
        .iter()
        .any(|c| c.tipo == TipoDeCambio::RepositorioCreado && c.archivo == ".git"));
}

#[test]
fn proyecto_con_repositorio_no_se_reinicializa() {
    let ejecutor = EjecutorFalso::default();
    ejecutor.dentro_de_repositorio.set(true);
    let mut informe = InformeDeInicializacion::new();

    ServicioGit::new(&ejecutor)
        .asegurar_repositorio(std::path::Path::new("/proyectos/demo"), &mut informe);

    assert!(!ejecutor
        .llamadas
        .borrow()
        .iter()
        .any(|l| l.iter().any(|a| a == "init")));
    assert!(!informe.hay_cambios());
}

#[test]
fn cada_orden_lleva_la_excepcion_de_seguridad_solo_para_ese_proyecto() {
    let ejecutor = EjecutorFalso::default();

    ServicioGit::new(&ejecutor).asegurar_repositorio(
        std::path::Path::new(r"D:\mis proyectos\Demo"),
        &mut InformeDeInicializacion::new(),
    );

    let llamadas = ejecutor.llamadas.borrow();
    assert_eq!(llamadas.len(), 2);
    for llamada in llamadas.iter() {
        assert_eq!(llamada.first().map(String::as_str), Some("-c"));
        assert_eq!(
            llamada.get(1).map(String::as_str),
            Some("safe.directory=D:/mis proyectos/Demo")
        );
    }
}

#[test]
fn git_no_instalado_se_avisa_sin_romper() {
    let ejecutor = EjecutorFalso::default();
    ejecutor.git_instalado.set(false);
    let mut informe = InformeDeInicializacion::new();

    ServicioGit::new(&ejecutor)
        .asegurar_repositorio(std::path::Path::new("/proyectos/demo"), &mut informe);

    assert_eq!(informe.avisos().len(), 1);
    assert!(informe.errores().is_empty());
}

#[test]
fn gitignore_sin_la_entrada_se_anade() {
    let bloque =
        "# Canal, diarios y actas de los agentes (Gestor Agentes): no se versionan.\n.gestor/\n";
    assert_eq!(git::anadir_entrada_gestor(None).as_deref(), Some(bloque));
    assert_eq!(
        git::anadir_entrada_gestor(Some("")).as_deref(),
        Some(bloque)
    );
    let esperado = format!("bin/\n\n{bloque}");
    assert_eq!(
        git::anadir_entrada_gestor(Some("bin/")),
        Some(esperado.clone())
    );
    assert_eq!(git::anadir_entrada_gestor(Some("bin/\n")), Some(esperado));
}

#[test]
fn gitignore_con_la_entrada_no_se_toca() {
    for actual in [
        ".gestor/\n",
        "bin/\n/.gestor\n",
        "  .gestor  \n",
        ".gestor/**",
    ] {
        assert_eq!(git::anadir_entrada_gestor(Some(actual)), None, "{actual}");
    }
}

// Registro de huellas.

#[test]
fn registro_se_escribe_como_system_text_json_y_se_relee_en_orden() {
    let carpeta = CarpetaTemporal::new();
    let mut registro = RegistroDeHuellas::default();
    let huellas: HuellasPorRegion = [("GESTOR:CABECERA", "aa"), ("NUCLEO-COMUN", "bb")]
        .into_iter()
        .collect();
    registro.registrar("CLAUDE.md", huellas.clone());
    registro.registrar(".gestor/memoria-compartida.md", HuellasPorRegion::new());
    registro.registrar("AGENTS.md", huellas.clone());

    let salto = if cfg!(windows) { "\r\n" } else { "\n" };
    let esperado = [
        "{",
        "  \"CLAUDE.md\": {",
        "    \"GESTOR:CABECERA\": \"aa\",",
        "    \"NUCLEO-COMUN\": \"bb\"",
        "  },",
        "  \".gestor/memoria-compartida.md\": {},",
        "  \"AGENTS.md\": {",
        "    \"GESTOR:CABECERA\": \"aa\",",
        "    \"NUCLEO-COMUN\": \"bb\"",
        "  }",
        "}",
    ]
    .join(salto);
    assert_eq!(registro.componer_json(), esperado);

    let guardado = registro.guardar(carpeta.ruta());
    assert!(guardado.is_ok(), "{guardado:?}");
    assert_eq!(
        carpeta.leer(registro_de_huellas::RUTA_RELATIVA),
        format!("{esperado}\n")
    );
    let mut informe = InformeDeInicializacion::new();
    let releido = RegistroDeHuellas::cargar(carpeta.ruta(), &mut informe);
    assert_eq!(releido, registro);
    assert!(informe.avisos().is_empty());
}

#[test]
fn registro_vacio_se_escribe_como_objeto_vacio() {
    assert_eq!(RegistroDeHuellas::default().componer_json(), "{}");
}

#[test]
fn registro_escapa_como_el_codificador_por_defecto_de_dotnet() {
    let mut registro = RegistroDeHuellas::default();
    registro.registrar(
        "a\"b\\c<d>&'+`ñ\n€😀",
        [("X", "y")].into_iter().collect::<HuellasPorRegion>(),
    );

    let json = registro.componer_json();

    assert!(json.contains(
        "\"a\\\"b\\\\c\\u003Cd\\u003E\\u0026\\u0027\\u002B\\u0060\\u00F1\\n\\u20AC\\uD83D\\uDE00\""
    ));
}

#[test]
fn registro_danado_se_reconstruye_con_aviso() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(registro_de_huellas::RUTA_RELATIVA, "{ esto no es json");
    let mut informe = InformeDeInicializacion::new();

    let registro = RegistroDeHuellas::cargar(carpeta.ruta(), &mut informe);

    assert_eq!(registro, RegistroDeHuellas::default());
    assert_eq!(informe.avisos().len(), 1);
    assert!(informe.avisos()[0].starts_with(registro_de_huellas::RUTA_RELATIVA));
    assert!(
        informe.avisos()[0].contains("se respaldarán antes de sustituirse"),
        "el aviso describe el archivo único, no las regiones de la 2.x"
    );
}

#[test]
fn registro_con_bom_y_null_se_lee_sin_aviso() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(registro_de_huellas::RUTA_RELATIVA, "\u{FEFF}null");
    let mut informe = InformeDeInicializacion::new();

    let registro = RegistroDeHuellas::cargar(carpeta.ruta(), &mut informe);

    assert_eq!(registro, RegistroDeHuellas::default());
    assert!(informe.avisos().is_empty());
}

// Informe y momento.

#[test]
fn informe_se_compone_como_en_la_2_3_2() {
    let mut informe = InformeDeInicializacion::new();
    assert_eq!(informe.componer_texto(), "No ha hecho falta cambiar nada.");

    informe.anotar_error("fallo");
    informe.anotar_aviso("aviso");
    informe.anotar_cambio(
        "CLAUDE.md",
        TipoDeCambio::Migrado,
        Some(String::from("detalle")),
    );
    informe.anotar_cambio(".git", TipoDeCambio::RepositorioCreado, None);

    assert_eq!(
        informe.componer_texto(),
        "Errores\n- fallo\n\nAvisos\n- aviso\n\nCambios\n- CLAUDE.md: migrado al formato por regiones; detalle.\n- .git: repositorio git creado."
    );
    assert!(informe.requiere_atencion());
}

#[test]
fn informe_solo_con_creaciones_no_requiere_atencion() {
    let mut informe = InformeDeInicializacion::new();
    informe.anotar_cambio("CLAUDE.md", TipoDeCambio::Creado, None);
    informe.anotar_cambio("AGENTS.md", TipoDeCambio::Actualizado, None);

    assert!(informe.hay_cambios());
    assert!(!informe.requiere_atencion());
}

#[test]
fn momento_valida_la_fecha_y_formatea_como_la_2_3_2() {
    let momento = Momento::new(2026, 2, 3, 4, 5, 6);
    assert_eq!(
        momento.map(|m| (m.fecha_iso(), m.marca_de_tiempo())),
        Ok((String::from("2026-02-03"), String::from("20260203-040506")))
    );
    assert!(Momento::new(2024, 2, 29, 0, 0, 0).is_ok());
    assert!(Momento::new(2026, 2, 29, 0, 0, 0).is_err());
    assert!(Momento::new(2100, 2, 29, 0, 0, 0).is_err());
    assert!(Momento::new(2000, 2, 29, 0, 0, 0).is_ok());
    assert!(Momento::new(2026, 4, 31, 0, 0, 0).is_err());
    assert!(Momento::new(2026, 1, 1, 24, 0, 0).is_err());
    assert!(Momento::new(2026, 13, 1, 0, 0, 0).is_err());
    assert!(Momento::new(0, 1, 1, 0, 0, 0).is_err());
}
