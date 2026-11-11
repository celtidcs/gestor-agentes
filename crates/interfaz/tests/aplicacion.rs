//! Barreras del estado de la ventana y de los trabajos, con dobles en lugar del disco, el reloj y
//! los procesos.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gestor_agentes::aplicacion::{Aplicacion, Dialogo, Servicios};
use gestor_agentes::perfiles::Disponibilidad;
use gestor_agentes::trabajos::{self, Lanzamiento, Motor, ResultadoDeTrabajo, Trabajador, Trabajo};
use lanzador::{ErrorDeLanzamiento, Herramienta, PeticionDeLanzamiento, PlanDeLanzamiento};
use nucleo::agentes::IdentidadAgente;
use nucleo::configuracion::{
    ErrorDeConfiguracion, HerramientaDelPerfil, PerfilDeAgente, ProyectoRegistrado,
};
use nucleo::cuentas::CarpetaPersonal;
use nucleo::proyecto::estado::EstadoDelProyecto;
use nucleo::proyecto::informe::{InformeDeInicializacion, TipoDeCambio};

const AHORA: &str = "2026-11-10T18:00:00.0000000+01:00";
const RUTA_PERFILES: &str = "/datos/perfiles-agentes.json";
const CASA: &str = "/home/u";

/// Disco de mentira: carpetas y archivos que existen, listas guardadas y si guardar falla.
#[derive(Default)]
struct Disco {
    carpetas: BTreeSet<String>,
    archivos: BTreeSet<String>,
    guardados: Vec<Vec<ProyectoRegistrado>>,
    perfiles_guardados: Vec<Vec<PerfilDeAgente>>,
    guardar_falla: bool,
    guardar_perfiles_falla: bool,
}

struct ServiciosDeMentira(Rc<RefCell<Disco>>);

impl Servicios for ServiciosDeMentira {
    fn guardar_proyectos(
        &self,
        proyectos: &[ProyectoRegistrado],
    ) -> Result<(), ErrorDeConfiguracion> {
        let mut disco = self.0.borrow_mut();
        if disco.guardar_falla {
            return Err(ErrorDeConfiguracion::Disco {
                ruta: PathBuf::from("/datos/config.json"),
                origen: std::io::Error::other("disco lleno"),
            });
        }
        disco.guardados.push(proyectos.to_vec());
        Ok(())
    }

    fn estado_del_proyecto(&self, carpeta: &Path) -> EstadoDelProyecto {
        if carpeta.ends_with("configurado") {
            EstadoDelProyecto::Configurado
        } else {
            EstadoDelProyecto::SinConfigurar
        }
    }

    fn existe_carpeta(&self, carpeta: &Path) -> bool {
        carpeta
            .to_str()
            .is_some_and(|ruta| self.0.borrow().carpetas.contains(ruta))
    }

    fn ahora(&self) -> String {
        AHORA.to_owned()
    }

    fn mostrar_fecha(&self, iso: &str) -> String {
        format!("visible:{iso}")
    }

    fn ruta_perfiles(&self) -> PathBuf {
        PathBuf::from(RUTA_PERFILES)
    }

    fn guardar_perfiles(&self, perfiles: &[PerfilDeAgente]) -> Result<(), ErrorDeConfiguracion> {
        let mut disco = self.0.borrow_mut();
        if disco.guardar_perfiles_falla {
            return Err(ErrorDeConfiguracion::Disco {
                ruta: PathBuf::from(RUTA_PERFILES),
                origen: std::io::Error::other("disco lleno"),
            });
        }
        disco.perfiles_guardados.push(perfiles.to_vec());
        Ok(())
    }

    fn existe_archivo(&self, ruta: &Path) -> bool {
        ruta.to_str()
            .is_some_and(|ruta| self.0.borrow().archivos.contains(ruta))
    }

    fn carpeta_personal(&self) -> Option<PathBuf> {
        Some(PathBuf::from(CASA))
    }
}

fn proyecto(nombre: &str, ruta: &str) -> ProyectoRegistrado {
    ProyectoRegistrado::new(nombre, ruta, "2026-01-01T00:00:00+01:00")
}

fn perfiles() -> Vec<PerfilDeAgente> {
    vec![
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 2),
        PerfilDeAgente::new(HerramientaDelPerfil::Gemini, 1),
        PerfilDeAgente::new(HerramientaDelPerfil::Programator, 1),
    ]
}

/// Aplicación recién creada, antes de que termine la detección de agentes.
fn montar_sin_detectar(proyectos: Vec<ProyectoRegistrado>) -> (Aplicacion, Rc<RefCell<Disco>>) {
    let disco = Rc::new(RefCell::new(Disco::default()));
    for registro in &proyectos {
        disco
            .borrow_mut()
            .carpetas
            .insert(registro.ruta().to_owned());
    }
    let aplicacion = Aplicacion::new(
        Box::new(ServiciosDeMentira(Rc::clone(&disco))),
        Ok(proyectos),
        Ok(perfiles()),
    );
    (aplicacion, disco)
}

fn disponible(programa: &str) -> Disponibilidad {
    Disponibilidad::Disponible(PathBuf::from(programa))
}

/// Termina una detección con los perfiles de [`perfiles`] y esa disponibilidad, sin cuentas nuevas.
fn detectado(aplicacion: &mut Aplicacion, disponibilidad: Vec<Disponibilidad>) {
    aplicacion.terminar(ResultadoDeTrabajo::Detectado {
        perfiles: perfiles(),
        disponibilidad,
        anadidos: 0,
    });
}

/// Aplicación con la detección terminada y todos los agentes encontrados.
fn montar(proyectos: Vec<ProyectoRegistrado>) -> (Aplicacion, Rc<RefCell<Disco>>) {
    let (mut aplicacion, disco) = montar_sin_detectar(proyectos);
    detectado(
        &mut aplicacion,
        vec![
            disponible("/bin/claude"),
            disponible("/bin/gemini"),
            disponible("/opt/programator"),
        ],
    );
    (aplicacion, disco)
}

fn nombres(aplicacion: &Aplicacion) -> Vec<&str> {
    aplicacion
        .filas()
        .iter()
        .map(|fila| fila.registro.nombre())
        .collect()
}

fn error_de_formato() -> ErrorDeConfiguracion {
    ErrorDeConfiguracion::FormatoInvalido {
        ruta: PathBuf::from("/datos/config.json"),
        motivo: String::from("se esperaba una lista"),
    }
}

// Carga

#[test]
fn al_cargar_ordena_por_nombre_y_cuenta_los_proyectos() {
    let (aplicacion, disco) = montar(vec![
        proyecto("beta", "/p/beta"),
        proyecto("Álvaro", "/p/alvaro"),
        proyecto("alba", "/p/configurado"),
    ]);
    assert_eq!(nombres(&aplicacion), ["alba", "Álvaro", "beta"]);
    assert_eq!(aplicacion.texto_de_estado(), "3 proyecto(s) registrado(s)");
    assert!(disco.borrow().guardados.is_empty(), "cargar no guarda nada");
    let primera = &aplicacion.filas()[0];
    assert_eq!(primera.estado, EstadoDelProyecto::Configurado);
    assert_eq!(primera.fecha_visible, "visible:2026-01-01T00:00:00+01:00");
}

#[test]
fn una_configuracion_danada_no_impide_arrancar() {
    let disco = Rc::new(RefCell::new(Disco::default()));
    let aplicacion = Aplicacion::new(
        Box::new(ServiciosDeMentira(disco)),
        Err(error_de_formato()),
        Ok(perfiles()),
    );
    assert!(aplicacion.filas().is_empty());
    assert_eq!(aplicacion.perfiles().len(), 3);
    assert!(aplicacion.texto_de_estado().contains("/datos/config.json"));
}

#[test]
fn unos_perfiles_danados_dejan_la_lista_y_lo_dicen() {
    let disco = Rc::new(RefCell::new(Disco::default()));
    let aplicacion = Aplicacion::new(
        Box::new(ServiciosDeMentira(disco)),
        Ok(vec![proyecto("uno", "/p/uno")]),
        Err(error_de_formato()),
    );
    assert_eq!(aplicacion.filas().len(), 1);
    assert!(aplicacion.perfiles().is_empty());
    assert!(aplicacion
        .texto_de_estado()
        .starts_with("Configuración no válida"));
}

// Botones

#[test]
fn sin_seleccion_los_botones_de_proyecto_estan_apagados() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    assert!(aplicacion.edicion_activa());
    assert!(!aplicacion.acciones_activas());
    assert_eq!(aplicacion.abrir_con(0), None);
    aplicacion.seleccionar("/p/uno");
    assert!(aplicacion.acciones_activas());
}

#[test]
fn seleccionar_una_ruta_que_no_esta_no_selecciona_nada() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.seleccionar("/p/otro");
    assert_eq!(aplicacion.seleccion(), None);
}

#[test]
fn con_un_trabajo_en_curso_todo_queda_apagado() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.seleccionar("/p/uno");
    assert!(aplicacion.abrir_con(0).is_some());
    assert!(aplicacion.ocupada());
    assert!(!aplicacion.edicion_activa());
    assert!(!aplicacion.acciones_activas());
    assert_eq!(aplicacion.abrir_con(0), None);
    assert_eq!(aplicacion.pedir_carpeta(), None);
    aplicacion.quitar("/p/uno");
    assert_eq!(
        aplicacion.filas().len(),
        1,
        "no se quita nada mientras se trabaja"
    );
}

#[test]
fn con_un_dialogo_abierto_no_se_edita_la_lista() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.informar("Aviso", String::from("texto"));
    assert!(!aplicacion.edicion_activa());
    assert_eq!(aplicacion.pedir_carpeta(), None);
}

// Agregar

fn agregar(aplicacion: &mut Aplicacion, carpeta: &str) {
    assert_eq!(aplicacion.pedir_carpeta(), Some(Trabajo::ElegirCarpeta));
    aplicacion.terminar(ResultadoDeTrabajo::CarpetaElegida(Some(PathBuf::from(
        carpeta,
    ))));
}

#[test]
fn agregar_un_proyecto_lo_guarda_ordenado_con_la_fecha_actual() {
    let (mut aplicacion, disco) = montar(vec![proyecto("zeta", "/p/zeta")]);
    agregar(&mut aplicacion, "/p/alfa/");
    assert_eq!(nombres(&aplicacion), ["alfa", "zeta"]);
    assert_eq!(aplicacion.texto_de_estado(), "Proyecto agregado: alfa");
    assert!(!aplicacion.ocupada());
    let guardado = disco.borrow().guardados.last().cloned().unwrap_or_default();
    assert_eq!(guardado.len(), 2);
    assert_eq!(guardado[0].ruta(), "/p/alfa/");
    assert_eq!(guardado[0].ultima_apertura(), AHORA);
}

#[test]
fn agregar_una_ruta_repetida_sin_distinguir_mayusculas_avisa_y_no_guarda() {
    let (mut aplicacion, disco) = montar(vec![proyecto("Uno", r"C:\P\Uno")]);
    agregar(&mut aplicacion, r"c:\p\UNO");
    assert_eq!(aplicacion.filas().len(), 1);
    assert!(disco.borrow().guardados.is_empty());
    assert_eq!(
        aplicacion.dialogo(),
        Some(&Dialogo::Informacion {
            titulo: String::from("Proyecto duplicado"),
            texto: String::from("Este proyecto ya está en la lista."),
        })
    );
}

#[test]
fn cancelar_el_selector_no_cambia_nada() {
    let (mut aplicacion, disco) = montar(vec![]);
    assert!(aplicacion.pedir_carpeta().is_some());
    aplicacion.terminar(ResultadoDeTrabajo::CarpetaElegida(None));
    assert!(aplicacion.filas().is_empty());
    assert!(!aplicacion.ocupada());
    assert!(disco.borrow().guardados.is_empty());
}

#[test]
fn nombre_de_un_proyecto_nuevo() {
    use gestor_agentes::aplicacion::nombre_de_carpeta;
    assert_eq!(nombre_de_carpeta(r"C:\Proyectos\Mi Web\"), "Mi Web");
    assert_eq!(nombre_de_carpeta("/home/u/uno//"), "uno");
    assert_eq!(nombre_de_carpeta("uno"), "uno");
}

#[test]
fn si_no_se_puede_guardar_lo_dice_la_barra_de_estado() {
    let (mut aplicacion, disco) = montar(vec![]);
    disco.borrow_mut().guardar_falla = true;
    agregar(&mut aplicacion, "/p/uno");
    assert_eq!(aplicacion.filas().len(), 1);
    assert!(aplicacion
        .texto_de_estado()
        .starts_with("No se pudo guardar la lista de proyectos"));
}

// Quitar

#[test]
fn quitar_saca_de_la_lista_guarda_y_deselecciona() {
    let (mut aplicacion, disco) =
        montar(vec![proyecto("uno", "/p/uno"), proyecto("dos", "/p/dos")]);
    aplicacion.seleccionar("/p/uno");
    aplicacion.quitar("/p/uno");
    assert_eq!(nombres(&aplicacion), ["dos"]);
    assert_eq!(aplicacion.seleccion(), None);
    assert_eq!(
        aplicacion.texto_de_estado(),
        "Proyecto eliminado de la lista: uno"
    );
    assert_eq!(disco.borrow().guardados.len(), 1);
    assert!(
        disco.borrow().carpetas.contains("/p/uno"),
        "quitar no toca el disco"
    );
}

// Abrir

fn abrir_claude(aplicacion: &mut Aplicacion) -> Trabajo {
    aplicacion.seleccionar("/p/uno");
    aplicacion
        .abrir_con(0)
        .unwrap_or_else(|| unreachable!("debía encargar el trabajo"))
}

fn resultado_abierto(
    informe: InformeDeInicializacion,
    lanzamiento: Lanzamiento,
) -> ResultadoDeTrabajo {
    ResultadoDeTrabajo::Abierto {
        nombre: String::from("uno"),
        ruta: String::from("/p/uno"),
        perfil: Box::new(perfiles()[0].clone()),
        informe,
        tamano_instrucciones: None,
        lanzamiento,
    }
}

#[test]
fn abrir_encarga_el_trabajo_con_el_perfil_elegido() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let trabajo = abrir_claude(&mut aplicacion);
    assert_eq!(
        trabajo,
        Trabajo::Abrir {
            nombre: String::from("uno"),
            ruta: String::from("/p/uno"),
            perfil: perfiles()[0].clone(),
        }
    );
    assert_eq!(
        aplicacion.texto_de_estado(),
        "Preparando uno para Claude 2…"
    );
}

#[test]
fn abrir_un_proyecto_cuya_carpeta_no_existe_avisa_y_no_encarga_nada() {
    let (mut aplicacion, disco) = montar(vec![proyecto("uno", "/p/uno")]);
    disco.borrow_mut().carpetas.clear();
    aplicacion.seleccionar("/p/uno");
    assert_eq!(aplicacion.abrir_con(0), None);
    assert!(!aplicacion.ocupada());
    let dialogo = aplicacion
        .dialogo()
        .cloned()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(dialogo.titulo(), "Error");
    assert_eq!(dialogo.texto(), "La carpeta no existe:\n/p/uno");
}

#[test]
fn abrir_con_un_perfil_que_no_existe_no_hace_nada() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.seleccionar("/p/uno");
    assert_eq!(aplicacion.abrir_con(9), None);
    assert!(!aplicacion.ocupada());
}

#[test]
fn al_abrir_se_anota_la_fecha_y_se_cuenta_lo_cambiado() {
    let (mut aplicacion, disco) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    let mut informe = InformeDeInicializacion::new();
    informe.anotar_cambio("CLAUDE.md", TipoDeCambio::Creado, None);
    informe.anotar_cambio("AGENTS.md", TipoDeCambio::Creado, None);
    aplicacion.terminar(resultado_abierto(informe, Lanzamiento::Abierto));
    assert!(!aplicacion.ocupada());
    assert_eq!(
        aplicacion.texto_de_estado(),
        "Claude 2 abierto en uno; 2 cambio(s) en sus archivos de gestión"
    );
    assert_eq!(aplicacion.filas()[0].registro.ultima_apertura(), AHORA);
    assert_eq!(
        aplicacion.filas()[0].fecha_visible,
        format!("visible:{AHORA}")
    );
    assert_eq!(disco.borrow().guardados.len(), 1);
    assert_eq!(aplicacion.dialogo(), None, "sin avisos no hay diálogo");
}

#[test]
fn un_informe_que_requiere_atencion_se_muestra() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    let mut informe = InformeDeInicializacion::new();
    informe.anotar_aviso("CLAUDE.md editado a mano");
    let esperado = informe.componer_texto();
    aplicacion.terminar(resultado_abierto(informe, Lanzamiento::Abierto));
    assert_eq!(
        aplicacion.dialogo(),
        Some(&Dialogo::Informacion {
            titulo: String::from("Preparación de uno"),
            texto: esperado,
        })
    );
}

#[test]
fn un_perfil_incompleto_dice_donde_se_edita() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    let error = ErrorDeLanzamiento::PerfilIncompleto {
        identidad: String::from("Programator 1"),
        detalle: String::from("falta la ruta del ejecutable de Programator"),
    };
    aplicacion.terminar(resultado_abierto(
        InformeDeInicializacion::new(),
        Lanzamiento::Fallido(error),
    ));
    let dialogo = aplicacion
        .dialogo()
        .cloned()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(dialogo.titulo(), "Perfil incompleto");
    assert!(
        dialogo.texto().contains(RUTA_PERFILES),
        "{}",
        dialogo.texto()
    );
    assert!(dialogo.texto().starts_with(
        "El perfil de Programator 1 no está completo: falta la ruta del ejecutable de Programator."
    ));
}

#[test]
fn una_herramienta_no_encontrada_usa_el_texto_de_la_2_3_2() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    let error = ErrorDeLanzamiento::NoEncontrada {
        herramienta: String::from("Codex"),
        detalle: String::from("no está en el PATH"),
    };
    aplicacion.terminar(resultado_abierto(
        InformeDeInicializacion::new(),
        Lanzamiento::Fallido(error),
    ));
    let dialogo = aplicacion
        .dialogo()
        .cloned()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(dialogo.titulo(), "Codex no está instalado");
    assert!(dialogo
        .texto()
        .starts_with("No se ha encontrado Codex en este equipo, así que no se ha abierto"));
    assert!(dialogo
        .texto()
        .ends_with("Los archivos de «uno» sí se han preparado."));
}

// Borrar

#[test]
fn borrar_pide_confirmacion_y_solo_el_si_encarga_el_borrado() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.seleccionar("/p/uno");
    aplicacion.pedir_borrado();
    let dialogo = aplicacion
        .dialogo()
        .cloned()
        .unwrap_or_else(|| unreachable!());
    assert!(dialogo.pide_confirmacion());
    assert!(dialogo.texto().contains("Proyecto: uno\nRuta: /p/uno"));
    assert_eq!(aplicacion.cerrar_dialogo(false), None);
    assert!(!aplicacion.ocupada());

    aplicacion.pedir_borrado();
    assert_eq!(
        aplicacion.cerrar_dialogo(true),
        Some(Trabajo::Borrar {
            nombre: String::from("uno"),
            ruta: String::from("/p/uno"),
        })
    );
    assert!(aplicacion.ocupada());
}

#[test]
fn borrado_correcto_quita_el_proyecto_y_guarda() {
    let (mut aplicacion, disco) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.seleccionar("/p/uno");
    aplicacion.pedir_borrado();
    let _ = aplicacion.cerrar_dialogo(true);
    aplicacion.terminar(ResultadoDeTrabajo::Borrado {
        nombre: String::from("uno"),
        ruta: String::from("/p/uno"),
        error: None,
    });
    assert!(aplicacion.filas().is_empty());
    assert_eq!(aplicacion.seleccion(), None);
    assert_eq!(
        aplicacion.texto_de_estado(),
        "Proyecto eliminado del disco: uno"
    );
    assert_eq!(disco.borrow().guardados.len(), 1);
}

#[test]
fn borrado_fallido_conserva_el_proyecto_y_lo_dice() {
    let (mut aplicacion, disco) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.seleccionar("/p/uno");
    aplicacion.pedir_borrado();
    let _ = aplicacion.cerrar_dialogo(true);
    aplicacion.terminar(ResultadoDeTrabajo::Borrado {
        nombre: String::from("uno"),
        ruta: String::from("/p/uno"),
        error: Some(String::from("acceso denegado")),
    });
    assert_eq!(aplicacion.filas().len(), 1);
    assert!(disco.borrow().guardados.is_empty());
    assert_eq!(
        aplicacion.dialogo().map(Dialogo::texto),
        Some(String::from(
            "No se pudo eliminar la carpeta:\nacceso denegado"
        ))
    );
}

#[test]
fn borrar_un_proyecto_sin_carpeta_lo_quita_de_la_lista_sin_preguntar() {
    let (mut aplicacion, disco) = montar(vec![proyecto("uno", "/p/uno")]);
    disco.borrow_mut().carpetas.clear();
    aplicacion.seleccionar("/p/uno");
    aplicacion.pedir_borrado();
    assert!(aplicacion.filas().is_empty());
    assert!(!aplicacion.ocupada());
    let dialogo = aplicacion
        .dialogo()
        .cloned()
        .unwrap_or_else(|| unreachable!());
    assert!(!dialogo.pide_confirmacion());
    assert_eq!(dialogo.texto(), "La carpeta no existe en disco:\n/p/uno");
    assert_eq!(disco.borrow().guardados.len(), 1);
}

#[test]
fn perder_el_hilo_de_trabajo_libera_la_ventana() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    aplicacion.trabajador_perdido();
    assert!(!aplicacion.ocupada());
    assert!(aplicacion.texto_de_estado().contains("hilo de trabajo"));
}

// Trabajos

/// Motor de mentira que anota lo que se le pide.
#[derive(Default, Clone)]
struct MotorDeMentira {
    registro: Arc<Mutex<Vec<String>>>,
    falla_ejecutar: bool,
    /// Lo que «mide» cualquier archivo que se le pida medir.
    tamano: Option<u64>,
    /// Herramientas cuyo programa se encuentra solo, en «/bin/<herramienta>».
    instaladas: Vec<Herramienta>,
    /// Carpetas de cuenta que hay en la carpeta personal.
    cuentas: Vec<CarpetaPersonal>,
    /// Carpetas que existen.
    carpetas: BTreeSet<PathBuf>,
    /// Sin carpeta personal conocida.
    sin_carpeta_personal: bool,
}

impl MotorDeMentira {
    fn anotar(&self, texto: String) {
        if let Ok(mut registro) = self.registro.lock() {
            registro.push(texto);
        }
    }

    fn anotado(&self) -> Vec<String> {
        self.registro.lock().map(|r| r.clone()).unwrap_or_default()
    }
}

fn plan() -> PlanDeLanzamiento {
    PlanDeLanzamiento {
        programa: PathBuf::from("/bin/claude"),
        argumentos: Vec::new(),
        variables: Vec::new(),
        carpeta: PathBuf::from("/p/uno"),
        carpetas_a_crear: Vec::new(),
    }
}

impl Motor for MotorDeMentira {
    fn elegir_carpeta(&self) -> Option<PathBuf> {
        self.anotar(String::from("elegir"));
        Some(PathBuf::from("/p/nueva"))
    }

    fn preparar(&self, raiz: &Path, identidad: Option<IdentidadAgente>) -> InformeDeInicializacion {
        let quien = identidad.map_or_else(|| String::from("ninguna"), |i| i.nombre());
        self.anotar(format!("preparar {} {quien}", raiz.display()));
        InformeDeInicializacion::new()
    }

    fn elegir_ejecutable(&self) -> Option<PathBuf> {
        self.anotar(String::from("elegir ejecutable"));
        Some(PathBuf::from("/opt/nuevo"))
    }

    fn carpeta_personal(&self) -> Option<PathBuf> {
        (!self.sin_carpeta_personal).then(|| PathBuf::from(CASA))
    }

    fn planificar(
        &self,
        peticion: &PeticionDeLanzamiento,
    ) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
        self.anotar(format!(
            "planificar {} {:?}",
            peticion.identidad, peticion.directorio_configuracion
        ));
        Ok(plan())
    }

    fn detectar_ejecutable(
        &self,
        herramienta: Herramienta,
        ruta_fijada: Option<&Path>,
    ) -> Option<PathBuf> {
        self.anotar(format!("detectar {herramienta:?} {ruta_fijada:?}"));
        match ruta_fijada {
            Some(ruta) => Some(ruta.to_path_buf()),
            None => self
                .instaladas
                .contains(&herramienta)
                .then(|| PathBuf::from(format!("/bin/{herramienta:?}"))),
        }
    }

    fn explorar_cuentas(&self, carpeta_personal: &Path) -> Vec<CarpetaPersonal> {
        self.anotar(format!("explorar {}", carpeta_personal.display()));
        self.cuentas.clone()
    }

    fn existe_carpeta(&self, ruta: &Path) -> bool {
        self.carpetas.contains(ruta)
    }

    fn ejecutar(&self, plan: &PlanDeLanzamiento) -> Result<(), ErrorDeLanzamiento> {
        self.anotar(format!("ejecutar {}", plan.programa.display()));
        if self.falla_ejecutar {
            return Err(ErrorDeLanzamiento::Proceso {
                programa: plan.programa.clone(),
                origen: std::io::Error::other("sin permiso"),
            });
        }
        Ok(())
    }

    fn borrar_carpeta(&self, ruta: &Path) -> std::io::Result<()> {
        self.anotar(format!("borrar {}", ruta.display()));
        Err(std::io::Error::other("en uso"))
    }

    fn tamano(&self, ruta: &Path) -> Option<u64> {
        self.anotar(format!("medir {}", ruta.display()));
        self.tamano
    }
}

fn trabajo_abrir(perfil: PerfilDeAgente) -> Trabajo {
    Trabajo::Abrir {
        nombre: String::from("uno"),
        ruta: String::from("/p/uno"),
        perfil,
    }
}

#[test]
fn abrir_prepara_antes_de_lanzar_y_con_la_carpeta_personal() {
    let motor = MotorDeMentira::default();
    let claude = PerfilDeAgente::new(HerramientaDelPerfil::Claude, 2).con_directorio("~/.claude2");
    let resultado = trabajos::ejecutar(trabajo_abrir(claude), &motor);
    assert!(matches!(
        resultado,
        ResultadoDeTrabajo::Abierto {
            lanzamiento: Lanzamiento::Abierto,
            ..
        }
    ));
    let esperado_config = Some(Path::new("/home/u").join(".claude2"));
    assert_eq!(
        motor.anotado(),
        [
            String::from("preparar /p/uno Claude 2"),
            format!("planificar Claude 2 {esperado_config:?}"),
            String::from("ejecutar /bin/claude"),
        ]
    );
}

#[test]
fn programator_se_prepara_sin_identidad_propia() {
    let motor = MotorDeMentira::default();
    let _ = trabajos::ejecutar(
        trabajo_abrir(PerfilDeAgente::new(HerramientaDelPerfil::Programator, 1)),
        &motor,
    );
    assert_eq!(motor.anotado()[0], "preparar /p/uno ninguna");
}

#[test]
fn un_fallo_al_ejecutar_vuelve_como_lanzamiento_fallido() {
    let motor = MotorDeMentira {
        falla_ejecutar: true,
        ..MotorDeMentira::default()
    };
    let resultado = trabajos::ejecutar(trabajo_abrir(perfiles()[0].clone()), &motor);
    assert!(matches!(
        resultado,
        ResultadoDeTrabajo::Abierto {
            lanzamiento: Lanzamiento::Fallido(ErrorDeLanzamiento::Proceso { .. }),
            ..
        }
    ));
}

#[test]
fn borrar_devuelve_el_error_del_sistema() {
    let motor = MotorDeMentira::default();
    let resultado = trabajos::ejecutar(
        Trabajo::Borrar {
            nombre: String::from("uno"),
            ruta: String::from("/p/uno"),
        },
        &motor,
    );
    assert!(matches!(
        resultado,
        ResultadoDeTrabajo::Borrado { error: Some(ref error), .. } if error == "en uso"
    ));
}

#[test]
fn el_trabajador_hace_el_trabajo_en_otro_hilo_y_avisa() {
    let motor = MotorDeMentira::default();
    let avisos = Arc::new(Mutex::new(0_u32));
    let contador = Arc::clone(&avisos);
    let trabajador = Trabajador::arrancar(
        Box::new(motor.clone()),
        Box::new(move || {
            if let Ok(mut avisos) = contador.lock() {
                *avisos += 1;
            }
        }),
    )
    .unwrap_or_else(|error| unreachable!("{error}"));
    assert!(matches!(trabajador.recoger(), Ok(None)));
    assert!(trabajador.encargar(Trabajo::ElegirCarpeta).is_ok());

    // Se espera, como mucho cinco segundos, a que lleguen el resultado y el aviso, que el hilo
    // da justo después de enviarlo.
    let mut resultado = None;
    for _ in 0..500 {
        if resultado.is_none() {
            resultado = trabajador.recoger().ok().flatten();
        }
        if resultado.is_some() && avisos.lock().map(|a| *a).unwrap_or_default() == 1 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(matches!(
        resultado,
        Some(ResultadoDeTrabajo::CarpetaElegida(Some(ref carpeta))) if carpeta == Path::new("/p/nueva")
    ));
    assert_eq!(avisos.lock().map(|a| *a).unwrap_or_default(), 1);
}

// Medir el archivo de instrucciones tras preparar y avisar si se corta.

fn abierto_con_tamano(perfil: PerfilDeAgente, tamano: Option<u64>) -> ResultadoDeTrabajo {
    ResultadoDeTrabajo::Abierto {
        nombre: String::from("uno"),
        ruta: String::from("/p/uno"),
        perfil: Box::new(perfil),
        informe: InformeDeInicializacion::new(),
        tamano_instrucciones: tamano,
        lanzamiento: Lanzamiento::Abierto,
    }
}

#[test]
fn abrir_gemini_mide_su_archivo_despues_de_preparar_y_antes_de_planificar() {
    let motor = MotorDeMentira {
        tamano: Some(30_000),
        ..MotorDeMentira::default()
    };
    let resultado = trabajos::ejecutar(
        trabajo_abrir(PerfilDeAgente::new(HerramientaDelPerfil::Gemini, 1)),
        &motor,
    );
    assert!(matches!(
        resultado,
        ResultadoDeTrabajo::Abierto {
            tamano_instrucciones: Some(30_000),
            ..
        }
    ));
    let anotado = motor.anotado();
    assert_eq!(anotado[0], "preparar /p/uno Gemini 1");
    assert_eq!(
        anotado[1],
        format!("medir {}", Path::new("/p/uno").join("AGENTS.md").display())
    );
    assert!(anotado[2].starts_with("planificar"));
}

#[test]
fn abrir_codex_mide_agents_md() {
    let motor = MotorDeMentira::default();
    let _ = trabajos::ejecutar(
        trabajo_abrir(PerfilDeAgente::new(HerramientaDelPerfil::Codex, 1)),
        &motor,
    );
    assert_eq!(
        motor.anotado()[1],
        format!("medir {}", Path::new("/p/uno").join("AGENTS.md").display())
    );
}

#[test]
fn abrir_claude_no_mide_nada_porque_no_tiene_limite() {
    let motor = MotorDeMentira {
        tamano: Some(99_999),
        ..MotorDeMentira::default()
    };
    let resultado = trabajos::ejecutar(trabajo_abrir(perfiles()[0].clone()), &motor);
    assert!(matches!(
        resultado,
        ResultadoDeTrabajo::Abierto {
            tamano_instrucciones: None,
            ..
        }
    ));
    assert!(motor
        .anotado()
        .iter()
        .all(|linea| !linea.starts_with("medir")));
}

#[test]
fn un_archivo_de_gemini_mayor_que_el_limite_avisa_en_la_barra() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.seleccionar("/p/uno");
    let _ = aplicacion.abrir_con(1);
    aplicacion.terminar(abierto_con_tamano(perfiles()[1].clone(), Some(24_001)));
    let estado = aplicacion.texto_de_estado();
    assert!(
        estado.ends_with(
            ". AVISO: AGENTS.md mide 24.001 bytes y Gemini solo lee los primeros 24.000"
        ),
        "{estado}"
    );
}

#[test]
fn un_archivo_justo_en_el_limite_no_avisa() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.seleccionar("/p/uno");
    let _ = aplicacion.abrir_con(1);
    aplicacion.terminar(abierto_con_tamano(perfiles()[1].clone(), Some(24_000)));
    assert!(!aplicacion.texto_de_estado().contains("AVISO"));
}

// Detección de agentes, botones visibles y rutas de los ejecutables.

#[test]
fn antes_de_detectar_no_hay_botones_ni_se_abre_nada() {
    let (mut aplicacion, _) = montar_sin_detectar(vec![proyecto("uno", "/p/uno")]);
    assert!(!aplicacion.deteccion_terminada());
    assert!(aplicacion.perfiles_visibles().is_empty());
    assert_eq!(aplicacion.disponibilidad(0), None);
    aplicacion.seleccionar("/p/uno");
    assert_eq!(aplicacion.abrir_con(0), None);
    assert!(!aplicacion.ocupada());
}

#[test]
fn detectar_encarga_el_trabajo_con_los_perfiles_guardados() {
    let (mut aplicacion, _) = montar_sin_detectar(vec![]);
    assert_eq!(
        aplicacion.detectar(),
        Some(Trabajo::Detectar {
            perfiles: perfiles()
        })
    );
    assert!(aplicacion.ocupada());
    assert_eq!(aplicacion.detectar(), None, "no se encarga dos veces");
}

#[test]
fn solo_se_ven_y_se_abren_los_agentes_disponibles() {
    let (mut aplicacion, _) = montar_sin_detectar(vec![proyecto("uno", "/p/uno")]);
    detectado(
        &mut aplicacion,
        vec![
            disponible("/bin/claude"),
            Disponibilidad::SinEjecutable,
            Disponibilidad::SinCuenta(PathBuf::from("/home/u/.x")),
        ],
    );
    assert!(aplicacion.deteccion_terminada());
    assert_eq!(aplicacion.perfiles_visibles(), [0]);
    aplicacion.seleccionar("/p/uno");
    assert_eq!(aplicacion.abrir_con(1), None);
    assert_eq!(aplicacion.abrir_con(2), None);
    assert!(!aplicacion.ocupada());
    assert!(aplicacion.abrir_con(0).is_some());
}

#[test]
fn las_cuentas_nuevas_se_guardan_y_se_dice_cuantas() {
    let (mut aplicacion, disco) = montar_sin_detectar(vec![]);
    let mut con_cuenta = perfiles();
    con_cuenta.push(PerfilDeAgente::new(HerramientaDelPerfil::Claude, 3));
    aplicacion.terminar(ResultadoDeTrabajo::Detectado {
        perfiles: con_cuenta.clone(),
        disponibilidad: vec![disponible("/bin/claude"); 4],
        anadidos: 1,
    });
    assert_eq!(disco.borrow().perfiles_guardados, [con_cuenta]);
    assert_eq!(aplicacion.perfiles().len(), 4);
    assert_eq!(
        aplicacion.texto_de_estado(),
        "Se ha añadido 1 cuenta de agente encontrada en este equipo."
    );
}

#[test]
fn sin_cuentas_nuevas_la_deteccion_no_guarda_nada() {
    let (_, disco) = montar(vec![]);
    assert!(disco.borrow().perfiles_guardados.is_empty());
}

#[test]
fn si_no_se_pueden_guardar_las_cuentas_se_usan_igual_y_se_dice() {
    let (mut aplicacion, disco) = montar_sin_detectar(vec![]);
    disco.borrow_mut().guardar_perfiles_falla = true;
    let mut con_cuenta = perfiles();
    con_cuenta.push(PerfilDeAgente::new(HerramientaDelPerfil::Codex, 2));
    aplicacion.terminar(ResultadoDeTrabajo::Detectado {
        perfiles: con_cuenta,
        disponibilidad: vec![disponible("/bin/x"); 4],
        anadidos: 1,
    });
    assert_eq!(aplicacion.perfiles().len(), 4);
    assert!(aplicacion
        .texto_de_estado()
        .starts_with("No se pudieron guardar los agentes"));
}

#[test]
fn editar_una_ruta_la_marca_como_cambiada_hasta_guardarla() {
    let (mut aplicacion, _) = montar(vec![]);
    assert_eq!(aplicacion.ruta_editada(2), "");
    assert!(!aplicacion.rutas_cambiadas());
    aplicacion.editar_ruta(2, String::from("/opt/programator"));
    assert_eq!(aplicacion.ruta_editada(2), "/opt/programator");
    assert!(aplicacion.rutas_cambiadas());
    aplicacion.busqueda_automatica(2);
    assert_eq!(aplicacion.ruta_editada(2), "");
    assert!(!aplicacion.rutas_cambiadas());
}

#[test]
fn guardar_una_ruta_que_no_existe_avisa_y_no_guarda_nada() {
    let (mut aplicacion, disco) = montar(vec![]);
    aplicacion.editar_ruta(2, String::from("/opt/no-esta"));
    assert_eq!(aplicacion.guardar_rutas(), None);
    assert!(disco.borrow().perfiles_guardados.is_empty());
    assert_eq!(aplicacion.perfiles()[2].ejecutable(), None);
    let dialogo = aplicacion
        .dialogo()
        .cloned()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(dialogo.titulo(), "Rutas no válidas");
    assert!(
        dialogo.texto().ends_with("Programator 1: /opt/no-esta"),
        "{}",
        dialogo.texto()
    );
}

#[test]
fn guardar_rutas_validas_las_guarda_y_vuelve_a_detectar() {
    let (mut aplicacion, disco) = montar(vec![]);
    // La virgulilla se expande con Path::join, que en Windows separa con «\».
    let expandida = Path::new(CASA).join("App/programator");
    disco
        .borrow_mut()
        .archivos
        .insert(expandida.to_string_lossy().into_owned());
    aplicacion.editar_ruta(2, String::from("  ~/App/programator  "));
    let trabajo = aplicacion.guardar_rutas();
    let guardado = disco
        .borrow()
        .perfiles_guardados
        .last()
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        guardado[2].ejecutable(),
        Some("~/App/programator"),
        "se guarda sin espacios y sin expandir"
    );
    assert_eq!(
        trabajo,
        Some(Trabajo::Detectar {
            perfiles: guardado.clone()
        })
    );
    assert_eq!(aplicacion.perfiles(), guardado.as_slice());
    assert!(!aplicacion.rutas_cambiadas());
    assert_eq!(
        aplicacion.texto_de_estado(),
        "Rutas guardadas; buscando de nuevo los agentes."
    );
}

#[test]
fn vaciar_una_ruta_guardada_vuelve_a_la_busqueda_automatica() {
    let (mut aplicacion, disco) = montar_sin_detectar(vec![]);
    let mut con_ruta = perfiles();
    con_ruta[0].fijar_ejecutable("/opt/claude");
    aplicacion.terminar(ResultadoDeTrabajo::Detectado {
        perfiles: con_ruta,
        disponibilidad: vec![disponible("/opt/claude"); 3],
        anadidos: 0,
    });
    assert_eq!(aplicacion.ruta_editada(0), "/opt/claude");
    aplicacion.busqueda_automatica(0);
    assert!(aplicacion.guardar_rutas().is_some());
    let guardado = disco
        .borrow()
        .perfiles_guardados
        .last()
        .cloned()
        .unwrap_or_default();
    assert_eq!(guardado[0].ejecutable(), None);
}

#[test]
fn el_ejecutable_elegido_rellena_la_ruta_sin_guardarla() {
    let (mut aplicacion, disco) = montar(vec![]);
    assert_eq!(
        aplicacion.pedir_ejecutable(1),
        Some(Trabajo::ElegirEjecutable { indice: 1 })
    );
    aplicacion.terminar(ResultadoDeTrabajo::EjecutableElegido {
        indice: 1,
        ruta: Some(PathBuf::from("/opt/gemini")),
    });
    assert_eq!(aplicacion.ruta_editada(1), "/opt/gemini");
    assert!(disco.borrow().perfiles_guardados.is_empty());
    assert_eq!(aplicacion.pedir_ejecutable(7), None);
}

#[test]
fn la_deteccion_anade_las_cuentas_y_calcula_la_disponibilidad() {
    let motor = MotorDeMentira {
        instaladas: vec![Herramienta::Claude],
        cuentas: vec![
            CarpetaPersonal {
                nombre: String::from(".claude"),
                archivos: vec![String::from(".credentials.json")],
            },
            CarpetaPersonal {
                nombre: String::from(".claude3"),
                archivos: vec![String::from(".credentials.json")],
            },
        ],
        ..MotorDeMentira::default()
    };
    let mut guardados = perfiles();
    guardados[1].fijar_ejecutable("~/gemini");
    guardados.insert(
        1,
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 4).con_directorio("~/.claude4"),
    );
    let resultado = trabajos::ejecutar(
        Trabajo::Detectar {
            perfiles: guardados,
        },
        &motor,
    );
    let ResultadoDeTrabajo::Detectado {
        perfiles,
        disponibilidad,
        anadidos,
    } = resultado
    else {
        unreachable!("debía ser una detección")
    };
    let nombres: Vec<String> = perfiles.iter().map(PerfilDeAgente::nombre).collect();
    assert_eq!(
        nombres,
        [
            "Claude 2",
            "Claude 4",
            "Claude 1",
            "Claude 3",
            "Gemini 1",
            "Programator 1"
        ]
    );
    assert_eq!(anadidos, 2);
    let casa = Path::new(CASA);
    assert_eq!(
        disponibilidad,
        [
            disponible("/bin/Claude"),
            Disponibilidad::SinCuenta(casa.join(".claude4")),
            disponible("/bin/Claude"),
            Disponibilidad::SinCuenta(casa.join(".claude3")),
            Disponibilidad::Disponible(casa.join("gemini")),
            Disponibilidad::SinEjecutable,
        ]
    );
    assert!(motor
        .anotado()
        .contains(&format!("detectar Gemini {:?}", Some(casa.join("gemini")))));
}

#[test]
fn sin_carpeta_personal_no_se_anaden_cuentas() {
    let motor = MotorDeMentira {
        sin_carpeta_personal: true,
        cuentas: vec![CarpetaPersonal {
            nombre: String::from(".codex"),
            archivos: vec![String::from("auth.json")],
        }],
        ..MotorDeMentira::default()
    };
    let resultado = trabajos::ejecutar(
        Trabajo::Detectar {
            perfiles: perfiles(),
        },
        &motor,
    );
    assert!(matches!(
        resultado,
        ResultadoDeTrabajo::Detectado { anadidos: 0, ref perfiles, .. } if perfiles.len() == 3
    ));
    assert!(motor
        .anotado()
        .iter()
        .all(|linea| !linea.starts_with("explorar")));
}

#[test]
fn elegir_ejecutable_devuelve_la_ruta_con_su_indice() {
    let motor = MotorDeMentira::default();
    assert!(matches!(
        trabajos::ejecutar(Trabajo::ElegirEjecutable { indice: 2 }, &motor),
        ResultadoDeTrabajo::EjecutableElegido { indice: 2, ruta: Some(ref ruta) } if ruta == Path::new("/opt/nuevo")
    ));
}
