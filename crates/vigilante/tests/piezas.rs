//! Barreras de las piezas puras del vigilante: opciones, entrega a Codex y lectura de su salida.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::path::Path;

use vigilante::entrega::{self, ResultadoDeEntrega};
use vigilante::hilo;
use vigilante::opciones::{self, Modo};
use vigilante::procesos::{self, DatosDeProceso};

fn texto(lista: &[&str]) -> Vec<String> {
    lista.iter().map(|a| (*a).to_owned()).collect()
}

#[test]
fn la_entrega_usa_queue_con_la_sesion_el_mensaje_resume_y_la_raiz() {
    assert_eq!(
        entrega::argumentos("hilo-1", "aviso", Path::new("/proyectos/Demo")),
        texto(&[
            "queue",
            "--thread",
            "hilo-1",
            "--message",
            "aviso",
            "--resume",
            "-C",
            "/proyectos/Demo"
        ])
    );
}

#[test]
fn una_reanudacion_fallida_con_codigo_cero_no_cuenta_como_entregada() {
    // La ayuda de codex queue avisa: «A failed resume produces a warning and the command still
    // succeeds».
    assert_eq!(
        entrega::interpretar_salida(Some(0), "queued\n"),
        ResultadoDeEntrega::Aceptada
    );
    assert!(matches!(
        entrega::interpretar_salida(Some(0), "queued\nwarning: failed to resume thread\n"),
        ResultadoDeEntrega::NoAcreditada(_)
    ));
    assert!(matches!(
        entrega::interpretar_salida(Some(1), ""),
        ResultadoDeEntrega::NoAcreditada(_)
    ));
}

#[test]
fn un_aviso_encolado_sin_reanudar_la_sesion_cuenta_como_encolado() {
    // Salidas literales de codex queue 0.167.1 (.gestor/codex/vigilante/avisos.log, 2026-11-21 y 22).
    for advertencia in [
        "start a local daemon or specify a server with --remote",
        "thread 01a1 already has an active writer",
    ] {
        let salida = format!(
            "Queued message 01a2 for thread 01a1.\n\
             Warning: message was queued, but could not resume thread 01a1: {advertencia}\n"
        );
        assert!(
            matches!(
                entrega::interpretar_salida(Some(0), &salida),
                ResultadoDeEntrega::EncoladaSinReanudar(_)
            ),
            "{salida}"
        );
    }
    // Sin la confirmación de la cola, o con otra advertencia, sigue sin acreditar.
    assert!(matches!(
        entrega::interpretar_salida(
            Some(0),
            "Warning: message was queued, but could not resume thread 01a1: x\n"
        ),
        ResultadoDeEntrega::NoAcreditada(_)
    ));
    assert!(matches!(
        entrega::interpretar_salida(
            Some(0),
            "Queued message 01a2 for thread 01a1.\n\
             Warning: message was queued, but could not resume thread 01a1: x\n\
             error: disk full\n"
        ),
        ResultadoDeEntrega::NoAcreditada(_)
    ));
    // Con un código distinto de 0 tampoco.
    assert!(matches!(
        entrega::interpretar_salida(
            Some(1),
            "Queued message 01a2 for thread 01a1.\n\
             Warning: message was queued, but could not resume thread 01a1: x\n"
        ),
        ResultadoDeEntrega::NoAcreditada(_)
    ));
}

#[test]
fn las_opciones_eligen_un_solo_modo_y_el_persistente_pide_entrega() {
    let hasta = opciones::interpretar(&texto(&["--hasta-cambio", "--agente", "Claude 2"])).unwrap();
    assert_eq!(hasta.modo, Modo::HastaCambio);
    assert_eq!(hasta.agente.as_deref(), Some("Claude 2"));
    assert!(opciones::interpretar(&texto(&["--una-vez", "--hasta-cambio"])).is_err());
    assert!(opciones::interpretar(&texto(&["--persistente"])).is_err());
    assert!(opciones::interpretar(&texto(&["--persistente", "--entrega", "codex"])).is_ok());
    assert!(opciones::interpretar(&texto(&["--desconocida"])).is_err());
}

#[test]
fn anotar_hilo_es_un_modo_y_desde_terminal_solo_vale_con_persistente() {
    let anotar = opciones::interpretar(&texto(&["--anotar-hilo"])).unwrap();
    assert_eq!(anotar.modo, Modo::AnotarHilo);
    assert!(opciones::interpretar(&texto(&["--anotar-hilo", "--una-vez"])).is_err());
    let desde = opciones::interpretar(&texto(&[
        "--persistente",
        "--entrega",
        "codex",
        "--desde-terminal",
        "4242",
    ]))
    .unwrap();
    assert_eq!(desde.desde_terminal, Some(4242));
    assert!(opciones::interpretar(&texto(&["--una-vez", "--desde-terminal", "4242"])).is_err());
    for malo in ["0", "-3", "12a", ""] {
        assert!(
            opciones::interpretar(&texto(&[
                "--persistente",
                "--entrega",
                "codex",
                "--desde-terminal",
                malo
            ]))
            .is_err(),
            "{malo}"
        );
    }
}

#[test]
fn solo_vale_una_sesion_anotada_despues_de_arrancar_el_vigilante() {
    assert_eq!(
        hilo::hilo_valido("01a2-b3\n", 100, 100),
        Some(String::from("01a2-b3"))
    );
    assert_eq!(
        hilo::hilo_valido("01a2-b3", 150, 100),
        Some(String::from("01a2-b3"))
    );
    assert_eq!(hilo::hilo_valido("01a2-b3", 99, 100), None);
    assert_eq!(hilo::hilo_valido("   \n", 150, 100), None);
    assert_eq!(hilo::hilo_valido("dos palabras", 150, 100), None);
}

fn proceso(pid: u32, padre: u32, nombre: &str) -> DatosDeProceso {
    DatosDeProceso {
        pid,
        padre: Some(padre),
        arranque: 1,
        nombre: nombre.to_owned(),
        ejecutable: None,
    }
}

#[test]
fn se_toma_el_codex_mas_cercano_de_la_terminal_y_no_sus_auxiliares() {
    let nombres = ["codex", "codex.exe"];
    let lista = vec![
        proceso(10, 1, "powershell.exe"),
        proceso(20, 10, "conhost.exe"),
        proceso(21, 10, "codex.exe"),
        proceso(30, 21, "codex-code-mode-host.exe"),
        proceso(31, 21, "codex.exe"),
        proceso(40, 1, "codex.exe"),
    ];
    assert_eq!(
        procesos::descendiente_llamado(&lista, 10, &nombres).map(|d| d.pid),
        Some(21)
    );
    // Un codex que no desciende de la terminal no cuenta.
    assert_eq!(
        procesos::descendiente_llamado(&lista[..2], 10, &nombres).map(|d| d.pid),
        None
    );
    // Solo un auxiliar: no es Codex.
    let auxiliar = vec![
        proceso(10, 1, "bash"),
        proceso(11, 10, "codex-code-mode-host"),
    ];
    assert_eq!(
        procesos::descendiente_llamado(&auxiliar, 10, &nombres),
        None
    );
    // Un ciclo en los padres no lo cuelga.
    let ciclo = vec![proceso(10, 11, "bash"), proceso(11, 10, "sh")];
    assert_eq!(procesos::descendiente_llamado(&ciclo, 10, &nombres), None);
}

#[test]
fn la_respuesta_de_programator_se_avisa_como_respuesta_y_una_sola_vez() {
    use canal::identidad::Identidad;
    use vigilante::pasada;

    let yo = Identidad::leer("Claude 2").unwrap();
    let antes = vec![
        (
            String::from("programator.md"),
            String::from("# Programator\n"),
        ),
        (String::from("codex.md"), String::from("# Codex\n")),
    ];
    let visto = pasada::primera(&antes, &yo);
    let respuesta =
        "# Programator\n\nLATIDO: 2026-11-22 13:50:00\nPara Claude 2: respuesta a PG-A\n150\n";
    let encargo = "# Codex\n\nLATIDO: 2026-11-22 13:50:00\nPara Claude 2: revisa esto\n";
    let ahora = vec![
        (String::from("programator.md"), String::from(respuesta)),
        (String::from("codex.md"), String::from(encargo)),
    ];
    let resultado = pasada::comparar(&visto, &ahora, &yo, "");
    let lineas: Vec<String> = resultado.cambios.iter().map(|c| c.linea(&yo)).collect();
    assert_eq!(
        lineas,
        [
            "CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]",
            "CANAL: ha cambiado .gestor/canal/programator.md [RESPUESTA DE PROGRAMATOR PARA Claude 2]",
        ]
    );
    assert!(resultado.hay_etiquetas());
    // Ya avisada, la misma respuesta no vuelve a despertar aunque Programator escriba después.
    let despues = vec![
        (
            String::from("programator.md"),
            format!("{respuesta}\nLATIDO: 2026-11-22 13:55:00\nnovedades sin encargo\n"),
        ),
        (String::from("codex.md"), String::from(encargo)),
    ];
    let otra = pasada::comparar(&resultado.visto, &despues, &yo, "");
    assert!(!otra.hay_etiquetas(), "{:?}", otra.cambios);
    // Una respuesta para otro agente no despierta.
    let ajena = vec![
        (
            String::from("programator.md"),
            String::from("# Programator\n\nPara Codex 1: respuesta a X\n"),
        ),
        (String::from("codex.md"), String::from("# Codex\n")),
    ];
    assert!(!pasada::comparar(&visto, &ajena, &yo, "").hay_etiquetas());
}

#[test]
fn un_aviso_en_la_cola_de_la_sesion_abierta_se_anota_como_entregado() {
    // Salida literal de codex queue 0.167.1 (.gestor/codex/vigilante/avisos.log, 2026-11-22 14:50).
    let salida = "Queued message 01a20385 for thread 01a20383.\n\
                  Warning: message was queued, but could not resume thread 01a20383: \
                  thread 01a20383 already has an active writer\n";
    let anotado = entrega::anotacion(&entrega::interpretar_salida(Some(0), salida));
    assert!(
        anotado.starts_with("entregado en la cola de la sesión abierta ("),
        "{anotado}"
    );
    assert!(!anotado.contains("no acreditado"), "{anotado}");
    assert!(
        entrega::anotacion(&entrega::interpretar_salida(Some(2), "fallo"))
            .starts_with("no acreditado: "),
    );
    assert_eq!(
        entrega::anotacion(&ResultadoDeEntrega::Aceptada),
        "entregado con codex queue"
    );
}

#[test]
fn dos_encargos_separados_por_cuarenta_segundos_se_entregan_por_separado() {
    assert!(entrega::toca_entregar(None, 1_000));
    assert!(entrega::toca_entregar(Some(1_000), 1_040));
    assert!(!entrega::toca_entregar(Some(1_000), 1_020));
}

#[test]
fn un_encargo_ya_acusado_en_la_bandeja_propia_no_se_avisa() {
    use canal::identidad::Identidad;
    use vigilante::pasada;

    let yo = Identidad::leer("Codex 1").unwrap();
    let visto = pasada::primera(
        &[(String::from("claude2.md"), String::from("# Claude 2\n"))],
        &yo,
    );
    let ahora = vec![(
        String::from("claude2.md"),
        String::from("# Claude 2\n\nLATIDO: 2026-11-22 13:17:48\n\nPara Codex 1: PING-1\n"),
    )];
    let propia = "LATIDO: 2026-11-22 13:27:10\n\nPING-1 recibido 13:27:10\n";

    let acusado = pasada::comparar(&visto, &ahora, &yo, propia);
    assert!(!acusado.hay_etiquetas(), "{:?}", acusado.cambios);
    assert_eq!(acusado.visto.encargos.len(), 1, "queda como visto");

    let pendiente = pasada::comparar(&visto, &ahora, &yo, "");
    assert!(pendiente.hay_etiquetas());
}

fn resultado_con(archivo: &str, encargo: bool, mutacion: bool) -> vigilante::pasada::Resultado {
    vigilante::pasada::Resultado {
        cambios: vec![vigilante::pasada::Cambio {
            archivo: archivo.to_owned(),
            encargo,
            mutacion,
        }],
        visto: vigilante::pasada::Visto::default(),
    }
}

#[test]
fn un_encargo_y_una_mutacion_juntos_en_una_entrega_conservan_las_dos_etiquetas() {
    use vigilante::avisos::AvisosPendientes;

    let mut pendientes = AvisosPendientes::default();
    pendientes.anadir(&resultado_con("claude2.md", true, false));
    let primera = pendientes
        .tomar_si_toca(1_000)
        .expect("la primera entrega va enseguida");
    assert_eq!(primera.etiquetas, "encargo");

    pendientes.anadir(&resultado_con("claude2.md", true, false));
    pendientes.anadir(&resultado_con("estado.md", false, true));
    pendientes.anadir(&resultado_con("claude2.md", true, false));
    assert!(
        pendientes.tomar_si_toca(1_010).is_none(),
        "antes de la cadencia"
    );
    let junta = pendientes.tomar_si_toca(1_030).expect("pasada la cadencia");
    assert_eq!(junta.etiquetas, "encargo,mutacion");
    assert_eq!(junta.archivos, vec!["claude2.md", "estado.md"]);
    assert!(pendientes.esta_vacio());
    assert!(pendientes.tomar_si_toca(2_000).is_none(), "nada pendiente");
}

#[test]
fn la_respuesta_de_programator_se_etiqueta_como_respuesta_y_no_como_encargo() {
    let lote = vigilante::avisos::Lote::de_resultado(&resultado_con("programator.md", true, false));
    assert_eq!(lote.etiquetas, "respuesta");
}

/// Orden que escribe más de lo que cabe en una tubería (64 KiB en Linux, 4 KiB a 64 KiB en
/// Windows) y termina.
fn orden_que_escribe_mucho(bytes: usize) -> std::process::Command {
    #[cfg(windows)]
    {
        let mut orden = std::process::Command::new("powershell");
        orden.args([
            "-NoProfile",
            "-Command",
            &format!("[Console]::Out.Write(('a' * {bytes}))"),
        ]);
        orden
    }
    #[cfg(not(windows))]
    {
        let mut orden = std::process::Command::new("sh");
        orden.args(["-c", &format!("head -c {bytes} /dev/zero | tr '\\0' a")]);
        orden
    }
}

#[test]
fn una_entrega_que_escribe_mucho_no_se_queda_bloqueada_en_la_tuberia() {
    use std::process::Stdio;
    use std::time::Duration;

    const BYTES: usize = 300_000;
    let hijo = orden_que_escribe_mucho(BYTES)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (codigo, salida) =
        entrega::esperar_salida(hijo, Duration::from_secs(20)).expect("termina a tiempo");
    assert_eq!(codigo, Some(0));
    assert!(salida.matches('a').count() >= BYTES, "{}", salida.len());
}

#[test]
fn una_entrega_sin_respuesta_dice_cuantos_segundos_espero() {
    use std::process::Stdio;
    use std::time::Duration;

    #[cfg(windows)]
    let mut orden = std::process::Command::new("powershell");
    #[cfg(windows)]
    orden.args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"]);
    #[cfg(not(windows))]
    let mut orden = std::process::Command::new("sleep");
    #[cfg(not(windows))]
    orden.arg("30");
    let hijo = orden
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let motivo = entrega::esperar_salida(hijo, Duration::from_secs(1)).unwrap_err();
    assert_eq!(motivo, "sin respuesta en 1 s");
}

/// Entregador que anota cada mensaje y responde siempre lo mismo.
struct EntregadorFalso {
    mensajes: std::cell::RefCell<Vec<String>>,
}

impl vigilante::puertos::Entregador for EntregadorFalso {
    fn entregar(&self, mensaje: &str) -> ResultadoDeEntrega {
        self.mensajes.borrow_mut().push(mensaje.to_owned());
        ResultadoDeEntrega::Aceptada
    }
}

#[test]
fn los_avisos_se_entregan_juntos_con_la_cadencia_sin_esperar_de_verdad() {
    use canal::identidad::Identidad;
    use vigilante::avisos::{self, AvisosPendientes};

    let yo = Identidad::leer("Codex 1").unwrap();
    let entregador = EntregadorFalso {
        mensajes: std::cell::RefCell::new(Vec::new()),
    };
    let mut pendientes = AvisosPendientes::default();
    let mut entregas = Vec::new();
    // Horas simuladas: un encargo a las 1000, una mutación a las 1010 y otro encargo a las 1020.
    for (ahora, archivo, encargo, mutacion) in [
        (1_000, "claude2.md", true, false),
        (1_010, "estado.md", false, true),
        (1_020, "claude2.md", true, false),
        (1_030, "", false, false),
    ] {
        if !archivo.is_empty() {
            pendientes.anadir(&resultado_con(archivo, encargo, mutacion));
        }
        if let Some((lote, _)) = avisos::entregar_si_toca(&mut pendientes, ahora, &yo, &entregador)
        {
            entregas.push((ahora, lote.etiquetas));
        }
    }
    assert_eq!(
        entregas,
        vec![
            (1_000, String::from("encargo")),
            (1_030, String::from("encargo,mutacion"))
        ]
    );
    let mensajes = entregador.mensajes.borrow();
    assert_eq!(mensajes.len(), 2);
    assert!(
        mensajes[1].contains(".gestor/canal/estado.md, .gestor/canal/claude2.md"),
        "{}",
        mensajes[1]
    );
}
