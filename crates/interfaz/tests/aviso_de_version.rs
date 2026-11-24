//! Barreras del módulo aviso_de_version, el único que habla con la biblioteca común
//! aviso-version-github, con una consulta falsa y sin red. Están los cinco casos mínimos de la
//! norma de integración de la biblioteca (desactivada, ajustes no válidos, versión mayor, misma
//! versión y fallo de red), sus constantes, y además qué se pide, que solo se avisa de una
//! versión más nueva, que el enlace sale de la configuración y nunca de la respuesta, que
//! Programator recibe la última publicada sea cual sea y que la consulta HTTPS usa el proxy que
//! se le da. También que el tiempo de espera de ajustes.json llega a la consulta, que la
//! identificación no válida se registra, qué dice cada línea del registro y que los avisos
//! llegan a la aplicación con la versión instalada con que se compararon.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use aviso_version_github::{
    ConsultaDeVersion, FalloDeComprobacion, OrigenDeLasVersiones, ProgramaQueConsulta,
};
use gestor_agentes::aplicacion::{
    AlmacenDeConfiguracion, Aplicacion, Dialogo, DiscoDeInterfaz, InspectorDeProyecto,
    RelojDeInterfaz,
};
use gestor_agentes::aviso_de_version::{
    self, AvisosAlArrancar, Registro, NOMBRE_DEL_PROGRAMA, REPOSITORIO_DE_VERSIONES_POR_DEFECTO,
    VERSION_INSTALADA,
};
use gestor_agentes::textos;
use nucleo::actualizaciones::{self, AvisoDeVersionNueva, Version};
use nucleo::ajustes::Ajustes;
use nucleo::configuracion::{ErrorDeConfiguracion, PerfilDeAgente, ProyectoRegistrado};
use nucleo::proyecto::estado::EstadoDelProyecto;

const API_DE_GESTOR: &str = "https://api.github.com/repos/celtidcs/gestor-agentes/releases/latest";
const API_DE_PROGRAMATOR: &str =
    "https://api.github.com/repos/celtidcs/Programator/releases/latest";

/// Consulta falsa que apunta lo que se le pide y responde lo que se le diga.
#[derive(Clone)]
struct ConsultaFalsa {
    respuesta: Option<String>,
    pedidos: Arc<Mutex<Vec<String>>>,
}

impl ConsultaFalsa {
    fn que_responde(respuesta: Option<&str>) -> Self {
        Self {
            respuesta: respuesta.map(str::to_owned),
            pedidos: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn pedidos(&self) -> Vec<String> {
        self.pedidos.lock().unwrap().clone()
    }
}

impl ConsultaDeVersion for ConsultaFalsa {
    fn pedir(&self, url: &str) -> Result<String, FalloDeComprobacion> {
        self.pedidos.lock().unwrap().push(url.to_owned());
        self.respuesta
            .clone()
            .ok_or_else(|| FalloDeComprobacion::Red("sin red".to_owned()))
    }
}

fn release(etiqueta: &str) -> String {
    format!(
        "{{\"tag_name\": \"{etiqueta}\", \"html_url\": \"https://github.com/celtidcs/gestor-agentes/releases/tag/{etiqueta}\"}}"
    )
}

/// Un registro que guarda lo que recibe.
fn registro_de_prueba() -> (Registro, Arc<Mutex<Vec<String>>>) {
    let apuntes = Arc::new(Mutex::new(Vec::new()));
    let destino = Arc::clone(&apuntes);
    let registro: Registro = Arc::new(move |fallo: &dyn std::fmt::Display| {
        destino.lock().unwrap().push(fallo.to_string());
    });
    (registro, apuntes)
}

/// Lo que pasó en una comprobación: si se fabricó la consulta, la versión del aviso que llegó y
/// lo que se registró.
#[derive(Debug, PartialEq, Eq)]
struct Resultado {
    fabricada: bool,
    aviso: Option<String>,
    registrado: Vec<String>,
}

/// Comprueba con los ajustes, la versión local y la consulta dados, y espera a que termine.
fn comprobar_con(
    ajustes: &Ajustes,
    version_local: &'static str,
    consulta: &ConsultaFalsa,
) -> Resultado {
    let fabricada = Cell::new(false);
    let (registro, apuntes) = registro_de_prueba();
    let falsa = consulta.clone();
    let canal = aviso_de_version::arrancar(
        ajustes,
        version_local,
        |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> {
            fabricada.set(true);
            Box::new(falsa)
        },
        registro,
        Box::new(|| {}),
    );
    // El hilo deja el aviso, si lo hay, y al terminar cierra el canal.
    let aviso = canal.recv().ok().map(|aviso| aviso.version);
    let registrado = apuntes.lock().unwrap().clone();
    Resultado {
        fabricada: fabricada.get(),
        aviso,
        registrado,
    }
}

/// Como comprobar_con, pero devuelve el aviso entero.
fn aviso_con(
    version_local: &'static str,
    consulta: &ConsultaFalsa,
) -> Option<actualizaciones::AvisoDeVersionNueva> {
    let (registro, _) = registro_de_prueba();
    let falsa = consulta.clone();
    aviso_de_version::arrancar(
        &Ajustes::default(),
        version_local,
        move |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> { Box::new(falsa) },
        registro,
        Box::new(|| {}),
    )
    .recv()
    .ok()
}

/// Ajustes leídos tal cual de un JSON, sin la validación de la configuración.
fn ajustes_sin_validar(json: &str) -> Ajustes {
    serde_json::from_str(json).unwrap()
}

// Los cinco casos mínimos de la norma de integración.

#[test]
fn desactivada_no_fabrica_la_consulta() {
    let ajustes = ajustes_sin_validar("{\"ComprobarActualizaciones\": false}");
    let consulta = ConsultaFalsa::que_responde(Some(&release("v999.0.0")));
    assert_eq!(
        comprobar_con(&ajustes, "3.1.0", &consulta),
        Resultado {
            fabricada: false,
            aviso: None,
            registrado: Vec::new()
        }
    );
    assert!(consulta.pedidos().is_empty());

    let fabricada = Cell::new(false);
    let de_programator = ConsultaFalsa::que_responde(Some(&release("v0.12.0")));
    let falsa = de_programator.clone();
    let versiones = aviso_de_version::arrancar_programator(
        &ajustes,
        |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> {
            fabricada.set(true);
            Box::new(falsa)
        },
        registro_de_prueba().0,
        Box::new(|| {}),
    );
    assert!(versiones.recv().is_err());
    assert!(!fabricada.get());
    assert!(de_programator.pedidos().is_empty());
}

#[test]
fn ajustes_no_validos_se_registran_sin_fabricar() {
    let ajustes = ajustes_sin_validar("{\"RepositorioDeActualizaciones\": \"no-vale\"}");
    let resultado = comprobar_con(&ajustes, "3.1.0", &ConsultaFalsa::que_responde(None));
    assert!(!resultado.fabricada && resultado.aviso.is_none());
    assert_eq!(resultado.registrado.len(), 1, "{:?}", resultado.registrado);
}

#[test]
fn version_mayor_llega_como_aviso() {
    let resultado = comprobar_con(
        &Ajustes::default(),
        "3.1.0",
        &ConsultaFalsa::que_responde(Some(&release("v3.2.0"))),
    );
    assert_eq!(
        resultado,
        Resultado {
            fabricada: true,
            aviso: Some("3.2.0".to_owned()),
            registrado: Vec::new()
        }
    );
}

#[test]
fn misma_version_no_avisa_ni_registra() {
    let resultado = comprobar_con(
        &Ajustes::default(),
        "3.1.0",
        &ConsultaFalsa::que_responde(Some(&release("v3.1.0"))),
    );
    assert_eq!(resultado.aviso, None);
    assert!(
        resultado.registrado.is_empty(),
        "{:?}",
        resultado.registrado
    );
}

#[test]
fn fallo_de_red_se_registra_sin_avisar() {
    let resultado = comprobar_con(
        &Ajustes::default(),
        "3.1.0",
        &ConsultaFalsa::que_responde(None),
    );
    assert_eq!(resultado.aviso, None);
    assert_eq!(resultado.registrado.len(), 1, "{:?}", resultado.registrado);
    assert!(
        resultado.registrado[0].contains("sin red"),
        "{:?}",
        resultado.registrado
    );
}

#[test]
fn las_constantes_y_los_valores_de_fabrica_son_validos() {
    assert!(ProgramaQueConsulta::nuevo(NOMBRE_DEL_PROGRAMA, VERSION_INSTALADA).is_ok());
    assert!(actualizaciones::repositorio_valido(
        REPOSITORIO_DE_VERSIONES_POR_DEFECTO
    ));
    assert_eq!(VERSION_INSTALADA, env!("CARGO_PKG_VERSION"));
    assert_eq!(textos::version_visible("3.1.0"), "v3.1.0");
}

// Barreras propias de Gestor Agentes.

#[test]
fn se_pide_la_ultima_release_del_repositorio_de_los_ajustes() {
    let consulta = ConsultaFalsa::que_responde(None);
    let _ = comprobar_con(&Ajustes::default(), "3.1.0", &consulta);
    assert_eq!(consulta.pedidos(), [API_DE_GESTOR.to_owned()]);

    let otro = ajustes_sin_validar("{\"RepositorioDeActualizaciones\": \"otro/gestor\"}");
    let consulta = ConsultaFalsa::que_responde(None);
    let _ = comprobar_con(&otro, "3.1.0", &consulta);
    assert_eq!(
        consulta.pedidos(),
        ["https://api.github.com/repos/otro/gestor/releases/latest".to_owned()]
    );
}

#[test]
fn solo_se_avisa_de_una_version_mas_nueva() {
    for (respuesta, se_registra) in [
        (release("v3.0.0"), false),
        (release("v3.2.0-rc.1"), true),
        ("<html>error</html>".to_owned(), true),
    ] {
        let resultado = comprobar_con(
            &Ajustes::default(),
            "3.1.0",
            &ConsultaFalsa::que_responde(Some(&respuesta)),
        );
        assert_eq!(resultado.aviso, None, "{respuesta}");
        assert_eq!(!resultado.registrado.is_empty(), se_registra, "{respuesta}");
    }
    let resultado = comprobar_con(
        &Ajustes::default(),
        "no es versión",
        &ConsultaFalsa::que_responde(Some(&release("v3.2.0"))),
    );
    assert_eq!(resultado.aviso, None);
    assert_eq!(resultado.registrado.len(), 1);
}

#[test]
fn el_enlace_sale_de_la_configuracion_y_nunca_de_la_respuesta() {
    let esperado = "https://github.com/celtidcs/gestor-agentes/releases/tag/v9.0.0";
    for cuerpo in [
        "{\"tag_name\": \"v9.0.0\", \"html_url\": \"https://ejemplo.com/trampa\"}",
        "{\"tag_name\": \"v9.0.0\", \"html_url\": \"https://github.com/otro/gestor-agentes/releases/tag/v9.0.0\"}",
        "{\"tag_name\": \"v9.0.0\"}",
    ] {
        let aviso = aviso_con("3.1.0", &ConsultaFalsa::que_responde(Some(cuerpo))).expect("hay aviso");
        assert_eq!(aviso.enlace, esperado, "{cuerpo}");
    }
}

#[test]
fn el_aviso_llega_a_la_ventana_con_su_texto() {
    let avisada = Arc::new(Mutex::new(false));
    let marca = Arc::clone(&avisada);
    let consulta = ConsultaFalsa::que_responde(Some(&release("v999.0.0")));
    let avisos = aviso_de_version::arrancar(
        &Ajustes::default(),
        VERSION_INSTALADA,
        move |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> { Box::new(consulta) },
        registro_de_prueba().0,
        Box::new(move || *marca.lock().unwrap() = true),
    );
    let aviso = avisos.recv_timeout(Duration::from_secs(10)).expect("llega");
    assert_eq!(aviso.version, "999.0.0");
    let texto = textos::actualizacion_disponible(&aviso.version, VERSION_INSTALADA, &aviso.enlace);
    assert!(texto.contains("999.0.0") && texto.contains(VERSION_INSTALADA));
    assert!(texto.contains("https://github.com/celtidcs/gestor-agentes/releases/tag/v999.0.0"));
    esperar_a(|| *avisada.lock().unwrap());
}

/// Pide la última versión de Programator con la respuesta dada.
fn programator_con(respuesta: Option<&str>) -> (Option<String>, Vec<String>, Vec<String>) {
    let consulta = ConsultaFalsa::que_responde(respuesta);
    let falsa = consulta.clone();
    let (registro, apuntes) = registro_de_prueba();
    let versiones = aviso_de_version::arrancar_programator(
        &Ajustes::default(),
        move |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> { Box::new(falsa) },
        registro,
        Box::new(|| {}),
    );
    let version = versiones.recv().ok().map(|v| v.to_string());
    let registrado = apuntes.lock().unwrap().clone();
    (version, consulta.pedidos(), registrado)
}

#[test]
fn programator_recibe_la_ultima_publicada_de_su_repositorio_sea_cual_sea() {
    for (etiqueta, esperada) in [("v0.0.1", "0.0.1"), ("v3.2.0", "3.2.0"), ("0.1.0", "0.1.0")] {
        let (version, pedidos, registrado) = programator_con(Some(&release(etiqueta)));
        assert_eq!(version.as_deref(), Some(esperada), "{etiqueta}");
        assert_eq!(pedidos, [API_DE_PROGRAMATOR.to_owned()]);
        assert!(registrado.is_empty(), "{registrado:?}");
    }
}

#[test]
fn una_respuesta_rara_de_programator_no_da_version_y_se_registra() {
    for respuesta in [
        None,
        Some(String::new()),
        Some("no es JSON".to_owned()),
        Some("{\"message\": \"Not Found\"}".to_owned()),
        Some("{\"tag_name\": 3}".to_owned()),
        Some(release("v3.2.0-beta")),
    ] {
        let (version, _, registrado) = programator_con(respuesta.as_deref());
        assert_eq!(version, None, "{respuesta:?}");
        assert_eq!(registrado.len(), 1, "{respuesta:?}: {registrado:?}");
    }
}

#[test]
fn la_consulta_https_usa_el_proxy_que_se_le_da() {
    // Un proxy en un puerto cerrado de esta máquina: si la consulta lo usa, falla al conectar con
    // él, sin salir a la red.
    let proxy = aviso_version_github::ureq::Proxy::new("http://127.0.0.1:1").expect("proxy válido");
    let ajustes = actualizaciones::ajustes_de_version(REPOSITORIO_DE_VERSIONES_POR_DEFECTO, 5);
    let origen = OrigenDeLasVersiones::desde_ajustes(&ajustes).expect("ajustes válidos");
    let (registro, apuntes) = registro_de_prueba();
    let https =
        aviso_de_version::consulta_https(&origen, Some(proxy), VERSION_INSTALADA, &registro);
    assert!(apuntes.lock().unwrap().is_empty());
    assert!(https.pedir(API_DE_GESTOR).is_err());
}

/// Espera, como mucho dos segundos, a que se cumpla una condición que pone otro hilo.
fn esperar_a(condicion: impl Fn() -> bool) {
    for _ in 0..200 {
        if condicion() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("la ventana no recibió el aviso");
}

/// Consulta falsa que apunta el nombre del hilo desde el que se le pide.
struct ConsultaQueMiraElHilo(Arc<Mutex<Option<String>>>);

impl ConsultaDeVersion for ConsultaQueMiraElHilo {
    fn pedir(&self, _url: &str) -> Result<String, FalloDeComprobacion> {
        *self.0.lock().unwrap() = std::thread::current().name().map(str::to_owned);
        Ok(release("v0.0.1"))
    }
}

#[test]
fn cada_comprobacion_va_en_su_hilo_con_nombre() {
    let de_gestor = Arc::new(Mutex::new(None));
    let visto = Arc::clone(&de_gestor);
    let _ = aviso_de_version::arrancar(
        &Ajustes::default(),
        "3.1.0",
        move |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> {
            Box::new(ConsultaQueMiraElHilo(visto))
        },
        registro_de_prueba().0,
        Box::new(|| {}),
    )
    .recv();
    assert_eq!(
        de_gestor.lock().unwrap().as_deref(),
        Some("aviso-de-version")
    );

    let de_programator = Arc::new(Mutex::new(None));
    let visto = Arc::clone(&de_programator);
    let _ = aviso_de_version::arrancar_programator(
        &Ajustes::default(),
        move |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> {
            Box::new(ConsultaQueMiraElHilo(visto))
        },
        registro_de_prueba().0,
        Box::new(|| {}),
    )
    .recv();
    assert_eq!(
        de_programator.lock().unwrap().as_deref(),
        Some("aviso-de-version-programator")
    );
}

// Barreras de la 3.8.1.

#[test]
fn el_tiempo_de_espera_de_los_ajustes_llega_a_la_consulta() {
    let ajustes = ajustes_sin_validar("{\"TiempoMaximoActualizacionSegundos\": 7}");
    let esperado = |repositorio: &str| {
        OrigenDeLasVersiones::desde_ajustes(&actualizaciones::ajustes_de_version(repositorio, 7))
            .unwrap()
    };
    let visto = Arc::new(Mutex::new(Vec::new()));
    let destino = Arc::clone(&visto);
    let avisos = AvisosAlArrancar::arrancar(
        &ajustes,
        "3.1.0",
        move |origen: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> {
            destino.lock().unwrap().push(origen.clone());
            Box::new(ConsultaFalsa::que_responde(None))
        },
        &registro_de_prueba().0,
        || Box::new(|| {}),
    );
    drop(avisos);
    assert_eq!(
        *visto.lock().unwrap(),
        [
            esperado(REPOSITORIO_DE_VERSIONES_POR_DEFECTO),
            esperado("celtidcs/Programator")
        ]
    );
    assert_ne!(
        esperado(REPOSITORIO_DE_VERSIONES_POR_DEFECTO),
        OrigenDeLasVersiones::desde_ajustes(&actualizaciones::ajustes_de_version(
            REPOSITORIO_DE_VERSIONES_POR_DEFECTO,
            5
        ))
        .unwrap(),
        "el valor de fábrica no puede pasar por el de los ajustes"
    );
}

#[test]
fn la_identificacion_no_valida_se_registra_y_la_consulta_sale_igual() {
    let proxy = aviso_version_github::ureq::Proxy::new("http://127.0.0.1:1").expect("proxy válido");
    let ajustes = actualizaciones::ajustes_de_version(REPOSITORIO_DE_VERSIONES_POR_DEFECTO, 5);
    let origen = OrigenDeLasVersiones::desde_ajustes(&ajustes).expect("ajustes válidos");
    let (registro, apuntes) = registro_de_prueba();
    let https = aviso_de_version::consulta_https(&origen, Some(proxy), "no es versión", &registro);
    let apuntes = apuntes.lock().unwrap().clone();
    assert_eq!(apuntes.len(), 1, "{apuntes:?}");
    assert!(
        apuntes[0].starts_with(textos::IDENTIFICACION_NO_VALIDA),
        "{apuntes:?}"
    );
    assert!(
        https.pedir(API_DE_GESTOR).is_err(),
        "la consulta sale por el proxy"
    );
}

#[test]
fn cada_fallo_dice_que_comprobacion_fallo_y_con_que_repositorio() {
    let resultado = comprobar_con(
        &Ajustes::default(),
        "3.1.0",
        &ConsultaFalsa::que_responde(None),
    );
    assert_eq!(resultado.registrado.len(), 1);
    assert!(
        resultado.registrado[0].starts_with("de Gestor Agentes (celtidcs/gestor-agentes): "),
        "{:?}",
        resultado.registrado
    );

    let (_, _, registrado) = programator_con(None);
    assert_eq!(registrado.len(), 1);
    assert!(
        registrado[0].starts_with("de Programator (celtidcs/Programator): "),
        "{registrado:?}"
    );

    let ajustes = ajustes_sin_validar("{\"RepositorioDeProgramator\": \"no-vale\"}");
    let (registro, apuntes) = registro_de_prueba();
    let versiones = aviso_de_version::arrancar_programator(
        &ajustes,
        |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> {
            Box::new(ConsultaFalsa::que_responde(None))
        },
        registro,
        Box::new(|| {}),
    );
    assert!(versiones.recv().is_err());
    let apuntes = apuntes.lock().unwrap().clone();
    assert_eq!(apuntes.len(), 1);
    assert!(
        apuntes[0].starts_with("de Programator (no-vale): "),
        "{apuntes:?}"
    );
}

#[test]
fn la_linea_del_registro_lleva_la_fecha_y_el_comienzo_documentado() {
    // El comienzo lo citan services.md, architecture.md y la guía de resolución de problemas.
    assert_eq!(
        aviso_de_version::linea_de_registro(
            "2026-11-23T16:30:00.0000000+01:00",
            &"de Programator (celtidcs/Programator): sin red"
        ),
        "2026-11-23T16:30:00.0000000+01:00 Gestor Agentes, comprobación de versión nueva de \
         Programator (celtidcs/Programator): sin red"
    );
}

/// Servicios mínimos para una aplicación sin proyectos ni perfiles.
struct ServiciosVacios;

impl AlmacenDeConfiguracion for ServiciosVacios {
    fn guardar_proyectos(&self, _: &[ProyectoRegistrado]) -> Result<(), ErrorDeConfiguracion> {
        Ok(())
    }
    fn guardar_perfiles(&self, _: &[PerfilDeAgente]) -> Result<(), ErrorDeConfiguracion> {
        Ok(())
    }
    fn ruta_perfiles(&self) -> PathBuf {
        PathBuf::from("/datos/perfiles-agentes.json")
    }
}

impl RelojDeInterfaz for ServiciosVacios {
    fn ahora(&self) -> String {
        "2026-11-23T12:00:00+01:00".to_owned()
    }
    fn mostrar_fecha(&self, iso: &str) -> String {
        iso.to_owned()
    }
}

impl DiscoDeInterfaz for ServiciosVacios {
    fn existe_carpeta(&self, _: &Path) -> bool {
        true
    }
    fn existe_archivo(&self, _: &Path) -> bool {
        true
    }
    fn carpeta_personal(&self) -> Option<PathBuf> {
        None
    }
}

impl InspectorDeProyecto for ServiciosVacios {
    fn estado_del_proyecto(&self, _: &Path) -> EstadoDelProyecto {
        EstadoDelProyecto::Configurado
    }
}

fn aplicacion_vacia() -> Aplicacion {
    let aplicacion = Aplicacion::new(Box::new(ServiciosVacios), Ok(Vec::new()), Ok(Vec::new()));
    assert!(aplicacion.dialogo().is_none());
    assert!(aplicacion.programator_publicado().is_none());
    aplicacion
}

/// Texto del diálogo informativo que haya en la aplicación, si lo hay.
fn dialogo_informativo(aplicacion: &Aplicacion) -> Option<(String, String)> {
    match aplicacion.dialogo()? {
        Dialogo::Informacion { titulo, texto } => Some((titulo.clone(), texto.clone())),
        _ => None,
    }
}

#[test]
fn el_aviso_llega_a_la_aplicacion_con_la_instalada_que_se_comparo() {
    let mut aplicacion = aplicacion_vacia();
    let mut avisos = AvisosAlArrancar::arrancar(
        &Ajustes::default(),
        "1.2.3",
        |_: &OrigenDeLasVersiones| -> Box<dyn ConsultaDeVersion> {
            Box::new(ConsultaFalsa::que_responde(Some(&release("v999.0.0"))))
        },
        &registro_de_prueba().0,
        || Box::new(|| {}),
    );
    for _ in 0..1000 {
        avisos.entregar(&mut aplicacion);
        if !avisos.esperando() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!avisos.esperando(), "las dos comprobaciones han terminado");
    assert_eq!(
        dialogo_informativo(&aplicacion),
        Some((
            textos::TITULO_ACTUALIZACION.to_owned(),
            textos::actualizacion_disponible(
                "999.0.0",
                "1.2.3",
                "https://github.com/celtidcs/gestor-agentes/releases/tag/v999.0.0"
            )
        ))
    );
    assert_eq!(
        aplicacion.programator_publicado(),
        Version::desde_etiqueta("v999.0.0")
    );
}

fn aviso(version: &str) -> AvisoDeVersionNueva {
    AvisoDeVersionNueva {
        version: version.to_owned(),
        enlace: format!("https://github.com/celtidcs/gestor-agentes/releases/tag/v{version}"),
    }
}

#[test]
fn cada_aviso_se_entrega_una_vez_y_sin_esperar() {
    let mut aplicacion = aplicacion_vacia();
    let (a_la_version, de_version) = mpsc::channel();
    let (a_programator, de_programator) = mpsc::channel();
    let mut avisos = AvisosAlArrancar::desde_canales("3.1.0", de_version, de_programator);

    // Sin nada todavía, no se entrega nada y se sigue mirando.
    avisos.entregar(&mut aplicacion);
    assert!(dialogo_informativo(&aplicacion).is_none());
    assert!(aplicacion.programator_publicado().is_none());
    assert!(avisos.esperando());

    // Llega el aviso: un diálogo, y ese canal se deja de mirar.
    a_la_version.send(aviso("3.2.0")).unwrap();
    avisos.entregar(&mut aplicacion);
    let (_, texto) = dialogo_informativo(&aplicacion).expect("hay diálogo");
    assert_eq!(
        texto,
        textos::actualizacion_disponible(
            "3.2.0",
            "3.1.0",
            "https://github.com/celtidcs/gestor-agentes/releases/tag/v3.2.0"
        )
    );
    assert!(a_la_version.send(aviso("3.3.0")).is_err(), "ya no se mira");
    assert!(avisos.esperando(), "falta Programator");

    // Programator termina sin nada: se deja de mirar sin cambiar nada.
    drop(a_programator);
    avisos.entregar(&mut aplicacion);
    assert!(aplicacion.programator_publicado().is_none());
    assert!(!avisos.esperando());

    // Y la última de Programator, cuando llega, se anota.
    let mut aplicacion = aplicacion_vacia();
    let (_a_la_version, de_version) = mpsc::channel();
    let (a_programator, de_programator) = mpsc::channel();
    let mut avisos = AvisosAlArrancar::desde_canales("3.1.0", de_version, de_programator);
    let publicada = Version::desde_etiqueta("v0.12.0").unwrap();
    a_programator.send(publicada).unwrap();
    avisos.entregar(&mut aplicacion);
    assert_eq!(aplicacion.programator_publicado(), Some(publicada));
    assert!(dialogo_informativo(&aplicacion).is_none());
}

#[test]
fn sin_comprobaciones_no_se_espera_nada() {
    let mut aplicacion = aplicacion_vacia();
    let mut avisos = AvisosAlArrancar::ninguno();
    assert!(!avisos.esperando());
    avisos.entregar(&mut aplicacion);
    assert!(aplicacion.dialogo().is_none());
}
