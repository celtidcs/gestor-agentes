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

Después se llevan a la carpeta de archivo de .gestor los archivos del sistema de trabajo anterior que la aplicación reconoce por su nombre, para que no convivan con los vigentes. El canal antiguo de Gemini, que son canal.md, historico.md, session.log e INSTRUCCIONES_GEMINI.md de .gestor/gemini, va a .gestor/archivo/canal-gemini-antiguo. GEMINI.md y .gestor/memoria-compartida.md van a .gestor/archivo/sistema-anterior. También van allí, a su subcarpeta agy, las reglas de proyecto que agy carga por su cuenta: .agents/AGENTS.md, .agents/GEMINI.md y los .md de .agents/rules y de .agent/rules, su ruta antigua. Si el proyecto no tenía todavía su archivo propio, su texto pasa antes al bloque pendiente de consolidar, igual que el de GEMINI.md. Las habilidades de .agents/skills no se tocan. Las instrucciones prohíben a los agentes leer o seguir lo archivado, y el agente que consolida el bloque pendiente, después de rescatar lo que siga vivo, borra esas dos carpetas del archivo para que no quede lastre. La aplicación, por su parte, nunca borra nada. No se borra nada: si en el destino ya hay un archivo con ese nombre, al nuevo se le añade la fecha. Cada movimiento queda en el informe, porque alguno puede guardar una tarea a medias.

Luego se crea la estructura de .gestor: el archivo de estado del canal, estado.md; el registro de órdenes, ordenes-del-director.md; una bandeja del canal por agente, como claude.md, claude2.md, codex.md o gemini.md; y la carpeta propia de cada instancia, con su tarea en curso, su acta de verificaciones y su registro de funciones. Solo se crea lo que falta, y no se sobrescribe nada de lo que ya existe.

A continuación, si falta, se crea el archivo propio del proyecto. Si el proyecto tenía texto propio en sus archivos anteriores, el archivo nace con ese texto dentro de un bloque pendiente de consolidar, sin los párrafos que se repetían entre los tres archivos, y con la lista de lo que se ha llevado al archivo. Si el archivo propio ya existía, no se toca, y el informe avisa de lo archivado.

Después se escriben enteros los archivos que son de la aplicación: AGENTS.md, CLAUDE.md, los documentos de consulta de .gestor/consulta y los dos guiones del vigilante del canal, en .gestor/vigilante. Si alguno tenía texto que no había escrito la aplicación, antes se guarda una copia exacta en .gestor/respaldos.

Por último se avisa de cada bandeja del canal que pasa de 16 KiB, el tamaño que corresponde a las unas ochenta líneas vivas que piden las instrucciones, para que su dueño pase lo resuelto a su histórico. Ese límite se cambia en datos/ajustes.json, con la clave LimiteDeBandejaBytes. La guía de Programator y los históricos del canal no cuentan como bandejas.

También se revisa el estado del canal, .gestor/canal/estado.md, y se avisa de lo que no cuadra: que le falte alguno de los apartados Papeles, Reservas activas o Mutaciones vivas; que haya mutaciones vivas, es decir, código cambiado a propósito para una prueba y todavía sin restaurar; que una reserva no siga el formato de la plantilla; que dos agentes tengan reservada la misma ruta, o uno una carpeta que contiene la ruta del otro; y que una reserva sea de un agente cuyo último latido es más antiguo que LimiteDeLatidoHoras, 12 horas por defecto, porque puede que ya no esté trabajando. Una reserva se escribe en una línea, con el agente, la fecha y la hora, las rutas separadas por comas y, tras una raya, el motivo; una carpeta termina en barra. La revisión es solo de la forma: no intenta entender lo que dicen los mensajes.

Al terminar, la barra de estado resume lo que ha cambiado. Si algo requiere atención, como un respaldo, un aviso o un error, se abre además un diálogo con el informe completo. Un paso que falla se anota en el informe y no impide los siguientes, y una segunda apertura seguida no cambia nada.


Archivo único de instrucciones

Todos los agentes leen las mismas reglas de un solo archivo, AGENTS.md, que la aplicación escribe entero cada vez que abre un agente. Lleva la cabecera de identidad común, el núcleo común y, al final, las normas de la carpeta que contiene los proyectos. No tiene zona propia: lo que se escriba a mano en él se pierde en la siguiente apertura, aunque antes se guarda una copia exacta en .gestor/respaldos y el informe lo avisa. Codex y Gemini leen AGENTS.md directamente. Claude Code lo carga a través de CLAUDE.md, que es solo un puntero con dos líneas de importación, @AGENTS.md y @.gestor/instrucciones-del-proyecto.md. GEMINI.md ya no se genera.

Lo propio de cada proyecto vive en .gestor/instrucciones-del-proyecto.md: sus reglas, sus órdenes particulares, sus prohibiciones y los detalles de su entorno. La aplicación lo crea una sola vez, si falta, y no lo vuelve a tocar nunca. Lo leen todos los agentes al empezar cada sesión.

Lo que solo se consulta en momentos concretos está fuera del núcleo, en tres documentos de consulta que la aplicación reescribe en cada apertura: .gestor/consulta/codigo.md, con los estándares de código; .gestor/consulta/verificacion.md, con el detalle de cómo se verifica un cambio; y .gestor/consulta/metodo.md, con el bloque pendiente, el control del proyecto, la documentación y la publicación. El núcleo dice en una tabla cuándo se lee cada uno. Así AGENTS.md mide unos 14 KB con las normas de la carpeta proyecto, muy por debajo de los 24.000 bytes que lee Gemini.

Los textos que leen los agentes, como AGENTS.md, CLAUDE.md, los documentos de consulta y los archivos iniciales de .gestor, están escritos para un agente y no para una persona: pasos numerados, listas cortas, tablas y las órdenes exactas que hay que ejecutar, con formato markdown. La documentación para el usuario, como esta, sigue en prosa.

Cuando un proyecto viene del sistema anterior, su texto propio pasa al archivo propio del proyecto dentro de un bloque pendiente de consolidar. Se descarta solo lo que se reconoce con seguridad como escrito por la aplicación, que son sus regiones, sus plantillas completas y los textos que dejaba fuera de ellas, y el bloque de la unidad de trabajo, que ya llega con las normas comunes. Si un archivo tenía las marcas dañadas, su texto pasa entero. Las instrucciones obligan al primer agente que arranque a consolidar ese bloque antes de cualquier otro trabajo, sin que nadie se lo pida: mira si algún archivo archivado guarda una tarea a medias y la traslada, deja lo que siga vigente como reglas del proyecto, pregunta al usuario lo que contradiga el núcleo y borra el bloque. La copia exacta de los CLAUDE.md y AGENTS.md anteriores queda en .gestor/respaldos, y GEMINI.md en .gestor/archivo/sistema-anterior.

La aplicación guarda una huella SHA-256 de cada archivo que escribe, para distinguir un archivo suyo sin cambios de uno editado a mano, que se respalda antes de reescribirlo. Se respetan la marca BOM y los finales de línea originales, y la escritura es atómica.


Identidad de cada agente

Al abrir un agente, la aplicación fija la variable de entorno GESTOR_AGENTE con su identidad, como Claude 1, Codex 2 o Gemini 1. La cabecera de AGENTS.md, común a todos, es un arranque obligatorio en nueve pasos: el agente lee su identidad, se presenta con una frase fija, lee las instrucciones del proyecto, el estado y todas las bandejas, consolida el bloque pendiente si lo hay, averigua su papel, atiende los encargos que tenga, arma el vigilante del canal y termina con un resumen para el usuario. En Claude Code se fija también CLAUDE_CONFIG_DIR, y en Codex CODEX_HOME, para que cada perfil use su propia carpeta de configuración.


Arranque automático

Para que el agente haga ese arranque sin esperar a que el usuario escriba, la aplicación le manda un primer mensaje al abrirlo. Claude Code y codex lo reciben como primer mensaje de su sesión interactiva, después de los argumentos del perfil, y agy con su opción -i, que abre la sesión con un primer mensaje y la deja abierta. Programator no lo recibe. El mensaje es una sola línea sin los caracteres que cmd.exe interpreta, para que llegue intacto también al codex.cmd que instala npm.


Papeles: Jefe de proyecto y ayudantes

El estado del proyecto, .gestor/canal/estado.md, tiene un apartado Papeles. Si un agente no encuentra allí su identidad, pregunta al usuario si es el Jefe de proyecto o un ayudante, espera la respuesta y la anota. Solo hay un Jefe. En la siguiente sesión el agente encuentra su papel anotado y no vuelve a preguntar. Si un proyecto ya tenía un estado.md sin ese apartado, el agente lo añade sin borrar nada.


Vigilante del canal

La aplicación escribe en .gestor/vigilante dos guiones equivalentes, vigilar-canal.sh para bash y vigilar-canal.ps1 para Windows PowerShell. Toman la identidad de GESTOR_AGENTE, vigilan los archivos de .gestor/canal salvo la bandeja del propio agente y escriben una línea por cada archivo que cambia. Sin opciones no terminan nunca, y es como los usa Claude Code, con su herramienta de vigilancia en segundo plano. Con --una-vez, o -UnaVez en PowerShell, comparan con la pasada anterior, que guardan en la carpeta propia del agente, dicen qué ha cambiado o que no hay cambios, y terminan. Es como los usa agy, que no tiene aviso automático, al terminar cada paso. Codex, en cambio, se monta él mismo al arrancar su propio sistema de vigilancia del canal, que lo avisa de los mensajes nuevos aunque esté esperando; sus instrucciones le dicen el objetivo y él decide cómo, y si no puede, usa la pasada única. Para saber qué es nuevo, los guiones guardan además una copia de cada archivo en esa misma carpeta. Cuando una línea nueva de una bandeja dice «Para» seguido de la identidad del agente, como Para Claude 2, la línea del aviso termina en [ENCARGO PARA Claude 2]; una mención en un «con copia a» no cuenta, y Para Claude 22 no es Para Claude 2. Cuando cambia estado.md y su apartado de mutaciones vivas no dice Ninguna, termina en [MUTACION VIVA]. La guía de Programator y los históricos del canal no se vigilan.


Detección de agentes y de cuentas

Al arrancar, en un hilo de trabajo que no bloquea la ventana, la aplicación busca el programa de cada perfil con la misma búsqueda que usa al abrirlo, de modo que un botón visible siempre se puede abrir. En Windows, codex se busca primero en %LOCALAPPDATA%\Programs\OpenAI\Codex\bin y agy en %LOCALAPPDATA%\agy\bin, donde los dejan sus instaladores oficiales. Después, todos se buscan en ~/.local/bin, en Linux luego en /usr/local/bin y /usr/bin, y por último en el PATH. Programator no se busca: solo se encuentra si su perfil lleva la ruta de su ejecutable. Si un perfil lleva una ruta fijada, solo vale esa, y si no existe el agente queda como no encontrado.

También reconoce las cuentas instaladas: ~/.claude y ~/.claudeN con .credentials.json, y ~/.codex y ~/.codexN con auth.json. Cada cuenta que no tenga perfil se añade a perfiles-agentes.json, después de los de su herramienta; los perfiles guardados no se cambian ni se borran. Un perfil de Claude o de Codex a partir de la instancia 2 solo tiene botón si existe su carpeta de cuenta.

La pestaña Agentes muestra el estado de cada perfil y permite cambiar la ruta de su programa, con Examinar y con Búsqueda automática. Las rutas que no existen se rechazan sin guardar nada.


Terminal

Todos los agentes se abren en una ventana de terminal situada en la carpeta del proyecto y titulada con la identidad y el nombre del proyecto, con su programa: claude, codex, agy o Programator, y los tres primeros con su mensaje inicial. En Windows es PowerShell, con un temporizador que restaura el título cuando el agente lo cambia. En Linux se usa el primer emulador disponible entre gnome-terminal, konsole, xfce4-terminal y xterm. No se abren las aplicaciones de escritorio de Codex ni de Antigravity, que tienen sus propios accesos directos. Si un programa deja de estar donde se encontró, la aplicación lo avisa al abrir, y los archivos del proyecto quedan preparados igualmente.


Normas comunes a todos los proyectos

Las normas que deben cumplir todos los proyectos se escriben una sola vez en un CLAUDE.md, AGENTS.md o GEMINI.md de la carpeta que los contiene. Cada vez que abre un agente, la aplicación sube desde la carpeta del proyecto hasta encontrar el primero de esos archivos que no lleve marcas propias de la aplicación, y copia su contenido al final de AGENTS.md, precedido de un encabezado que dice de qué archivo sale. Los que llevan esas marcas son de otro proyecto gestionado y se saltan; las marcas con otros identificadores, como un bloque delimitado a mano, no cuentan. Las marcas que pudiera contener el texto copiado se desactivan. Si no encuentra ningún archivo, el bloque lo dice. Los cambios en ese archivo llegan a cada proyecto la siguiente vez que se abre.

Se copia en vez de confiar en que el agente lo lea porque Codex y Gemini no leen nada por encima de la raíz del repositorio. Claude Code, en cambio, sí carga por su cuenta el CLAUDE.md de las carpetas superiores, así que recibe esas normas dos veces, la original y la copia de AGENTS.md. Si las dos difieren, manda la original, y la copia se pone al día en la siguiente apertura.


Pestaña Instrucciones de los agentes

Muestra el núcleo común en modo de solo lectura. El núcleo va dentro del ejecutable y cambia con cada versión de la aplicación, así que no se edita desde aquí. Lo que aprenda un proyecto se escribe en su .gestor/instrucciones-del-proyecto.md.


Configuración

La aplicación es portable: la configuración vive en la carpeta datos, junto al ejecutable. config.json guarda la lista de proyectos, y perfiles-agentes.json los perfiles. ajustes.json guarda los ajustes de funcionamiento. TiempoMaximoGitSegundos son los segundos que se espera a una orden de git antes de pararla, 30 por defecto. LimiteDeBandejaBytes es el tamaño a partir del cual se avisa de que una bandeja del canal está sin podar, 16.384 por defecto. ComprobarActualizaciones dice si se consulta al arrancar si hay una versión nueva, true por defecto. RepositorioDeActualizaciones es el repositorio de GitHub donde se busca, celtidcs/gestor-agentes por defecto. TiempoMaximoActualizacionSegundos son los segundos que se espera a GitHub, 5 por defecto. LimiteDeLatidoHoras son las horas sin latido a partir de las cuales se avisa de que una reserva puede estar abandonada, 12 por defecto. Si el archivo falta, la aplicación lo crea con esos valores; una clave que falte toma su valor por defecto, y un cero o un repositorio que no tenga la forma propietario/nombre se rechazan al arrancar con un error que nombra la clave.


Versión y aviso de actualización

La versión de la aplicación se ve en la esquina superior derecha de la ventana. Al arrancar, en un hilo aparte que no retrasa nada, la aplicación pide a la API de GitHub la última versión publicada en su repositorio. Si es más nueva que la instalada, muestra el aviso Actualización disponible, con las dos versiones y la dirección de la página donde se descarga. Solo acepta direcciones de la web de GitHub dentro del propio repositorio. No descarga nada ni cambia nada por su cuenta. Si no hay red, GitHub no responde a tiempo o la respuesta no se entiende, no muestra nada. La petición no lleva ningún dato del usuario: solo el nombre y la versión del programa. Con ComprobarActualizaciones en false no se hace ninguna petición.


Estado del equipo en la ventana

Al seleccionar un proyecto, la pestaña Proyectos muestra encima de los botones una línea con el estado de su equipo: quién es el Jefe de proyecto, cuánto hace del último latido de cada agente, cuántas reservas hay y, si las hay, una alerta de mutaciones vivas o de autoexclusiones vigentes. Se lee de .gestor/canal al seleccionar el proyecto y cada vez que termina un trabajo, como abrir un agente, no en cada fotograma. Si el proyecto todavía no tiene estado del canal, no se muestra nada.


Reglas aprendidas

Cuando un trabajo se rechaza al verificarlo, o un agente tiene que parar por la regla de los dos intentos, y la causa es una trampa del proyecto o de su entorno que se puede reproducir, las instrucciones piden convertirla en una regla de una línea en el apartado Reglas aprendidas del archivo propio del proyecto, con su fecha, su autor y su origen, para que ningún otro agente vuelva a caer. Antes se mira si ya está, y si el apartado pasa de treinta reglas, el Jefe las agrupa. Los proyectos nuevos reciben ese apartado vacío.


Programator en los proyectos

La aplicación escribe en cada proyecto .gestor/consulta/programator.md, una guía corta para los agentes: qué es Programator, cómo se le hace un encargo en el canal, qué sabe hacer y qué no, dónde deja sus propuestas y que quien le encarga algo verifica el resultado. AGENTS.md manda leerla antes de encargarle nada. Programator publica además, al arrancar, su propia guía en el canal con los datos de su instalación, y la de la aplicación remite a ella. Esa guía y los históricos del canal no cuentan como bandejas: no reciben el aviso de tamaño y el vigilante no avisa de sus cambios.

La primera vez que se abre, si la carpeta datos no tiene todavía config.json ni perfiles-agentes.json, copia la configuración de Gestor Claude, la versión anterior, que estaba en %APPDATA%\GestorClaude en Windows y en ~/.config/GestorClaude en Linux. Si existe un claude-instructions.txt de versiones anteriores, solo se lee, para reconocer como plantilla antigua el contenido de los proyectos que lo usaban.
