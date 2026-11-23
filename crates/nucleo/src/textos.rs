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

/// Motivo del rechazo de un límite de latido de cero horas.
pub const AJUSTE_LATIDO_CERO: &str =
    "LimiteDeLatidoHoras tiene que ser mayor que cero; con cero toda reserva parecería huérfana";

/// Aviso: a estado.md le falta un apartado obligatorio.
#[must_use]
pub fn estado_sin_apartado(estado: &str, apartado: &str) -> String {
    format!(
        "{estado} no tiene el apartado «{apartado}» de la plantilla; el primer agente que arranque lo añade sin borrar nada."
    )
}

/// Aviso: hay mutaciones de control aplicadas.
#[must_use]
pub fn mutaciones_vivas(estado: &str) -> String {
    format!(
        "{estado} anota mutaciones vivas: hay código cambiado a propósito para una prueba y sin restaurar. Que su dueño lo restaure antes de seguir."
    )
}

/// Aviso: una reserva no sigue el formato de la plantilla.
#[must_use]
pub fn reserva_ilegible(estado: &str, linea: &str) -> String {
    format!(
        "{estado} tiene una reserva que no sigue el formato «- <Agente>, <fecha hora>: <rutas> — <motivo>»: {linea}"
    )
}

/// Aviso: dos agentes reservan la misma ruta.
#[must_use]
pub fn reserva_repetida(ruta: &str, uno: &str, otro: &str) -> String {
    format!("{uno} y {otro} tienen reservada a la vez {ruta}; uno de los dos tiene que liberarla.")
}

/// Aviso: la reserva es de un agente sin latido reciente.
#[must_use]
pub fn reserva_sin_latido(agente: &str, horas: Option<i64>, limite: u64) -> String {
    match horas {
        Some(horas) => format!(
            "{agente} tiene reservas, pero su último latido es de hace {horas} horas, más de las {limite} de LimiteDeLatidoHoras; puede que ya no esté trabajando. Que el Jefe compruebe si siguen vigentes."
        ),
        None => format!(
            "{agente} tiene reservas, pero su bandeja no tiene ningún latido legible; puede que ya no esté trabajando. Que el Jefe compruebe si siguen vigentes."
        ),
    }
}

/// Motivo del rechazo de un tiempo máximo de cero segundos para la consulta de versión.
pub const AJUSTE_TIEMPO_ACTUALIZACION_CERO: &str =
    "TiempoMaximoActualizacionSegundos tiene que ser mayor que cero; con cero GitHub no tendría tiempo de responder";

/// Motivo del rechazo de un repositorio de actualizaciones mal escrito.
pub const AJUSTE_REPOSITORIO_INVALIDO: &str =
    "RepositorioDeActualizaciones tiene que tener la forma propietario/nombre, como celtidcs/gestor-agentes";

// Informe de preparación: estructura.

/// Título del apartado de errores.
pub const APARTADO_ERRORES: &str = "Errores";
/// Título del apartado de avisos.
pub const APARTADO_AVISOS: &str = "Avisos";
/// Título del apartado de cambios.
pub const APARTADO_CAMBIOS: &str = "Cambios";
/// Informe sin nada que contar.
pub const INFORME_SIN_CAMBIOS: &str = "No ha hecho falta cambiar nada.";

/// Cabecera del archivo de avisos de la última preparación, hecha en `momento`.
#[must_use]
pub fn cabecera_de_avisos(momento: &str) -> String {
    format!(
        "# Avisos de la preparación del proyecto

Gestor Agentes reescribe este archivo cada vez que prepara el proyecto y lo borra cuando no queda ningún aviso: no lo edites. Última preparación: {momento}.

Atiende los avisos que te tocan; por ejemplo, si tu bandeja está sin podar, pasa lo resuelto a tu histórico.

"
    )
}

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

/// Descripción de un cambio: se ha borrado algo que ya no se usa.
pub const CAMBIO_BORRADO: &str = "borrado, porque ya no se usa";

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
/// Detalle del cambio en el archivo de permisos de una herramienta.
pub const DETALLE_PERMISOS: &str = "se autoriza por defecto lo que pide el arranque de los agentes";

/// Aviso de un archivo de permisos que no se entiende y por eso no se toca.
#[must_use]
pub fn permisos_ilegibles(archivo: &str, motivo: &str) -> String {
    format!(
        "{archivo} no se ha podido leer ({motivo}); no se toca, y los agentes preguntarán los permisos del arranque hasta que se corrija."
    )
}

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

/// Aviso de un archivo que se lee entero en cada arranque y ha crecido demasiado.
#[must_use]
pub fn archivo_de_arranque_grande(archivo: &str, tamano: u64, limite: u64) -> String {
    format!(
        "{archivo} mide {tamano} bytes, más de los {limite} que conviene leer en cada arranque; lo cerrado debe pasar a su histórico o a .gestor/archivo/."
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

/// Aviso: Gestor Agentes no lleva a su lado el ejecutable del vigilante del canal.
#[must_use]
pub fn vigilante_sin_ejecutable(destino: &str) -> String {
    format!(
        "No se ha encontrado gestor-vigilante junto a Gestor Agentes, así que no se ha copiado {destino}: los agentes no tendrán vigilante del canal. Reinstala Gestor Agentes desde su paquete completo."
    )
}

/// Aviso: se han anotado reglas candidatas a aprendizaje común.
#[must_use]
pub fn candidatas_recogidas(destino: &str) -> String {
    format!(
        "Se han anotado reglas candidatas a aprendizaje común en {destino}. El Director decide cuáles pasan a ficha."
    )
}

/// Aviso: el código del proyecto cita los archivos de reglas que la aplicación reescribe.
#[must_use]
pub fn codigo_que_cita_las_reglas(archivos: &[String]) -> String {
    format!(
        "El código del proyecto cita AGENTS.md, CLAUDE.md o GEMINI.md en {}. Gestor Agentes reescribe esos archivos en cada lanzamiento: ninguna prueba ni programa del proyecto debe depender de su contenido.",
        archivos.join(", ")
    )
}
