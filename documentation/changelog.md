Historial de cambios

Este historial recoge lo que cambia en cada versión, contado para quien usa la aplicación. Las versiones siguen la numeración de Semantic Versioning: mayor, menor y corrección.


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
