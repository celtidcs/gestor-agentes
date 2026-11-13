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

/// Orden que ejecuta el guion en el proyecto, con la identidad en GESTOR_AGENTE.
fn orden(carpeta: &CarpetaTemporal, una_vez: bool) -> Command {
    let guion = guion_de_la_plataforma();
    let ruta = carpeta.archivo(guion.ruta_relativa());
    let mut orden = if cfg!(windows) {
        let mut orden = Command::new("powershell.exe");
        orden.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
        orden.arg(&ruta);
        if una_vez {
            orden.arg("-UnaVez");
        }
        orden
    } else {
        let mut orden = Command::new("bash");
        orden.arg(&ruta);
        if una_vez {
            orden.arg("--una-vez");
        }
        orden
    };
    orden
        .current_dir(carpeta.ruta())
        .env("GESTOR_AGENTE", IDENTIDAD);
    orden
}

/// Una pasada del modo --una-vez: sus líneas de salida, exigiendo que termine bien.
fn una_pasada(carpeta: &CarpetaTemporal) -> Vec<String> {
    let salida = orden(carpeta, true).output().expect("se ejecuta el guion");
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
    let salida = orden(&carpeta, true)
        .env_remove("GESTOR_AGENTE")
        .output()
        .expect("se ejecuta el guion");

    assert_eq!(salida.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&salida.stderr).contains("GESTOR_AGENTE"));
}

#[test]
fn el_modo_continuo_escribe_una_linea_por_cambio_ajeno() {
    let carpeta = proyecto();
    let mut hijo = orden(&carpeta, false)
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
