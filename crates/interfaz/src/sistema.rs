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
use nucleo::actualizaciones::{self, Version};
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
use nucleo::proyecto::momento::{Reloj, RelojDelSistema};
use nucleo::proyecto::papeles::{self, Papel};
use nucleo::proyecto::revision_del_canal;
use nucleo::proyecto::vigilancia;
use vigilante::opciones::ProcesoMarcado;
use vigilante::procesos::{self, ProcesosDelSistema};

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
        let mut resumen = revision_del_canal::resumir(&estado, &bandejas);
        resumen.vigilancia = vigilancia_del_equipo(carpeta, &bandejas);
        Some((resumen, Local::now().naive_local()))
    }
}

/// Vigilancia de cada agente con bandeja: su vivo.json, comprobado contra los procesos del sistema,
/// y los encargos de su avisos.log posteriores a su último acuse.
fn vigilancia_del_equipo(
    carpeta: &Path,
    bandejas: &[revision_del_canal::Bandeja],
) -> Vec<vigilancia::VigilanciaDeAgente> {
    let mut procesos = ProcesosDelSistema::new();
    bandejas
        .iter()
        .filter_map(|bandeja| {
            let identificador = bandeja.nombre.strip_suffix(".md")?.to_owned();
            let recuerdo = carpeta
                .join(".gestor")
                .join(&identificador)
                .join(vigilancia::CARPETA_DEL_VIGILANTE);
            let vigilante = fs::read_to_string(recuerdo.join(vigilancia::ARCHIVO_VIVO))
                .ok()
                .and_then(|texto| vigilancia::leer_vivo(&texto))
                .map(|anotado| {
                    let marcado = ProcesoMarcado {
                        pid: anotado.pid,
                        arranque: anotado.arranque,
                    };
                    if procesos::vivo(&mut procesos, marcado) {
                        vigilancia::EstadoDelVigilante::Vigilando
                    } else {
                        vigilancia::EstadoDelVigilante::Caido
                    }
                });
            let desde = vigilancia::ultimo_acuse(&bandeja.texto)
                .and_then(|hora| hora.and_local_timezone(Local).earliest())
                .map(|hora| hora.timestamp());
            let encargos_sin_acuse =
                fs::read_to_string(recuerdo.join(vigilancia::ARCHIVO_DE_AVISOS))
                    .map(|avisos| vigilancia::encargos_sin_acuse(&avisos, desde))
                    .unwrap_or(0);
            (vigilante.is_some() || encargos_sin_acuse > 0).then_some(
                vigilancia::VigilanciaDeAgente {
                    identificador,
                    vigilante,
                    encargos_sin_acuse,
                },
            )
        })
        .collect()
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

    fn carpeta_de_datos(&self) -> Option<PathBuf> {
        self.configuracion.carpeta_de_datos()
    }

    fn limite_de_latido_horas(&self) -> u64 {
        self.ajustes.limite_de_latido_horas()
    }

    fn limite_de_bandeja(&self) -> u64 {
        self.ajustes.limite_de_bandeja()
    }

    /// El gestor-vigilante que va junto al ejecutable de Gestor Agentes, en su paquete.
    fn ejecutable_del_vigilante(&self) -> Option<PathBuf> {
        let propio = std::env::current_exe().ok()?;
        Some(
            propio
                .parent()?
                .join(nucleo::proyecto::vigilante::nombre_del_ejecutable()),
        )
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

    fn elegir_carpeta_de_cuenta(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(textos::TITULO_SELECTOR_CUENTA)
            .pick_folder()
    }

    fn version_del_ejecutable(&self, ruta: &Path) -> Option<Version> {
        fs::read(ruta)
            .ok()
            .and_then(|bytes| nucleo::version_de_programa::version_en_recurso(&bytes))
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

    fn anotar_papel(&self, raiz: &Path, identidad: &str, papel: Papel) -> std::io::Result<bool> {
        papeles::anotar_en_disco(raiz, identidad, papel, RelojDelSistema.ahora())
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
