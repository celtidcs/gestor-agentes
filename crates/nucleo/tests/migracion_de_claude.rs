//! Barreras de la migración única, desde la 3.9.0, del estado de la cuenta por defecto de Claude Code
//! al dejar de abrirla con CLAUDE_CONFIG_DIR. Las rutas salen de la biblioteca; el disco es un
//! doble o, con el adaptador real, una carpeta temporal.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod comun;

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use comun::CarpetaTemporal;
use nucleo::migracion_de_claude::{
    decidir, migrar, Decision, DiscoDeLaMigracion, DiscoReal, EstadoDelDisco, RutasDeLaMigracion,
    ARCHIVO_DE_COPIA, SUFIJO_DEL_ORIGEN_MIGRADO,
};

/// Estado del disco con el origen y, según se pida, el destino y la copia.
const fn con_origen(existe_el_destino: bool, existe_la_copia: bool) -> EstadoDelDisco {
    EstadoDelDisco {
        existe_el_origen: true,
        existe_el_migrado: false,
        existe_el_destino,
        existe_la_copia,
    }
}

#[test]
fn la_decision_solo_migra_con_origen_y_sin_migrar_y_no_repite_la_copia() {
    let sin_origen = EstadoDelDisco {
        existe_el_origen: false,
        ..con_origen(true, false)
    };
    let ya_migrado = EstadoDelDisco {
        existe_el_migrado: true,
        ..con_origen(true, false)
    };
    assert_eq!(decidir(sin_origen), Decision::Nada);
    assert_eq!(decidir(ya_migrado), Decision::Nada);
    assert_eq!(
        decidir(con_origen(true, false)),
        Decision::Migrar { copiar_antes: true }
    );
    assert_eq!(
        decidir(con_origen(false, false)),
        Decision::Migrar {
            copiar_antes: false
        }
    );
    assert_eq!(
        decidir(con_origen(true, true)),
        Decision::Migrar {
            copiar_antes: false
        }
    );
}

#[test]
fn las_rutas_son_las_de_la_biblioteca_y_las_de_datos() {
    let personal = Path::new("/home/u");
    let datos = Path::new("/gestor/datos");
    let rutas = RutasDeLaMigracion::new(personal, datos);

    assert_eq!(rutas.origen, personal.join(".claude").join(".claude.json"));
    assert_eq!(
        rutas.migrado,
        personal
            .join(".claude")
            .join(format!(".claude.json{SUFIJO_DEL_ORIGEN_MIGRADO}"))
    );
    assert_eq!(rutas.destino, personal.join(".claude.json"));
    assert_eq!(rutas.copia, datos.join(ARCHIVO_DE_COPIA));
}

/// Disco de mentira: los archivos que existen, lo que se hace y en qué ruta falla.
#[derive(Default)]
struct DiscoDeMentira {
    archivos: RefCell<BTreeSet<PathBuf>>,
    hecho: RefCell<Vec<String>>,
    falla_en: RefCell<Option<PathBuf>>,
}

impl DiscoDeMentira {
    fn con(archivos: &[&Path]) -> Self {
        Self {
            archivos: RefCell::new(archivos.iter().map(|a| a.to_path_buf()).collect()),
            ..Self::default()
        }
    }

    fn fallando_en(self, ruta: &Path) -> Self {
        *self.falla_en.borrow_mut() = Some(ruta.to_path_buf());
        self
    }

    fn falla(&self, ruta: &Path) -> io::Result<()> {
        if self.falla_en.borrow().as_deref() == Some(ruta) {
            return Err(io::Error::other("disco lleno"));
        }
        Ok(())
    }
}

impl DiscoDeLaMigracion for DiscoDeMentira {
    fn existe(&self, ruta: &Path) -> bool {
        self.archivos.borrow().contains(ruta)
    }
    fn copiar(&self, origen: &Path, destino: &Path) -> io::Result<()> {
        self.hecho
            .borrow_mut()
            .push(format!("copiar {} {}", origen.display(), destino.display()));
        self.falla(destino)?;
        self.archivos.borrow_mut().insert(destino.to_path_buf());
        Ok(())
    }
    fn renombrar(&self, origen: &Path, destino: &Path) -> io::Result<()> {
        self.hecho.borrow_mut().push(format!(
            "renombrar {} {}",
            origen.display(),
            destino.display()
        ));
        self.falla(origen)?;
        let mut archivos = self.archivos.borrow_mut();
        archivos.remove(origen);
        archivos.insert(destino.to_path_buf());
        Ok(())
    }
}

fn rutas() -> RutasDeLaMigracion {
    RutasDeLaMigracion::new(Path::new("/home/u"), Path::new("/datos"))
}

#[test]
fn migra_guardando_antes_la_copia_y_renombra_el_origen_al_final() {
    let r = rutas();
    let disco = DiscoDeMentira::con(&[&r.origen, &r.destino]);

    let hecha = migrar(r.clone(), &disco).unwrap().unwrap();

    assert!(hecha.con_copia);
    assert_eq!(
        *disco.hecho.borrow(),
        [
            format!("copiar {} {}", r.destino.display(), r.copia.display()),
            format!("copiar {} {}", r.origen.display(), r.destino.display()),
            format!("renombrar {} {}", r.origen.display(), r.migrado.display()),
        ]
    );
}

#[test]
fn sin_archivo_en_la_personal_migra_sin_copia() {
    let r = rutas();
    let disco = DiscoDeMentira::con(&[&r.origen]);

    let hecha = migrar(r, &disco).unwrap().unwrap();

    assert!(!hecha.con_copia);
    assert_eq!(disco.hecho.borrow().len(), 2);
}

#[test]
fn sin_origen_no_toca_nada() {
    let r = rutas();
    let disco = DiscoDeMentira::con(&[&r.destino]);

    assert!(migrar(r, &disco).unwrap().is_none());
    assert!(disco.hecho.borrow().is_empty());
}

#[test]
fn no_se_repite_una_vez_renombrado_el_origen() {
    let r = rutas();
    let disco = DiscoDeMentira::con(&[&r.origen, &r.destino]);

    assert!(migrar(r.clone(), &disco).unwrap().is_some());
    let hechas = disco.hecho.borrow().len();
    assert!(migrar(r, &disco).unwrap().is_none());
    assert_eq!(disco.hecho.borrow().len(), hechas);
}

/// La marca va con lo migrado. Otra instalación de Gestor, con otra
/// carpeta de datos y sin copia, no vuelve a migrar.
#[test]
fn otra_carpeta_de_datos_no_vuelve_a_migrar() {
    let r = rutas();
    let disco = DiscoDeMentira::con(&[&r.origen, &r.destino]);
    migrar(r, &disco).unwrap();

    let otra = RutasDeLaMigracion::new(Path::new("/home/u"), Path::new("/otra/datos"));
    let hechas = disco.hecho.borrow().len();
    assert!(migrar(otra, &disco).unwrap().is_none());
    assert_eq!(disco.hecho.borrow().len(), hechas);
}

#[test]
fn si_falla_la_copia_de_seguridad_no_toca_el_destino_ni_renombra() {
    let r = rutas();
    let disco = DiscoDeMentira::con(&[&r.origen, &r.destino]).fallando_en(&r.copia);

    let error = migrar(r.clone(), &disco).unwrap_err();

    assert_eq!(error.ruta, r.copia);
    assert_eq!(disco.hecho.borrow().len(), 1);
    assert!(disco.existe(&r.origen));
    assert!(!disco.existe(&r.migrado));
}

/// Si la migración se corta después de la copia de seguridad, el siguiente
/// intento no la sobrescribe con el destino, que ya es el estado migrado.
#[test]
fn un_reintento_no_sobrescribe_la_copia_de_seguridad() {
    let r = rutas();
    let disco = DiscoDeMentira::con(&[&r.origen, &r.destino]).fallando_en(&r.origen);
    assert_eq!(migrar(r.clone(), &disco).unwrap_err().ruta, r.origen);

    *disco.falla_en.borrow_mut() = None;
    disco.hecho.borrow_mut().clear();
    let hecha = migrar(r.clone(), &disco).unwrap().unwrap();

    assert!(hecha.con_copia);
    assert!(!disco
        .hecho
        .borrow()
        .iter()
        .any(|hecho| hecho.ends_with(&r.copia.display().to_string())));
}

/// Con el adaptador real, en carpetas temporales: el destino pasa a tener el estado del origen, la
/// copia guarda el anterior y el origen queda renombrado.
#[test]
fn en_disco_real_el_destino_queda_con_el_estado_de_la_cuenta() {
    let personal = CarpetaTemporal::new();
    let datos = CarpetaTemporal::new();
    personal.escribir(".claude/.claude.json", "nuevo");
    personal.escribir(".claude.json", "viejo");
    let r = RutasDeLaMigracion::new(personal.ruta(), datos.ruta());

    migrar(r, &DiscoReal).unwrap();

    assert_eq!(personal.leer(".claude.json"), "nuevo");
    assert!(!personal.existe(".claude/.claude.json"));
    assert_eq!(
        personal.leer(&format!(".claude/.claude.json{SUFIJO_DEL_ORIGEN_MIGRADO}")),
        "nuevo"
    );
    assert_eq!(datos.leer(ARCHIVO_DE_COPIA), "viejo");
}

/// Con el disco real, una copia de seguridad de un intento anterior se conserva.
#[test]
fn en_disco_real_una_copia_anterior_se_conserva() {
    let personal = CarpetaTemporal::new();
    let datos = CarpetaTemporal::new();
    personal.escribir(".claude/.claude.json", "nuevo");
    personal.escribir(".claude.json", "nuevo");
    datos.escribir(ARCHIVO_DE_COPIA, "original");
    let r = RutasDeLaMigracion::new(personal.ruta(), datos.ruta());

    let hecha = migrar(r, &DiscoReal).unwrap().unwrap();

    assert!(hecha.con_copia);
    assert_eq!(datos.leer(ARCHIVO_DE_COPIA), "original");
}
