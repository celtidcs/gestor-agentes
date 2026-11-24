Historial de cambios

Este historial recoge lo que cambia en cada versión, contado para quien usa la aplicación. Las versiones siguen la numeración de Semantic Versioning: mayor, menor y corrección.


3.9.1, del 24 de noviembre de 2026. Código publicado más limpio

No cambia nada de lo que hace la aplicación. Los comentarios del código y de las pruebas ya no llevan las claves con que se organizaba el trabajo interno, como los códigos de cada tarea o las referencias a sus revisiones, y explican solo lo que hace el código o lo que comprueba cada prueba.

3.9.0, del 24 de noviembre de 2026. Las cuentas, con la biblioteca común

Las cuentas las encuentra la biblioteca común. La búsqueda de las cuentas de Claude Code, Codex y agy la hace ahora la biblioteca compartida deteccion-agentes, en su versión 0.2.0, incluida en la carpeta bibliotecas. Es la misma que usan otros programas, así que todos ven las mismas cuentas de la misma forma.

Más carpetas son cuentas. Una carpeta de tu carpeta de usuario que se llame .claude o .codex seguido de cualquier cosa es una cuenta si tiene alguno de los archivos que la herramienta crea al usarla, no solo el de la sesión: en Claude Code, .claude.json, .credentials.json o settings.json, y en Codex, config.toml o auth.json. Antes solo contaban las carpetas con número, como .claude2, y solo con el archivo de la sesión. Ahora .claude-trabajo también es una cuenta, que toma el número siguiente al mayor que ya tengan tus agentes de Claude. La primera cuenta tiene botón si existe su carpeta, aunque esté vacía. Si guardabas una copia de una cuenta con un nombre como .claude-old para que la aplicación no la usara, cámbiale el nombre por otro que no empiece por .claude. La carpeta que señale CLAUDE_CONFIG_DIR o CODEX_HOME en el entorno desde el que abres la aplicación también cuenta, si existe.

Cada cuenta conserva su número. Una cuenta que ya tiene agente lo conserva aunque aparezcan otras carpetas, y un agente se reconoce por su carpeta, aunque esté escrita con otras mayúsculas o con otra barra en Windows. Si un agente ya usa la primera cuenta con otro número, no se añade un Claude 1 aparte.

Claude 1 y Codex 1 se abren como su cuenta por defecto. La primera cuenta de cada herramienta se abre ahora quitando CLAUDE_CONFIG_DIR o CODEX_HOME, de modo que la herramienta use su cuenta por defecto aunque abras la aplicación desde una terminal que tenga puesta la variable de otra cuenta. Claude 1 ya no fija la carpeta ~/.claude en tu lista de agentes, y si la fijaba, se le quita al buscar los agentes.

El estado de Claude 1 se conserva. Claude Code guarda el estado de su cuenta por defecto, con tus proyectos y ajustes, en el archivo .claude.json de tu carpeta de usuario, y no en el de .claude, que es donde lo guardaba mientras la aplicación lo abría con la variable puesta. La primera vez que abras Claude 1 con esta versión, la aplicación copia el de .claude sobre el de tu carpeta de usuario, guarda antes el anterior en la carpeta datos, cambia el nombre del de .claude para que no se vuelva a copiar y te lo cuenta en la ventana de preparación del proyecto. Si no existe el de .claude, no toca nada, y si algo falla lo vuelve a intentar la próxima vez sin sobrescribir la copia guardada.

Si tienes copias de cuentas, cámbiales el nombre. El manual de uso explica, en el apartado Si guardas copias de cuentas, por qué una copia con un nombre como .claude-old aparece ahora como un agente nuevo, qué pasa si no haces nada y cómo evitarlo o quitar ese agente de la lista.

Una lista de agentes dañada ya no se pierde. Si perfiles-agentes.json no se puede leer al arrancar, por ejemplo porque se ha editado a mano y ha quedado mal escrito, la aplicación ya no lo sobrescribe con las cuentas que encuentra al buscar los agentes, que borraba todo lo que tenías en él. Ahora no guarda nada en ese archivo durante esa sesión y te lo dice en la barra de estado.

El alta a mano, con las mismas reglas. Añadir cuenta valida la carpeta con la biblioteca, contra los agentes que ya tienes y contra las cuentas encontradas en el equipo, y te dice qué agente usa ya esa carpeta. Una carpeta de tu carpeta de usuario se guarda igual que la de una cuenta encontrada, y .claude5, por ejemplo, da Claude 5 si ese número está libre. Si la aplicación no sabe cuál es tu carpeta de usuario, no da de alta la cuenta y te dice por qué.


3.8.2, del 23 de noviembre de 2026. La biblioteca, lista para publicarse

La biblioteca aviso-version-github pasa a su versión 0.2.3, que no cambia su código: su documentación y sus archivos ya no citan rutas del equipo en que se escribe, ni nombres de personas, programas o máquinas, para que pueda publicarse con Gestor Agentes. El comportamiento de la aplicación no cambia. Se corrige además una prueba del vigilante que en Windows podía leer el registro de avisos en el instante en que estaba creado pero aún vacío.


3.8.1, del 23 de noviembre de 2026. El aviso de versión, mejor vigilado

Registro de fallos más claro. Cuando la comprobación de versión nueva falla, la línea que queda en la salida de errores lleva ahora la fecha y la hora y dice si fallaba la de Gestor Agentes o la de Programator, con su repositorio. Antes las dos escribían lo mismo y no se podían distinguir.

El aviso dice siempre la versión con que se comparó. La versión instalada que aparece en el aviso de Actualización disponible es ahora, por construcción, la misma con que se comparó la publicada.

Más pruebas. Hay pruebas nuevas de que el tiempo máximo de espera de ajustes.json llega de verdad a la consulta, de que el aviso y la versión de Programator llegan a la aplicación y de que un fallo al identificar a Gestor Agentes ante GitHub queda registrado. El ejemplo de consulta real dice, junto a su resultado, si la comprobación falló.

Documentación más precisa. Las guías dicen en qué orden se miran las variables del proxy y que hay una comprobación para cada programa.


3.8.0, del 23 de noviembre de 2026. Un aviso de versión compartido

Aviso de versión con la biblioteca común. La comprobación de si hay una versión nueva de Gestor Agentes, y la de la última versión de Programator, la hace ahora la biblioteca compartida aviso-version-github, incluida en la carpeta bibliotecas. Por fuera todo sigue igual y tu ajustes.json no cambia. Por dentro gana tres cosas: la dirección de descarga del aviso se compone siempre con el repositorio configurado y nunca con la que venga en la respuesta, la petición respeta el proxy del sistema, también el de las opciones de internet de Windows, y la respuesta se lee con un tamaño máximo.

La biblioteca, en su versión 0.2.0. Gestor Agentes usa la versión 0.2.0 de esa biblioteca. Con ella, Gestor Agentes se presenta con su nombre y su versión en la petición a GitHub, y si la comprobación falla, porque no hay red o la respuesta no sirve, el motivo queda escrito en la salida de errores en vez de perderse, que en Windows solo se ve si abres la aplicación desde una terminal. Sigue sin salir ningún aviso en ese caso, y en ajustes.json no cambia nada.

Mensaje de error más claro. Si RepositorioDeProgramator está mal escrito en ajustes.json, el aviso lo dice con ese nombre y con su propio mensaje; antes culpaba a RepositorioDeActualizaciones.

Vigilante de Codex más fiable. El vigilante que avisa a Codex ya no puede quedarse esperando cuando la orden codex queue escribe mucho por su salida, y su registro de avisos dice el tiempo de espera real cuando Codex no contesta. Cuando junta en una sola entrega un encargo y una mutación viva, el registro conserva las dos etiquetas, y no solo una.

El Jefe que eliges se ve en el panel. Si eliges Jefe de proyecto en la ventana al abrir un agente, la línea del equipo lo muestra ya como Jefe; antes solo reconocía el Jefe anotado con la forma de la plantilla.

Un mensaje más limpio. El aviso de que la ruta de Programator apunta a una versión antigua ya no lleva espacios de más en medio de la frase.

Orden por dentro y documentación al día. El código se ha reorganizado por dentro, en piezas más pequeñas y mejor separadas, sin cambiar lo que hace la aplicación. Y la documentación se ha puesto al día con el comportamiento real de Codex, de agy y del aviso de versión.


3.7.0, del 22 de noviembre de 2026. Nada pendiente: cuentas propias, avisos más finos y Programator al día

Verificaciones con más constancia. La guía de verificación que la aplicación deja en cada proyecto pide ahora, por cada orden, la orden literal, sus horas, su código de salida y sus líneas de resumen tal cual; que se ejecuten todos los criterios publicados y no solo las cuatro puertas; que ningún valor esperado se copie de otro agente; que en los proyectos con varios manifiestos las puertas se pasen en cada uno; y que las pruebas de Windows y Linux se hagan en las máquinas virtuales y no en el equipo del usuario. También enumera las señales que deben hacer desconfiar a quien acepta un resultado.

La lista de agentes ya no trae las cuentas de nadie. Una instalación nueva empezaba con un Claude 2 que solo existía en el equipo donde se escribió la aplicación. Ahora empieza con una cuenta de cada herramienta, y las demás las añade la búsqueda de cuentas al abrir la ventana. Para las cuentas que viven en otra carpeta, la pestaña Agentes tiene los botones Añadir cuenta de Claude y Añadir cuenta de Codex. Si ya usabas la aplicación, tu lista no cambia.

Avisos a Codex más rápidos y mejor anotados. Entre dos avisos a la misma sesión de Codex pasan ahora 30 segundos como mínimo, en lugar de dos minutos, y los que llegan mientras tanto se juntan en uno. Cuando Codex recibe el aviso en la cola de la sesión abierta, el registro de avisos lo anota como entregado, en lugar de dar a entender que falló.

Sin avisos repetidos de encargos ya atendidos. Si un agente ya ha acusado recibo de un encargo en su bandeja, con una línea que nombra la etiqueta del encargo y dice recibido, su vigilante no se lo vuelve a avisar, aunque arranque después de que el encargo se escribiera.

Aviso de Programator antiguo. La pestaña Agentes muestra la versión del Programator al que apunta su ruta y la compara con la última publicada. Si hay una más nueva, lo dice en su fila y, al abrir Programator, la barra de estado recuerda que la ruta apunta a una versión antigua. Así no se sigue abriendo una versión vieja cuando la nueva se ha instalado en otra carpeta.

Licencia en el paquete. El paquete portable lleva el texto de la licencia, Licencia.txt, en la carpeta documentacion, y el LEEME dice dónde está el código fuente. Así se puede repartir el programa, o el vigilante dentro de otro programa, cumpliendo la licencia.

Política de seguridad. El repositorio tiene un archivo SECURITY.md que explica qué versión recibe correcciones y cómo informar de una vulnerabilidad en privado a través de GitHub.


3.6.0, del 19 de noviembre de 2026. Cada agente en su proyecto y un vigilante propio de la aplicación

Cada agente trabaja solo en su proyecto. agy se abría en su proyecto por defecto, que podía ser otro, y un agente llegó a escribir en la carpeta de un proyecto ajeno creyendo estar en el suyo. Ahora la aplicación abre agy con el proyecto de agy que corresponde a la carpeta, y si agy todavía no tiene ninguno con esa carpeta lo abre pidiéndole que cree uno nuevo para ella, de modo que el proyecto queda creado desde ese momento. Además, cada agente recibe la ruta del proyecto en la variable GESTOR_PROYECTO y, antes de hacer nada, comprueba que está en esa carpeta; si no lo está, para y te avisa. Fuera de esa carpeta no escribe nada ni lee otros proyectos, pero sí usa las herramientas compartidas que citan las normas de la carpeta de proyectos, como las de las máquinas virtuales de pruebas. La ventana de PowerShell en la que se abre cada agente en Windows se cerraba sola a los dos segundos, por un fallo del temporizador que restaura su título, aunque el agente seguía funcionando en ella; ahora ese temporizador ya no la tumba y la ventana sigue viva mientras el agente trabaja. Codex ejecuta sus órdenes desde un servicio que comparten todas sus sesiones, y con dos Codex abiertos en proyectos distintos las órdenes de uno podían ver la identidad y la carpeta del otro; ahora la aplicación se las pasa a cada sesión de Codex como configuración propia, así que cada una ve las suyas. Un agente que abras por tu cuenta, sin la aplicación, no tiene identidad y ya no hace el arranque del equipo ni escribe en la carpeta .gestor.

Un vigilante nuevo, gestor-vigilante. Los guiones de bash y de PowerShell del vigilante del canal se sustituyen por un programa pequeño, gestor-vigilante, que va en el paquete junto a la aplicación y que esta copia a la carpeta .gestor/vigilante de cada proyecto, donde retira los guiones anteriores. Reconoce un encargo aunque nombre a varios agentes («Para Claude 2 y Codex 1») o a la primera instancia sin número («Para Codex»), no vuelve a despertar por un encargo ya avisado que solo ha cambiado de sitio y solo avisa de una mutación viva cuando cambia. Claude Code y agy lo dejan en segundo plano esperando al primer encargo, como hasta ahora. Para Codex lo lanza la propia aplicación en modo persistente, desde la terminal en la que lo abre y fuera del entorno aislado de Codex, que no le deja ver los procesos del equipo; Codex solo le deja anotada su sesión al arrancar, y el vigilante le entrega cada aviso dentro de ella con codex queue, así que Codex ya no tiene que montarse su propia vigilancia. Cuando Codex deja el aviso en la cola pero no puede reanudar la sesión, que la recoge sola al terminar su turno, el registro del vigilante lo anota como encolado y no como entrega fallida.

Las respuestas de Programator despiertan a quien le hizo el encargo. Cuando Programator encabeza su respuesta con una línea Para dirigida al agente que le encargó el trabajo, el vigilante de ese agente lo despierta con un aviso de respuesta de Programator, distinto del de un encargo. La guía para trabajar con Programator explica ese acuerdo y cómo esperar la respuesta mientras la versión instalada de Programator todavía no lo cumpla.

El panel del equipo dice quién está atento. Junto al último latido de cada agente, la línea del equipo indica si su vigilante está vigilando o se ha caído y cuántos encargos le han llegado desde su último acuse.

Acuse de recibo. Al leer un encargo, cada agente escribe en su bandeja una línea RECIBIDO con la hora, como prueba de que está vivo y de que lo ha leído. Esas líneas no se borran al podar la bandeja.

Eliges el papel al abrir. Encima de los botones de los agentes puedes elegir si el próximo agente que abras será Jefe de proyecto o ayudante. La aplicación lo anota en el estado del proyecto y el agente no tiene que preguntártelo. La elección vale para una sola apertura; si no eliges nada, el agente lo pregunta al arrancar como antes.

Codex trabaja sin pedirte permiso. Codex se abre con su entorno aislado de escritura en el proyecto, con acceso a la red y sin pedir nunca aprobación, así que puede atender los encargos del jefe de proyecto sin que tengas que aprobar cada orden. Su entorno aislado no le da permiso para escribir fuera de la carpeta del proyecto, y no te pide aprobación para saltárselo. En Windows prefiere MXC, el entorno aislado que OpenAI recomienda allí. Antes se abría pidiendo aprobación cuando hacía falta, y en Windows eso acababa en peticiones constantes porque su entorno aislado fallaba a menudo. Si tu perfil de Codex ya fija el sandbox o la aprobación, manda el perfil.

Preparación más completa. La aplicación añade al estado del proyecto los apartados de papeles y de mutaciones vivas si le faltan, para que no los añadan varios agentes a la vez. Avisa si el código del proyecto cita AGENTS.md, CLAUDE.md o GEMINI.md, porque la aplicación reescribe esos archivos y una prueba que dependa de ellos se rompería sola. Y cuando un agente termina de consolidar las instrucciones antiguas, borra las carpetas de archivo que dejó ella, cosa que el agente no siempre tiene permiso para hacer.

Aprendizaje común. Las soluciones comprobadas en un proyecto viajan ahora en la aplicación como fichas, y cada proyecto recibe las comprobadas en .gestor/consulta/aprendizaje-comun.md. Si un proyecto marca una de sus reglas aprendidas como candidata a común, la aplicación la anota en candidatas-comunes.md, en su carpeta de datos, para que decidas si pasa a ficha.

La pestaña Agentes enseña dónde vive cada agente. La columna Estado mostraba solo la ruta del programa, que es la misma para todas las cuentas de una herramienta. Ahora dice primero el estado con una palabra, luego la carpeta donde vive el agente y si existe, y por último el programa. La celda con el nombre del agente se colorea según ese estado: verde si se puede abrir, ámbar si falta la carpeta de su cuenta, rojo si no se encuentra el programa y gris mientras está sin comprobar. Mientras se buscan los agentes, un indicador giratorio lo dice junto a los botones de la pestaña.

La primera cuenta también necesita su carpeta. Si faltaba la carpeta .claude o .codex, Claude 1 o Codex 1 seguían apareciendo como encontrados y, al abrirlos, la herramienta creaba una carpeta vacía y pedía iniciar sesión otra vez. Ahora la primera cuenta se comprueba igual que las demás y, sin su carpeta, sale en ámbar y sin botón.

Método. Las instrucciones de verificación piden ahora que cada resultado cite el registro de su ejecución y que una mutación se compruebe aplicada antes de leer su resultado, y las de método recogen el rigor con las citas, las páginas web y los experimentos que dejan rastro fuera del proyecto.

3.5.1, del 18 de noviembre de 2026. Vigilantes que no se quedan colgados

Corrige un fallo que podía dejar cientos de vigilantes del canal en marcha. Se vio porque una ventana de consola se abría y se cerraba sin parar en Windows. Al contar los procesos aparecieron más de seiscientos vigilantes vivos, de varias sesiones de días anteriores, y se seguían creando cada pocos segundos. La causa era doble. Claude Code vigilaba con su herramienta Monitor, que caduca a los pocos minutos; al caducar dejaba de escuchar, pero en Windows el guion seguía vivo y en bucle, y las instrucciones pedían a Claude rearmarlo, así que cada rearme sumaba un vigilante más. Además, el guion de bash lanzaba varios programas por cada archivo del canal en cada comprobación, y en Windows cada uno abría su propia consola, que era la ventana que parpadeaba. También se vio que una sesión de agente que se daba por cerrada pero seguía abierta continuaba rearmando su vigilante.

Cada agente vigila el canal por su cuenta, con su identidad, tengas uno o varios y sea cual sea su herramienta, y ninguno da por hecho que existan los demás. Todos siguen ahora la misma regla: el vigilante solo despierta al agente cuando hay un encargo para él, así no gasta un turno con cada cambio del canal que no le toca, nunca hay dos del mismo agente a la vez y ninguno sigue en marcha cuando se cierra su sesión. Claude Code y agy lo consiguen con el mismo guion, que dejan en segundo plano y termina con el primer encargo; al terminar, la herramienta despierta al agente, que lo atiende y vuelve a lanzarlo. Claude Code ya no usa el modo continuo con su herramienta Monitor, que era el origen del fallo. Codex no se despierta al terminar una tarea en segundo plano, así que sigue montando su propia vigilancia, ahora con esas mismas condiciones.

Los dos guiones del vigilante, que puede usar cualquier agente, se protegen además solos. Solo puede haber uno en marcha por agente: si se lanza otro, el anterior lo ve y termina. Terminan también si se cierra el proceso que los lanzó, y tienen una vida máxima, de 29 minutos en el modo continuo y de 8 horas en el que espera al cambio, que se puede cambiar con la opción --vida-segundos, o -VidaSegundos en PowerShell. En Windows las instrucciones usan el guion de PowerShell, que hace cada comprobación sin lanzar ningún programa, y el de bash lanza ahora solo dos por comprobación, la espera y el cálculo de las sumas, cuando antes lanzaba varios por cada archivo.

Se estudió también tener un solo vigilante para todos los agentes, pero no es posible de la misma forma en las tres herramientas: agy no deja que un programa externo despierte una sesión abierta, Codex no se despierta al terminar una tarea en segundo plano y Claude Code solo lo permite con una función todavía en pruebas. Por eso cada agente sigue teniendo el suyo, ahora con estas protecciones.


3.5.0, del 16 de noviembre de 2026. agy vigila el canal sin gastar

agy ya no mira el canal a cada rato. Sus instrucciones le dan una orden lista para usar: deja en segundo plano el vigilante con la opción nueva --hasta-cambio, o -HastaCambio en PowerShell, que termina en cuanto llega un encargo para el agente o alguien deja una mutación viva, y al terminar agy despierta al agente. Mientras no hay nada para él no gasta nada, y los demás cambios los ve en ese mismo aviso. La orden va completa en las instrucciones, sin código que escribir, porque agy reescribe las variables de PowerShell de una orden escrita a mano y el vigilante dejaba de funcionar sin avisar.

Avisos sin nada que decir. Las instrucciones piden ahora a todos los agentes que, si un aviso del vigilante no trae nada para ellos, no escriban nada y sigan vigilando, en lugar de contar en la terminal lo que ha pasado en el canal.

Modelo y esfuerzo. El manual explica cómo elegir el modelo y el esfuerzo de cada agente una sola vez, dentro de su herramienta, para que valgan en todos tus proyectos.

3.4.0, del 16 de noviembre de 2026. Menos interrupciones y menos gasto

Sin preguntas de permiso al arrancar en Claude Code. La aplicación deja autorizado en el archivo .claude/settings.local.json del proyecto lo que todo agente hace al arrancar, y nada más: saber quién es, mirar la fecha, listar archivos, ver el estado de git, escribir en la carpeta .gestor y poner en marcha su vigilante. Conserva lo que ya tuvieras en ese archivo. Para agy, cuyos permisos se guardan fuera del proyecto, el manual explica qué reglas añadir una sola vez.

Los avisos ya no interrumpen. Los avisos de la preparación que debe atender un agente, como una bandeja sin podar, ya no abren una ventana en cada lanzamiento: la barra de estado dice cuántos hay y quedan en .gestor/avisos-de-preparacion.md, que cada agente lee al arrancar. La ventana sigue saliendo si algo falla o si la aplicación ha guardado copia de algún archivo.

agy también vigila el canal. Sus instrucciones le piden ahora que monte una vigilancia continua que lo avise aunque esté esperando, como ya hacía Codex. Solo si su herramienta no puede, mira el canal al terminar cada paso y lo dice.

Mensajes más baratos. Los agentes escriben en el canal solo cuando tienen un resultado, una pregunta o un bloqueo, sin acuses de recibo; citan dónde está cada cosa en vez de copiarla; no repiten en la terminal lo que ya está en el canal; y tus órdenes las copia solo quien las recibe de ti.

Arranques más ligeros. El diario de cada agente guarda solo la tarea en curso y el estado del proyecto solo lo vigente; lo cerrado pasa a su histórico. La aplicación avisa, como ya hacía con las bandejas, cuando un diario o el estado crecen demasiado.


3.3.2, del 13 de noviembre de 2026. Pruebas con un proyecto de ejemplo neutro

Sin cambios en la aplicación. Las pruebas del lanzador usan ahora como ejemplo un proyecto ficticio llamado Demo.


3.3.1, del 13 de noviembre de 2026. Programator en el mismo canal

Programator habla como los demás agentes. Desde su versión 0.12.0, Programator escribe sus reservas, sus latidos y sus respuestas con el mismo formato que el resto del equipo, así que la revisión del canal ya no da avisos por su culpa y su último latido aparece en la pestaña Proyectos. La guía que reciben los agentes para trabajar con él les explica ahora que un encargo para Programator se escribe como cualquier otro, con una línea que empieza por «Para Programator».


3.3.0, del 13 de noviembre de 2026. Codex vigila por su cuenta

Codex se monta su propio vigilante. Al arrancar, las instrucciones de Codex le piden que monte él mismo un sistema de vigilancia del canal que lo avise de los mensajes nuevos aunque esté esperando, de la forma que mejor sepa hacerlo; si no puede, mira el canal al terminar cada paso, como hasta ahora. Solo agy se queda sin aviso automático.

Documentación al día. El manual de uso, la resolución de problemas y el archivo LEEME del paquete explican con detalle cómo arrancan los agentes, cómo contestar a la pregunta de su papel, cómo trabajan juntos con un Jefe y sus ayudantes, cómo se enteran de los mensajes, qué significa cada aviso sobre el equipo y cómo actualizar la aplicación conservando tu configuración.

3.2.0, del 13 de noviembre de 2026. El equipo, a la vista

El estado del equipo en la ventana. Al seleccionar un proyecto, la pestaña Proyectos dice quién es su Jefe, cuánto hace que cada agente dio señales de vida, cuántas reservas hay y si queda código cambiado a propósito sin restaurar.

Revisión del estado del canal. Al abrir un proyecto, la aplicación avisa si dos agentes reservan el mismo archivo, si una reserva es de un agente que lleva más de doce horas sin dar señales, si queda alguna mutación viva o si al estado del proyecto le falta algún apartado. Las horas se cambian en ajustes.json, con LimiteDeLatidoHoras.

El vigilante señala los encargos. Cuando un mensaje nuevo va dirigido al agente, el aviso del vigilante lo marca como encargo, y también avisa cuando se anota una mutación viva.

Reglas aprendidas. Cuando un fallo se debe a una trampa del proyecto que se puede repetir, los agentes la dejan escrita como regla de una línea en el archivo propio del proyecto, para que nadie vuelva a caer.

Gestor Agentes y la ingeniería de arneses. Un documento nuevo explica qué ideas de la ingeniería de arneses para agentes aplica la aplicación y con qué funcionalidad, con sus fuentes.


3.1.0, del 13 de noviembre de 2026. Los agentes arrancan solos

Cada agente empieza a trabajar en cuanto se abre. La aplicación le manda un primer mensaje que le pide seguir el arranque de AGENTS.md, así que ya no hace falta escribirle nada para que se presente, lea el estado del proyecto y los mensajes de los demás y retome lo que hubiera pendiente. Claude Code y Codex lo reciben como primer mensaje de su sesión, y agy con su opción -i.

El papel de cada agente lo decides tú. Si el estado del proyecto todavía no dice si un agente es el Jefe de proyecto o un ayudante, el agente te lo pregunta al arrancar y espera tu respuesta antes de hacer nada más. La respuesta queda anotada, y la siguiente vez ya no lo pregunta.

Vigilante del canal incluido. La aplicación deja en cada proyecto dos guiones, uno para bash y otro para PowerShell, que avisan de los mensajes nuevos de los demás agentes. Claude Code los deja vigilando en segundo plano; Codex y agy, que no tienen aviso automático, los ejecutan al terminar cada paso.

La versión, a la vista, y aviso de versión nueva. La versión de la aplicación se ve arriba a la derecha. Al arrancar, la aplicación mira en GitHub si hay una versión más nueva y, si la hay, te dice cuál es y dónde descargarla. No descarga nada por su cuenta, no envía datos tuyos y se puede desactivar en ajustes.json.

Programator en todos los proyectos. Cada proyecto recibe una guía para que los agentes sepan encargarle trabajo a Programator, aunque Programator no se haya abierto todavía en él.

Menos lastre del sistema anterior. Las reglas antiguas que agy cargaba desde la carpeta .agents se tratan ahora como el resto de instrucciones antiguas: su texto pasa al bloque pendiente y el archivo se aparta. Los agentes ya no siguen nada de lo apartado, y el que consolida el bloque lo borra cuando ha rescatado lo que siga vivo.

Instrucciones más cortas y pensadas para los agentes. AGENTS.md pasa de unos 21 KB a unos 14 KB y está escrito como una lista de pasos y reglas que el agente sigue con facilidad. Lo que solo hace falta en momentos concretos, como el detalle de cómo se verifica un cambio, está en los documentos de consulta, que el agente lee solo cuando le hacen falta.


3.0.0, del 11 de noviembre de 2026. Gestor Agentes

La aplicación se llamaba Gestor Claude y estaba escrita en C Sharp con .NET. Desde esta versión se llama Gestor Agentes y está escrita en Rust. El ejecutable es uno solo, gestor-agentes.exe en Windows o gestor-agentes en Linux, no necesita .NET ni ningún otro componente, y la ventana es la misma en los dos sistemas.

Portable de verdad. La configuración ya no está en la carpeta del usuario, sino en la carpeta datos, junto al ejecutable, y viaja con él. La primera vez que se abre sin lista de proyectos ni perfiles en datos, copia la configuración de Gestor Claude sin tocar la original. Desaparecen los guiones de guardar y restaurar la configuración, que ya no hacen falta. Los ajustes nuevos, el tiempo máximo de git y el límite de tamaño de una bandeja, van en datos/ajustes.json.

Todos los agentes en terminal. Claude Code, Codex, Gemini con agy y Programator se abren en una terminal, situada en la carpeta del proyecto. Ya no se abren las aplicaciones de escritorio de Codex ni de Antigravity, y con ello desaparece el aviso de que Antigravity ya estaba abierto. Programator es un tipo de agente nuevo, que se abre con la ruta de su ejecutable.

Solo los botones que sirven. Al arrancar, la aplicación busca el programa de cada perfil y muestra solo los botones de los agentes instalados, con la misma búsqueda que se usa al abrirlos. En Windows, codex se busca primero donde lo deja su instalador oficial. También reconoce las cuentas instaladas, como ~/.claude, ~/.claude2 o ~/.codex, y añade un perfil por cada una que falte, sin tocar los que ya había. La pestaña nueva Agentes muestra el estado de cada perfil y permite fijar la ruta de su programa, con Examinar o con Búsqueda automática.

Un solo archivo de instrucciones. AGENTS.md pasa a ser el único archivo de reglas de todos los agentes, y la aplicación lo escribe entero cada vez que abre uno. CLAUDE.md queda como un puntero que lo carga, y GEMINI.md deja de generarse, porque Antigravity lee AGENTS.md. Lo propio de cada proyecto va en .gestor/instrucciones-del-proyecto.md, que la aplicación crea una vez y no vuelve a tocar. En un proyecto preparado con una versión anterior, recoge el texto propio de sus archivos antiguos en un bloque pendiente de consolidar, que el primer agente que arranque ordena por su cuenta, y lleva a .gestor/archivo lo que ya no se usa. Lo que solo se consulta a veces sale a .gestor/consulta, y así AGENTS.md cabe entero en lo que leen Codex y Gemini.

Avisos nuevos. Si AGENTS.md mide más de lo que lee Codex o Gemini, la barra de estado lo avisa al abrirlos, y un perfil puede fijar otro límite. El informe de preparación avisa de las bandejas del canal que han crecido demasiado. Si falta OpenGL 2.0, la aplicación lo dice con un mensaje claro en vez de cerrarse sin más.

Correcciones. En Windows, un nombre de carpeta con comillas tipográficas ya no puede romper la orden con la que se abre la terminal: se escapan las cinco variantes de la comilla simple que reconoce PowerShell. Los archivos de instrucciones ya no llegan cortados a Codex ni a Gemini.

Para desarrollar. El código se organiza en tres partes, nucleo, lanzador e interfaz, con sus pruebas aparte. Los guiones setup, build-portable, prueba-portable y puertas-rust, en versión para Windows y para Linux, preparan el entorno, generan y comprueban el paquete portable y pasan las cuatro comprobaciones de calidad.


Versiones anteriores

Las versiones 1.x y 2.x se publicaron con el nombre Gestor Claude y estaban escritas en C Sharp, primero con WPF y después con Avalonia. La 3.0.0 las sustituye por completo y conserva su configuración.
