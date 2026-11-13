//! Barreras de la consulta de versión de la interfaz: con el consultor
//! inyectado, sin red. Se comprueba qué se pide, con qué cabecera y que solo se avisa de una versión
//! más nueva, y que con la consulta desactivada no se pide nada.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gestor_agentes::actualizaciones::{
    self, Consulta, ConsultorDePublicaciones, VERSION_INSTALADA,
};
use gestor_agentes::textos;
use nucleo::ajustes::Ajustes;

/// Consultor que apunta lo que se le pide y responde lo que se le diga.
struct ConsultorFalso {
    respuesta: Option<String>,
    pedidos: RefCell<Vec<(String, String, Duration)>>,
}

impl ConsultorFalso {
    fn que_responde(respuesta: Option<&str>) -> Self {
        Self {
            respuesta: respuesta.map(str::to_owned),
            pedidos: RefCell::new(Vec::new()),
        }
    }
}

impl ConsultorDePublicaciones for ConsultorFalso {
    fn consultar(&self, direccion: &str, agente: &str, tiempo: Duration) -> Option<String> {
        self.pedidos
            .borrow_mut()
            .push((direccion.to_owned(), agente.to_owned(), tiempo));
        self.respuesta.clone()
    }
}

fn release(etiqueta: &str) -> String {
    format!(
        "{{\"tag_name\": \"{etiqueta}\", \"html_url\": \"https://github.com/celtidcs/gestor-agentes/releases/tag/{etiqueta}\"}}"
    )
}

fn consulta() -> Consulta {
    Consulta::desde(&Ajustes::default()).expect("activada por defecto")
}

#[test]
fn se_pide_la_ultima_release_con_un_agente_de_usuario_generico() {
    let consultor = ConsultorFalso::que_responde(None);

    let _ = actualizaciones::comprobar(&consulta(), "3.1.0", &consultor);

    assert_eq!(
        consultor.pedidos.borrow().as_slice(),
        [(
            "https://api.github.com/repos/celtidcs/gestor-agentes/releases/latest".to_owned(),
            "gestor-agentes/3.1.0".to_owned(),
            Duration::from_secs(5),
        )]
    );
}

#[test]
fn solo_se_avisa_de_una_version_mas_nueva() {
    let nueva = ConsultorFalso::que_responde(Some(&release("v3.2.0")));
    let igual = ConsultorFalso::que_responde(Some(&release("v3.1.0")));
    let vieja = ConsultorFalso::que_responde(Some(&release("v3.0.0")));
    let sin_red = ConsultorFalso::que_responde(None);
    let rara = ConsultorFalso::que_responde(Some("<html>error</html>"));

    let aviso = actualizaciones::comprobar(&consulta(), "3.1.0", &nueva);
    assert_eq!(
        aviso.map(|p| p.version.to_string()).as_deref(),
        Some("3.2.0")
    );
    for consultor in [&igual, &vieja, &sin_red, &rara] {
        assert_eq!(
            actualizaciones::comprobar(&consulta(), "3.1.0", consultor),
            None
        );
    }
}

#[test]
fn con_la_consulta_desactivada_no_se_pide_nada() {
    let ajustes: Ajustes = ajustes_desde_json("{\"ComprobarActualizaciones\": false}");
    assert_eq!(Consulta::desde(&ajustes), None);

    struct Prohibido(Arc<Mutex<u32>>);
    impl ConsultorDePublicaciones for Prohibido {
        fn consultar(&self, _: &str, _: &str, _: Duration) -> Option<String> {
            *self.0.lock().unwrap() += 1;
            None
        }
    }
    let veces = Arc::new(Mutex::new(0));
    let avisos = actualizaciones::en_segundo_plano(
        Consulta::desde(&ajustes),
        Box::new(Prohibido(Arc::clone(&veces))),
        Box::new(|| {}),
    );

    assert!(avisos.recv_timeout(Duration::from_secs(2)).is_err());
    assert_eq!(*veces.lock().unwrap(), 0);
}

#[test]
fn en_segundo_plano_llega_la_version_nueva_y_se_avisa_a_la_ventana() {
    struct Responde;
    impl ConsultorDePublicaciones for Responde {
        fn consultar(&self, _: &str, _: &str, _: Duration) -> Option<String> {
            Some(release("v999.0.0"))
        }
    }
    let avisada = Arc::new(Mutex::new(false));
    let marca = Arc::clone(&avisada);

    let avisos = actualizaciones::en_segundo_plano(
        Some(consulta()),
        Box::new(Responde),
        Box::new(move || *marca.lock().unwrap() = true),
    );

    let publicacion = avisos.recv_timeout(Duration::from_secs(10)).expect("llega");
    assert_eq!(publicacion.version.to_string(), "999.0.0");
    let texto = textos::actualizacion_disponible(
        &publicacion.version.to_string(),
        VERSION_INSTALADA,
        &publicacion.enlace,
    );
    assert!(texto.contains("999.0.0") && texto.contains(VERSION_INSTALADA));
    assert!(texto.contains("https://github.com/celtidcs/gestor-agentes/releases/tag/v999.0.0"));
    esperar_a(|| *avisada.lock().unwrap());
}

#[test]
fn la_version_visible_es_la_de_cargo_toml() {
    assert_eq!(VERSION_INSTALADA, env!("CARGO_PKG_VERSION"));
    assert_eq!(textos::version_visible("3.1.0"), "v3.1.0");
}

/// Ajustes leídos de un JSON, como los lee la configuración.
fn ajustes_desde_json(json: &str) -> Ajustes {
    let carpeta = std::env::temp_dir().join(format!(
        "gestor-pruebas-ajustes-{}-{}",
        std::process::id(),
        json.len()
    ));
    std::fs::create_dir_all(&carpeta).unwrap();
    std::fs::write(carpeta.join("ajustes.json"), json).unwrap();
    let ajustes = nucleo::configuracion::Configuracion::new(&carpeta).cargar_ajustes();
    let _ = std::fs::remove_dir_all(&carpeta);
    ajustes.unwrap()
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
