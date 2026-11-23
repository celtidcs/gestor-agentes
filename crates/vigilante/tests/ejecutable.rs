//! Barreras del ejecutable gestor-vigilante en un proyecto temporal. Sustituyen a las de los
//! guiones vigilar-canal.ps1 y vigilar-canal.sh (crates/nucleo/tests/vigilante.rs hasta la 3.5.1),
//! con los mismos casos salvo el modo continuo, que desaparece, y con los casos nuevos de la 3.6.0.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use vigilante::procesos::{Procesos, ProcesosDelSistema};

const IDENTIDAD: &str = "Claude 2";
const EJECUTABLE: &str = env!("CARGO_BIN_EXE_gestor-vigilante");
/// Intervalo de las pruebas, en segundos: un segundo en vez de diez.
const INTERVALO: &str = "1";
/// Espera máxima a que el vigilante haga algo: muchas veces su intervalo.
const ESPERA_MAXIMA: Duration = Duration::from_secs(20);
/// Algo más que un intervalo, para que haya hecho al menos una pasada más.
const MAS_DE_UN_INTERVALO: Duration = Duration::from_secs(3);
const PASO_DE_ESPERA: Duration = Duration::from_millis(100);
const RELEVO: &str = ".gestor/claude2/vigilante.pid";
const VIVO: &str = ".gestor/claude2/vigilante/vivo.json";
const RECUERDO: &str = ".gestor/claude2/vigilante/archivos.txt";

static CONTADOR: AtomicU64 = AtomicU64::new(0);

/// Proyecto temporal con un canal de tres archivos; se borra al salir de ámbito.
struct Proyecto {
    ruta: PathBuf,
}

impl Proyecto {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let numero = CONTADOR.fetch_add(1, Ordering::Relaxed);
        let ruta = std::env::temp_dir().join(format!(
            "gestor-vigilante-{}-{nanos}-{numero}",
            std::process::id()
        ));
        fs::create_dir_all(ruta.join(".gestor/canal")).unwrap();
        let proyecto = Self { ruta };
        proyecto.escribir(".gestor/canal/estado.md", "# Estado del proyecto\n");
        proyecto.escribir(".gestor/canal/claude2.md", "# Bandeja de Claude 2\n");
        proyecto.escribir(".gestor/canal/codex.md", "# Bandeja de Codex 1\n");
        proyecto
    }

    fn escribir(&self, relativa: &str, texto: &str) {
        let ruta = self.ruta.join(relativa);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, texto).unwrap();
    }

    fn existe(&self, relativa: &str) -> bool {
        self.ruta.join(relativa).exists()
    }

    fn gestor(&self) -> PathBuf {
        self.ruta.join(".gestor")
    }
}

impl Drop for Proyecto {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.ruta);
    }
}

fn orden(proyecto: &Proyecto, opciones: &[&str]) -> Command {
    let mut orden = Command::new(EJECUTABLE);
    orden
        .args(opciones)
        .arg("--gestor")
        .arg(proyecto.gestor())
        .args(["--intervalo-segundos", INTERVALO])
        .current_dir(&proyecto.ruta)
        .env("GESTOR_AGENTE", IDENTIDAD);
    orden
}

fn lineas(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::to_owned)
        .collect()
}

fn una_pasada(proyecto: &Proyecto) -> Vec<String> {
    let salida = orden(proyecto, &["--una-vez"]).output().unwrap();
    assert!(
        salida.status.success(),
        "{:?}: {}",
        salida.status,
        String::from_utf8_lossy(&salida.stderr)
    );
    lineas(&salida.stdout)
}

fn terminado_en(hijo: &mut Child, plazo: Duration) -> Option<ExitStatus> {
    let mut esperado = Duration::ZERO;
    while esperado < plazo {
        if let Some(estado) = hijo.try_wait().unwrap() {
            return Some(estado);
        }
        thread::sleep(PASO_DE_ESPERA);
        esperado += PASO_DE_ESPERA;
    }
    let _ = hijo.kill();
    let _ = hijo.wait();
    None
}

fn hasta_cambio(proyecto: &Proyecto, extra: &[&str]) -> Child {
    let mut opciones = vec!["--hasta-cambio"];
    opciones.extend_from_slice(extra);
    orden(proyecto, &opciones)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

fn lineas_de(hijo: Child) -> Vec<String> {
    lineas(&hijo.wait_with_output().unwrap().stdout)
}

fn lector(hijo: &mut Child) -> mpsc::Receiver<String> {
    let salida = hijo.stdout.take().unwrap();
    let (envio, recibidas) = mpsc::channel();
    thread::spawn(move || {
        for linea in BufReader::new(salida).lines().map_while(Result::ok) {
            if envio.send(linea).is_err() {
                break;
            }
        }
    });
    recibidas
}

fn esperar_archivo(ruta: &Path) -> bool {
    let mut esperado = Duration::ZERO;
    while esperado < ESPERA_MAXIMA {
        if ruta.exists() {
            return true;
        }
        thread::sleep(PASO_DE_ESPERA);
        esperado += PASO_DE_ESPERA;
    }
    false
}

#[test]
fn una_pasada_avisa_de_lo_ajeno_y_nunca_de_la_bandeja_propia() {
    let proyecto = Proyecto::new();
    let primera = una_pasada(&proyecto);
    assert_eq!(primera.len(), 1, "{primera:?}");
    assert!(
        primera[0].starts_with("CANAL: primera pasada"),
        "{primera:?}"
    );
    assert!(proyecto.existe(RECUERDO));
    assert_eq!(una_pasada(&proyecto), ["CANAL: sin cambios"]);

    proyecto.escribir(
        ".gestor/canal/claude2.md",
        "# Bandeja de Claude 2\n\nPropio.\n",
    );
    assert_eq!(una_pasada(&proyecto), ["CANAL: sin cambios"]);

    proyecto.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo.\n",
    );
    proyecto.escribir(".gestor/canal/gemini.md", "# Bandeja de Gemini 1\n");
    assert_eq!(
        una_pasada(&proyecto),
        [
            "CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]",
            "CANAL: ha cambiado .gestor/canal/gemini.md",
        ]
    );
    assert_eq!(una_pasada(&proyecto), ["CANAL: sin cambios"]);
}

#[test]
fn una_pasada_no_avisa_de_la_guia_de_programator_ni_de_los_historicos() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    proyecto.escribir(
        ".gestor/canal/COMO-ENCARGAR-A-PROGRAMATOR.md",
        "# Guía nueva\n",
    );
    proyecto.escribir(".gestor/canal/historico-programator.md", "# Histórico\n");
    assert_eq!(una_pasada(&proyecto), ["CANAL: sin cambios"]);
}

#[test]
fn solo_se_etiqueta_un_encargo_nuevo_para_la_propia_identidad() {
    let proyecto = Proyecto::new();
    proyecto.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo viejo.\n",
    );
    let _ = una_pasada(&proyecto);
    proyecto.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo viejo.\n\nPara Claude 22. Otro.\nPara Gemini 1, con copia a Claude 2.\n",
    );
    assert_eq!(
        una_pasada(&proyecto),
        ["CANAL: ha cambiado .gestor/canal/codex.md"]
    );
    proyecto.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo viejo.\n\nPara Claude 22. Otro.\nPara Gemini 1, con copia a Claude 2.\n\nPara Claude 2, con copia a Gemini 1. Encargo nuevo.\n",
    );
    assert_eq!(
        una_pasada(&proyecto),
        ["CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]"]
    );
}

#[test]
fn un_encargo_ya_avisado_que_cambia_de_sitio_no_vuelve_a_etiquetarse() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    let encargo = "2026-11-19 10:00:00. Para Claude 2: revisa F5.\n";
    proyecto.escribir(".gestor/canal/codex.md", &format!("# Bandeja\n\n{encargo}"));
    assert_eq!(
        una_pasada(&proyecto),
        ["CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]"]
    );
    proyecto.escribir(
        ".gestor/canal/codex.md",
        &format!("# Bandeja\n\nRECIBIDO 2026-11-19 10:01:00.\n\n{encargo}\nOtra cosa.\n"),
    );
    assert_eq!(
        una_pasada(&proyecto),
        ["CANAL: ha cambiado .gestor/canal/codex.md"]
    );
}

#[test]
fn varios_destinatarios_y_la_instancia_1_sin_numero() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    proyecto.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja\n\n## Para Claude 1 y Claude 2 — reparto\n",
    );
    assert_eq!(
        una_pasada(&proyecto),
        ["CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]"]
    );
    let primera = orden(&proyecto, &["--una-vez", "--agente", "Codex"])
        .output()
        .unwrap();
    assert!(primera.status.success());
    proyecto.escribir(
        ".gestor/canal/claude2.md",
        "# Bandeja de Claude 2\n\nPara Codex: empieza.\n",
    );
    let salida = orden(&proyecto, &["--una-vez", "--agente", "Codex"])
        .output()
        .unwrap();
    assert_eq!(
        lineas(&salida.stdout),
        ["CANAL: ha cambiado .gestor/canal/claude2.md [ENCARGO PARA Codex 1]"]
    );
}

#[test]
fn las_mutaciones_vivas_se_etiquetan_solo_cuando_cambian() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    let con_mutacion = "# Estado\n\n## Mutaciones vivas\n\n- Codex 1: crates/nucleo/src/git.rs\n\n## Prohibiciones vigentes\n\nNinguna.\n";
    proyecto.escribir(".gestor/canal/estado.md", con_mutacion);
    assert_eq!(
        una_pasada(&proyecto),
        ["CANAL: ha cambiado .gestor/canal/estado.md [MUTACION VIVA]"]
    );
    // Otro cambio de estado.md con la misma mutación ya no se etiqueta.
    proyecto.escribir(
        ".gestor/canal/estado.md",
        &format!("{con_mutacion}- Otra prohibición.\n"),
    );
    assert_eq!(
        una_pasada(&proyecto),
        ["CANAL: ha cambiado .gestor/canal/estado.md"]
    );
    // Una nota delante de «Ninguna.» y un título con espacio delante no son mutaciones.
    proyecto.escribir(
        ".gestor/canal/estado.md",
        "# Estado\n\n## Mutaciones vivas\n\n- (Añadido por Claude 1.) Ninguna.\n ## Raíz reorganizada\n\nTexto.\n",
    );
    assert_eq!(
        una_pasada(&proyecto),
        ["CANAL: ha cambiado .gestor/canal/estado.md"]
    );
}

#[test]
fn sin_identidad_falla_y_lo_dice() {
    let proyecto = Proyecto::new();
    let salida = orden(&proyecto, &["--una-vez"])
        .env_remove("GESTOR_AGENTE")
        .output()
        .unwrap();
    assert_eq!(salida.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&salida.stderr).contains("GESTOR_AGENTE"));
}

#[test]
fn hasta_cambio_espera_sin_escribir_y_termina_con_el_primer_encargo() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    let mut hijo = hasta_cambio(&proyecto, &[]);
    assert!(
        esperar_archivo(&proyecto.ruta.join(VIVO)),
        "no escribió vivo.json"
    );
    proyecto.escribir(
        ".gestor/canal/claude2.md",
        "# Bandeja de Claude 2\n\nPropio.\n",
    );
    proyecto.escribir(
        ".gestor/canal/gemini.md",
        "# Bandeja de Gemini 1\n\nPara Codex 1, con copia a Claude 2.\n",
    );
    thread::sleep(MAS_DE_UN_INTERVALO);
    assert!(hijo.try_wait().unwrap().is_none(), "terminó sin un encargo");

    proyecto.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja de Codex 1\n\nPara Claude 2. Encargo.\n",
    );
    let estado = terminado_en(&mut hijo, ESPERA_MAXIMA);
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    assert_eq!(
        lineas_de(hijo),
        [
            "CANAL: ha cambiado .gestor/canal/codex.md [ENCARGO PARA Claude 2]",
            "CANAL: ha cambiado .gestor/canal/gemini.md",
        ]
    );
    assert_eq!(una_pasada(&proyecto), ["CANAL: sin cambios"]);
    assert!(!proyecto.existe(VIVO), "vivo.json queda tras terminar");
    assert!(!proyecto.existe(RELEVO), "el relevo queda tras terminar");
    let avisos =
        fs::read_to_string(proyecto.ruta.join(".gestor/claude2/vigilante/avisos.log")).unwrap();
    assert!(avisos.contains("\tencargo\tcodex.md\t"), "{avisos}");
}

#[test]
fn hasta_cambio_ve_en_el_momento_lo_que_cambio_mientras_no_vigilaba() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    proyecto.escribir(
        ".gestor/canal/estado.md",
        "# Estado\n\n## Mutaciones vivas\n\n- Codex 1: crates/nucleo/src/git.rs\n",
    );
    let mut hijo = hasta_cambio(&proyecto, &[]);
    let estado = terminado_en(&mut hijo, MAS_DE_UN_INTERVALO);
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    assert_eq!(
        lineas_de(hijo),
        ["CANAL: ha cambiado .gestor/canal/estado.md [MUTACION VIVA]"]
    );
}

#[test]
fn hasta_cambio_sin_referencia_termina_con_la_primera_pasada() {
    let proyecto = Proyecto::new();
    let mut hijo = hasta_cambio(&proyecto, &[]);
    let estado = terminado_en(&mut hijo, MAS_DE_UN_INTERVALO);
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    let lineas = lineas_de(hijo);
    assert_eq!(lineas.len(), 1, "{lineas:?}");
    assert!(lineas[0].starts_with("CANAL: primera pasada"), "{lineas:?}");
}

#[test]
fn dos_modos_a_la_vez_es_un_error() {
    let proyecto = Proyecto::new();
    let salida = orden(&proyecto, &["--una-vez", "--hasta-cambio"])
        .output()
        .unwrap();
    assert_eq!(salida.status.code(), Some(2));
    assert!(!proyecto.existe(RECUERDO));
}

#[test]
fn una_vida_que_no_es_un_entero_positivo_es_un_error() {
    let proyecto = Proyecto::new();
    for vida in ["0", "abc", "-5", "1.5", "+3"] {
        let salida = orden(&proyecto, &["--una-vez", "--vida-segundos", vida])
            .output()
            .unwrap();
        assert_eq!(salida.status.code(), Some(2), "{vida}");
        assert!(!proyecto.existe(RECUERDO), "{vida}");
    }
}

#[test]
fn hasta_cambio_termina_al_cumplir_su_vida_y_suelta_el_relevo() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    let mut hijo = hasta_cambio(&proyecto, &["--vida-segundos", "2"]);
    let lineas = lector(&mut hijo);
    let fin = lineas.recv_timeout(ESPERA_MAXIMA);
    let estado = terminado_en(&mut hijo, ESPERA_MAXIMA);
    assert_eq!(fin.as_deref(), Ok("VIGILANTE: fin por tiempo; rearma"));
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    assert!(!proyecto.existe(RELEVO));
}

#[test]
fn un_vigilante_nuevo_del_mismo_agente_releva_al_anterior() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    let mut primero = hasta_cambio(&proyecto, &[]);
    let lineas_del_primero = lector(&mut primero);
    assert!(esperar_archivo(&proyecto.ruta.join(RELEVO)));
    let marca_del_primero = fs::read_to_string(proyecto.ruta.join(RELEVO)).unwrap();
    let mut segundo = hasta_cambio(&proyecto, &[]);
    let mut esperado = Duration::ZERO;
    while fs::read_to_string(proyecto.ruta.join(RELEVO)).unwrap_or_default() == marca_del_primero
        && esperado < ESPERA_MAXIMA
    {
        thread::sleep(PASO_DE_ESPERA);
        esperado += PASO_DE_ESPERA;
    }
    let relevo = lineas_del_primero.recv_timeout(ESPERA_MAXIMA);
    let estado = terminado_en(&mut primero, ESPERA_MAXIMA);
    let segundo_sigue = segundo.try_wait().unwrap().is_none();
    let _ = segundo.kill();
    let _ = segundo.wait();
    assert_eq!(
        relevo.as_deref(),
        Ok("VIGILANTE: relevado por otro vigilante de Claude 2; no lo rearmes")
    );
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    assert!(segundo_sigue, "el relevo no puede parar al nuevo");
}

/// Envoltura que lanza el vigilante y espera a que termine, como la orden de una herramienta.
fn envoltura(proyecto: &Proyecto) -> Command {
    let mut orden = if cfg!(windows) {
        let mut orden = Command::new("powershell.exe");
        orden.args([
            "-NoProfile",
            "-Command",
            "& $env:VIGILANTE_RUTA --hasta-cambio --gestor $env:VIGILANTE_GESTOR --intervalo-segundos 1; exit 0",
        ]);
        orden
    } else {
        let mut orden = Command::new("sh");
        orden.args([
            "-c",
            "\"$VIGILANTE_RUTA\" --hasta-cambio --gestor \"$VIGILANTE_GESTOR\" --intervalo-segundos 1; exit 0",
        ]);
        orden
    };
    orden
        .env("VIGILANTE_RUTA", EJECUTABLE)
        .env("VIGILANTE_GESTOR", proyecto.gestor())
        .env("GESTOR_AGENTE", IDENTIDAD)
        .current_dir(&proyecto.ruta);
    orden
}

#[test]
fn si_muere_quien_lo_lanzo_el_vigilante_termina_y_no_queda_huerfano() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    let mut envoltura = envoltura(&proyecto)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let lineas = lector(&mut envoltura);
    assert!(esperar_archivo(&proyecto.ruta.join(RELEVO)));
    let _ = envoltura.kill();
    let _ = envoltura.wait();
    let fin = lineas.recv_timeout(ESPERA_MAXIMA);
    let cerrada = lineas.recv_timeout(ESPERA_MAXIMA);
    assert_eq!(
        fin.as_deref(),
        Ok("VIGILANTE: fin porque se cerró quien lo lanzó; relánzalo si sigues trabajando")
    );
    assert_eq!(cerrada, Err(mpsc::RecvTimeoutError::Disconnected));
    assert!(!proyecto.existe(RELEVO));
}

#[test]
fn persistente_sin_codex_dice_por_que_no_puede() {
    let proyecto = Proyecto::new();
    let salida = orden(&proyecto, &["--persistente", "--entrega", "codex"])
        .env_remove("CODEX_THREAD_ID")
        .output()
        .unwrap();
    assert_eq!(salida.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&salida.stdout).contains("CODEX_THREAD_ID"));
}

#[test]
fn persistente_entrega_cada_encargo_y_lo_anota() {
    let proyecto = Proyecto::new();
    let _ = una_pasada(&proyecto);
    let propio = ProcesosDelSistema::new()
        .datos(std::process::id())
        .unwrap()
        .marcado();
    // Como «Codex» se usa el propio vigilante: rechaza «queue» con código 2, así que la entrega
    // queda anotada como no acreditada. Basta para ver que se intenta, con qué y cuándo.
    let mut hijo = orden(
        &proyecto,
        &[
            "--persistente",
            "--entrega",
            "codex",
            "--hijo",
            "--dueno",
            &format!("{}:{}", propio.pid, propio.arranque),
            "--codex",
            EJECUTABLE,
            "--hilo",
            "00000000-0000-0000-0000-000000000000",
        ],
    )
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .unwrap();
    assert!(
        esperar_archivo(&proyecto.ruta.join(VIVO)),
        "no escribió vivo.json"
    );
    proyecto.escribir(
        ".gestor/canal/codex.md",
        "# Bandeja\n\nPara Claude 2: encargo.\n",
    );
    let avisos = proyecto.ruta.join(".gestor/claude2/vigilante/avisos.log");
    assert!(esperar_archivo(&avisos), "no intentó la entrega");
    let texto = fs::read_to_string(&avisos).unwrap();
    assert!(
        texto.contains("\tencargo\tcodex.md\tno acreditado: código Some(2)"),
        "{texto}"
    );
    // Un vigilante nuevo del mismo agente lo releva.
    proyecto.escribir(RELEVO, "otro\n");
    let estado = terminado_en(&mut hijo, ESPERA_MAXIMA);
    assert!(estado.is_some_and(|e| e.success()), "{estado:?}");
    assert!(!proyecto.existe(VIVO));
}

#[test]
fn anotar_hilo_deja_la_sesion_de_codex_en_el_proyecto() {
    let proyecto = Proyecto::new();
    let salida = orden(&proyecto, &["--anotar-hilo"])
        .env("CODEX_THREAD_ID", "01a2025b-3391")
        .output()
        .unwrap();
    assert_eq!(salida.status.code(), Some(0));
    let anotado =
        fs::read_to_string(proyecto.ruta.join(".gestor/claude2/vigilante/hilo.txt")).unwrap();
    assert_eq!(anotado.trim(), "01a2025b-3391");
}

#[test]
fn anotar_hilo_sin_sesion_de_codex_falla_y_lo_dice() {
    let proyecto = Proyecto::new();
    let salida = orden(&proyecto, &["--anotar-hilo"])
        .env_remove("CODEX_THREAD_ID")
        .output()
        .unwrap();
    assert_eq!(salida.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&salida.stdout).contains("CODEX_THREAD_ID"));
    assert!(!proyecto.existe(".gestor/claude2/vigilante/hilo.txt"));
}

#[test]
fn desde_terminal_termina_cuando_se_cierra_la_terminal_sin_codex() {
    let proyecto = Proyecto::new();
    // Hace de terminal un vigilante con vida corta, que no abre ningún Codex. Con su referencia ya
    // escrita espera su vida entera en lugar de terminar con la primera pasada.
    let referencia = orden(&proyecto, &["--una-vez"])
        .env("GESTOR_AGENTE", "Gemini 1")
        .output()
        .unwrap();
    assert_eq!(referencia.status.code(), Some(0));
    let mut terminal = orden(&proyecto, &["--hasta-cambio", "--vida-segundos", "2"])
        .env("GESTOR_AGENTE", "Gemini 1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let pid = terminal.id().to_string();
    let mut vigilante = orden(
        &proyecto,
        &[
            "--persistente",
            "--entrega",
            "codex",
            "--desde-terminal",
            &pid,
        ],
    )
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .unwrap();
    // La terminal se recoge al terminar, como hace quien la abrió: sin recoger, en Linux sigue
    // existiendo como proceso zombi.
    let _ = terminal.wait();
    let estado = terminado_en(&mut vigilante, ESPERA_MAXIMA);
    assert_eq!(estado.and_then(|e| e.code()), Some(0));
    assert!(!proyecto.existe(".gestor/claude2/vigilante/avisos.log"));
}

#[test]
fn desde_terminal_con_una_terminal_que_no_existe_no_vigila() {
    let proyecto = Proyecto::new();
    let salida = orden(
        &proyecto,
        &[
            "--persistente",
            "--entrega",
            "codex",
            "--desde-terminal",
            "4294967294",
        ],
    )
    .output()
    .unwrap();
    assert_eq!(salida.status.code(), Some(2));
}
