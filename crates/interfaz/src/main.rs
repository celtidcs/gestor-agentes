//! Gestor Agentes: punto de entrada de la aplicación de escritorio. Compone las piezas de la
//! biblioteca y abre la ventana.

// En Windows es una aplicación de ventana: sin esto, cada arranque abriría también una consola.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use eframe::egui;
use gestor_agentes::actualizaciones::{self, Consulta};
use gestor_agentes::arranque;
use gestor_agentes::icono;
use gestor_agentes::sistema::{ConsultorGithub, MotorReal, ServiciosReales};
use gestor_agentes::tema;
use gestor_agentes::textos;
use gestor_agentes::trabajos::Trabajador;
use gestor_agentes::ventana::Ventana;
use lanzador::Sistema;
use nucleo::configuracion::Configuracion;

fn main() -> ExitCode {
    match abrir() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Sin consola en Windows, el error se enseña también en un diálogo del sistema.
            eprintln!("{}: {error}", textos::TITULO);
            rfd::MessageDialog::new()
                .set_title(textos::TITULO)
                .set_description(error.to_string())
                .set_level(rfd::MessageLevel::Error)
                .show();
            ExitCode::FAILURE
        }
    }
}

/// Abre la ventana y espera a que se cierre.
fn abrir() -> Result<(), Box<dyn std::error::Error>> {
    let configuracion = Configuracion::new(arranque::carpeta_de_datos()?);
    let ajustes = configuracion.cargar_ajustes()?;
    let consulta = Consulta::desde(&ajustes);
    let aplicacion = arranque::estado_inicial(
        &configuracion,
        Box::new(ServiciosReales::new(configuracion.clone())),
        Sistema::actual(),
        |nombre| std::env::var_os(nombre),
    );
    let viewport = egui::ViewportBuilder::default()
        .with_title(textos::TITULO)
        .with_inner_size(tema::TAMANO_VENTANA);
    // Un icono ilegible no impide abrir: la ventana sale sin él y el motivo queda en la consola.
    let viewport = match icono::decodificar(icono::ICONO_PNG) {
        Ok(imagen) => viewport.with_icon(imagen),
        Err(error) => {
            eprintln!("{}: {error}", textos::TITULO);
            viewport
        }
    };
    let opciones = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        textos::TITULO,
        opciones,
        Box::new(move |creacion| {
            tema::aplicar(&creacion.egui_ctx);
            let contexto = creacion.egui_ctx.clone();
            let trabajador = Trabajador::arrancar(
                Box::new(MotorReal::new(configuracion, ajustes)),
                Box::new(move || contexto.request_repaint()),
            )?;
            let contexto = creacion.egui_ctx.clone();
            let avisos = actualizaciones::en_segundo_plano(
                consulta,
                Box::new(ConsultorGithub),
                Box::new(move || contexto.request_repaint()),
            );
            Ok(Box::new(
                Ventana::new(aplicacion, trabajador, nucleo::plantillas::nucleo_comun())
                    .con_avisos_de_version(avisos),
            ))
        }),
    )?;
    Ok(())
}
