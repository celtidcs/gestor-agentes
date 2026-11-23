//! Árbol de procesos: quién lanzó al vigilante y si sigue vivo. Lo que decide no depende del
//! sistema: recibe un [`Procesos`], que en producción es [`ProcesosDelSistema`] (sysinfo) y en las
//! pruebas una tabla fija.

use std::path::PathBuf;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::opciones::ProcesoMarcado;

/// Lo que se sabe de un proceso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatosDeProceso {
    /// Número de proceso.
    pub pid: u32,
    /// Número del padre, si se conoce.
    pub padre: Option<u32>,
    /// Hora de arranque, en segundos desde 1970.
    pub arranque: u64,
    /// Nombre del programa, como «codex.exe» o «claude».
    pub nombre: String,
    /// Ruta del ejecutable, si se conoce.
    pub ejecutable: Option<PathBuf>,
}

impl DatosDeProceso {
    /// El proceso con su hora de arranque.
    #[must_use]
    pub const fn marcado(&self) -> ProcesoMarcado {
        ProcesoMarcado {
            pid: self.pid,
            arranque: self.arranque,
        }
    }
}

/// Fuente de datos de procesos.
pub trait Procesos {
    /// Datos del proceso `pid` en este momento, o nada si no existe.
    fn datos(&mut self, pid: u32) -> Option<DatosDeProceso>;

    /// Todos los procesos en este momento. Por defecto ninguno: solo lo necesita el vigilante que
    /// lanza la terminal de Codex ([`descendiente_llamado`]).
    fn todos(&mut self) -> Vec<DatosDeProceso> {
        Vec::new()
    }
}

/// Procesos del sistema, leídos con sysinfo.
pub struct ProcesosDelSistema {
    sistema: System,
}

impl ProcesosDelSistema {
    /// Crea la fuente sin leer nada todavía.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sistema: System::new(),
        }
    }
}

impl Default for ProcesosDelSistema {
    fn default() -> Self {
        Self::new()
    }
}

impl Procesos for ProcesosDelSistema {
    fn datos(&mut self, pid: u32) -> Option<DatosDeProceso> {
        let buscado = Pid::from_u32(pid);
        self.sistema.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[buscado]),
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        );
        let proceso = self.sistema.process(buscado)?;
        Some(DatosDeProceso {
            pid,
            padre: proceso.parent().map(Pid::as_u32),
            arranque: proceso.start_time(),
            nombre: proceso.name().to_string_lossy().into_owned(),
            ejecutable: proceso.exe().map(PathBuf::from),
        })
    }

    fn todos(&mut self) -> Vec<DatosDeProceso> {
        self.sistema.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        );
        self.sistema
            .processes()
            .iter()
            .map(|(pid, proceso)| DatosDeProceso {
                pid: pid.as_u32(),
                padre: proceso.parent().map(Pid::as_u32),
                arranque: proceso.start_time(),
                nombre: proceso.name().to_string_lossy().into_owned(),
                ejecutable: proceso.exe().map(PathBuf::from),
            })
            .collect()
    }
}

/// El descendiente de `raiz` más cercano (por niveles) cuyo nombre, sin distinguir mayúsculas, es
/// uno de `nombres`. Así se encuentra el codex que abrió la terminal y no uno de sus procesos
/// auxiliares, como codex-code-mode-host.
#[must_use]
pub fn descendiente_llamado(
    lista: &[DatosDeProceso],
    raiz: u32,
    nombres: &[&str],
) -> Option<DatosDeProceso> {
    let mut nivel = vec![raiz];
    let mut vistos = vec![raiz];
    while !nivel.is_empty() {
        let hijos: Vec<&DatosDeProceso> = lista
            .iter()
            .filter(|datos| datos.padre.is_some_and(|padre| nivel.contains(&padre)))
            .filter(|datos| !vistos.contains(&datos.pid))
            .collect();
        if let Some(encontrado) = hijos.iter().find(|datos| {
            nombres
                .iter()
                .any(|nombre| datos.nombre.eq_ignore_ascii_case(nombre))
        }) {
            return Some((*encontrado).clone());
        }
        nivel = hijos.iter().map(|datos| datos.pid).collect();
        vistos.extend(&nivel);
    }
    None
}

/// Hasta `cuantos` ascendientes de `pid`, del padre hacia arriba.
pub fn ascendientes(procesos: &mut dyn Procesos, pid: u32, cuantos: usize) -> Vec<DatosDeProceso> {
    let mut lista = Vec::new();
    let mut actual = procesos.datos(pid).and_then(|datos| datos.padre);
    while let Some(padre) = actual {
        if lista.len() >= cuantos {
            break;
        }
        let Some(datos) = procesos.datos(padre) else {
            break;
        };
        actual = datos.padre.filter(|siguiente| *siguiente != padre);
        lista.push(datos);
    }
    lista
}

/// El primer ascendiente de `pid`, entre los `cuantos` más cercanos, cuyo nombre empieza por
/// `prefijo` sin distinguir mayúsculas.
pub fn ascendiente_llamado(
    procesos: &mut dyn Procesos,
    pid: u32,
    prefijo: &str,
    cuantos: usize,
) -> Option<DatosDeProceso> {
    let prefijo = prefijo.to_lowercase();
    ascendientes(procesos, pid, cuantos)
        .into_iter()
        .find(|datos| datos.nombre.to_lowercase().starts_with(&prefijo))
}

/// Si sigue vivo el proceso con ese número y esa hora de arranque.
pub fn vivo(procesos: &mut dyn Procesos, marcado: ProcesoMarcado) -> bool {
    procesos
        .datos(marcado.pid)
        .is_some_and(|datos| datos.arranque == marcado.arranque)
}
