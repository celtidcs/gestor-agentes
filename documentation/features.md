Funcionalidades

Todas las funcionalidades están disponibles en Windows y en Linux. Gestor Agentes es una sola aplicación, con la misma ventana en los dos sistemas.


Lista de proyectos

La pestaña Proyectos muestra cada proyecto registrado con su nombre, su ruta, la fecha de la última apertura y su estado. El estado se calcula en el momento mirando los archivos de instrucciones del proyecto. Es Sin configurar si no hay ninguno; Configurado si AGENTS.md lleva el núcleo común y existe .gestor/instrucciones-del-proyecto.md; y Formato antiguo en cualquier otro caso, por ejemplo un proyecto que todavía tiene los archivos de una versión anterior. Un proyecto en formato antiguo se pone al día la próxima vez que se abre. Los proyectos se seleccionan con un clic. El doble clic está desactivado, para que nadie abra un proyecto con el agente equivocado por accidente.


Agregar, quitar y eliminar

El botón Agregar Proyecto abre el selector de carpetas del sistema y añade la carpeta elegida, sin admitir duplicados. El botón Quitar de cada fila borra solo el registro de la lista y no toca el disco. El botón Eliminar del disco borra la carpeta entera después de pedir confirmación expresa; si la carpeta ya no existía, simplemente la quita de la lista.


Botones de perfil

Bajo la lista aparece un botón por cada perfil de agente cuyo programa se ha encontrado en el equipo, con el nombre de su identidad, por ejemplo Abrir · Claude 2. La aplicación no inicia sesión ni guarda credenciales: reconoce las carpetas de cuenta que ya existen y abre la herramienta con la suya.

Los perfiles se leen de perfiles-agentes.json, en la carpeta datos. La primera vez que arranca, la aplicación crea ese archivo con Claude 1, Claude 2, Codex 1, Gemini 1 y Programator 1. Se pueden añadir más instancias de cualquier tipo. Los perfiles de Codex y de Gemini pueden llevar en Argumentos opciones para su programa, como el modelo. Al abrir con Codex o con Gemini, si AGENTS.md mide más de lo que lee la herramienta, la barra de estado lo avisa. Ese límite es de 32.768 bytes para Codex y de 24.000 para Gemini, salvo que el perfil fije otro en LimiteDeInstrucciones.


Preparación automática del proyecto al abrirlo

Cada vez que se abre un proyecto con cualquier perfil, la aplicación comprueba sus archivos: si no existen los crea y, si existen, los pone al día. El trabajo se hace en segundo plano para no congelar la ventana, y sigue este orden.

Primero se asegura el repositorio git. Si la carpeta no es un repositorio se ejecuta git init, y la entrada .gestor/ se añade al .gitignore, porque el canal de los agentes no se versiona. Si git no está instalado, se avisa y se sigue adelante.

Si el proyecto todavía no tiene su archivo propio, .gestor/instrucciones-del-proyecto.md, se lee antes el texto propio que haya en sus archivos de instrucciones anteriores, CLAUDE.md, AGENTS.md y GEMINI.md, para no perderlo en los pasos siguientes.

Después se llevan a la carpeta de archivo de .gestor los archivos del sistema de trabajo anterior que la aplicación reconoce por su nombre, para que no convivan con los vigentes. El canal antiguo de Gemini, que son canal.md, historico.md, session.log e INSTRUCCIONES_GEMINI.md de .gestor/gemini, va a .gestor/archivo/canal-gemini-antiguo. GEMINI.md y .gestor/memoria-compartida.md van a .gestor/archivo/sistema-anterior. No se borra nada: si en el destino ya hay un archivo con ese nombre, al nuevo se le añade la fecha. Cada movimiento queda en el informe, porque alguno puede guardar una tarea a medias.

Luego se crea la estructura de .gestor: el archivo de estado del canal, estado.md; el registro de órdenes, ordenes-del-director.md; una bandeja del canal por agente, como claude.md, claude2.md, codex.md o gemini.md; y la carpeta propia de cada instancia, con su tarea en curso, su acta de verificaciones y su registro de funciones. Solo se crea lo que falta, y no se sobrescribe nada de lo que ya existe.

A continuación, si falta, se crea el archivo propio del proyecto. Si el proyecto tenía texto propio en sus archivos anteriores, el archivo nace con ese texto dentro de un bloque pendiente de consolidar, sin los párrafos que se repetían entre los tres archivos, y con la lista de lo que se ha llevado al archivo. Si el archivo propio ya existía, no se toca, y el informe avisa de lo archivado.

Después se escriben enteros los archivos que son de la aplicación: AGENTS.md, CLAUDE.md y los documentos de consulta de .gestor/consulta. Si alguno tenía texto que no había escrito la aplicación, antes se guarda una copia exacta en .gestor/respaldos.

Por último se avisa de cada bandeja del canal que pasa de 16 KiB, el tamaño que corresponde a las unas ochenta líneas vivas que piden las instrucciones, para que su dueño pase lo resuelto a su histórico. Ese límite se cambia en datos/ajustes.json, con la clave LimiteDeBandejaBytes.

Al terminar, la barra de estado resume lo que ha cambiado. Si algo requiere atención, como un respaldo, un aviso o un error, se abre además un diálogo con el informe completo. Un paso que falla se anota en el informe y no impide los siguientes, y una segunda apertura seguida no cambia nada.


Archivo único de instrucciones

Todos los agentes leen las mismas reglas de un solo archivo, AGENTS.md, que la aplicación escribe entero cada vez que abre un agente. Lleva la cabecera de identidad común, el núcleo común y, al final, las normas de la carpeta que contiene los proyectos. No tiene zona propia: lo que se escriba a mano en él se pierde en la siguiente apertura, aunque antes se guarda una copia exacta en .gestor/respaldos y el informe lo avisa. Codex y Gemini leen AGENTS.md directamente. Claude Code lo carga a través de CLAUDE.md, que es solo un puntero con dos líneas de importación, @AGENTS.md y @.gestor/instrucciones-del-proyecto.md. GEMINI.md ya no se genera.

Lo propio de cada proyecto vive en .gestor/instrucciones-del-proyecto.md: sus reglas, sus órdenes particulares, sus prohibiciones y los detalles de su entorno. La aplicación lo crea una sola vez, si falta, y no lo vuelve a tocar nunca. Lo leen todos los agentes al empezar cada sesión.

Lo que solo se consulta en momentos concretos está fuera del núcleo, en dos documentos de consulta que la aplicación reescribe en cada apertura: .gestor/consulta/codigo.md y .gestor/consulta/metodo.md. El núcleo remite a ellos. Así AGENTS.md cabe en los 24.000 bytes que lee Gemini.

Cuando un proyecto viene del sistema anterior, su texto propio pasa al archivo propio del proyecto dentro de un bloque pendiente de consolidar. Se descarta solo lo que se reconoce con seguridad como escrito por la aplicación, que son sus regiones, sus plantillas completas y los textos que dejaba fuera de ellas, y el bloque de la unidad de trabajo, que ya llega con las normas comunes. Si un archivo tenía las marcas dañadas, su texto pasa entero. Las instrucciones obligan al primer agente que arranque a consolidar ese bloque antes de cualquier otro trabajo, sin que nadie se lo pida: mira si algún archivo archivado guarda una tarea a medias y la traslada, deja lo que siga vigente como reglas del proyecto, pregunta al usuario lo que contradiga el núcleo y borra el bloque. La copia exacta de los CLAUDE.md y AGENTS.md anteriores queda en .gestor/respaldos, y GEMINI.md en .gestor/archivo/sistema-anterior.

La aplicación guarda una huella SHA-256 de cada archivo que escribe, para distinguir un archivo suyo sin cambios de uno editado a mano, que se respalda antes de reescribirlo. Se respetan la marca BOM y los finales de línea originales, y la escritura es atómica.


Identidad de cada agente

Al abrir un agente, la aplicación fija la variable de entorno GESTOR_AGENTE con su identidad, como Claude 1, Codex 2 o Gemini 1. La cabecera de AGENTS.md, común a todos, pide al agente que la lea y que anuncie al arrancar quién es y qué bandeja del canal ocupa. En Claude Code se fija también CLAUDE_CONFIG_DIR, y en Codex CODEX_HOME, para que cada perfil use su propia carpeta de configuración.


Detección de agentes y de cuentas

Al arrancar, en un hilo de trabajo que no bloquea la ventana, la aplicación busca el programa de cada perfil con la misma búsqueda que usa al abrirlo, de modo que un botón visible siempre se puede abrir. En Windows, codex se busca primero en %LOCALAPPDATA%\Programs\OpenAI\Codex\bin y agy en %LOCALAPPDATA%\agy\bin, donde los dejan sus instaladores oficiales. Después, todos se buscan en ~/.local/bin, en Linux luego en /usr/local/bin y /usr/bin, y por último en el PATH. Programator no se busca: solo se encuentra si su perfil lleva la ruta de su ejecutable. Si un perfil lleva una ruta fijada, solo vale esa, y si no existe el agente queda como no encontrado.

También reconoce las cuentas instaladas: ~/.claude y ~/.claudeN con .credentials.json, y ~/.codex y ~/.codexN con auth.json. Cada cuenta que no tenga perfil se añade a perfiles-agentes.json, después de los de su herramienta; los perfiles guardados no se cambian ni se borran. Un perfil de Claude o de Codex a partir de la instancia 2 solo tiene botón si existe su carpeta de cuenta.

La pestaña Agentes muestra el estado de cada perfil y permite cambiar la ruta de su programa, con Examinar y con Búsqueda automática. Las rutas que no existen se rechazan sin guardar nada.


Terminal

Todos los agentes se abren en una ventana de terminal situada en la carpeta del proyecto y titulada con la identidad y el nombre del proyecto, con su programa: claude, codex, agy o Programator. En Windows es PowerShell, con un temporizador que restaura el título cuando el agente lo cambia. En Linux se usa el primer emulador disponible entre gnome-terminal, konsole, xfce4-terminal y xterm. No se abren las aplicaciones de escritorio de Codex ni de Antigravity, que tienen sus propios accesos directos. Si un programa deja de estar donde se encontró, la aplicación lo avisa al abrir, y los archivos del proyecto quedan preparados igualmente.


Normas comunes a todos los proyectos

Las normas que deben cumplir todos los proyectos se escriben una sola vez en un CLAUDE.md, AGENTS.md o GEMINI.md de la carpeta que los contiene. Cada vez que abre un agente, la aplicación sube desde la carpeta del proyecto hasta encontrar el primero de esos archivos que no lleve marcas propias de la aplicación, y copia su contenido al final de AGENTS.md, precedido de un encabezado que dice de qué archivo sale. Los que llevan esas marcas son de otro proyecto gestionado y se saltan; las marcas con otros identificadores, como un bloque delimitado a mano, no cuentan. Las marcas que pudiera contener el texto copiado se desactivan. Si no encuentra ningún archivo, el bloque lo dice. Los cambios en ese archivo llegan a cada proyecto la siguiente vez que se abre.

Se copia en vez de confiar en que el agente lo lea porque Codex y Gemini no leen nada por encima de la raíz del repositorio. Claude Code, en cambio, sí carga por su cuenta el CLAUDE.md de las carpetas superiores, así que recibe esas normas dos veces, la original y la copia de AGENTS.md. Si las dos difieren, manda la original, y la copia se pone al día en la siguiente apertura.


Pestaña Instrucciones de los agentes

Muestra el núcleo común en modo de solo lectura. El núcleo va dentro del ejecutable y cambia con cada versión de la aplicación, así que no se edita desde aquí. Lo que aprenda un proyecto se escribe en su .gestor/instrucciones-del-proyecto.md.


Configuración

La aplicación es portable: la configuración vive en la carpeta datos, junto al ejecutable. config.json guarda la lista de proyectos, y perfiles-agentes.json los perfiles. ajustes.json guarda dos ajustes de funcionamiento. TiempoMaximoGitSegundos son los segundos que se espera a una orden de git antes de pararla, 30 por defecto. LimiteDeBandejaBytes es el tamaño a partir del cual se avisa de que una bandeja del canal está sin podar, 16.384 por defecto. Si el archivo falta, la aplicación lo crea con esos valores; una clave que falte toma su valor por defecto, y un cero se rechaza al arrancar con un error que nombra la clave.

La primera vez que se abre, si la carpeta datos no tiene todavía config.json ni perfiles-agentes.json, copia la configuración de Gestor Claude, la versión anterior, que estaba en %APPDATA%\GestorClaude en Windows y en ~/.config/GestorClaude en Linux. Si existe un claude-instructions.txt de versiones anteriores, solo se lee, para reconocer como plantilla antigua el contenido de los proyectos que lo usaban.
