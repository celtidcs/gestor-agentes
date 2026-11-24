//! Textos que ve el usuario, compuestos en un solo sitio.
//!
//! Son los de la 2.3.2, con «Gestor
//! Claude» cambiado por «Gestor Agentes». Los textos nuevos de la 3.0.0 lo dicen en su comentario.

use std::path::Path;

use lanzador::ErrorDeLanzamiento;
use nucleo::actualizaciones::Version;
use nucleo::configuracion::ErrorDeConfiguracion;
use nucleo::cuentas::AltaRechazada;
use nucleo::migracion_de_claude::{ErrorDeMigracion, MigracionHecha};

use crate::perfiles::{CarpetaDelAgente, Disponibilidad, EstadoDelAgente};

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
        let vigilancia = resumen
            .vigilancia
            .iter()
            .find(|v| v.identificador == latido.identificador)
            .map(nota_de_vigilancia)
            .unwrap_or_default();
        partes.push(format!(
            "{} {}{vigilancia}",
            latido.identificador,
            antiguedad(minutos)
        ));
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

/// Lo que se añade tras el latido de un agente: si su vigilante vigila o ha caído y cuántos
/// encargos tiene sin acuse.
fn nota_de_vigilancia(vigilancia: &nucleo::proyecto::vigilancia::VigilanciaDeAgente) -> String {
    use nucleo::proyecto::vigilancia::EstadoDelVigilante;
    let mut notas = Vec::new();
    match vigilancia.vigilante {
        Some(EstadoDelVigilante::Vigilando) => notas.push("vigila".to_owned()),
        Some(EstadoDelVigilante::Caido) => notas.push("VIGILANTE CAÍDO".to_owned()),
        None => {}
    }
    match vigilancia.encargos_sin_acuse {
        0 => {}
        1 => notas.push("1 encargo sin RECIBIDO".to_owned()),
        n => notas.push(format!("{n} encargos sin RECIBIDO")),
    }
    if notas.is_empty() {
        String::new()
    } else {
        format!(" ({})", notas.join(", "))
    }
}

/// Etiqueta de la elección del papel del próximo agente que se abra.
pub const ETIQUETA_PAPEL: &str = "Papel del agente que abras:";

/// Opción de no elegir papel: el agente lo pregunta al arrancar.
pub const PAPEL_SIN_ELEGIR: &str = "que lo pregunte";

/// Opción de Jefe de proyecto.
pub const PAPEL_JEFE: &str = "Jefe de proyecto";

/// Opción de ayudante.
pub const PAPEL_AYUDANTE: &str = "ayudante";

/// Detalle del cambio en estado.md al anotar el papel elegido.
#[must_use]
pub fn papel_anotado(identidad: &str, papel: nucleo::proyecto::papeles::Papel) -> String {
    format!("{identidad} anotado como {}", papel.nombre())
}

/// Título del aviso de versión nueva.
pub const TITULO_ACTUALIZACION: &str = "Actualización disponible";

/// Comienzo de la línea que deja en el registro un fallo al comprobar si hay una versión nueva.
pub const REGISTRO_DEL_AVISO_DE_VERSION: &str = "Gestor Agentes, comprobación de versión nueva";

/// Nombre de Programator en los textos.
pub const PROGRAMATOR: &str = "Programator";

/// Qué comprobación de versión falló, para el registro: «de Programator (celtidcs/Programator)».
#[must_use]
pub fn que_se_comprueba(programa: &str, repositorio: &str) -> String {
    format!("de {programa} ({repositorio})")
}

/// Fallo al identificar a Gestor Agentes ante GitHub; la consulta sale entonces sin su nombre.
pub const IDENTIFICACION_NO_VALIDA: &str =
    "de Gestor Agentes: no se puede identificar ante GitHub y la consulta sale sin su nombre";

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

/// Botón de la pestaña Agentes que da de alta a mano una cuenta de Claude Code.
pub const BOTON_CUENTA_CLAUDE: &str = "Añadir cuenta de Claude…";

/// Botón de la pestaña Agentes que da de alta a mano una cuenta de Codex.
pub const BOTON_CUENTA_CODEX: &str = "Añadir cuenta de Codex…";

/// Título del selector de la carpeta de una cuenta.
pub const TITULO_SELECTOR_CUENTA: &str =
    "Seleccionar la carpeta de la cuenta (la de CLAUDE_CONFIG_DIR o CODEX_HOME)";

/// Título del aviso cuando una cuenta no se puede dar de alta.
pub const TITULO_CUENTA: &str = "Cuenta no añadida";

/// Por qué no se ha dado de alta la cuenta.
#[must_use]
pub fn cuenta_rechazada(rechazo: &AltaRechazada) -> String {
    match rechazo {
        AltaRechazada::SinCuentas => {
            String::from("Esa herramienta no tiene cuentas separadas por carpeta.")
        }
        AltaRechazada::CarpetaEnUso(perfil) => {
            format!("Esa carpeta ya es la de {perfil}. Cada cuenta necesita su propia carpeta.")
        }
        AltaRechazada::SinCarpetaPersonal => String::from(
            "No se sabe cuál es tu carpeta de usuario, así que no se puede comprobar si esa \
             carpeta ya es la de otra cuenta.",
        ),
        AltaRechazada::RutaNoValida => String::from(
            "La ruta de esa carpeta tiene caracteres que la lista de agentes no puede guardar. \
             Cámbiale el nombre o elige otra.",
        ),
    }
}

/// Detalle del informe de apertura para la migración única del estado de la cuenta por defecto de
/// Claude Code (3.9.0), que abre el agente `identidad`: con qué se ha sustituido el archivo, dónde
/// queda la copia del anterior y cómo queda marcada.
#[must_use]
pub fn estado_de_claude_migrado(hecha: &MigracionHecha, identidad: &str) -> String {
    let copia = if hecha.con_copia {
        format!(
            " La versión anterior está guardada en {}.",
            hecha.rutas.copia.display()
        )
    } else {
        String::new()
    };
    format!(
        "{identidad} se abre ahora sin CLAUDE_CONFIG_DIR, como la cuenta por defecto de Claude \
         Code; para que conserve sus proyectos y ajustes, se ha puesto en su lugar {}.{copia} \
         Ese archivo se ha renombrado a {} para que no se vuelva a copiar",
        hecha.rutas.origen.display(),
        hecha.rutas.migrado.display()
    )
}

/// Error de la migración del estado de la cuenta por defecto de Claude Code: el agente `identidad`
/// se abre igual, y se volverá a intentar la próxima vez.
#[must_use]
pub fn estado_de_claude_sin_migrar(error: &ErrorDeMigracion, identidad: &str) -> String {
    format!(
        "No se ha podido pasar el estado de {identidad} a la cuenta por defecto de Claude Code \
         ({error}). Se volverá a intentar la próxima vez que se abra {identidad}."
    )
}

/// Título del selector de archivos del ejecutable.
pub const TITULO_SELECTOR_EJECUTABLE: &str = "Seleccionar el ejecutable del agente";

/// Barra de botones mientras se buscan los agentes al arrancar.
pub const BUSCANDO_AGENTES: &str = "Buscando agentes…";

/// Barra de botones cuando no se encuentra ningún agente.
pub const NINGUN_AGENTE: &str =
    "No se ha encontrado ningún agente. Indica la ruta de sus programas en la pestaña Agentes.";

/// Separador entre las partes del estado de un agente.
const SEPARADOR_DE_ESTADO: &str = " · ";

/// Palabra del estado de un agente, que acompaña al color de su celda para que no dependa solo
/// del color.
#[must_use]
pub const fn palabra_de_estado(estado: EstadoDelAgente) -> &'static str {
    match estado {
        EstadoDelAgente::Localizado => "Localizado",
        EstadoDelAgente::FaltaLaCuenta => "Falta la cuenta",
        EstadoDelAgente::NoEncontrado => "No encontrado",
        EstadoDelAgente::SinComprobar => "Sin comprobar",
    }
}

/// Columna Estado de un perfil en la pestaña de agentes: la palabra del estado, la carpeta donde
/// vive el agente y si existe, y el programa encontrado. Sin comprobar solo lleva la palabra,
/// porque lo demás puede estar desfasado.
#[must_use]
pub fn estado_de_agente(
    estado: EstadoDelAgente,
    disponibilidad: Option<&Disponibilidad>,
    carpeta: Option<&CarpetaDelAgente>,
) -> String {
    let mut partes = vec![palabra_de_estado(estado).to_owned()];
    if estado == EstadoDelAgente::SinComprobar {
        return partes.concat();
    }
    if let Some(carpeta) = carpeta {
        partes.push(if carpeta.existe() {
            format!("carpeta {}", carpeta.ruta().display())
        } else {
            format!("carpeta {} (no existe)", carpeta.ruta().display())
        });
    }
    if let Some(Disponibilidad::Disponible(programa)) = disponibilidad {
        partes.push(format!("programa {}", programa.display()));
    }
    partes.join(SEPARADOR_DE_ESTADO)
}

/// Aviso de que la ruta de Programator apunta a una versión anterior a la publicada.
#[must_use]
pub fn programator_anticuado(nueva: Version) -> String {
    format!(
        "la ruta de Programator apunta a una versión antigua; hay la {nueva}: descárgala y cambia la ruta en la pestaña Agentes"
    )
}

/// Estado de un agente con la versión de su programa, si se conoce, y la nueva si la hay.
#[must_use]
pub fn con_version(estado: String, instalada: Option<Version>, nueva: Option<Version>) -> String {
    match (instalada, nueva) {
        (Some(instalada), Some(nueva)) => format!(
            "{estado}{SEPARADOR_DE_ESTADO}versión {instalada} (hay la {nueva}: cambia la ruta)"
        ),
        (Some(instalada), None) => format!("{estado}{SEPARADOR_DE_ESTADO}versión {instalada}"),
        (None, _) => estado,
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

/// Barra de estado cuando no se guarda la lista de agentes porque perfiles-agentes.json no se pudo
/// leer al arrancar: guardarla borraría lo que el usuario tiene en ese archivo.
pub const PERFILES_ILEGIBLES_SIN_GUARDAR: &str = "No se guarda la lista de agentes porque \
perfiles-agentes.json no se pudo leer al arrancar. Corrígelo con la aplicación cerrada y vuelve a \
abrirla.";

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

/// Aviso de la barra de estado cuando la preparación deja avisos para los agentes, que no abren
/// ventana: cuántos son y dónde los leen.
#[must_use]
pub fn avisos_para_los_agentes(avisos: usize) -> String {
    format!(
        "{avisos} aviso(s) para los agentes en {}",
        nucleo::proyecto::avisos_de_preparacion::RUTA_RELATIVA
    )
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
