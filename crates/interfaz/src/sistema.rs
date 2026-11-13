//! Implementaciones reales de [`Servicios`] y [`Motor`], sobre el disco, el reloj, el núcleo y
//! el lanzador. Solo componen piezas que ya tienen sus propias pruebas.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Local;
use lanzador::{
    EntornoDelSistema, EntornoReal, ErrorDeLanzamiento, Herramienta, PeticionDeLanzamiento,
    PlanDeLanzamiento, Sistema,
};
use nucleo::actualizaciones;
use nucleo::agentes::IdentidadAgente;
use nucleo::ajustes::Ajustes;
use nucleo::configuracion::{
    Configuracion, ErrorDeConfiguracion, PerfilDeAgente, ProyectoRegistrado,
};
use nucleo::cuentas::{self, CarpetaPersonal};
use nucleo::git::{EjecutorDelSistema, ServicioGit};
use nucleo::proyecto::estado::EstadoDelProyecto;
use nucleo::proyecto::informe::InformeDeInicializacion;
use nucleo::proyecto::inicializador::{
    self, ConfiguracionDeInstrucciones, InicializadorDeProyecto,
};
use nucleo::proyecto::localizador::LocalizadorDeInstrucciones;
use nucleo::proyecto::momento::RelojDelSistema;
use nucleo::proyecto::revision_del_canal;

use crate::actualizaciones::ConsultorDePublicaciones;
use crate::aplicacion::Servicios;
use crate::fecha;
use crate::textos;
use crate::trabajos::Motor;

/// Consulta real a la API de GitHub con ureq. Solo manda las cabeceras User-Agent y Accept.
pub struct ConsultorGithub;

impl ConsultorDePublicaciones for ConsultorGithub {
    fn consultar(&self, direccion: &str, agente: &str, tiempo: Duration) -> Option<String> {
        ureq::AgentBuilder::new()
            .timeout(tiempo)
            .user_agent(agente)
            .build()
            .get(direccion)
            .set("Accept", actualizaciones::ACEPTA_LA_API)
            .call()
            .ok()?
            .into_string()
            .ok()
    }
}

/// Servicios sobre la configuración de datos/ y el reloj del equipo.
pub struct ServiciosReales {
    configuracion: Configuracion,
}

impl ServiciosReales {
    /// Servicios sobre esa configuración.
    #[must_use]
    pub const fn new(configuracion: Configuracion) -> Self {
        Self { configuracion }
    }
}

impl Servicios for ServiciosReales {
    fn guardar_proyectos(
        &self,
        proyectos: &[ProyectoRegistrado],
    ) -> Result<(), ErrorDeConfiguracion> {
        self.configuracion.guardar_proyectos(proyectos)
    }

    fn estado_del_proyecto(&self, carpeta: &Path) -> EstadoDelProyecto {
        EstadoDelProyecto::de(carpeta)
    }

    fn existe_carpeta(&self, carpeta: &Path) -> bool {
        carpeta.is_dir()
    }

    fn ahora(&self) -> String {
        fecha::iso(&Local::now())
    }

    fn mostrar_fecha(&self, iso: &str) -> String {
        fecha::mostrar(iso, &Local)
    }

    fn ruta_perfiles(&self) -> PathBuf {
        self.configuracion.ruta_perfiles()
    }

    fn guardar_perfiles(&self, perfiles: &[PerfilDeAgente]) -> Result<(), ErrorDeConfiguracion> {
        self.configuracion.guardar_perfiles(perfiles)
    }

    fn existe_archivo(&self, ruta: &Path) -> bool {
        ruta.is_file()
    }

    fn carpeta_personal(&self) -> Option<PathBuf> {
        EntornoReal.carpeta_personal()
    }

    fn equipo_del_proyecto(
        &self,
        carpeta: &Path,
    ) -> Option<(revision_del_canal::ResumenDelEquipo, chrono::NaiveDateTime)> {
        let estado = fs::read_to_string(carpeta.join(".gestor/canal/estado.md")).ok()?;
        let bandejas = inicializador::leer_bandejas(carpeta).unwrap_or_default();
        Some((
            revision_del_canal::resumir(&estado, &bandejas),
            Local::now().naive_local(),
        ))
    }
}

/// Motor real: selector de carpetas del sistema, inicializador del núcleo y lanzador.
pub struct MotorReal {
    configuracion: Configuracion,
    ajustes: Ajustes,
}

impl MotorReal {
    /// Motor que lee las instrucciones antiguas de esa configuración y usa esos ajustes.
    #[must_use]
    pub const fn new(configuracion: Configuracion, ajustes: Ajustes) -> Self {
        Self {
            configuracion,
            ajustes,
        }
    }
}

impl ConfiguracionDeInstrucciones for MotorReal {
    fn leer_instrucciones_antiguas(&self) -> Option<String> {
        self.configuracion.leer_instrucciones_antiguas()
    }

    fn limite_de_latido_horas(&self) -> u64 {
        self.ajustes.limite_de_latido_horas()
    }

    fn limite_de_bandeja(&self) -> u64 {
        self.ajustes.limite_de_bandeja()
    }
}

impl Motor for MotorReal {
    fn elegir_carpeta(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(textos::TITULO_SELECTOR)
            .pick_folder()
    }

    fn elegir_ejecutable(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(textos::TITULO_SELECTOR_EJECUTABLE)
            .pick_file()
    }

    fn preparar(&self, raiz: &Path, identidad: Option<IdentidadAgente>) -> InformeDeInicializacion {
        let ejecutor = EjecutorDelSistema::con_tiempo_maximo(self.ajustes.tiempo_maximo_git());
        InicializadorDeProyecto::new(
            self,
            &LocalizadorDeInstrucciones,
            ServicioGit::new(&ejecutor),
            &RelojDelSistema,
        )
        .inicializar(raiz, identidad)
    }

    fn carpeta_personal(&self) -> Option<PathBuf> {
        EntornoReal.carpeta_personal()
    }

    fn planificar(
        &self,
        peticion: &PeticionDeLanzamiento,
    ) -> Result<PlanDeLanzamiento, ErrorDeLanzamiento> {
        lanzador::planificar(peticion, Sistema::actual(), &EntornoReal)
    }

    fn detectar_ejecutable(
        &self,
        herramienta: Herramienta,
        ruta_fijada: Option<&Path>,
    ) -> Option<PathBuf> {
        lanzador::detectar_ejecutable(herramienta, ruta_fijada, Sistema::actual(), &EntornoReal)
    }

    fn explorar_cuentas(&self, carpeta_personal: &Path) -> Vec<CarpetaPersonal> {
        cuentas::explorar(carpeta_personal)
    }

    fn existe_carpeta(&self, ruta: &Path) -> bool {
        ruta.is_dir()
    }

    fn ejecutar(&self, plan: &PlanDeLanzamiento) -> Result<(), ErrorDeLanzamiento> {
        lanzador::ejecutar(plan)
    }

    fn borrar_carpeta(&self, ruta: &Path) -> std::io::Result<()> {
        fs::remove_dir_all(ruta)
    }

    fn tamano(&self, ruta: &Path) -> Option<u64> {
        fs::metadata(ruta).ok().map(|datos| datos.len())
    }
}
