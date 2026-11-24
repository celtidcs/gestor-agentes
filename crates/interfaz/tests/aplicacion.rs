//! Barreras del estado de la ventana y de los trabajos, con dobles en lugar del disco, el reloj y
//! los procesos.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gestor_agentes::aplicacion::{Aplicacion, Dialogo, RespuestaDeDialogo};
use gestor_agentes::perfiles::{CarpetaDelAgente, Disponibilidad, EstadoDelAgente};
use gestor_agentes::trabajos::{self, Lanzamiento, ResultadoDeTrabajo, Trabajador, Trabajo};
use lanzador::{
    AjusteDeEntorno, ErrorDeLanzamiento, Herramienta, PeticionDeLanzamiento, PlanDeLanzamiento,
};
use nucleo::actualizaciones::Version;
use nucleo::agentes::IdentidadAgente;
use nucleo::configuracion::{
    ErrorDeConfiguracion, HerramientaDelPerfil, PerfilDeAgente, ProyectoRegistrado,
};
use nucleo::cuentas::{Cuenta, OrigenDeLaCuenta};
use nucleo::cuentas_de_agentes::CuentaManual;
use nucleo::migracion_de_claude::{ErrorDeMigracion, MigracionHecha, RutasDeLaMigracion};
use nucleo::proyecto::estado::EstadoDelProyecto;
use nucleo::proyecto::informe::{InformeDeInicializacion, TipoDeCambio};
use nucleo::proyecto::papeles::Papel;

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

impl gestor_agentes::aplicacion::AlmacenDeConfiguracion for ServiciosDeMentira {
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
    fn ruta_perfiles(&self) -> PathBuf {
        PathBuf::from(RUTA_PERFILES)
    }
}

impl gestor_agentes::aplicacion::RelojDeInterfaz for ServiciosDeMentira {
    fn ahora(&self) -> String {
        AHORA.to_owned()
    }
    fn mostrar_fecha(&self, iso: &str) -> String {
        format!("visible:{iso}")
    }
}

impl gestor_agentes::aplicacion::DiscoDeInterfaz for ServiciosDeMentira {
    fn existe_carpeta(&self, carpeta: &Path) -> bool {
        carpeta
            .to_str()
            .is_some_and(|ruta| self.0.borrow().carpetas.contains(ruta))
    }
    fn existe_archivo(&self, ruta: &Path) -> bool {
        ruta.to_str()
            .is_some_and(|ruta| self.0.borrow().archivos.contains(ruta))
    }
    fn carpeta_personal(&self) -> Option<PathBuf> {
        Some(PathBuf::from(CASA))
    }
}

impl gestor_agentes::aplicacion::InspectorDeProyecto for ServiciosDeMentira {
    fn estado_del_proyecto(&self, carpeta: &Path) -> EstadoDelProyecto {
        if carpeta.ends_with("configurado") {
            EstadoDelProyecto::Configurado
        } else {
            EstadoDelProyecto::SinConfigurar
        }
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
        carpetas: Vec::new(),
        anadidos: 0,
        normalizados: 0,
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

/// Si perfiles-agentes.json no se pudo leer, la búsqueda de agentes no lo
/// sobrescribe con las cuentas que encuentre, porque borraría la lista del usuario.
#[test]
fn unos_perfiles_danados_no_se_sobrescriben_al_buscar_agentes() {
    let disco = Rc::new(RefCell::new(Disco::default()));
    let mut aplicacion = Aplicacion::new(
        Box::new(ServiciosDeMentira(Rc::clone(&disco))),
        Ok(vec![proyecto("uno", "/p/uno")]),
        Err(error_de_formato()),
    );
    let _ = aplicacion.detectar();
    aplicacion.terminar(ResultadoDeTrabajo::Detectado {
        perfiles: perfiles(),
        disponibilidad: vec![disponible("/bin/claude"); 3],
        carpetas: Vec::new(),
        anadidos: 3,
        normalizados: 0,
    });
    assert!(disco.borrow().perfiles_guardados.is_empty());
    assert_eq!(
        aplicacion.texto_de_estado(),
        gestor_agentes::textos::PERFILES_ILEGIBLES_SIN_GUARDAR
    );
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
            papel: None,
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
fn un_informe_con_errores_se_muestra() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    let mut informe = InformeDeInicializacion::new();
    informe.anotar_error("CLAUDE.md: acceso denegado");
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
fn un_informe_con_errores_y_avisos_se_muestra_entero() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    let mut informe = InformeDeInicializacion::new();
    informe.anotar_aviso(".gestor/canal/claude.md mide 30000 bytes");
    informe.anotar_error("CLAUDE.md: acceso denegado");
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
fn un_informe_solo_con_avisos_no_abre_ventana_y_lo_dice_la_barra() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    let mut informe = InformeDeInicializacion::new();
    informe.anotar_aviso(".gestor/canal/claude.md mide 30000 bytes");
    informe.anotar_aviso("Falta el apartado Papeles");
    aplicacion.terminar(resultado_abierto(informe, Lanzamiento::Abierto));
    assert_eq!(aplicacion.dialogo(), None);
    assert!(
        aplicacion
            .texto_de_estado()
            .ends_with(". 2 aviso(s) para los agentes en .gestor/avisos-de-preparacion.md"),
        "{}",
        aplicacion.texto_de_estado()
    );
}

#[test]
fn un_informe_sin_avisos_no_toca_la_barra() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    let _ = abrir_claude(&mut aplicacion);
    aplicacion.terminar(resultado_abierto(
        InformeDeInicializacion::new(),
        Lanzamiento::Abierto,
    ));
    assert_eq!(aplicacion.dialogo(), None);
    assert!(!aplicacion.texto_de_estado().contains("aviso(s)"));
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
    assert_eq!(
        aplicacion.cerrar_dialogo(RespuestaDeDialogo::Cancelar),
        None
    );
    assert!(!aplicacion.ocupada());

    aplicacion.pedir_borrado();
    assert_eq!(
        aplicacion.cerrar_dialogo(RespuestaDeDialogo::Aceptar),
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
    let _ = aplicacion.cerrar_dialogo(RespuestaDeDialogo::Aceptar);
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
    let _ = aplicacion.cerrar_dialogo(RespuestaDeDialogo::Aceptar);
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
    /// Versión que «declara» el ejecutable de Programator.
    version_de_programator: Option<Version>,
    /// Herramientas cuyo programa se encuentra solo, en «/bin/<herramienta>».
    instaladas: Vec<Herramienta>,
    /// Cuentas que «encuentra» la biblioteca, antes de añadir las manuales.
    cuentas: Vec<Cuenta>,
    /// Lo que devuelve la migración del estado de Claude 1: nada, lo hecho o un error.
    migracion: Option<Result<bool, String>>,
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

impl gestor_agentes::trabajos::SelectoresDeArchivo for MotorDeMentira {
    fn elegir_carpeta(&self) -> Option<PathBuf> {
        self.anotar(String::from("elegir"));
        Some(PathBuf::from("/p/nueva"))
    }
    fn elegir_ejecutable(&self) -> Option<PathBuf> {
        self.anotar(String::from("elegir ejecutable"));
        Some(PathBuf::from("/opt/nuevo"))
    }
    fn elegir_carpeta_de_cuenta(&self) -> Option<PathBuf> {
        self.anotar(String::from("elegir cuenta"));
        Some(PathBuf::from("/cuentas/claude-trabajo"))
    }
}

impl gestor_agentes::trabajos::PreparadorDeProyecto for MotorDeMentira {
    fn preparar(&self, raiz: &Path, identidad: Option<IdentidadAgente>) -> InformeDeInicializacion {
        let quien = identidad.map_or_else(|| String::from("ninguna"), |i| i.nombre());
        self.anotar(format!("preparar {} {quien}", raiz.display()));
        InformeDeInicializacion::new()
    }
    fn anotar_papel(&self, raiz: &Path, identidad: &str, papel: Papel) -> std::io::Result<bool> {
        self.anotar(format!("papel {} {identidad} {papel:?}", raiz.display()));
        Ok(true)
    }
}

impl gestor_agentes::trabajos::LanzadorDeAgentes for MotorDeMentira {
    fn planificar(
        &self,
        peticion: &PeticionDeLanzamiento,
    ) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
        self.anotar(format!(
            "planificar {} {:?}",
            peticion.identidad, peticion.ajuste_de_entorno
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
    fn version_del_ejecutable(&self, ruta: &Path) -> Option<Version> {
        self.anotar(format!("version {}", ruta.display()));
        self.version_de_programator
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
}

impl gestor_agentes::trabajos::DiscoDelMotor for MotorDeMentira {
    fn carpeta_personal(&self) -> Option<PathBuf> {
        (!self.sin_carpeta_personal).then(|| PathBuf::from(CASA))
    }
    fn existe_carpeta(&self, ruta: &Path) -> bool {
        self.carpetas.contains(ruta)
    }
    fn borrar_carpeta(&self, ruta: &Path) -> std::io::Result<()> {
        self.anotar(format!("borrar {}", ruta.display()));
        Err(std::io::Error::other("en uso"))
    }
    fn tamano(&self, ruta: &Path) -> Option<u64> {
        self.anotar(format!("medir {}", ruta.display()));
        self.tamano
    }
    fn cuentas_del_equipo(
        &self,
        carpeta_personal: &Path,
        manuales: &[CuentaManual],
    ) -> Vec<Cuenta> {
        self.anotar(format!("cuentas {}", carpeta_personal.display()));
        let mut cuentas = self.cuentas.clone();
        cuentas.extend(manuales.iter().map(|manual| Cuenta {
            herramienta: manual.herramienta,
            carpeta: manual.carpeta.clone(),
            origen: OrigenDeLaCuenta::Manual,
        }));
        cuentas
    }
    fn migrar_estado_de_claude(
        &self,
        carpeta_personal: &Path,
    ) -> Result<Option<MigracionHecha>, ErrorDeMigracion> {
        self.anotar(format!("migrar {}", carpeta_personal.display()));
        let rutas = RutasDeLaMigracion::new(carpeta_personal, Path::new("/datos"));
        match self.migracion.clone() {
            None => Ok(None),
            Some(Ok(con_copia)) => Ok(Some(MigracionHecha { rutas, con_copia })),
            Some(Err(error)) => Err(ErrorDeMigracion {
                ruta: rutas.destino,
                origen: std::io::Error::other(error),
            }),
        }
    }
}

fn trabajo_abrir(perfil: PerfilDeAgente) -> Trabajo {
    Trabajo::Abrir {
        nombre: String::from("uno"),
        ruta: String::from("/p/uno"),
        perfil,
        papel: None,
    }
}

#[test]
fn el_papel_elegido_va_en_la_siguiente_apertura_y_solo_en_ella() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    aplicacion.elegir_papel(Some(Papel::Ayudante));
    assert_eq!(aplicacion.papel(), Some(Papel::Ayudante));
    let Trabajo::Abrir { papel, .. } = abrir_claude(&mut aplicacion) else {
        unreachable!("se esperaba un trabajo de apertura");
    };
    assert_eq!(papel, Some(Papel::Ayudante));
    assert_eq!(aplicacion.papel(), None);
}

#[test]
fn abrir_con_papel_lo_anota_antes_de_lanzar_y_lo_dice_en_el_informe() {
    let motor = MotorDeMentira::default();
    let claude = PerfilDeAgente::new(HerramientaDelPerfil::Claude, 2).con_directorio("~/.claude2");
    let Trabajo::Abrir {
        nombre,
        ruta,
        perfil,
        ..
    } = trabajo_abrir(claude)
    else {
        unreachable!();
    };
    let trabajo = Trabajo::Abrir {
        nombre,
        ruta,
        perfil,
        papel: Some(Papel::Jefe),
    };
    let ResultadoDeTrabajo::Abierto { informe, .. } = trabajos::ejecutar(trabajo, &motor) else {
        unreachable!("se esperaba Abierto");
    };
    let anotado = motor.anotado();
    let papel = anotado
        .iter()
        .position(|l| l == "papel /p/uno Claude 2 Jefe");
    let ejecutar = anotado.iter().position(|l| l.starts_with("ejecutar"));
    assert!(
        matches!((papel, ejecutar), (Some(a), Some(b)) if a < b),
        "{anotado:?}"
    );
    assert!(informe
        .cambios()
        .iter()
        .any(|c| c.archivo == ".gestor/canal/estado.md" && c.tipo == TipoDeCambio::Actualizado));
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
    let esperado_config = Some(AjusteDeEntorno::Poner {
        variable: "CLAUDE_CONFIG_DIR",
        valor: Path::new("/home/u").join(".claude2"),
    });
    assert_eq!(
        motor.anotado(),
        [
            String::from("preparar /p/uno Claude 2"),
            format!("planificar Claude 2 {esperado_config:?}"),
            String::from("ejecutar /bin/claude"),
        ]
    );
}

/// La cuenta por defecto de Claude se abre quitando CLAUDE_CONFIG_DIR, y antes de
/// planificarla se hace la migración única de su estado. Se anota como archivo sustituido, con la
/// identidad del agente, y abre la ventana del informe, para que el usuario lo vea.
#[test]
fn abrir_la_cuenta_por_defecto_de_claude_migra_antes_de_planificar_y_lo_avisa() {
    let motor = MotorDeMentira {
        migracion: Some(Ok(true)),
        ..MotorDeMentira::default()
    };
    let claude = PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1);
    let ResultadoDeTrabajo::Abierto {
        informe,
        lanzamiento: Lanzamiento::Abierto,
        ..
    } = trabajos::ejecutar(trabajo_abrir(claude), &motor)
    else {
        unreachable!("se esperaba Abierto");
    };
    let quitar = Some(AjusteDeEntorno::Quitar {
        variable: "CLAUDE_CONFIG_DIR",
    });
    assert_eq!(
        motor.anotado(),
        [
            String::from("preparar /p/uno Claude 1"),
            format!("migrar {CASA}"),
            format!("planificar Claude 1 {quitar:?}"),
            String::from("ejecutar /bin/claude"),
        ]
    );
    let rutas = RutasDeLaMigracion::new(Path::new(CASA), Path::new("/datos"));
    assert!(informe.requiere_ventana());
    let Some(cambio) = informe
        .cambios()
        .iter()
        .find(|cambio| cambio.tipo == TipoDeCambio::Sustituido)
    else {
        unreachable!("la migración se anota como archivo sustituido");
    };
    assert_eq!(cambio.archivo, rutas.destino.display().to_string());
    let detalle = cambio.detalle.as_deref().unwrap_or_default();
    for parte in [
        String::from("Claude 1"),
        rutas.origen.display().to_string(),
        rutas.copia.display().to_string(),
        rutas.migrado.display().to_string(),
    ] {
        assert!(detalle.contains(&parte), "{detalle}");
    }
}

/// Un fallo de la migración se anota como error y Claude 1 se abre igual.
#[test]
fn un_fallo_de_la_migracion_se_anota_y_claude_se_abre_igual() {
    let motor = MotorDeMentira {
        migracion: Some(Err(String::from("disco lleno"))),
        ..MotorDeMentira::default()
    };
    let ResultadoDeTrabajo::Abierto {
        informe,
        lanzamiento: Lanzamiento::Abierto,
        ..
    } = trabajos::ejecutar(
        trabajo_abrir(PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1)),
        &motor,
    )
    else {
        unreachable!("se esperaba Abierto");
    };
    assert!(informe
        .errores()
        .iter()
        .any(|e| e.contains("disco lleno") && e.contains("Claude 1")));
    assert!(informe.requiere_ventana());
    assert!(motor.anotado().iter().any(|l| l.starts_with("ejecutar")));
}

/// Ni Codex ni una cuenta de Claude que no es la por defecto migran nada.
#[test]
fn solo_la_cuenta_por_defecto_de_claude_migra() {
    for perfil in [
        PerfilDeAgente::new(HerramientaDelPerfil::Codex, 1),
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 2).con_directorio("~/.claude2"),
        PerfilDeAgente::new(HerramientaDelPerfil::Gemini, 1),
    ] {
        let motor = MotorDeMentira {
            migracion: Some(Ok(true)),
            ..MotorDeMentira::default()
        };
        let _ = trabajos::ejecutar(trabajo_abrir(perfil), &motor);
        assert!(
            motor.anotado().iter().all(|l| !l.starts_with("migrar")),
            "{:?}",
            motor.anotado()
        );
    }
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

/// Quitar la carpeta por defecto a un perfil se guarda, pero no se
/// anuncia como una cuenta nueva.
#[test]
fn un_perfil_normalizado_se_guarda_sin_anunciar_cuentas_nuevas() {
    let (mut aplicacion, disco) = montar_sin_detectar(vec![]);
    let antes = aplicacion.texto_de_estado().to_owned();
    aplicacion.terminar(ResultadoDeTrabajo::Detectado {
        perfiles: perfiles(),
        disponibilidad: vec![disponible("/bin/claude"); 3],
        carpetas: Vec::new(),
        anadidos: 0,
        normalizados: 1,
    });
    assert_eq!(disco.borrow().perfiles_guardados, [perfiles()]);
    assert_eq!(aplicacion.texto_de_estado(), antes);
}

#[test]
fn las_cuentas_nuevas_se_guardan_y_se_dice_cuantas() {
    let (mut aplicacion, disco) = montar_sin_detectar(vec![]);
    let mut con_cuenta = perfiles();
    con_cuenta.push(PerfilDeAgente::new(HerramientaDelPerfil::Claude, 3));
    aplicacion.terminar(ResultadoDeTrabajo::Detectado {
        perfiles: con_cuenta.clone(),
        disponibilidad: vec![disponible("/bin/claude"); 4],
        carpetas: Vec::new(),
        anadidos: 1,
        normalizados: 0,
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
        carpetas: Vec::new(),
        anadidos: 1,
        normalizados: 0,
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
        carpetas: Vec::new(),
        anadidos: 0,
        normalizados: 0,
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
        // Claude 2 sin carpeta fijada es la cuenta por defecto, ~/.claude, que existe (AG1). Desde
        // la 3.9.0 la cuenta por defecto no añade un Claude 1: ya tiene perfil.
        carpetas: BTreeSet::from([Path::new(CASA).join(".claude")]),
        cuentas: vec![
            Cuenta {
                herramienta: nucleo::cuentas::Herramienta::ClaudeCode,
                carpeta: Path::new(CASA).join(".claude"),
                origen: OrigenDeLaCuenta::PorDefecto,
            },
            Cuenta {
                herramienta: nucleo::cuentas::Herramienta::ClaudeCode,
                carpeta: Path::new(CASA).join(".claude3"),
                origen: OrigenDeLaCuenta::Busqueda,
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
        carpetas,
        anadidos,
        ..
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
            "Claude 3",
            "Gemini 1",
            "Programator 1"
        ]
    );
    assert_eq!(anadidos, 1);
    assert_eq!(
        perfiles[2].directorio_configuracion(),
        Some("~/.claude3"),
        "la carpeta de la personal se guarda con «~/»"
    );
    let casa = Path::new(CASA);
    assert_eq!(
        disponibilidad,
        [
            disponible("/bin/Claude"),
            Disponibilidad::SinCuenta(casa.join(".claude4")),
            Disponibilidad::SinCuenta(casa.join(".claude3")),
            Disponibilidad::Disponible(casa.join("gemini")),
            Disponibilidad::SinEjecutable,
        ]
    );
    assert!(motor
        .anotado()
        .contains(&format!("detectar Gemini {:?}", Some(casa.join("gemini")))));
    let carpeta = |nombre: &str, existe| Some(CarpetaDelAgente::new(casa.join(nombre), existe));
    assert_eq!(
        carpetas,
        [
            carpeta(".claude", true),
            carpeta(".claude4", false),
            carpeta(".claude3", false),
            carpeta(".gemini", false),
            None,
        ]
    );
}

#[test]
fn la_primera_cuenta_de_claude_sin_su_carpeta_falta_la_cuenta() {
    let motor = MotorDeMentira {
        instaladas: vec![Herramienta::Claude],
        ..MotorDeMentira::default()
    };
    let claude_uno =
        PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1).con_directorio("~/.claude");
    let ResultadoDeTrabajo::Detectado { disponibilidad, .. } = trabajos::ejecutar(
        Trabajo::Detectar {
            perfiles: vec![claude_uno],
        },
        &motor,
    ) else {
        unreachable!("debía ser una detección")
    };
    assert_eq!(
        disponibilidad,
        [Disponibilidad::SinCuenta(Path::new(CASA).join(".claude"))]
    );
}

#[test]
fn solo_la_busqueda_de_agentes_marca_que_se_esta_buscando() {
    let (mut aplicacion, _) = montar(vec![proyecto("uno", "/p/uno")]);
    assert!(!aplicacion.buscando_agentes());
    let _ = abrir_claude(&mut aplicacion);
    assert!(aplicacion.ocupada());
    assert!(!aplicacion.buscando_agentes());
    aplicacion.terminar(ResultadoDeTrabajo::CarpetaElegida(None));

    assert!(aplicacion.detectar().is_some());
    assert!(aplicacion.buscando_agentes());
    assert_eq!(
        aplicacion.estado_del_agente(0),
        EstadoDelAgente::SinComprobar
    );
    detectado(
        &mut aplicacion,
        vec![
            disponible("/bin/claude"),
            Disponibilidad::SinCuenta(PathBuf::from("/h/.gemini")),
            Disponibilidad::SinEjecutable,
        ],
    );
    assert!(!aplicacion.buscando_agentes());
    assert_eq!(aplicacion.estado_del_agente(0), EstadoDelAgente::Localizado);
    assert_eq!(
        aplicacion.estado_del_agente(1),
        EstadoDelAgente::FaltaLaCuenta
    );
    assert_eq!(
        aplicacion.estado_del_agente(2),
        EstadoDelAgente::NoEncontrado
    );
}

#[test]
fn una_ruta_editada_sin_guardar_deja_el_agente_sin_comprobar() {
    let (mut aplicacion, _) = montar(vec![]);
    assert_eq!(aplicacion.estado_del_agente(0), EstadoDelAgente::Localizado);
    aplicacion.editar_ruta(0, String::from("/opt/otro"));
    assert_eq!(
        aplicacion.estado_del_agente(0),
        EstadoDelAgente::SinComprobar
    );
    aplicacion.editar_ruta(0, String::new());
    assert_eq!(aplicacion.estado_del_agente(0), EstadoDelAgente::Localizado);
}

#[test]
fn si_se_pierde_el_trabajador_deja_de_buscar() {
    let (mut aplicacion, _) = montar(vec![]);
    assert!(aplicacion.detectar().is_some());
    aplicacion.trabajador_perdido();
    assert!(!aplicacion.buscando_agentes());
}

#[test]
fn sin_carpeta_personal_no_se_anaden_cuentas() {
    let motor = MotorDeMentira {
        sin_carpeta_personal: true,
        cuentas: vec![Cuenta {
            herramienta: nucleo::cuentas::Herramienta::CodexCli,
            carpeta: Path::new(CASA).join(".codex"),
            origen: OrigenDeLaCuenta::PorDefecto,
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
        .all(|linea| !linea.starts_with("cuentas")));
}

#[test]
fn elegir_ejecutable_devuelve_la_ruta_con_su_indice() {
    let motor = MotorDeMentira::default();
    assert!(matches!(
        trabajos::ejecutar(Trabajo::ElegirEjecutable { indice: 2 }, &motor),
        ResultadoDeTrabajo::EjecutableElegido { indice: 2, ruta: Some(ref ruta) } if ruta == Path::new("/opt/nuevo")
    ));
}

#[test]
fn pedir_cuenta_encarga_el_alta_con_los_perfiles_guardados() {
    let (mut aplicacion, _) = montar_sin_detectar(vec![]);
    assert_eq!(
        aplicacion.pedir_cuenta(HerramientaDelPerfil::Claude),
        Some(Trabajo::DarDeAltaCuenta {
            herramienta: HerramientaDelPerfil::Claude,
            perfiles: perfiles(),
        })
    );
    assert!(aplicacion.ocupada());
    assert_eq!(
        aplicacion.pedir_cuenta(HerramientaDelPerfil::Codex),
        None,
        "no se encarga dos veces"
    );
}

#[test]
fn el_alta_de_cuenta_anade_el_perfil_con_su_carpeta_y_lo_cuenta_para_guardar() {
    let motor = MotorDeMentira::default();
    let resultado = trabajos::ejecutar(
        Trabajo::DarDeAltaCuenta {
            herramienta: HerramientaDelPerfil::Claude,
            perfiles: perfiles(),
        },
        &motor,
    );
    let ResultadoDeTrabajo::Detectado {
        perfiles, anadidos, ..
    } = resultado
    else {
        unreachable!("debía ser una detección")
    };
    let nombres: Vec<String> = perfiles.iter().map(PerfilDeAgente::nombre).collect();
    assert_eq!(
        nombres,
        ["Claude 2", "Claude 3", "Gemini 1", "Programator 1"]
    );
    assert_eq!(
        perfiles[1].directorio_configuracion(),
        Some("/cuentas/claude-trabajo")
    );
    assert_eq!(anadidos, 1);
    assert!(motor.anotado().contains(&String::from("elegir cuenta")));
    // El alta busca las cuentas del equipo una sola vez.
    assert_eq!(
        motor
            .anotado()
            .iter()
            .filter(|l| l.starts_with("cuentas "))
            .count(),
        1
    );
}

/// Sin carpeta personal no se puede saber si la carpeta es la de
/// otra cuenta, y el alta se rechaza en vez de validarse contra rutas relativas.
#[test]
fn sin_carpeta_personal_el_alta_se_rechaza_con_su_motivo() {
    let motor = MotorDeMentira {
        sin_carpeta_personal: true,
        ..MotorDeMentira::default()
    };
    let resultado = trabajos::ejecutar(
        Trabajo::DarDeAltaCuenta {
            herramienta: HerramientaDelPerfil::Claude,
            perfiles: perfiles(),
        },
        &motor,
    );
    assert!(matches!(
        resultado,
        ResultadoDeTrabajo::CuentaNoAnadida(Some(
            nucleo::cuentas::AltaRechazada::SinCarpetaPersonal
        ))
    ));
}

#[test]
fn una_cuenta_rechazada_se_explica_y_no_cambia_los_perfiles() {
    let (mut aplicacion, disco) = montar(vec![]);
    let antes = disco.borrow().perfiles_guardados.clone();
    aplicacion.terminar(ResultadoDeTrabajo::CuentaNoAnadida(Some(
        nucleo::cuentas::AltaRechazada::CarpetaEnUso(String::from("Claude 2")),
    )));
    let Some(dialogo) = aplicacion.dialogo() else {
        unreachable!("debía avisar del rechazo")
    };
    assert_eq!(dialogo.titulo(), "Cuenta no añadida");
    assert!(matches!(
        dialogo,
        Dialogo::Informacion { texto, .. } if texto.contains("ya es la de Claude 2")
    ));
    assert_eq!(disco.borrow().perfiles_guardados, antes);
}

#[test]
fn la_deteccion_lee_la_version_de_programator_y_la_compara_con_la_publicada() {
    let motor = MotorDeMentira {
        version_de_programator: Version::desde_etiqueta("0.12.0"),
        ..MotorDeMentira::default()
    };
    let mut guardados = perfiles();
    guardados[2].fijar_ejecutable("/opt/programator/programator.exe");
    let resultado = trabajos::ejecutar(
        Trabajo::Detectar {
            perfiles: guardados,
        },
        &motor,
    );
    let ResultadoDeTrabajo::Detectado { carpetas, .. } = &resultado else {
        unreachable!("debía ser una detección")
    };
    assert_eq!(
        carpetas[2].as_ref().and_then(CarpetaDelAgente::version),
        Version::desde_etiqueta("0.12.0")
    );
    assert!(
        carpetas[0]
            .as_ref()
            .and_then(CarpetaDelAgente::version)
            .is_none(),
        "solo se lee la de Programator"
    );

    let (mut aplicacion, _) = montar_sin_detectar(vec![]);
    aplicacion.terminar(resultado);
    assert_eq!(
        aplicacion.version_mas_nueva(2),
        None,
        "sin consulta no se avisa"
    );
    let Some(nueva) = Version::desde_etiqueta("0.13.0") else {
        unreachable!("versión válida")
    };
    aplicacion.fijar_programator_publicado(nueva);
    assert_eq!(
        aplicacion.version_mas_nueva(2),
        Version::desde_etiqueta("0.13.0")
    );
    assert_eq!(aplicacion.version_mas_nueva(0), None);
}
