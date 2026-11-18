//! Barreras del vigilante del canal: ejecutan el guion de la plataforma (bash en
//! Linux, Windows PowerShell en Windows) en un proyecto temporal y comprueban que avisa de los
//! cambios en las bandejas ajenas y en estado.md, y nunca de la bandeja propia.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use comun::CarpetaTemporal;
use nucleo::plantillas::GuionDelVigilante;

/// Identidad con la que se ejecuta el guion: su bandeja es claude2.md.
const IDENTIDAD: &str = "Claude 2";

/// Espera máxima a una línea del modo continuo: tres veces su intervalo de diez segundos.
const ESPERA_MAXIMA: Duration = Duration::from_secs(30);

/// Proyecto temporal con el guion de la plataforma y un canal con tres archivos.
fn proyecto() -> CarpetaTemporal {
    let carpeta = CarpetaTemporal::new();
    let guion = guion_de_la_plataforma();
    carpeta.escribir(guion.ruta_relativa(), &(guion.texto() + "\n"));
    carpeta.escribir(".gestor/canal/estado.md", "# Estado del proyecto\n");
    carpeta.escribir(".gestor/canal/claude2.md", "# Bandeja de Claude 2\n");
    carpeta.escribir(".gestor/canal/codex.md", "# Bandeja de Codex 1\n");
    carpeta
}

fn guion_de_la_plataforma() -> GuionDelVigilante {
    if cfg!(windows) {
        GuionDelVigilante::PowerShell
    } else {
        GuionDelVigilante::Bash
    }
}

/// Modo del guion, con su opción en cada plataforma. El continuo no lleva opción.
#[derive(Clone, Copy)]
enum Modo {
    UnaVez,
    HastaCambio,
}

impl Modo {
    const fn opcion(self) -> &'static str {
        match (self, cfg!(windows)) {
            (Self::UnaVez, true) => "-UnaVez",
            (Self::UnaVez, false) => "--una-vez",
            (Self::HastaCambio, true) => "-HastaCambio",
            (Self::HastaCambio, false) => "--hasta-cambio",
        }
    }
}

/// Orden que ejecuta el guion en el proyecto con esas opciones de modo (ninguna es el modo
/// continuo), con la identidad en GESTOR_AGENTE.
fn orden(carpeta: &CarpetaTemporal, modos: &[Modo]) -> Command {
    let guion = guion_de_la_plataforma();
    let ruta = carpeta.archivo(guion.ruta_relativa());
    let mut orden = if cfg!(windows) {
        let mut orden = Command::new("powershell.exe");
        orden.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
        orden
    } else {
        Command::new("bash")
    };
    orden
        .arg(&ruta)
        .args(modos.iter().map(|modo| modo.opcion()))
        .current_dir(carpeta.ruta())
        .env("GESTOR_AGENTE", IDENTIDAD);
    orden
}

/// Una pasada del modo --una-vez: sus líneas de salida, exigiendo que termine bien.
fn una_pasada(carpeta: &CarpetaTemporal) -> Vec<String> {
    let salida = orden(carpeta, &[Modo::UnaVez])
        .output()
        .expect("se ejecuta el guion");
    assert!(
        salida.status.success(),
        "{:?}: {}",
        salida.status,
        String::from_utf8_lossy(&salida.stderr)
    );
    String::from_utf8_lossy(&salida.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn una_pasada_avisa_de_lo_ajeno_y_nunca_de_la_bandeja_propia() {
    let carpeta = proyecto();

    let primera = una_pasada(&carpeta);
    assert_eq!(primera.len(), 1, "{primera:?}");
    assert!(
        primera[0].starts_with("CANAL: primera pasada"),
        "{primera:?}"
    );
    assert!(carpeta.existe(".gestor/claude2/canal-visto.txt"));

    assert_eq!(una_pasada(&carpeta), ["CANAL: sin cambios"]);

    carpeta.escribir(
        ".gestor/canal/claude2.md",
        "# Bandeja de Claude 2\n\nMensaje propio.\n",
    );
    assert_eq!(una_pasada(&carpeta), ["CANAL: sin cambios"]);

    carpeta.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo.\n",
    );
    carpeta.escribir(".gestor/canal/gemini.md", "# Bandeja de Gemini 1\n");
    let mut avisos = una_pasada(&carpeta);
    avisos.sort();
    assert_eq!(
        avisos,
        [
            // Desde la 3.2.0, la línea nueva «Para Claude 2» lleva su etiqueta.
            "CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]",
            "CANAL: ha cambiado .gestor/canal/gemini.md",
        ]
    );

    assert_eq!(una_pasada(&carpeta), ["CANAL: sin cambios"]);
}

#[test]
fn una_pasada_no_avisa_de_la_guia_de_programator_ni_de_los_historicos() {
    let carpeta = proyecto();
    let _ = una_pasada(&carpeta);

    carpeta.escribir(
        ".gestor/canal/COMO-ENCARGAR-A-PROGRAMATOR.md",
        "# Guía nueva\n",
    );
    carpeta.escribir(".gestor/canal/historico-programator.md", "# Histórico\n");

    assert_eq!(una_pasada(&carpeta), ["CANAL: sin cambios"]);
}

#[test]
fn solo_se_etiqueta_un_encargo_nuevo_para_la_propia_identidad() {
    let carpeta = proyecto();
    carpeta.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo viejo.\n",
    );
    let _ = una_pasada(&carpeta);

    // El encargo viejo sigue ahí; lo nuevo no es para Claude 2.
    carpeta.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo viejo.\n\nPara Claude 22. Otro.\nPara Gemini 1, con copia a Claude 2.\n",
    );
    assert_eq!(
        una_pasada(&carpeta),
        ["CANAL: ha cambiado .gestor/canal/codex.md"]
    );

    carpeta.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo viejo.\n\nPara Claude 22. Otro.\nPara Gemini 1, con copia a Claude 2.\n\nPara Claude 2, con copia a Gemini 1. Encargo nuevo.\n",
    );
    assert_eq!(
        una_pasada(&carpeta),
        ["CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]"]
    );
}

#[test]
fn un_estado_con_mutaciones_vivas_se_etiqueta() {
    let carpeta = proyecto();
    let _ = una_pasada(&carpeta);

    carpeta.escribir(
        ".gestor/canal/estado.md",
        "# Estado del proyecto\n\n## Mutaciones vivas\n\n- Codex 1: crates/nucleo/src/git.rs\n\n## Prohibiciones vigentes\n\nNinguna.\n",
    );
    assert_eq!(
        una_pasada(&carpeta),
        ["CANAL: ha cambiado .gestor/canal/estado.md [MUTACION VIVA]"]
    );

    carpeta.escribir(
        ".gestor/canal/estado.md",
        "# Estado del proyecto\n\n## Mutaciones vivas\n\nNinguna.\n\n## Prohibiciones vigentes\n\n- Algo.\n",
    );
    assert_eq!(
        una_pasada(&carpeta),
        ["CANAL: ha cambiado .gestor/canal/estado.md"]
    );
}

#[test]
fn sin_identidad_el_guion_falla_y_lo_dice() {
    let carpeta = proyecto();
    let salida = orden(&carpeta, &[Modo::UnaVez])
        .env_remove("GESTOR_AGENTE")
        .output()
        .expect("se ejecuta el guion");

    assert_eq!(salida.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&salida.stderr).contains("GESTOR_AGENTE"));
}

#[test]
fn el_modo_continuo_escribe_una_linea_por_cambio_ajeno() {
    let carpeta = proyecto();
    let mut hijo = orden(&carpeta, &[])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("arranca el vigilante");
    let salida = hijo.stdout.take().expect("salida del vigilante");
    let (envio, lineas) = mpsc::channel();
    thread::spawn(move || {
        for linea in BufReader::new(salida).lines().map_while(Result::ok) {
            if envio.send(linea).is_err() {
                break;
            }
        }
    });

    let inicio = lineas.recv_timeout(ESPERA_MAXIMA);
    carpeta.escribir(
        ".gestor/canal/claude2.md",
        "# Bandeja de Claude 2\n\nPropio.\n",
    );
    carpeta.escribir(
        ".gestor/canal/estado.md",
        "# Estado del proyecto\n\nReserva nueva.\n",
    );
    let aviso = lineas.recv_timeout(ESPERA_MAXIMA);
    let _ = hijo.kill();
    let _ = hijo.wait();

    assert!(
        inicio
            .as_deref()
            .is_ok_and(|l| l.starts_with("CANAL: vigilando")),
        "{inicio:?}"
    );
    assert_eq!(
        aviso.as_deref(),
        Ok("CANAL: ha cambiado .gestor/canal/estado.md")
    );
}

/// Espera algo más que un intervalo del guion (diez segundos), para que haya hecho al menos una
/// comprobación más antes de mirar si sigue en marcha.
const MAS_DE_UN_INTERVALO: Duration = Duration::from_secs(12);

/// Cada cuánto se mira si el guion ha terminado.
const PASO_DE_ESPERA: Duration = Duration::from_millis(100);

/// Espera a que el guion termine, como mucho `plazo`. Si no termina, lo para y devuelve nada.
fn terminado_en(
    hijo: &mut std::process::Child,
    plazo: Duration,
) -> Option<std::process::ExitStatus> {
    let mut esperado = Duration::ZERO;
    while esperado < plazo {
        if let Some(estado) = hijo.try_wait().expect("estado del guion") {
            return Some(estado);
        }
        thread::sleep(PASO_DE_ESPERA);
        esperado += PASO_DE_ESPERA;
    }
    let _ = hijo.kill();
    let _ = hijo.wait();
    None
}

/// Lanza el guion en modo --hasta-cambio, con su salida recogida.
fn hasta_cambio(carpeta: &CarpetaTemporal) -> std::process::Child {
    orden(carpeta, &[Modo::HastaCambio])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("arranca el vigilante")
}

/// Líneas de salida de un guion que ya ha terminado.
fn lineas_de(hijo: std::process::Child) -> Vec<String> {
    let salida = hijo.wait_with_output().expect("salida del guion");
    String::from_utf8_lossy(&salida.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn hasta_cambio_espera_sin_escribir_y_termina_con_el_primer_encargo() {
    let carpeta = proyecto();
    let _ = una_pasada(&carpeta);
    let mut hijo = hasta_cambio(&carpeta);

    // Ni la bandeja propia ni un mensaje ajeno que no es para el agente lo despiertan: pasado un
    // intervalo, sigue esperando (orden del Director, para no gastar un turno en cada cambio).
    carpeta.escribir(
        ".gestor/canal/claude2.md",
        "# Bandeja de Claude 2\n\nPropio.\n",
    );
    carpeta.escribir(
        ".gestor/canal/gemini.md",
        "# Bandeja de Gemini 1\n\nPara Codex 1, con copia a Claude 2.\n",
    );
    thread::sleep(MAS_DE_UN_INTERVALO);
    assert!(
        hijo.try_wait().expect("estado del guion").is_none(),
        "terminó sin un encargo"
    );

    carpeta.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo.\n",
    );
    let estado = terminado_en(&mut hijo, ESPERA_MAXIMA);
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    // El aviso trae también lo que cambió mientras esperaba.
    let mut lineas = lineas_de(hijo);
    lineas.sort();
    assert_eq!(
        lineas,
        [
            "CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]",
            "CANAL: ha cambiado .gestor/canal/gemini.md",
        ]
    );

    // Guardó la referencia: la pasada siguiente ya no ve esos cambios.
    assert_eq!(una_pasada(&carpeta), ["CANAL: sin cambios"]);
}

#[test]
fn hasta_cambio_ve_en_el_momento_lo_que_cambio_mientras_no_vigilaba() {
    let carpeta = proyecto();
    let _ = una_pasada(&carpeta);
    carpeta.escribir(
        ".gestor/canal/estado.md",
        "# Estado del proyecto\n\n## Mutaciones vivas\n\n- Codex 1: crates/nucleo/src/git.rs\n",
    );

    let mut hijo = hasta_cambio(&carpeta);
    // Sin esperar un intervalo: la primera comprobación es inmediata.
    let estado = terminado_en(&mut hijo, MAS_DE_UN_INTERVALO / 2);

    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    assert_eq!(
        lineas_de(hijo),
        ["CANAL: ha cambiado .gestor/canal/estado.md [MUTACION VIVA]"]
    );
}

#[test]
fn hasta_cambio_sin_referencia_termina_con_la_primera_pasada() {
    let carpeta = proyecto();
    let mut hijo = hasta_cambio(&carpeta);
    let estado = terminado_en(&mut hijo, MAS_DE_UN_INTERVALO / 2);

    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    let lineas = lineas_de(hijo);
    assert_eq!(lineas.len(), 1, "{lineas:?}");
    assert!(lineas[0].starts_with("CANAL: primera pasada"), "{lineas:?}");
}

#[test]
fn una_vez_y_hasta_cambio_a_la_vez_es_un_error() {
    let carpeta = proyecto();
    let salida = orden(&carpeta, &[Modo::UnaVez, Modo::HastaCambio])
        .output()
        .expect("se ejecuta el guion");

    assert_eq!(salida.status.code(), Some(2));
    assert!(!carpeta.existe(".gestor/claude2/canal-visto.txt"));
}

// Desde la 3.5.1: vida máxima, relevo y fin con quien lo lanzó (incidente del 2026-11-18, en el
// que el modo continuo dejó cientos de procesos vivos al caducar el Monitor de Claude Code).

/// Opción de vida máxima del guion de la plataforma.
const fn opcion_de_vida() -> &'static str {
    if cfg!(windows) {
        "-VidaSegundos"
    } else {
        "--vida-segundos"
    }
}

/// Marcador del vigilante en bucle de la identidad de las pruebas.
const MARCADOR: &str = ".gestor/claude2/vigilante.pid";

/// Recoge en un canal cada línea de la salida del guion, hasta que la salida se cierra.
fn lector(hijo: &mut std::process::Child) -> mpsc::Receiver<String> {
    let salida = hijo.stdout.take().expect("salida del vigilante");
    let (envio, lineas) = mpsc::channel();
    thread::spawn(move || {
        for linea in BufReader::new(salida).lines().map_while(Result::ok) {
            if envio.send(linea).is_err() {
                break;
            }
        }
    });
    lineas
}

/// Arranca el guion con esos modos y opciones extra, con la salida recogida.
fn arrancar(
    carpeta: &CarpetaTemporal,
    modos: &[Modo],
    extra: &[&str],
) -> (std::process::Child, mpsc::Receiver<String>) {
    let mut hijo = orden(carpeta, modos)
        .args(extra)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("arranca el vigilante");
    let lineas = lector(&mut hijo);
    (hijo, lineas)
}

#[test]
fn el_modo_continuo_termina_solo_al_cumplir_su_vida_y_suelta_el_marcador() {
    let carpeta = proyecto();
    let (mut hijo, lineas) = arrancar(&carpeta, &[], &[opcion_de_vida(), "1"]);

    let inicio = lineas.recv_timeout(ESPERA_MAXIMA);
    assert!(
        inicio
            .as_deref()
            .is_ok_and(|l| l.starts_with("CANAL: vigilando")),
        "{inicio:?}"
    );
    assert!(carpeta.existe(MARCADOR));
    let fin = lineas.recv_timeout(ESPERA_MAXIMA);
    let estado = terminado_en(&mut hijo, ESPERA_MAXIMA);

    assert_eq!(fin.as_deref(), Ok("VIGILANTE: fin por tiempo; rearma"));
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    assert!(!carpeta.existe(MARCADOR));
}

#[test]
fn un_vigilante_nuevo_del_mismo_agente_releva_al_anterior() {
    let carpeta = proyecto();
    let _ = una_pasada(&carpeta);
    let (mut primero, lineas_del_primero) = arrancar(&carpeta, &[], &[]);
    let inicio = lineas_del_primero.recv_timeout(ESPERA_MAXIMA);
    assert!(inicio.is_ok(), "{inicio:?}");

    // El relevo vale entre modos: el nuevo espera al cambio.
    let mut segundo = hasta_cambio(&carpeta);
    let relevo = lineas_del_primero.recv_timeout(ESPERA_MAXIMA);
    let estado = terminado_en(&mut primero, ESPERA_MAXIMA);
    let segundo_sigue = segundo.try_wait().expect("estado del segundo").is_none();
    let _ = segundo.kill();
    let _ = segundo.wait();

    assert_eq!(
        relevo.as_deref(),
        Ok("VIGILANTE: relevado por otro vigilante de Claude 2; no lo rearmes")
    );
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    assert!(segundo_sigue, "el relevo no puede parar al nuevo");
}

/// Variable con la ruta del guion para la envoltura de la prueba de abajo.
const RUTA_DEL_GUION: &str = "VIGILANTE_RUTA";

/// Envoltura que lanza el guion en modo continuo y espera a que termine, como la orden de una
/// herramienta: el guion es su hijo y no la reemplaza.
fn envoltura(carpeta: &CarpetaTemporal) -> Command {
    let ruta = carpeta.archivo(guion_de_la_plataforma().ruta_relativa());
    let mut orden = if cfg!(windows) {
        let mut orden = Command::new("powershell.exe");
        orden.args([
            "-NoProfile",
            "-Command",
            "& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $env:VIGILANTE_RUTA; exit 0",
        ]);
        orden
    } else {
        let mut orden = Command::new("bash");
        orden.args(["-c", "bash \"$VIGILANTE_RUTA\"; exit 0"]);
        orden
    };
    orden
        .env(RUTA_DEL_GUION, ruta)
        .current_dir(carpeta.ruta())
        .env("GESTOR_AGENTE", IDENTIDAD);
    orden
}

#[test]
fn si_muere_quien_lo_lanzo_el_vigilante_termina_y_no_queda_huerfano() {
    let carpeta = proyecto();
    let mut envoltura = envoltura(&carpeta)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("arranca la envoltura");
    let lineas = lector(&mut envoltura);
    let inicio = lineas.recv_timeout(ESPERA_MAXIMA);
    assert!(
        inicio
            .as_deref()
            .is_ok_and(|l| l.starts_with("CANAL: vigilando")),
        "{inicio:?}"
    );

    // Se mata solo la envoltura, como cuando caduca el Monitor o se cierra la sesión: el guion
    // sigue vivo y tiene que verlo él.
    let _ = envoltura.kill();
    let _ = envoltura.wait();
    let fin = lineas.recv_timeout(ESPERA_MAXIMA);
    // La salida se cierra cuando termina el guion, que es el último que la tiene abierta.
    let cerrada = lineas.recv_timeout(ESPERA_MAXIMA);

    assert_eq!(
        fin.as_deref(),
        Ok("VIGILANTE: fin porque se cerro quien lo lanzo; relanzalo si sigues trabajando")
    );
    assert_eq!(cerrada, Err(mpsc::RecvTimeoutError::Disconnected));
    assert!(!carpeta.existe(MARCADOR));
}

#[test]
fn una_vida_que_no_es_un_entero_positivo_es_un_error() {
    // Con --una-vez, para que un guion que no rechace el valor termine igual y la prueba falle
    // en vez de quedarse esperando: la vida se comprueba antes de elegir el modo.
    let carpeta = proyecto();
    for vida in ["0", "abc", "-5", "1.5"] {
        let salida = orden(&carpeta, &[Modo::UnaVez])
            .args([opcion_de_vida(), vida])
            .output()
            .expect("se ejecuta el guion");

        assert_eq!(salida.status.code(), Some(2), "{vida}");
        assert!(!carpeta.existe(".gestor/claude2/canal-visto.txt"), "{vida}");
    }
}
