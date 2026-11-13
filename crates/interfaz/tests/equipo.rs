//! Barreras de la línea del equipo: la pestaña Proyectos enseña, para el proyecto seleccionado,
//! una línea con el Jefe, el último latido de cada agente, las reservas y las alertas. Se lee al
//! seleccionar y al terminar un trabajo; sin estado del canal no se enseña nada.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use chrono::NaiveDateTime;
use gestor_agentes::aplicacion::{Aplicacion, Servicios};
use gestor_agentes::textos;
use gestor_agentes::trabajos::ResultadoDeTrabajo;
use nucleo::configuracion::{ErrorDeConfiguracion, PerfilDeAgente, ProyectoRegistrado};
use nucleo::proyecto::estado::EstadoDelProyecto;
use nucleo::proyecto::revision_del_canal::{Latido, ResumenDelEquipo};

const CON_CANAL: &str = "/proyectos/con-canal";
const SIN_CANAL: &str = "/proyectos/sin-canal";

fn hora(texto: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(texto, "%Y-%m-%d %H:%M:%S").unwrap()
}

fn resumen() -> ResumenDelEquipo {
    ResumenDelEquipo {
        jefe: Some("Claude 2".to_owned()),
        latidos: vec![
            Latido {
                identificador: "claude2".to_owned(),
                momento: hora("2026-11-13 11:55:00"),
            },
            Latido {
                identificador: "gemini".to_owned(),
                momento: hora("2026-11-13 09:00:00"),
            },
        ],
        reservas: 2,
        mutaciones_vivas: true,
        autoexclusiones: false,
    }
}

/// Servicios que solo saben del equipo, y cuentan cuántas veces se les pregunta.
struct ServiciosDelEquipo {
    lecturas: Rc<Cell<u32>>,
}

impl Servicios for ServiciosDelEquipo {
    fn guardar_proyectos(&self, _: &[ProyectoRegistrado]) -> Result<(), ErrorDeConfiguracion> {
        Ok(())
    }
    fn estado_del_proyecto(&self, _: &Path) -> EstadoDelProyecto {
        EstadoDelProyecto::Configurado
    }
    fn existe_carpeta(&self, _: &Path) -> bool {
        true
    }
    fn ahora(&self) -> String {
        "2026-11-13T12:00:00+01:00".to_owned()
    }
    fn mostrar_fecha(&self, iso: &str) -> String {
        iso.to_owned()
    }
    fn ruta_perfiles(&self) -> PathBuf {
        PathBuf::from("/datos/perfiles-agentes.json")
    }
    fn guardar_perfiles(&self, _: &[PerfilDeAgente]) -> Result<(), ErrorDeConfiguracion> {
        Ok(())
    }
    fn existe_archivo(&self, _: &Path) -> bool {
        true
    }
    fn carpeta_personal(&self) -> Option<PathBuf> {
        None
    }
    fn equipo_del_proyecto(&self, carpeta: &Path) -> Option<(ResumenDelEquipo, NaiveDateTime)> {
        self.lecturas.set(self.lecturas.get() + 1);
        (carpeta == Path::new(CON_CANAL)).then(|| (resumen(), hora("2026-11-13 12:00:00")))
    }
}

fn aplicacion() -> (Aplicacion, Rc<Cell<u32>>) {
    let lecturas = Rc::new(Cell::new(0));
    let proyectos = vec![
        ProyectoRegistrado::new("con-canal", CON_CANAL, "2026-01-01T00:00:00+01:00"),
        ProyectoRegistrado::new("sin-canal", SIN_CANAL, "2026-01-01T00:00:00+01:00"),
    ];
    let aplicacion = Aplicacion::new(
        Box::new(ServiciosDelEquipo {
            lecturas: Rc::clone(&lecturas),
        }),
        Ok(proyectos),
        Ok(Vec::new()),
    );
    (aplicacion, lecturas)
}

#[test]
fn la_linea_dice_jefe_latidos_reservas_y_alertas() {
    assert_eq!(
        textos::linea_del_equipo(&resumen(), hora("2026-11-13 12:00:00")),
        "Equipo · Jefe: Claude 2 · claude2 hace 5 min · gemini hace 3 h · 2 reservas · MUTACIONES VIVAS"
    );
}

#[test]
fn sin_jefe_ni_reservas_lo_dice() {
    let vacio = ResumenDelEquipo::default();

    assert_eq!(
        textos::linea_del_equipo(&vacio, hora("2026-11-13 12:00:00")),
        "Equipo · Jefe sin asignar · sin reservas"
    );
}

#[test]
fn la_antiguedad_se_dice_en_minutos_horas_o_dias() {
    for (minutos, texto) in [
        (0, "ahora"),
        (-5, "ahora"),
        (59, "hace 59 min"),
        (60, "hace 1 h"),
        (47 * 60, "hace 47 h"),
        (48 * 60, "hace 2 días"),
    ] {
        assert_eq!(textos::antiguedad(minutos), texto, "{minutos}");
    }
}

#[test]
fn se_lee_al_seleccionar_y_sin_canal_no_se_ensena_nada() {
    let (mut aplicacion, lecturas) = aplicacion();
    assert_eq!(aplicacion.equipo(), None);
    assert_eq!(lecturas.get(), 0, "no se lee nada antes de seleccionar");

    aplicacion.seleccionar(CON_CANAL);
    assert!(aplicacion
        .equipo()
        .is_some_and(|l| l.contains("Jefe: Claude 2")));

    aplicacion.seleccionar(SIN_CANAL);
    assert_eq!(aplicacion.equipo(), None);
    assert_eq!(lecturas.get(), 2);
}

#[test]
fn se_vuelve_a_leer_al_terminar_un_trabajo() {
    let (mut aplicacion, lecturas) = aplicacion();
    aplicacion.seleccionar(CON_CANAL);

    aplicacion.terminar(ResultadoDeTrabajo::CarpetaElegida(None));

    assert_eq!(lecturas.get(), 2);
}
