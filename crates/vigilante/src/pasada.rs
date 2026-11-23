//! Una pasada del vigilante: compara el canal de ahora con lo visto antes y dice qué ha cambiado y
//! con qué etiquetas. Lógica pura: los textos y lo visto llegan como argumentos.

use std::collections::{BTreeMap, BTreeSet};

use canal::acuses;
use canal::encargos::{self, Encargo};
use canal::huella;
use canal::identidad::Identidad;
use canal::mutaciones;

/// Archivo del canal con el estado común.
pub const ESTADO: &str = "estado.md";

/// Bandeja de Programator. Programator no recibe encargos de nadie por ella: lo que va «Para
/// <identidad>» en ella es su respuesta al encargo de esa identidad (acuerdo con el proyecto
/// Programator, 2026-11-22).
pub const BANDEJA_DE_PROGRAMATOR: &str = "programator.md";

/// Archivos del canal que no se vigilan, en minúsculas: la guía de Programator.
const NO_VIGILADOS: [&str; 1] = ["como-encargar-a-programator.md"];

/// Prefijos de los históricos del canal, con y sin tilde, que no se vigilan.
const PREFIJOS_DE_HISTORICO: [&str; 2] = ["historico-", "histórico-"];

/// Lo visto hasta la pasada anterior.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Visto {
    /// Huella de cada archivo vigilado, por nombre.
    pub archivos: BTreeMap<String, u64>,
    /// Huellas de los encargos para el agente que ya se le han avisado.
    pub encargos: BTreeSet<u64>,
    /// Huella de las mutaciones vivas avisadas, o nada si no había.
    pub mutaciones: Option<u64>,
}

/// Un archivo que ha cambiado, con sus etiquetas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cambio {
    /// Nombre dentro de .gestor/canal/.
    pub archivo: String,
    /// Si trae un encargo nuevo para el agente.
    pub encargo: bool,
    /// Si trae mutaciones vivas nuevas o cambiadas.
    pub mutacion: bool,
}

impl Cambio {
    /// Si lleva alguna etiqueta.
    #[must_use]
    pub const fn etiquetado(&self) -> bool {
        self.encargo || self.mutacion
    }

    /// Si lo nuevo para el agente es la respuesta de Programator a un encargo suyo.
    #[must_use]
    pub fn respuesta_de_programator(&self) -> bool {
        self.encargo && self.archivo.eq_ignore_ascii_case(BANDEJA_DE_PROGRAMATOR)
    }

    /// Línea de aviso, como la escribían los guiones de la 3.5.1.
    #[must_use]
    pub fn linea(&self, identidad: &Identidad) -> String {
        let mut linea = format!("CANAL: ha cambiado .gestor/canal/{}", self.archivo);
        if self.respuesta_de_programator() {
            linea.push_str(&format!(" [RESPUESTA DE PROGRAMATOR PARA {identidad}]"));
        } else if self.encargo {
            linea.push_str(&format!(" [ENCARGO PARA {identidad}]"));
        }
        if self.mutacion {
            linea.push_str(" [MUTACION VIVA]");
        }
        linea
    }
}

/// Resultado de una pasada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resultado {
    /// Archivos cambiados, por orden de nombre.
    pub cambios: Vec<Cambio>,
    /// Lo visto tras esta pasada.
    pub visto: Visto,
}

impl Resultado {
    /// Si algún cambio lleva etiqueta.
    #[must_use]
    pub fn hay_etiquetas(&self) -> bool {
        self.cambios.iter().any(Cambio::etiquetado)
    }
}

/// Si un archivo de .gestor/canal/ se vigila para el agente cuya bandeja es `propia`: los .md
/// que no son la bandeja propia, la guía de Programator ni un histórico. No distingue mayúsculas.
#[must_use]
pub fn se_vigila(nombre: &str, propia: &str) -> bool {
    let nombre = nombre.to_lowercase();
    nombre.ends_with(".md")
        && nombre != propia.to_lowercase()
        && !NO_VIGILADOS.contains(&nombre.as_str())
        && !PREFIJOS_DE_HISTORICO
            .iter()
            .any(|prefijo| nombre.starts_with(prefijo))
}

/// Lo visto en una primera pasada: todo lo de ahora, sin avisar de nada.
#[must_use]
pub fn primera(archivos: &[(String, String)], identidad: &Identidad) -> Visto {
    let mut visto = Visto::default();
    for (nombre, texto) in archivos {
        visto
            .archivos
            .insert(nombre.clone(), huella::de_texto(texto));
        visto.encargos.extend(
            encargos::encargos_para(texto, identidad)
                .iter()
                .map(|e| e.huella),
        );
        if nombre == ESTADO {
            visto.mutaciones = mutaciones::huella_de_las_vivas(texto);
        }
    }
    visto
}

/// Si el agente ya acusó el encargo en su bandeja `propia` (ver [`canal::acuses`]).
fn ya_acusado(encargo: &Encargo, propia: &str) -> bool {
    acuses::etiqueta(&encargo.primera_linea)
        .is_some_and(|etiqueta| acuses::acusado(&etiqueta, encargo.latido.as_deref(), propia))
}

/// Compara `archivos` (nombre y texto de cada archivo vigilado) con `antes`. `propia` es el texto
/// de la bandeja del agente: un encargo nuevo que ya acusó allí se da por visto sin avisar.
#[must_use]
pub fn comparar(
    antes: &Visto,
    archivos: &[(String, String)],
    identidad: &Identidad,
    propia: &str,
) -> Resultado {
    let mut visto = Visto {
        archivos: BTreeMap::new(),
        encargos: antes.encargos.clone(),
        mutaciones: antes.mutaciones,
    };
    let mut cambios = Vec::new();
    let mut ordenados: Vec<&(String, String)> = archivos.iter().collect();
    ordenados.sort_by(|a, b| a.0.cmp(&b.0));
    for (nombre, texto) in ordenados {
        let huella_actual = huella::de_texto(texto);
        visto.archivos.insert(nombre.clone(), huella_actual);
        if antes.archivos.get(nombre) == Some(&huella_actual) {
            continue;
        }
        let mut encargo = false;
        for nuevo in encargos::encargos_para(texto, identidad) {
            if visto.encargos.insert(nuevo.huella) && !ya_acusado(&nuevo, propia) {
                encargo = true;
            }
        }
        let mut mutacion = false;
        if nombre == ESTADO {
            let vivas = mutaciones::huella_de_las_vivas(texto);
            mutacion = vivas.is_some() && vivas != antes.mutaciones;
            visto.mutaciones = vivas;
        }
        cambios.push(Cambio {
            archivo: nombre.clone(),
            encargo,
            mutacion,
        });
    }
    Resultado { cambios, visto }
}
