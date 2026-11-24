//! Estado de la ventana y sus reglas, sin egui.
//!
//! La ventana pinta lo que hay aquí y traduce cada clic en una llamada. Las operaciones que no
//! pueden correr en el hilo de la ventana devuelven un [`Trabajo`]; la ventana lo encarga y, al
//! terminar, pasa el resultado a [`Aplicacion::terminar`]. Lo que toca el disco va por
//! [`Servicios`], que se inyecta.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use nucleo::actualizaciones::Version;
use nucleo::configuracion::{
    ErrorDeConfiguracion, HerramientaDelPerfil, PerfilDeAgente, ProyectoRegistrado,
};
use nucleo::cuentas::Incorporacion;
use nucleo::proyecto::estado::EstadoDelProyecto;
use nucleo::proyecto::informe::InformeDeInicializacion;
use nucleo::proyecto::papeles::Papel;
use nucleo::proyecto::revision_del_canal::ResumenDelEquipo;
use nucleo::version_de_programa;

use crate::orden::{self, ClaveDeOrden};
use crate::perfiles::{self, CarpetaDelAgente, Disponibilidad, EstadoDelAgente};
use crate::textos;
use crate::trabajos::{Lanzamiento, ResultadoDeTrabajo, Trabajo};
pub use nucleo::orden::nombre_de_carpeta;

/// Respuesta del usuario a un diálogo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespuestaDeDialogo {
    /// «Sí» o «Aceptar».
    Aceptar,
    /// «No», «Cancelar» o cerrar la ventana del diálogo.
    Cancelar,
}

/// Guardado de la lista de proyectos y de los perfiles.
pub trait AlmacenDeConfiguracion {
    /// Guarda la lista de proyectos.
    ///
    /// # Errors
    ///
    /// El de la configuración si no se puede escribir.
    fn guardar_proyectos(
        &self,
        proyectos: &[ProyectoRegistrado],
    ) -> Result<(), ErrorDeConfiguracion>;
    /// Guarda los perfiles.
    ///
    /// # Errors
    ///
    /// El de la configuración si un perfil no es válido o no se puede escribir.
    fn guardar_perfiles(&self, perfiles: &[PerfilDeAgente]) -> Result<(), ErrorDeConfiguracion>;
    /// Archivo donde se editan los perfiles, para los mensajes.
    fn ruta_perfiles(&self) -> PathBuf;
}

/// Hora actual y su forma de mostrarla.
pub trait RelojDeInterfaz {
    /// Momento actual en ISO 8601 con desfase, para la última apertura.
    fn ahora(&self) -> String;
    /// Texto visible de una última apertura guardada.
    fn mostrar_fecha(&self, iso: &str) -> String;
}

/// Comprobaciones de disco y carpeta personal.
pub trait DiscoDeInterfaz {
    /// Indica si existe la carpeta.
    fn existe_carpeta(&self, carpeta: &Path) -> bool;
    /// Indica si existe el archivo.
    fn existe_archivo(&self, ruta: &Path) -> bool;
    /// Carpeta personal del usuario, para sustituir «~» en las rutas.
    fn carpeta_personal(&self) -> Option<PathBuf>;
}

/// Lo que se lee de un proyecto: su estado y su equipo.
pub trait InspectorDeProyecto {
    /// Estado de las instrucciones de un proyecto.
    fn estado_del_proyecto(&self, carpeta: &Path) -> EstadoDelProyecto;
    /// Cómo está el equipo del proyecto, leído de su .gestor/canal/, y la hora local con la que
    /// se calcula la antigüedad de los latidos. Nada si el proyecto no tiene estado del canal.
    fn equipo_del_proyecto(&self, _carpeta: &Path) -> Option<(ResumenDelEquipo, NaiveDateTime)> {
        None
    }
}

/// Lo que la aplicación necesita del sistema.
///
/// Junta los rasgos pequeños de cada responsabilidad.
pub trait Servicios:
    AlmacenDeConfiguracion + RelojDeInterfaz + DiscoDeInterfaz + InspectorDeProyecto
{
}

impl<T: AlmacenDeConfiguracion + RelojDeInterfaz + DiscoDeInterfaz + InspectorDeProyecto> Servicios
    for T
{
}

/// Fila de la tabla de proyectos, con lo que se muestra ya calculado para no leer el disco en
/// cada fotograma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fila {
    /// Proyecto tal como se guarda.
    pub registro: ProyectoRegistrado,
    /// Última apertura en el formato visible.
    pub fecha_visible: String,
    /// Estado de sus instrucciones.
    pub estado: EstadoDelProyecto,
}

/// Diálogo modal pendiente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialogo {
    /// Mensaje con un botón «Aceptar».
    Informacion {
        /// Título.
        titulo: String,
        /// Texto, que puede ser largo y se desplaza.
        texto: String,
    },
    /// Pregunta «Sí» o «No» antes de borrar la carpeta de un proyecto.
    ConfirmarBorrado {
        /// Nombre del proyecto.
        nombre: String,
        /// Carpeta que se borraría.
        ruta: String,
    },
}

impl Dialogo {
    /// Título del diálogo.
    #[must_use]
    pub fn titulo(&self) -> &str {
        match self {
            Self::Informacion { titulo, .. } => titulo,
            Self::ConfirmarBorrado { .. } => textos::TITULO_BORRADO,
        }
    }

    /// Texto del diálogo.
    #[must_use]
    pub fn texto(&self) -> String {
        match self {
            Self::Informacion { texto, .. } => texto.clone(),
            Self::ConfirmarBorrado { nombre, ruta } => textos::confirmar_borrado(nombre, ruta),
        }
    }

    /// Indica si el diálogo pide confirmación («Sí» y «No») en lugar de solo «Aceptar».
    #[must_use]
    pub const fn pide_confirmacion(&self) -> bool {
        matches!(self, Self::ConfirmarBorrado { .. })
    }
}

/// Estado completo de la ventana.
pub struct Aplicacion {
    servicios: Box<dyn Servicios>,
    filas: Vec<Fila>,
    perfiles: Vec<PerfilDeAgente>,
    /// Disponibilidad de cada perfil, en su orden; nada mientras no ha terminado la detección.
    disponibilidad: Option<Vec<Disponibilidad>>,
    /// Carpeta donde vive cada perfil y si existe, en su orden, de la última detección.
    carpetas: Vec<Option<CarpetaDelAgente>>,
    /// Indica si está en curso una búsqueda de agentes (al arrancar, Buscar de nuevo o al guardar
    /// las rutas). Solo la marca [`Self::detectar`] y la borra el final del trabajo.
    buscando: bool,
    /// Última versión publicada de Programator, si ha llegado la consulta.
    programator_publicado: Option<Version>,
    /// Ruta del ejecutable de cada perfil tal como se está editando en la pestaña de agentes.
    rutas: Vec<String>,
    seleccion: Option<String>,
    estado: String,
    dialogos: VecDeque<Dialogo>,
    ocupada: bool,
    /// Línea con el estado del equipo del proyecto seleccionado, ya compuesta.
    equipo: Option<String>,
    /// Papel elegido para el próximo agente que se abra. Vale para una sola apertura.
    papel: Option<Papel>,
    /// Si perfiles-agentes.json no se pudo leer al arrancar. Entonces la lista de esta sesión no
    /// es la del usuario, y guardarla sobrescribiría su archivo: no se guarda nada hasta que lo
    /// corrija y vuelva a abrir la aplicación.
    perfiles_ilegibles: bool,
}

impl Aplicacion {
    /// Estado inicial a partir de lo que se cargó de la configuración. Un archivo dañado no
    /// impide arrancar: la lista o los perfiles quedan vacíos y el error se ve en la barra de
    /// estado, como en la 2.3.2.
    #[must_use]
    pub fn new(
        servicios: Box<dyn Servicios>,
        proyectos: Result<Vec<ProyectoRegistrado>, ErrorDeConfiguracion>,
        perfiles: Result<Vec<PerfilDeAgente>, ErrorDeConfiguracion>,
    ) -> Self {
        let mut aplicacion = Self {
            servicios,
            filas: Vec::new(),
            perfiles: Vec::new(),
            disponibilidad: None,
            carpetas: Vec::new(),
            buscando: false,
            programator_publicado: None,
            rutas: Vec::new(),
            seleccion: None,
            estado: String::new(),
            dialogos: VecDeque::new(),
            ocupada: false,
            equipo: None,
            papel: None,
            perfiles_ilegibles: false,
        };
        match proyectos {
            Ok(proyectos) => {
                aplicacion.estado = textos::proyectos_registrados(proyectos.len());
                aplicacion.reconstruir(proyectos);
            }
            Err(error) => aplicacion.estado = textos::error_de_configuracion(&error),
        }
        match perfiles {
            Ok(perfiles) => aplicacion.poner_perfiles(perfiles),
            Err(error) => {
                aplicacion.estado = textos::error_de_configuracion(&error);
                aplicacion.perfiles_ilegibles = true;
            }
        }
        aplicacion
    }

    /// Filas de la tabla, ya ordenadas.
    #[must_use]
    pub fn filas(&self) -> &[Fila] {
        &self.filas
    }

    /// Perfiles, en el orden del archivo.
    #[must_use]
    pub fn perfiles(&self) -> &[PerfilDeAgente] {
        &self.perfiles
    }

    /// Posiciones de los perfiles cuyo botón se muestra: los disponibles. Vacío mientras no ha
    /// terminado la detección.
    #[must_use]
    pub fn perfiles_visibles(&self) -> Vec<usize> {
        self.disponibilidad
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, disponibilidad)| disponibilidad.disponible())
            .map(|(indice, _)| indice)
            .collect()
    }

    /// Indica si ya ha terminado la detección de agentes.
    #[must_use]
    pub const fn deteccion_terminada(&self) -> bool {
        self.disponibilidad.is_some()
    }

    /// Disponibilidad del perfil de esa posición; nada si no se ha detectado todavía.
    #[must_use]
    pub fn disponibilidad(&self, indice: usize) -> Option<&Disponibilidad> {
        self.disponibilidad.as_ref()?.get(indice)
    }

    /// Carpeta donde vive el perfil de esa posición y si existe; nada si no se sabe.
    #[must_use]
    pub fn carpeta_del_agente(&self, indice: usize) -> Option<&CarpetaDelAgente> {
        self.carpetas.get(indice)?.as_ref()
    }

    /// Anota la última versión publicada de Programator.
    pub fn fijar_programator_publicado(&mut self, version: Version) {
        self.programator_publicado = Some(version);
    }

    /// Última versión publicada de Programator, si ya ha llegado.
    #[must_use]
    pub const fn programator_publicado(&self) -> Option<Version> {
        self.programator_publicado
    }

    /// Versión publicada de Programator más nueva que la del programa del perfil, si la hay.
    #[must_use]
    pub fn version_mas_nueva(&self, indice: usize) -> Option<Version> {
        let instalada = self.carpeta_del_agente(indice)?.version();
        version_de_programa::mas_nueva(instalada, self.programator_publicado)
    }

    /// Indica si hay una búsqueda de agentes en curso. Abrir o borrar no cuentan.
    #[must_use]
    pub const fn buscando_agentes(&self) -> bool {
        self.buscando
    }

    /// Estado del perfil de esa posición en la pestaña Agentes (ver
    /// [`perfiles::estado_del_agente`]).
    #[must_use]
    pub fn estado_del_agente(&self, indice: usize) -> EstadoDelAgente {
        let ruta_cambiada = self.perfiles.get(indice).is_some_and(|perfil| {
            perfil.ejecutable().unwrap_or("") != self.ruta_editada(indice).trim()
        });
        let comprobacion = if ruta_cambiada || self.buscando {
            perfiles::Comprobacion::Pendiente
        } else {
            perfiles::Comprobacion::Vigente
        };
        perfiles::estado_del_agente(self.disponibilidad(indice), comprobacion)
    }

    /// Ruta del ejecutable del perfil tal como se está editando.
    #[must_use]
    pub fn ruta_editada(&self, indice: usize) -> &str {
        self.rutas.get(indice).map_or("", String::as_str)
    }

    /// Cambia la ruta que se está editando de un perfil. No se guarda hasta
    /// [`Self::guardar_rutas`].
    pub fn editar_ruta(&mut self, indice: usize, ruta: String) {
        if !self.edicion_activa() {
            return;
        }
        if let Some(editada) = self.rutas.get_mut(indice) {
            *editada = ruta;
        }
    }

    /// Deja vacía la ruta de un perfil, para que su programa se busque solo al guardar.
    pub fn busqueda_automatica(&mut self, indice: usize) {
        self.editar_ruta(indice, String::new());
    }

    /// Indica si alguna ruta editada difiere de la guardada.
    #[must_use]
    pub fn rutas_cambiadas(&self) -> bool {
        self.perfiles
            .iter()
            .zip(&self.rutas)
            .any(|(perfil, ruta)| perfil.ejecutable().unwrap_or("") != ruta.trim())
    }

    /// Pide el selector de archivos para el ejecutable de un perfil.
    #[must_use]
    pub fn pedir_ejecutable(&mut self, indice: usize) -> Option<Trabajo> {
        if !self.edicion_activa() || indice >= self.perfiles.len() {
            return None;
        }
        self.empezar_trabajo(Trabajo::ElegirEjecutable { indice })
    }

    /// Busca los agentes del equipo: añade las cuentas nuevas y calcula qué botones se muestran.
    #[must_use]
    pub fn detectar(&mut self) -> Option<Trabajo> {
        if !self.edicion_activa() {
            return None;
        }
        self.buscando = true;
        self.empezar_trabajo(Trabajo::Detectar {
            perfiles: self.perfiles.clone(),
        })
    }

    /// Da de alta a mano una cuenta de `herramienta`: pide su carpeta, la añade a los perfiles y
    /// vuelve a buscar los agentes (botones «Añadir cuenta de…» de la pestaña Agentes).
    #[must_use]
    pub fn pedir_cuenta(&mut self, herramienta: HerramientaDelPerfil) -> Option<Trabajo> {
        if !self.edicion_activa() {
            return None;
        }
        self.buscando = true;
        self.empezar_trabajo(Trabajo::DarDeAltaCuenta {
            herramienta,
            perfiles: self.perfiles.clone(),
        })
    }

    /// Guarda las rutas editadas y vuelve a buscar los agentes. Una ruta que no existe no se
    /// guarda: se avisa con la lista de las que fallan y no se cambia nada.
    #[must_use]
    pub fn guardar_rutas(&mut self) -> Option<Trabajo> {
        if !self.edicion_activa() {
            return None;
        }
        let personal = self.servicios.carpeta_personal();
        let mut nuevos = self.perfiles.clone();
        let mut inexistentes = Vec::new();
        for (perfil, ruta) in nuevos.iter_mut().zip(&self.rutas) {
            let ruta = ruta.trim();
            perfil.fijar_ejecutable(ruta);
            let Some(fijada) = perfiles::ejecutable_fijado(perfil, personal.as_deref()) else {
                continue;
            };
            if !self.servicios.existe_archivo(&fijada) {
                inexistentes.push((perfil.nombre(), ruta.to_owned()));
            }
        }
        if !inexistentes.is_empty() {
            self.informar(
                textos::TITULO_RUTAS,
                textos::rutas_inexistentes(&inexistentes),
            );
            return None;
        }
        if let Err(motivo) = self.guardar_perfiles(&nuevos) {
            self.informar(textos::TITULO_ERROR, motivo);
            return None;
        }
        self.poner_perfiles(nuevos);
        self.estado = textos::RUTAS_GUARDADAS.to_owned();
        self.detectar()
    }

    /// Texto de la barra de estado.
    #[must_use]
    pub fn texto_de_estado(&self) -> &str {
        &self.estado
    }

    /// Diálogo que hay que mostrar, si hay alguno.
    #[must_use]
    pub fn dialogo(&self) -> Option<&Dialogo> {
        self.dialogos.front()
    }

    /// Ruta del proyecto seleccionado.
    #[must_use]
    pub fn seleccion(&self) -> Option<&str> {
        self.seleccion.as_deref()
    }

    /// Indica si hay un trabajo en curso.
    #[must_use]
    pub const fn ocupada(&self) -> bool {
        self.ocupada
    }

    /// Indica si se pueden usar los botones que añaden o quitan proyectos.
    #[must_use]
    pub fn edicion_activa(&self) -> bool {
        !self.ocupada && self.dialogos.is_empty()
    }

    /// Indica si se pueden usar los botones que actúan sobre el proyecto seleccionado (abrir con
    /// un perfil y borrar del disco).
    #[must_use]
    pub fn acciones_activas(&self) -> bool {
        self.edicion_activa() && self.fila_seleccionada().is_some()
    }

    /// Selecciona el proyecto de esa ruta y lee cómo está su equipo.
    pub fn seleccionar(&mut self, ruta: &str) {
        if self.filas.iter().any(|fila| fila.registro.ruta() == ruta) {
            self.seleccion = Some(ruta.to_owned());
            self.leer_equipo();
        }
    }

    /// Línea con el estado del equipo del proyecto seleccionado, si tiene estado del canal.
    #[must_use]
    pub fn equipo(&self) -> Option<&str> {
        self.equipo.as_deref()
    }

    /// Vuelve a leer el equipo del proyecto seleccionado. Son unos pocos archivos pequeños de
    /// .gestor/canal/, y se leen al seleccionar y al terminar un trabajo, no en cada fotograma.
    fn leer_equipo(&mut self) {
        self.equipo = self.seleccion.as_deref().and_then(|ruta| {
            self.servicios
                .equipo_del_proyecto(Path::new(ruta))
                .map(|(resumen, ahora)| textos::linea_del_equipo(&resumen, ahora))
        });
    }

    /// Pide el selector de carpetas para añadir un proyecto.
    #[must_use]
    pub fn pedir_carpeta(&mut self) -> Option<Trabajo> {
        if !self.edicion_activa() {
            return None;
        }
        self.empezar_trabajo(Trabajo::ElegirCarpeta)
    }

    /// Quita un proyecto de la lista sin tocar el disco (botón «Quitar» de su fila).
    pub fn quitar(&mut self, ruta: &str) {
        if !self.edicion_activa() {
            return;
        }
        let Some(posicion) = self
            .filas
            .iter()
            .position(|fila| fila.registro.ruta() == ruta)
        else {
            return;
        };
        let fila = self.filas.remove(posicion);
        if self.seleccion.as_deref() == Some(ruta) {
            self.seleccion = None;
        }
        self.estado = textos::proyecto_quitado(fila.registro.nombre());
        self.guardar();
    }

    /// Prepara el proyecto seleccionado y abre en él el perfil indicado por su posición.
    #[must_use]
    pub fn abrir_con(&mut self, indice_perfil: usize) -> Option<Trabajo> {
        if !self.acciones_activas() {
            return None;
        }
        if !self.disponibilidad(indice_perfil)?.disponible() {
            return None;
        }
        let perfil = self.perfiles.get(indice_perfil)?.clone();
        let registro = self.fila_seleccionada()?.registro.clone();
        if !self.servicios.existe_carpeta(Path::new(registro.ruta())) {
            self.informar(
                textos::TITULO_ERROR,
                textos::carpeta_inexistente(registro.ruta()),
            );
            return None;
        }
        self.estado = textos::preparando(registro.nombre(), &perfil.nombre());
        let papel = self.papel.take();
        self.empezar_trabajo(Trabajo::Abrir {
            nombre: registro.nombre().to_owned(),
            ruta: registro.ruta().to_owned(),
            perfil,
            papel,
        })
    }

    /// Papel elegido para el próximo agente que se abra, si lo hay.
    #[must_use]
    pub const fn papel(&self) -> Option<Papel> {
        self.papel
    }

    /// Elige el papel del próximo agente que se abra; nada para que lo pregunte él al arrancar.
    pub fn elegir_papel(&mut self, papel: Option<Papel>) {
        self.papel = papel;
    }

    /// Pide borrar del disco el proyecto seleccionado. Si su carpeta ya no existe, lo quita de la
    /// lista y lo dice; si existe, pide confirmación.
    pub fn pedir_borrado(&mut self) {
        if !self.acciones_activas() {
            return;
        }
        let Some(registro) = self.fila_seleccionada().map(|fila| fila.registro.clone()) else {
            return;
        };
        if self.servicios.existe_carpeta(Path::new(registro.ruta())) {
            self.dialogos.push_back(Dialogo::ConfirmarBorrado {
                nombre: registro.nombre().to_owned(),
                ruta: registro.ruta().to_owned(),
            });
            return;
        }
        self.informar(
            textos::TITULO_ERROR,
            textos::carpeta_inexistente_en_disco(registro.ruta()),
        );
        self.filas
            .retain(|fila| fila.registro.ruta() != registro.ruta());
        self.seleccion = None;
        self.guardar();
    }

    /// Cierra el diálogo actual con la respuesta del usuario. Confirmar un borrado devuelve el
    /// trabajo que lo hace.
    #[must_use]
    pub fn cerrar_dialogo(&mut self, respuesta: RespuestaDeDialogo) -> Option<Trabajo> {
        match self.dialogos.pop_front()? {
            Dialogo::ConfirmarBorrado { nombre, ruta }
                if respuesta == RespuestaDeDialogo::Aceptar =>
            {
                self.empezar_trabajo(Trabajo::Borrar { nombre, ruta })
            }
            _ => None,
        }
    }

    /// Recoge el resultado de un trabajo.
    pub fn terminar(&mut self, resultado: ResultadoDeTrabajo) {
        self.ocupada = false;
        self.buscando = false;
        match resultado {
            ResultadoDeTrabajo::CarpetaElegida(Some(carpeta)) => self.agregar(&carpeta),
            ResultadoDeTrabajo::CarpetaElegida(None) => {}
            ResultadoDeTrabajo::Abierto {
                nombre,
                ruta,
                perfil,
                informe,
                tamano_instrucciones,
                lanzamiento,
            } => self.recibir_apertura(
                &nombre,
                &ruta,
                &perfil,
                &informe,
                tamano_instrucciones,
                lanzamiento,
            ),
            ResultadoDeTrabajo::Borrado {
                nombre,
                ruta,
                error: None,
            } => {
                self.filas.retain(|fila| fila.registro.ruta() != ruta);
                if self.seleccion.as_deref() == Some(ruta.as_str()) {
                    self.seleccion = None;
                }
                self.estado = textos::proyecto_borrado(&nombre);
                self.guardar();
            }
            ResultadoDeTrabajo::Borrado {
                error: Some(error), ..
            } => self.informar(textos::TITULO_ERROR, textos::borrado_fallido(&error)),
            ResultadoDeTrabajo::Detectado {
                perfiles,
                disponibilidad,
                carpetas,
                anadidos,
                normalizados,
            } => self.recibir_deteccion(
                perfiles,
                disponibilidad,
                carpetas,
                Incorporacion {
                    normalizados,
                    anadidos,
                },
            ),
            ResultadoDeTrabajo::EjecutableElegido {
                indice,
                ruta: Some(ruta),
            } => match ruta.to_str() {
                Some(texto) => self.editar_ruta(indice, texto.to_owned()),
                None => self.informar(textos::TITULO_ERROR, textos::ruta_no_unicode(&ruta)),
            },
            ResultadoDeTrabajo::EjecutableElegido { ruta: None, .. } => {}
            ResultadoDeTrabajo::CuentaNoAnadida(None) => {}
            ResultadoDeTrabajo::CuentaNoAnadida(Some(rechazo)) => {
                self.informar(textos::TITULO_CUENTA, textos::cuenta_rechazada(&rechazo));
            }
        }
        // Un trabajo, como abrir un agente, puede haber cambiado el canal del proyecto.
        self.leer_equipo();
    }

    /// Recoge la preparación y el lanzamiento de un agente: la barra de estado con sus avisos, el
    /// informe si pide ventana y el fallo del lanzamiento, si lo hubo.
    fn recibir_apertura(
        &mut self,
        nombre: &str,
        ruta: &str,
        perfil: &PerfilDeAgente,
        informe: &InformeDeInicializacion,
        tamano_instrucciones: Option<u64>,
        lanzamiento: Lanzamiento,
    ) {
        self.marcar_apertura(ruta);
        self.estado =
            self.estado_tras_abrir(nombre, perfil, informe, tamano_instrucciones, &lanzamiento);
        if informe.requiere_ventana() {
            self.informar(
                &textos::titulo_preparacion(nombre),
                informe.componer_texto(),
            );
        }
        if let Lanzamiento::Fallido(error) = lanzamiento {
            let (titulo, texto) =
                textos::fallo_de_lanzamiento(&error, nombre, &self.servicios.ruta_perfiles());
            self.informar(&titulo, texto);
        }
    }

    /// Texto de la barra de estado tras abrir un agente, con cada aviso que toque.
    fn estado_tras_abrir(
        &self,
        nombre: &str,
        perfil: &PerfilDeAgente,
        informe: &InformeDeInicializacion,
        tamano_instrucciones: Option<u64>,
        lanzamiento: &Lanzamiento,
    ) -> String {
        let identidad = perfil.nombre();
        let mut estado = match lanzamiento {
            Lanzamiento::Abierto => textos::abierto(&identidad, nombre, informe.cambios().len()),
            Lanzamiento::Fallido(_) => textos::preparado_sin_abrir(nombre, &identidad),
        };
        if let Some(aviso) = perfiles::aviso_de_tamano(perfil, tamano_instrucciones) {
            estado = textos::con_aviso(&estado, &aviso);
        }
        if let Some(nueva) = self.version_mas_nueva_de(perfil) {
            estado = textos::con_aviso(&estado, &textos::programator_anticuado(nueva));
        }
        match informe.avisos().len() {
            0 => estado,
            avisos => textos::con_aviso(&estado, &textos::avisos_para_los_agentes(avisos)),
        }
    }

    /// Pone los perfiles detectados y su disponibilidad; si han cambiado, los guarda. Solo las
    /// cuentas nuevas se anuncian en la barra de estado: quitar a un perfil la carpeta por defecto
    /// no cambia nada que el usuario vea. Si no se pueden guardar, se usan igualmente en esta
    /// sesión y se avisa en la barra de estado.
    fn recibir_deteccion(
        &mut self,
        perfiles: Vec<PerfilDeAgente>,
        disponibilidad: Vec<Disponibilidad>,
        carpetas: Vec<Option<CarpetaDelAgente>>,
        cambios: Incorporacion,
    ) {
        if cambios.hay_cambios() {
            match self.guardar_perfiles(&perfiles) {
                Ok(()) if cambios.anadidos > 0 => {
                    self.estado = textos::cuentas_anadidas(cambios.anadidos);
                }
                Ok(()) => {}
                Err(motivo) => self.estado = motivo,
            }
        }
        self.poner_perfiles(perfiles);
        self.disponibilidad = Some(disponibilidad);
        self.carpetas = carpetas;
    }

    /// Guarda la lista de agentes, salvo que la del usuario no se pudiera leer al arrancar (ver
    /// el campo perfiles_ilegibles). Devuelve el texto que explica por qué no se ha guardado.
    fn guardar_perfiles(&self, perfiles: &[PerfilDeAgente]) -> Result<(), String> {
        if self.perfiles_ilegibles {
            return Err(textos::PERFILES_ILEGIBLES_SIN_GUARDAR.to_owned());
        }
        self.servicios
            .guardar_perfiles(perfiles)
            .map_err(|error| textos::perfiles_no_guardados(&error))
    }

    /// [`Self::version_mas_nueva`] del perfil `perfil`, buscado por su identidad.
    fn version_mas_nueva_de(&self, perfil: &PerfilDeAgente) -> Option<Version> {
        let indice = self
            .perfiles
            .iter()
            .position(|guardado| guardado.nombre() == perfil.nombre())?;
        self.version_mas_nueva(indice)
    }

    /// Cambia los perfiles y deja las rutas editables como las guardadas.
    fn poner_perfiles(&mut self, perfiles: Vec<PerfilDeAgente>) {
        self.rutas = perfiles
            .iter()
            .map(|perfil| perfil.ejecutable().unwrap_or_default().to_owned())
            .collect();
        self.perfiles = perfiles;
    }

    /// El hilo de trabajo se ha perdido: se deja de esperar y se avisa.
    pub fn trabajador_perdido(&mut self) {
        self.ocupada = false;
        self.buscando = false;
        self.estado = textos::TRABAJADOR_PERDIDO.to_owned();
    }

    /// Añade un aviso informativo, como el de la configuración importada de la 2.x.
    pub fn informar(&mut self, titulo: &str, texto: String) {
        self.dialogos.push_back(Dialogo::Informacion {
            titulo: titulo.to_owned(),
            texto,
        });
    }

    /// Cambia el texto de la barra de estado.
    pub fn anunciar(&mut self, texto: String) {
        self.estado = texto;
    }

    /// Añade la carpeta elegida, salvo que ya esté en la lista.
    fn agregar(&mut self, carpeta: &Path) {
        let Some(ruta) = carpeta.to_str() else {
            self.informar(textos::TITULO_ERROR, textos::ruta_no_unicode(carpeta));
            return;
        };
        if self
            .filas
            .iter()
            .any(|fila| orden::misma_ruta(fila.registro.ruta(), ruta))
        {
            self.informar(
                textos::TITULO_DUPLICADO,
                textos::PROYECTO_DUPLICADO.to_owned(),
            );
            return;
        }
        let registro =
            ProyectoRegistrado::new(nombre_de_carpeta(ruta), ruta, self.servicios.ahora());
        self.estado = textos::proyecto_agregado(registro.nombre());
        let mut proyectos = self.registros();
        proyectos.push(registro);
        self.reconstruir(proyectos);
        self.guardar();
    }

    /// Anota ahora como última apertura del proyecto y recalcula su estado.
    fn marcar_apertura(&mut self, ruta: &str) {
        let ahora = self.servicios.ahora();
        let mut proyectos = self.registros();
        let Some(registro) = proyectos
            .iter_mut()
            .find(|registro| registro.ruta() == ruta)
        else {
            return;
        };
        registro.anotar_apertura(ahora);
        self.reconstruir(proyectos);
        self.guardar();
    }

    /// Marca la ventana como ocupada y devuelve el trabajo. Quien llama ya ha comprobado que no
    /// había otro en curso, con [`Self::edicion_activa`] o [`Self::acciones_activas`].
    fn empezar_trabajo(&mut self, trabajo: Trabajo) -> Option<Trabajo> {
        self.ocupada = true;
        Some(trabajo)
    }

    fn fila_seleccionada(&self) -> Option<&Fila> {
        let ruta = self.seleccion.as_deref()?;
        self.filas.iter().find(|fila| fila.registro.ruta() == ruta)
    }

    fn registros(&self) -> Vec<ProyectoRegistrado> {
        self.filas
            .iter()
            .map(|fila| fila.registro.clone())
            .collect()
    }

    /// Vuelve a calcular las filas, ordenadas por nombre, a partir de los registros.
    fn reconstruir(&mut self, mut proyectos: Vec<ProyectoRegistrado>) {
        proyectos.sort_by_cached_key(|registro| ClaveDeOrden::de(registro.nombre()));
        self.filas = proyectos
            .into_iter()
            .map(|registro| Fila {
                fecha_visible: self.servicios.mostrar_fecha(registro.ultima_apertura()),
                estado: self
                    .servicios
                    .estado_del_proyecto(Path::new(registro.ruta())),
                registro,
            })
            .collect();
    }

    fn guardar(&mut self) {
        if let Err(error) = self.servicios.guardar_proyectos(&self.registros()) {
            self.estado = textos::guardado_fallido(&error);
        }
    }
}
