//! Textos que ve el usuario, compuestos en un solo sitio.
//!
//! Son los de la 2.3.2, con «Gestor
//! Claude» cambiado por «Gestor Agentes». Los textos nuevos de la 3.0.0 lo dicen en su comentario.

use std::path::Path;

use lanzador::ErrorDeLanzamiento;
use nucleo::configuracion::ErrorDeConfiguracion;

use crate::perfiles::Disponibilidad;

/// Título de la ventana y encabezado.
pub const TITULO: &str = "Gestor Agentes";

/// Versión que se ve en la esquina superior derecha de la ventana.
#[must_use]
pub fn version_visible(version: &str) -> String {
    format!("v{version}")
}

/// Minutos de una hora y horas a partir de las cuales la antigüedad de un latido se dice en días:
/// dos días en horas todavía se leen bien; más, no.
const MINUTOS_POR_HORA: i64 = 60;
const HORAS_ANTES_DE_DECIR_DIAS: i64 = 48;
const HORAS_POR_DIA: i64 = 24;

/// Antigüedad de un latido, como «hace 5 min», «hace 3 h» o «hace 4 días».
#[must_use]
pub fn antiguedad(minutos: i64) -> String {
    if minutos < 1 {
        return "ahora".to_owned();
    }
    if minutos < MINUTOS_POR_HORA {
        return format!("hace {minutos} min");
    }
    let horas = minutos / MINUTOS_POR_HORA;
    if horas < HORAS_ANTES_DE_DECIR_DIAS {
        return format!("hace {horas} h");
    }
    format!("hace {} días", horas / HORAS_POR_DIA)
}

/// Línea del estado del equipo del proyecto seleccionado: Jefe, último latido de cada agente,
/// reservas y alertas.
#[must_use]
pub fn linea_del_equipo(
    resumen: &nucleo::proyecto::revision_del_canal::ResumenDelEquipo,
    ahora: chrono::NaiveDateTime,
) -> String {
    let mut partes = vec![match &resumen.jefe {
        Some(jefe) => format!("Jefe: {jefe}"),
        None => "Jefe sin asignar".to_owned(),
    }];
    for latido in &resumen.latidos {
        let minutos = (ahora - latido.momento).num_minutes();
        partes.push(format!("{} {}", latido.identificador, antiguedad(minutos)));
    }
    partes.push(match resumen.reservas {
        0 => "sin reservas".to_owned(),
        1 => "1 reserva".to_owned(),
        n => format!("{n} reservas"),
    });
    if resumen.mutaciones_vivas {
        partes.push("MUTACIONES VIVAS".to_owned());
    }
    if resumen.autoexclusiones {
        partes.push("autoexclusiones vigentes".to_owned());
    }
    format!("Equipo · {}", partes.join(" · "))
}

/// Título del aviso de versión nueva.
pub const TITULO_ACTUALIZACION: &str = "Actualización disponible";

/// Texto del aviso de versión nueva: cuál hay, cuál tienes y dónde se descarga.
#[must_use]
pub fn actualizacion_disponible(nueva: &str, instalada: &str, enlace: &str) -> String {
    format!(
        "Hay una versión nueva de Gestor Agentes, la {nueva}. Tienes la {instalada}.\n\n\
         Puedes descargarla de su página en GitHub:\n{enlace}\n\n\
         Descarga el paquete de tu sistema y sustituye el programa conservando tu carpeta \
         datos, que guarda tu configuración."
    )
}

/// Pestaña de la lista de proyectos.
pub const PESTANA_PROYECTOS: &str = "Proyectos";

/// Pestaña del núcleo común.
pub const PESTANA_INSTRUCCIONES: &str = "Instrucciones de los agentes";

/// Pestaña de los agentes y las rutas de sus ejecutables. Nueva en la 3.0.0.
pub const PESTANA_AGENTES: &str = "Agentes";

/// Explicación de la pestaña de agentes. Nueva en la 3.0.0.
pub const EXPLICACION_AGENTES: &str = "Cada agente se abre en una terminal, en la carpeta del \
proyecto. Gestor Agentes busca cada programa en sus ubicaciones habituales y crea un botón por \
cada cuenta que encuentra (~/.claude, ~/.claude2, ~/.codex...). Solo se muestran los botones de \
los agentes que se encuentran. Si un programa está en otro sitio, escribe aquí su ruta o búscala \
con Examinar; deja la ruta vacía para que se busque solo.";

/// Cabeceras de la tabla de agentes. Nuevas en la 3.0.0.
pub const COLUMNAS_AGENTES: [&str; 3] = ["Agente", "Estado", "Ruta del ejecutable"];

/// Botón que abre el selector de archivos para la ruta de un ejecutable.
pub const BOTON_EXAMINAR: &str = "Examinar";

/// Botón que borra la ruta fijada para que el programa se busque solo.
pub const BOTON_BUSQUEDA_AUTOMATICA: &str = "Búsqueda automática";

/// Botón que guarda las rutas y vuelve a buscar los agentes.
pub const BOTON_GUARDAR_RUTAS: &str = "Guardar rutas";

/// Botón que vuelve a buscar los agentes sin cambiar nada.
pub const BOTON_BUSCAR_DE_NUEVO: &str = "Buscar de nuevo";

/// Título del selector de archivos del ejecutable.
pub const TITULO_SELECTOR_EJECUTABLE: &str = "Seleccionar el ejecutable del agente";

/// Barra de botones mientras se buscan los agentes al arrancar.
pub const BUSCANDO_AGENTES: &str = "Buscando agentes…";

/// Barra de botones cuando no se encuentra ningún agente.
pub const NINGUN_AGENTE: &str =
    "No se ha encontrado ningún agente. Indica la ruta de sus programas en la pestaña Agentes.";

/// Estado de un perfil en la pestaña de agentes.
#[must_use]
pub fn disponibilidad(disponibilidad: Option<&Disponibilidad>) -> String {
    match disponibilidad {
        None => BUSCANDO_AGENTES.to_owned(),
        Some(Disponibilidad::Disponible(ruta)) => format!("Encontrado: {}", ruta.display()),
        Some(Disponibilidad::SinEjecutable) => String::from("No encontrado"),
        Some(Disponibilidad::SinCuenta(carpeta)) => {
            format!("Falta la carpeta de su cuenta: {}", carpeta.display())
        }
    }
}

/// Título del aviso de rutas que no existen.
pub const TITULO_RUTAS: &str = "Rutas no válidas";

/// Aviso de rutas que no existen, una por línea con su agente. No se guarda nada.
#[must_use]
pub fn rutas_inexistentes(rutas: &[(String, String)]) -> String {
    let lineas: Vec<String> = rutas
        .iter()
        .map(|(agente, ruta)| format!("{agente}: {ruta}"))
        .collect();
    format!(
        "No existe ningún archivo en estas rutas, así que no se ha guardado nada:\n\n{}",
        lineas.join("\n")
    )
}

/// Barra de estado tras guardar las rutas.
pub const RUTAS_GUARDADAS: &str = "Rutas guardadas; buscando de nuevo los agentes.";

/// Barra de estado cuando la detección ha añadido cuentas nuevas.
#[must_use]
pub fn cuentas_anadidas(cuantas: usize) -> String {
    if cuantas == 1 {
        String::from("Se ha añadido 1 cuenta de agente encontrada en este equipo.")
    } else {
        format!("Se han añadido {cuantas} cuentas de agente encontradas en este equipo.")
    }
}

/// Barra de estado cuando no se pueden guardar los perfiles.
#[must_use]
pub fn perfiles_no_guardados(error: &ErrorDeConfiguracion) -> String {
    format!("No se pudieron guardar los agentes: {error}")
}

/// Explicación encima del núcleo común.
pub const EXPLICACION_NUCLEO: &str = "Núcleo común que Gestor Agentes escribe en AGENTS.md, el \
único archivo de reglas de todos los agentes; CLAUDE.md solo hace que Claude Code lo cargue. Va \
dentro del ejecutable y cambia con cada versión de la aplicación, así que aquí solo se consulta. \
Lo que cada proyecto aprende se escribe en .gestor/instrucciones-del-proyecto.md, que Gestor \
Agentes crea una vez y no vuelve a tocar. El bloque de entorno de cada proyecto se copia \
automáticamente del CLAUDE.md, AGENTS.md o GEMINI.md de la carpeta superior más cercana (las \
normas comunes de la carpeta proyecto).";

/// Cabeceras de la tabla de proyectos.
pub const COLUMNAS: [&str; 4] = ["Nombre", "Ruta", "Última apertura", "Estado"];

/// Botón de cada fila que quita el proyecto de la lista.
pub const BOTON_QUITAR: &str = "Quitar";

/// Botón que añade un proyecto.
pub const BOTON_AGREGAR: &str = "Agregar Proyecto";

/// Botón que borra la carpeta del proyecto.
pub const BOTON_ELIMINAR: &str = "Eliminar del disco";

/// Botón del diálogo informativo.
pub const BOTON_ACEPTAR: &str = "Aceptar";

/// Botón afirmativo del diálogo de confirmación.
pub const BOTON_SI: &str = "Sí";

/// Botón negativo del diálogo de confirmación.
pub const BOTON_NO: &str = "No";

/// Título del selector de carpetas.
pub const TITULO_SELECTOR: &str = "Seleccionar carpeta del proyecto";

/// Título de los diálogos de error.
pub const TITULO_ERROR: &str = "Error";

/// Título del aviso de proyecto repetido.
pub const TITULO_DUPLICADO: &str = "Proyecto duplicado";

/// Aviso de proyecto repetido.
pub const PROYECTO_DUPLICADO: &str = "Este proyecto ya está en la lista.";

/// Título de la confirmación de borrado.
pub const TITULO_BORRADO: &str = "Eliminar proyecto del disco";

/// Texto del botón de un perfil: «Abrir · Claude 2», como en la 2.3.2.
#[must_use]
pub fn etiqueta_de_perfil(identidad: &str) -> String {
    format!("Abrir · {identidad}")
}

/// Aviso de la barra de estado cuando el archivo de instrucciones pasa de lo que lee la
/// herramienta. Nuevo en la 3.0.0: Gemini y Codex reciben esas instrucciones cortadas.
#[must_use]
pub fn instrucciones_cortadas(
    archivo: &str,
    tamano: u64,
    herramienta: &str,
    limite: u64,
) -> String {
    format!(
        "AVISO: {archivo} mide {} bytes y {herramienta} solo lee los primeros {}",
        con_puntos(tamano),
        con_puntos(limite)
    )
}

/// Barra de estado con un aviso añadido detrás.
#[must_use]
pub fn con_aviso(estado: &str, aviso: &str) -> String {
    format!("{estado}. {aviso}")
}

/// Número con punto de millares, como se escribe en español: 40.434.
fn con_puntos(numero: u64) -> String {
    let cifras = numero.to_string();
    let mut resultado = String::new();
    for (posicion, cifra) in cifras.chars().enumerate() {
        if posicion > 0 && (cifras.len() - posicion).is_multiple_of(3) {
            resultado.push('.');
        }
        resultado.push(cifra);
    }
    resultado
}

/// Barra de estado tras cargar la lista.
#[must_use]
pub fn proyectos_registrados(cuantos: usize) -> String {
    format!("{cuantos} proyecto(s) registrado(s)")
}

/// Barra de estado tras añadir un proyecto.
#[must_use]
pub fn proyecto_agregado(nombre: &str) -> String {
    format!("Proyecto agregado: {nombre}")
}

/// Barra de estado tras quitar un proyecto de la lista.
#[must_use]
pub fn proyecto_quitado(nombre: &str) -> String {
    format!("Proyecto eliminado de la lista: {nombre}")
}

/// Barra de estado tras borrar la carpeta de un proyecto.
#[must_use]
pub fn proyecto_borrado(nombre: &str) -> String {
    format!("Proyecto eliminado del disco: {nombre}")
}

/// Barra de estado mientras se prepara un proyecto.
#[must_use]
pub fn preparando(proyecto: &str, identidad: &str) -> String {
    format!("Preparando {proyecto} para {identidad}…")
}

/// Barra de estado con el agente ya abierto.
#[must_use]
pub fn abierto(identidad: &str, proyecto: &str, cambios: usize) -> String {
    if cambios == 0 {
        format!("{identidad} abierto en {proyecto}")
    } else {
        format!("{identidad} abierto en {proyecto}; {cambios} cambio(s) en sus archivos de gestión")
    }
}

/// Barra de estado cuando el proyecto se ha preparado pero no se ha abierto ninguna ventana. Es
/// nuevo en la 3.0.0: la 2.3.2 decía «abierto» también en ese caso.
#[must_use]
pub fn preparado_sin_abrir(proyecto: &str, identidad: &str) -> String {
    format!("{proyecto} preparado para {identidad}, pero no se ha abierto ninguna ventana")
}

/// Mensaje de carpeta inexistente al abrir un proyecto.
#[must_use]
pub fn carpeta_inexistente(ruta: &str) -> String {
    format!("La carpeta no existe:\n{ruta}")
}

/// Mensaje de carpeta inexistente al borrar un proyecto.
#[must_use]
pub fn carpeta_inexistente_en_disco(ruta: &str) -> String {
    format!("La carpeta no existe en disco:\n{ruta}")
}

/// Pregunta de confirmación del borrado. Va sin el signo «⚠️» de la 2.3.2: las fuentes que
/// trae egui no lo tienen y lo pintaban como un cuadro.
#[must_use]
pub fn confirmar_borrado(nombre: &str, ruta: &str) -> String {
    format!(
        "ATENCIÓN: Esto borrará permanentemente la carpeta del proyecto y todos sus archivos.\n\n\
         Proyecto: {nombre}\nRuta: {ruta}\n\n\
         ¿Estás seguro? Esta acción no se puede deshacer."
    )
}

/// Mensaje cuando el borrado falla.
#[must_use]
pub fn borrado_fallido(detalle: &str) -> String {
    format!("No se pudo eliminar la carpeta:\n{detalle}")
}

/// Título del diálogo con el informe de preparación.
#[must_use]
pub fn titulo_preparacion(proyecto: &str) -> String {
    format!("Preparación de {proyecto}")
}

/// Título y texto del diálogo cuando el agente no se ha podido abrir.
///
/// Si no se encuentra la herramienta se usa el texto de la 2.3.2. Si el perfil está incompleto se
/// dice en qué archivo se edita (nuevo en la 3.0.0, por Programator).
#[must_use]
pub fn fallo_de_lanzamiento(
    error: &ErrorDeLanzamiento,
    proyecto: &str,
    ruta_perfiles: &Path,
) -> (String, String) {
    match error {
        ErrorDeLanzamiento::NoEncontrada {
            herramienta,
            detalle,
        } => (
            format!("{herramienta} no está instalado"),
            format!(
                "No se ha encontrado {herramienta} en este equipo, así que no se ha abierto \
                 ({detalle}).\n\nLos archivos de «{proyecto}» sí se han preparado."
            ),
        ),
        ErrorDeLanzamiento::PerfilIncompleto { .. } => (
            String::from("Perfil incompleto"),
            format!(
                "{}.\n\nEl perfil se edita en {}.\n\nLos archivos de «{proyecto}» sí se han \
                 preparado.",
                con_mayuscula(&error.to_string()),
                ruta_perfiles.display()
            ),
        ),
        _ => (
            String::from("No se ha podido abrir"),
            format!("No se ha podido abrir: {error}.\n\nLos archivos de «{proyecto}» sí se han preparado."),
        ),
    }
}

/// El texto con su primera letra en mayúscula. Los errores de los crates empiezan en minúscula,
/// como pide la convención de Rust, y aquí abren una frase.
fn con_mayuscula(texto: &str) -> String {
    let mut letras = texto.chars();
    letras.next().map_or_else(String::new, |primera| {
        primera.to_uppercase().chain(letras).collect()
    })
}

/// Barra de estado cuando falla la configuración.
#[must_use]
pub fn error_de_configuracion(error: &ErrorDeConfiguracion) -> String {
    format!("Configuración no válida: {error}")
}

/// Barra de estado cuando no se puede guardar la lista. Nuevo en la 3.0.0: la 2.3.2 no lo
/// trataba.
#[must_use]
pub fn guardado_fallido(error: &ErrorDeConfiguracion) -> String {
    format!("No se pudo guardar la lista de proyectos: {error}")
}

/// Mensaje cuando la carpeta elegida no tiene una ruta Unicode válida (solo puede pasar en
/// Linux). Nuevo en la 3.0.0: la lista de proyectos guarda las rutas como texto.
#[must_use]
pub fn ruta_no_unicode(ruta: &Path) -> String {
    format!(
        "La ruta {} no es Unicode válido y no se puede guardar en la lista.",
        ruta.display()
    )
}

/// Título del aviso de la configuración importada de la 2.x.
pub const TITULO_MIGRACION: &str = "Configuración importada";

/// Aviso de la configuración importada de la 2.x en el primer arranque.
#[must_use]
pub fn migracion(origen: &Path, destino: &Path, archivos: &[&str]) -> String {
    format!(
        "Se ha copiado la configuración de Gestor Claude 2.x a la carpeta de datos de Gestor \
         Agentes.\n\nDesde: {}\nHasta: {}\nArchivos: {}\n\nEl original no se ha tocado.",
        origen.display(),
        destino.display(),
        archivos.join(", ")
    )
}

/// Barra de estado cuando la migración falla.
#[must_use]
pub fn migracion_fallida(error: &ErrorDeConfiguracion) -> String {
    format!("No se pudo copiar la configuración de la 2.x: {error}")
}

/// Mensaje cuando el hilo de trabajo ha dejado de responder.
pub const TRABAJADOR_PERDIDO: &str =
    "El hilo de trabajo se ha detenido; reinicia la aplicación para seguir.";
