//! Textos del núcleo que ve el usuario, compuestos en un solo sitio (orden 5.10): el informe de
//! preparación de un proyecto, los estados de la lista de proyectos y los motivos de los
//! rechazos de configuración. Los mensajes de cada tipo de error viven en su #[error], que es su
//! texto; cuando uno repite una frase de aquí, la toma de aquí.
//!
//! No están aquí los textos que se escriben dentro de los archivos de los proyectos (plantillas,
//! .gitignore, bloque pendiente): son contenido de esos archivos, no mensajes al usuario.

// Estados de un proyecto en la lista.

/// Estado de un proyecto sin archivos de instrucciones.
pub const ESTADO_SIN_CONFIGURAR: &str = "Sin configurar";
/// Estado de un proyecto con el archivo único de reglas y su archivo propio.
pub const ESTADO_CONFIGURADO: &str = "Configurado";
/// Estado de un proyecto con instrucciones de una versión anterior.
pub const ESTADO_FORMATO_ANTIGUO: &str = "Formato antiguo";

// Perfiles y ajustes.

/// Motivo del rechazo de una instancia 0.
#[must_use]
pub fn instancia_desde_uno(herramienta: &str) -> String {
    format!("la instancia de {herramienta} empieza en 1")
}

/// Motivo del rechazo de un perfil con un límite de instrucciones de 0 bytes.
#[must_use]
pub fn limite_de_instrucciones_cero(perfil: &str) -> String {
    format!("{perfil}: el límite de instrucciones no puede ser 0")
}

/// Motivo del rechazo de un tiempo máximo de git de cero segundos.
pub const AJUSTE_TIEMPO_CERO: &str =
    "TiempoMaximoGitSegundos tiene que ser mayor que cero; con cero git no tendría tiempo de responder";

/// Motivo del rechazo de un límite de bandeja de cero bytes.
pub const AJUSTE_LIMITE_CERO: &str =
    "LimiteDeBandejaBytes tiene que ser mayor que cero; con cero se avisaría de todas las bandejas";

// Informe de preparación: estructura.

/// Título del apartado de errores.
pub const APARTADO_ERRORES: &str = "Errores";
/// Título del apartado de avisos.
pub const APARTADO_AVISOS: &str = "Avisos";
/// Título del apartado de cambios.
pub const APARTADO_CAMBIOS: &str = "Cambios";
/// Informe sin nada que contar.
pub const INFORME_SIN_CAMBIOS: &str = "No ha hecho falta cambiar nada.";

/// Línea del informe para un cambio en `archivo`, con su descripción y su detalle, si lo hay.
#[must_use]
pub fn linea_de_cambio(archivo: &str, descripcion: &str, detalle: Option<&str>) -> String {
    match detalle {
        Some(detalle) => format!("{archivo}: {descripcion}; {detalle}."),
        None => format!("{archivo}: {descripcion}."),
    }
}

// Informe de preparación: tipos de cambio.

/// Cambio: archivo creado.
pub const CAMBIO_CREADO: &str = "creado";
/// Cambio: archivo actualizado.
pub const CAMBIO_ACTUALIZADO: &str = "actualizado";
/// Cambio: archivo migrado al formato por regiones de la 2.x.
pub const CAMBIO_MIGRADO: &str = "migrado al formato por regiones";
/// Cambio: archivo llevado al archivo de .gestor/.
pub const CAMBIO_ARCHIVADO: &str = "archivado";
/// Cambio: copia exacta guardada antes de modificar.
pub const CAMBIO_RESPALDADO: &str = "respaldado";
/// Cambio: repositorio git creado.
pub const CAMBIO_REPOSITORIO_CREADO: &str = "repositorio git creado";

// Informe de preparación: git.

/// Error de git init.
#[must_use]
pub fn git_init_fallido(salida_de_error: &str) -> String {
    format!("git init ha fallado: {salida_de_error}")
}

/// Aviso cuando no se puede ejecutar git.
#[must_use]
pub fn git_no_comprobado(motivo: &str) -> String {
    format!(
        "No se ha podido comprobar git ({motivo}). El núcleo común exige que el proyecto esté bajo git."
    )
}

/// Detalle del cambio en .gitignore.
pub const DETALLE_GITIGNORE: &str = "se excluye .gestor/ del control de versiones";

// Informe de preparación: proyecto e instrucciones.

/// Error cuando la carpeta del proyecto no existe.
#[must_use]
pub fn carpeta_inexistente(carpeta: &str) -> String {
    format!("La carpeta del proyecto no existe: {carpeta}")
}

/// Aviso de un archivo anterior con las marcas de región dañadas.
#[must_use]
pub fn marcas_danadas(archivo: &str, archivo_propio: &str) -> String {
    format!(
        "{archivo} tenía las marcas de región dañadas; su texto entero pasa al bloque pendiente de {archivo_propio}."
    )
}

/// Detalle de la creación del archivo propio con un bloque pendiente.
pub const DETALLE_PROPIO_CON_PENDIENTE: &str =
    "lleva un bloque pendiente de consolidar con el texto propio de los archivos de instrucciones anteriores";

/// Aviso de lo archivado del sistema anterior cuando el archivo propio ya existía. `archivados`
/// son pares de ruta de origen y de destino.
#[must_use]
pub fn archivados_sin_propio(archivados: &[(&str, &str)], archivo_propio: &str) -> String {
    let lista: Vec<String> = archivados
        .iter()
        .map(|(origen, destino)| format!("{origen} (ahora en {destino})"))
        .collect();
    format!(
        "Se han llevado al archivo archivos del sistema anterior: {}. {archivo_propio} no se ha tocado; si alguno guardaba una tarea viva, hay que trasladarla.",
        lista.join(", ")
    )
}

/// Detalle del respaldo de un archivo.
#[must_use]
pub fn copia_exacta_en(respaldo: &str) -> String {
    format!("copia exacta en {respaldo}")
}

/// Detalle de la sustitución de un archivo con texto ajeno a la aplicación.
#[must_use]
pub fn sustituido_con_texto_ajeno(archivo_propio: &str) -> String {
    format!(
        "tenía texto que no había escrito Gestor Agentes; lo propio del proyecto va en {archivo_propio}"
    )
}

/// Detalle de la puesta al día de un archivo que había escrito la aplicación.
pub const DETALLE_PUESTO_AL_DIA: &str = "puesto al día con el texto vigente";

/// Aviso de una bandeja del canal sin podar.
#[must_use]
pub fn bandeja_sin_podar(bandeja: &str, tamano: u64, limite: u64) -> String {
    format!(
        "{bandeja} mide {tamano} bytes, más de los {limite} de una bandeja podada; su dueño debe pasar lo resuelto a su histórico."
    )
}

/// Detalle de un archivo del sistema anterior llevado al archivo.
#[must_use]
pub fn movido_al_archivo(destino: &str) -> String {
    format!("movido a {destino}; era del sistema anterior y puede guardar una tarea viva")
}

/// Aviso de un registro de huellas ilegible.
#[must_use]
pub fn registro_ilegible(registro: &str, motivo: &str) -> String {
    format!(
        "{registro} no se ha podido leer ({motivo}); se reconstruye, y los archivos que Gestor Agentes escribe enteros se respaldarán antes de sustituirse si no coinciden con su texto vigente."
    )
}
