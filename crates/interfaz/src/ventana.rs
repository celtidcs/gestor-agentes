//! Ventana principal. Solo pinta el estado de [`Aplicacion`] y le pasa cada clic; las reglas
//! viven allí y los trabajos largos en el [`Trabajador`].

use eframe::egui::{self, Color32, RichText};

use crate::aplicacion::Aplicacion;
use crate::tema;
use crate::textos;
use crate::trabajos::{Trabajador, Trabajo};

/// Pestaña visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pestana {
    Proyectos,
    Agentes,
    Instrucciones,
}

/// Lo que el usuario ha pedido en este fotograma. Se recoge mientras se pinta y se aplica al
/// final, para no cambiar el estado a mitad del dibujo.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Orden {
    Seleccionar(String),
    Quitar(String),
    Agregar,
    AbrirCon(usize),
    Borrar,
    CerrarDialogo(bool),
    EditarRuta(usize, String),
    Examinar(usize),
    BusquedaAutomatica(usize),
    GuardarRutas,
    BuscarDeNuevo,
}

/// Ventana de Gestor Agentes.
pub struct Ventana {
    aplicacion: Aplicacion,
    trabajador: Trabajador,
    nucleo_comun: String,
    pestana: Pestana,
}

impl Ventana {
    /// Ventana sobre ese estado y ese hilo de trabajo. Encarga en seguida la detección de los
    /// agentes del equipo, que decide qué botones se muestran.
    #[must_use]
    pub fn new(aplicacion: Aplicacion, trabajador: Trabajador, nucleo_comun: String) -> Self {
        let mut ventana = Self {
            aplicacion,
            trabajador,
            nucleo_comun,
            pestana: Pestana::Proyectos,
        };
        if let Some(trabajo) = ventana.aplicacion.detectar() {
            ventana.encargar(trabajo);
        }
        ventana
    }

    fn aplicar(&mut self, orden: Orden) {
        let trabajo = match orden {
            Orden::Seleccionar(ruta) => {
                self.aplicacion.seleccionar(&ruta);
                None
            }
            Orden::Quitar(ruta) => {
                self.aplicacion.quitar(&ruta);
                None
            }
            Orden::Agregar => self.aplicacion.pedir_carpeta(),
            Orden::AbrirCon(indice) => self.aplicacion.abrir_con(indice),
            Orden::Borrar => {
                self.aplicacion.pedir_borrado();
                None
            }
            Orden::CerrarDialogo(aceptado) => self.aplicacion.cerrar_dialogo(aceptado),
            Orden::EditarRuta(indice, ruta) => {
                self.aplicacion.editar_ruta(indice, ruta);
                None
            }
            Orden::Examinar(indice) => self.aplicacion.pedir_ejecutable(indice),
            Orden::BusquedaAutomatica(indice) => {
                self.aplicacion.busqueda_automatica(indice);
                None
            }
            Orden::GuardarRutas => self.aplicacion.guardar_rutas(),
            Orden::BuscarDeNuevo => self.aplicacion.detectar(),
        };
        if let Some(trabajo) = trabajo {
            self.encargar(trabajo);
        }
    }

    fn encargar(&mut self, trabajo: Trabajo) {
        if self.trabajador.encargar(trabajo).is_err() {
            self.aplicacion.trabajador_perdido();
        }
    }

    fn encabezado(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(textos::TITULO)
                .size(tema::TAMANO_ENCABEZADO)
                .strong(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (pestana, texto) in [
                (Pestana::Proyectos, textos::PESTANA_PROYECTOS),
                (Pestana::Agentes, textos::PESTANA_AGENTES),
                (Pestana::Instrucciones, textos::PESTANA_INSTRUCCIONES),
            ] {
                let elegida = self.pestana == pestana;
                let color = if elegida {
                    tema::TEXTO
                } else {
                    tema::TEXTO_SECUNDARIO
                };
                let rotulo = RichText::new(texto)
                    .size(tema::TAMANO_TEXTO)
                    .strong()
                    .color(color);
                if ui.selectable_label(elegida, rotulo).clicked() {
                    self.pestana = pestana;
                }
            }
        });
        ui.add_space(8.0);
    }

    fn proyectos(&self, ui: &mut egui::Ui, ordenes: &mut Vec<Orden>) {
        let edicion = self.aplicacion.edicion_activa();
        let acciones = self.aplicacion.acciones_activas();

        egui::Panel::bottom("acciones").show(ui, |ui| {
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                if boton(ui, textos::BOTON_AGREGAR, tema::MORADO, edicion) {
                    ordenes.push(Orden::Agregar);
                }
                let visibles = self.aplicacion.perfiles_visibles();
                for &indice in &visibles {
                    let Some(perfil) = self.aplicacion.perfiles().get(indice) else {
                        continue;
                    };
                    let etiqueta = crate::perfiles::etiqueta(perfil);
                    if boton(ui, &etiqueta, tema::color_de_perfil(indice), acciones) {
                        ordenes.push(Orden::AbrirCon(indice));
                    }
                }
                if visibles.is_empty() {
                    let aviso = if self.aplicacion.deteccion_terminada() {
                        textos::NINGUN_AGENTE
                    } else {
                        textos::BUSCANDO_AGENTES
                    };
                    ui.label(
                        RichText::new(aviso)
                            .size(tema::TAMANO_PEQUENO)
                            .color(tema::TEXTO_SECUNDARIO),
                    );
                }
                if boton(ui, textos::BOTON_ELIMINAR, tema::ROJO, acciones) {
                    ordenes.push(Orden::Borrar);
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::Frame::new().fill(tema::SUPERFICIE).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    egui::Grid::new("proyectos")
                        .num_columns(textos::COLUMNAS.len() + 1)
                        .striped(true)
                        .spacing([16.0, 6.0])
                        .show(ui, |ui| self.tabla(ui, edicion, ordenes));
                });
            });
        });
    }

    fn tabla(&self, ui: &mut egui::Ui, edicion: bool, ordenes: &mut Vec<Orden>) {
        for cabecera in textos::COLUMNAS {
            ui.label(
                RichText::new(cabecera)
                    .strong()
                    .color(tema::TEXTO_SECUNDARIO),
            );
        }
        ui.label("");
        ui.end_row();

        for fila in self.aplicacion.filas() {
            let ruta = fila.registro.ruta();
            let elegida = self.aplicacion.seleccion() == Some(ruta);
            let nombre = RichText::new(fila.registro.nombre()).size(tema::TAMANO_TEXTO);
            if ui.selectable_label(elegida, nombre).clicked() {
                ordenes.push(Orden::Seleccionar(ruta.to_owned()));
            }
            ui.label(ruta);
            ui.label(fila.fecha_visible.as_str());
            ui.label(fila.estado.texto());
            let quitar = egui::Button::new(
                RichText::new(textos::BOTON_QUITAR)
                    .size(tema::TAMANO_PEQUENO)
                    .strong()
                    .color(tema::FONDO),
            )
            .fill(tema::ROJO);
            if ui.add_enabled(edicion, quitar).clicked() {
                ordenes.push(Orden::Quitar(ruta.to_owned()));
            }
            ui.end_row();
        }
    }

    fn agentes(&self, ui: &mut egui::Ui, ordenes: &mut Vec<Orden>) {
        let edicion = self.aplicacion.edicion_activa();
        ui.label(
            RichText::new(textos::EXPLICACION_AGENTES)
                .size(tema::TAMANO_PEQUENO)
                .color(tema::TEXTO_SECUNDARIO),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let guardar = edicion && self.aplicacion.rutas_cambiadas();
            if boton(ui, textos::BOTON_GUARDAR_RUTAS, tema::MORADO, guardar) {
                ordenes.push(Orden::GuardarRutas);
            }
            if boton(ui, textos::BOTON_BUSCAR_DE_NUEVO, tema::VERDE, edicion) {
                ordenes.push(Orden::BuscarDeNuevo);
            }
        });
        ui.add_space(8.0);
        egui::Frame::new()
            .fill(tema::SUPERFICIE)
            .inner_margin(tema::MARGEN_INTERIOR)
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    egui::Grid::new("agentes")
                        .num_columns(textos::COLUMNAS_AGENTES.len() + 2)
                        .striped(true)
                        .spacing([16.0, 6.0])
                        .show(ui, |ui| self.tabla_de_agentes(ui, edicion, ordenes));
                });
            });
    }

    fn tabla_de_agentes(&self, ui: &mut egui::Ui, edicion: bool, ordenes: &mut Vec<Orden>) {
        for cabecera in textos::COLUMNAS_AGENTES {
            ui.label(
                RichText::new(cabecera)
                    .strong()
                    .color(tema::TEXTO_SECUNDARIO),
            );
        }
        ui.label("");
        ui.label("");
        ui.end_row();

        for (indice, perfil) in self.aplicacion.perfiles().iter().enumerate() {
            ui.label(RichText::new(perfil.nombre()).size(tema::TAMANO_TEXTO));
            ui.label(textos::disponibilidad(
                self.aplicacion.disponibilidad(indice),
            ));
            let mut ruta = self.aplicacion.ruta_editada(indice).to_owned();
            let campo = egui::TextEdit::singleline(&mut ruta).desired_width(tema::ANCHO_RUTA);
            if ui.add_enabled(edicion, campo).changed() {
                ordenes.push(Orden::EditarRuta(indice, ruta));
            }
            if ui
                .add_enabled(edicion, egui::Button::new(textos::BOTON_EXAMINAR))
                .clicked()
            {
                ordenes.push(Orden::Examinar(indice));
            }
            if ui
                .add_enabled(
                    edicion,
                    egui::Button::new(textos::BOTON_BUSQUEDA_AUTOMATICA),
                )
                .clicked()
            {
                ordenes.push(Orden::BusquedaAutomatica(indice));
            }
            ui.end_row();
        }
    }

    fn instrucciones(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(textos::EXPLICACION_NUCLEO)
                .size(tema::TAMANO_PEQUENO)
                .color(tema::TEXTO_SECUNDARIO),
        );
        ui.add_space(6.0);
        egui::Frame::new()
            .fill(tema::SUPERFICIE)
            .inner_margin(tema::MARGEN_INTERIOR)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        // Solo lectura: se pinta como texto seleccionable, no como campo editable.
                        ui.add(
                            egui::Label::new(
                                RichText::new(&self.nucleo_comun)
                                    .monospace()
                                    .size(tema::TAMANO_PEQUENO),
                            )
                            .selectable(true)
                            .wrap(),
                        );
                    });
            });
    }

    fn dialogo(&self, contexto: &egui::Context, ordenes: &mut Vec<Orden>) {
        let Some(dialogo) = self.aplicacion.dialogo() else {
            return;
        };
        let respuesta = egui::Modal::new(egui::Id::unique("dialogo")).show(contexto, |ui| {
            ui.set_width(tema::ANCHO_DIALOGO);
            ui.label(
                RichText::new(dialogo.titulo())
                    .strong()
                    .size(tema::TAMANO_TEXTO + 2.0),
            );
            ui.add_space(12.0);
            egui::ScrollArea::vertical()
                .max_height(tema::ALTO_MAXIMO_DIALOGO)
                .show(ui, |ui| {
                    ui.add(egui::Label::new(dialogo.texto()).selectable(true).wrap());
                });
            ui.add_space(16.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if dialogo.pide_confirmacion() {
                    if ui.button(textos::BOTON_NO).clicked() {
                        ordenes.push(Orden::CerrarDialogo(false));
                    }
                    if ui.button(textos::BOTON_SI).clicked() {
                        ordenes.push(Orden::CerrarDialogo(true));
                    }
                } else if ui.button(textos::BOTON_ACEPTAR).clicked() {
                    ordenes.push(Orden::CerrarDialogo(true));
                }
            });
        });
        // Escape o un clic fuera cierran el diálogo como «No» o «Aceptar»: nunca confirman un
        // borrado.
        if respuesta.should_close() && ordenes.is_empty() {
            ordenes.push(Orden::CerrarDialogo(!dialogo.pide_confirmacion()));
        }
    }
}

/// Botón de color con texto oscuro, como los de la 2.3.2. Devuelve si se ha pulsado.
fn boton(ui: &mut egui::Ui, texto: &str, color: Color32, activo: bool) -> bool {
    // Desactivado, como en la 2.3.2: fondo #45475a y texto #6c7086.
    let (fondo, letra) = if activo {
        (color, tema::FONDO)
    } else {
        (tema::SUPERFICIE_ALTA, tema::TEXTO_APAGADO)
    };
    let rotulo = RichText::new(texto)
        .size(tema::TAMANO_TEXTO)
        .strong()
        .color(letra);
    let boton = egui::Button::new(rotulo).fill(fondo).corner_radius(6);
    ui.add_enabled(activo, boton).clicked()
}

impl eframe::App for Ventana {
    fn logic(&mut self, _contexto: &egui::Context, _marco: &mut eframe::Frame) {
        loop {
            match self.trabajador.recoger() {
                Ok(Some(resultado)) => self.aplicacion.terminar(resultado),
                Ok(None) => break,
                Err(_) => {
                    if self.aplicacion.ocupada() {
                        self.aplicacion.trabajador_perdido();
                    }
                    break;
                }
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _marco: &mut eframe::Frame) {
        let mut ordenes = Vec::new();
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(tema::FONDO)
                    .inner_margin(tema::MARGEN),
            )
            .show(ui, |ui| {
                egui::Panel::top("encabezado").show(ui, |ui| self.encabezado(ui));
                egui::Panel::bottom("estado").show(ui, |ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(self.aplicacion.texto_de_estado())
                            .size(tema::TAMANO_PEQUENO)
                            .color(tema::TEXTO_APAGADO),
                    );
                });
                egui::CentralPanel::default().show(ui, |ui| match self.pestana {
                    Pestana::Proyectos => self.proyectos(ui, &mut ordenes),
                    Pestana::Agentes => self.agentes(ui, &mut ordenes),
                    Pestana::Instrucciones => self.instrucciones(ui),
                });
            });
        self.dialogo(ui.ctx(), &mut ordenes);
        for orden in ordenes {
            self.aplicar(orden);
        }
    }
}
