Manual de uso de Gestor Agentes

Este manual está pensado para quien abre Gestor Agentes por primera vez y no sabe todavía para qué sirve. No hace falta saber nada del programa. Se lee entero en unos veinte minutos, y después basta con volver al apartado que haga falta. Si algo falla, el documento Resolución de problemas, que acompaña a este, explica qué hacer en cada caso.


Qué es Gestor Agentes

Gestor Agentes es una pequeña aplicación de escritorio para Windows y Linux que guarda la lista de tus proyectos de programación y los abre con un agente de inteligencia artificial. Los agentes con los que trabaja son cuatro: Claude Code, de Anthropic; Codex, de OpenAI; Gemini, a través de la herramienta agy de Google Antigravity; y Programator. Todos se abren en una ventana de terminal, es decir, en una ventana de texto donde escribes al agente y él te contesta.

Sin esta aplicación, cada vez que quieres trabajar en un proyecto tienes que abrir una terminal, ir hasta la carpeta del proyecto y arrancar allí el agente. Si además tienes dos cuentas del mismo agente, tienes que acordarte de decirle cuál usar. Gestor Agentes lo reduce a dos clics: eliges el proyecto en la lista y pulsas el botón del agente.

Hace además algo que no se ve a simple vista. Cada vez que abres un proyecto, deja dentro de su carpeta las instrucciones de trabajo de los agentes, las mismas para todos, y un espacio donde se dejan mensajes entre ellos. Gracias a eso, varios agentes pueden trabajar a la vez en el mismo proyecto sin pisarse, revisar cada uno el trabajo de los otros y retomar una tarea que otro dejó a medias.

Antes se llamaba Gestor Claude. Si usabas aquella versión, todo lo que tenías se aprovecha, como se explica más abajo.


Qué necesitas en el ordenador

El programa funciona en Windows 10 u 11 de 64 bits y en Linux de 64 bits con escritorio gráfico. No hay que instalarlo: es una carpeta que se copia donde quieras y se usa tal cual. Tampoco necesita ningún otro componente, porque el ejecutable lleva dentro todo lo que le hace falta. Lo único que pide al ordenador es OpenGL 2.0 o superior para dibujar su ventana, que proporciona el controlador de la tarjeta gráfica. Algunas máquinas virtuales no lo tienen, y en ellas el programa avisa de que falta OpenGL y no se abre. En Windows se arregla instalando el complemento gratuito de Microsoft OpenCL, OpenGL, and Vulkan Compatibility Pack, como explica el documento Resolución de problemas.

Lo que sí necesitas es al menos uno de los agentes, porque son programas aparte y Gestor Agentes no los incluye. También conviene tener git, el programa de control de versiones, porque la aplicación lo usa para crear el repositorio de los proyectos que no lo tienen. Sin git todo lo demás funciona, y la aplicación te avisa.


Cómo instalar cada agente

Cada agente se instala con el método oficial de su fabricante. Las órdenes de este apartado se escriben en una ventana de PowerShell en Windows, o en una terminal en Linux, y están copiadas de la documentación oficial que se cita en cada caso. Después de instalar cualquier agente, cierra Gestor Agentes si lo tenías abierto y vuelve a abrirlo. Es necesario porque un programa que ya estaba en marcha no se entera de los cambios que hace el instalador, y sin reiniciarlo el agente seguiría apareciendo como no encontrado.

Claude Code. En Windows, abre PowerShell y escribe esta orden:

irm https://claude.ai/install.ps1 | iex

En Linux, en una terminal:

curl -fsSL https://claude.ai/install.sh | bash

El programa queda en la carpeta .local\bin de tu carpeta de usuario, que es justo donde Gestor Agentes lo busca primero. La primera vez que lo abras te pedirá que inicies sesión con tu cuenta. Fuente: https://code.claude.com/docs/en/setup

Codex. Necesitas la versión de terminal de Codex. La aplicación de escritorio de Codex que se descarga de la Microsoft Store no sirve para esto: lleva dentro un programa llamado codex.exe, pero no funciona por su cuenta en una terminal. En Windows, abre PowerShell y escribe esta orden, que es el instalador oficial de OpenAI:

powershell -ExecutionPolicy ByPass -c "irm https://chatgpt.com/codex/install.ps1 | iex"

Deja el programa en la carpeta AppData\Local\Programs\OpenAI\Codex\bin de tu carpeta de usuario y añade esa carpeta a las rutas de búsqueda del sistema, lo que se conoce como PATH. En Linux, OpenAI ofrece la instalación con npm, que necesita tener Node.js instalado:

npm install -g @openai/codex

También puedes usar npm en Windows, pero entonces no queda un codex.exe sino un archivo intermedio llamado codex.cmd. Funciona igual, con una limitación pequeña que explica el documento Resolución de problemas, así que en Windows es preferible el instalador oficial. Fuentes: https://github.com/openai/codex y https://releases.openai.com/codex/install.ps1

Gemini, con agy de Google Antigravity. En Windows, en PowerShell:

irm https://antigravity.google/cli/install.ps1 | iex

En Linux, en una terminal:

curl -fsSL https://antigravity.google/cli/install.sh | bash

En Windows queda en la carpeta AppData\Local\agy\bin de tu carpeta de usuario, y en Linux en .local/bin. La primera vez que lo abras te preguntará por los colores, por el modo de dibujar la pantalla y si confías en la carpeta del proyecto. Fuente: https://antigravity.google/docs/getting-started

Programator. No tiene instalador ni se busca solo. Copia su carpeta donde quieras y, en la pestaña Agentes de Gestor Agentes, escribe la ruta de su ejecutable, programator.exe en Windows.

Para saber si un agente ya está instalado, abre una ventana de PowerShell o una terminal nueva y escribe su nombre, claude, codex o agy, seguido de un espacio y --version, por ejemplo claude --version. Si contesta con un número de versión, está instalado. Si el sistema dice que no reconoce la orden, no lo está, o está en una carpeta que el sistema no conoce.


Arrancar el programa

En Windows, haz doble clic en gestor-agentes.exe, dentro de la carpeta del programa. La primera vez, Windows puede mostrar un aviso azul que dice que ha protegido tu equipo. Sale porque el programa no lleva firma digital, no porque tenga nada malo. Pulsa en Más información y después en Ejecutar de todas formas. Windows recordará tu decisión.

En Linux, abre una terminal en la carpeta del programa y escribe ./gestor-agentes. Si dice que no tiene permiso, dáselo una sola vez con la orden chmod +x gestor-agentes.

La primera vez que lo abras, si su carpeta datos todavía no tiene tu lista de proyectos ni tus perfiles y en ese ordenador habías usado Gestor Claude, la aplicación copia tu lista de proyectos y tus perfiles de agente a su carpeta datos. Te lo cuenta en una ventana titulada Configuración importada, que dice de dónde ha copiado, adónde y qué archivos. La configuración antigua no se borra ni se modifica.

Mientras arranca, la barra inferior dice Buscando agentes. En ese momento la aplicación mira qué agentes tienes instalados, lo que tarda uno o dos segundos.


La ventana

La ventana tiene tres pestañas en la parte de arriba, en este orden: Proyectos, Agentes e Instrucciones de los agentes. Abajo del todo está la barra de estado, una línea de texto donde la aplicación te cuenta lo que acaba de hacer: cuántos proyectos tienes, qué agente acaba de abrir o qué ha ido mal.

En la pestaña Proyectos pasarás casi todo el tiempo. En la pestaña Agentes ves qué agentes ha encontrado la aplicación y puedes decirle dónde están los que no encuentra. La pestaña Instrucciones de los agentes solo sirve para leer las normas que la aplicación da a los agentes, y al principio puedes ignorarla.


La pestaña Proyectos

Muestra tus proyectos en una lista con cuatro columnas: Nombre, Ruta, Última apertura y Estado. El nombre es el de la carpeta del proyecto, la ruta dice dónde está, la última apertura dice cuándo lo abriste por última vez desde la aplicación, y el estado dice cómo están sus instrucciones.

El estado puede ser uno de estos tres. Sin configurar quiere decir que el proyecto no tiene todavía archivos de instrucciones. Formato antiguo quiere decir que tiene instrucciones que no escribió esta versión, por ejemplo de una versión anterior o escritas a mano. Configurado quiere decir que está al día. No tienes que hacer nada con ninguno de ellos, porque los dos primeros se arreglan solos la primera vez que abres el proyecto.

Debajo de la lista aparece un botón por cada agente que la aplicación ha encontrado en tu ordenador, con un texto como Abrir · Claude 1 o Abrir · Codex 1.


Añadir un proyecto

Pulsa el botón Agregar Proyecto. Se abre la ventana normal de tu sistema para elegir una carpeta. Elige la carpeta principal del proyecto, la que contiene todo lo demás, y no una subcarpeta. Es la decisión más importante que vas a tomar en la aplicación, porque esa carpeta es la que el agente tomará como su lugar de trabajo y la que recibirá los archivos de instrucciones.

El proyecto aparece en la lista; si la carpeta no tenía archivos de instrucciones, con el estado Sin configurar. Si eliges una carpeta que ya estaba en la lista, la aplicación no la repite y te dice que este proyecto ya está en la lista.


Abrir un proyecto con un agente

Haz un clic sobre el proyecto para seleccionarlo y pulsa el botón del agente con el que quieras trabajar. El doble clic sobre un proyecto no hace nada, y es a propósito: así no se abre ningún proyecto sin querer con el agente equivocado.

Al pulsar el botón, la barra de estado dice que está preparando el proyecto para ese agente. Durante esa preparación, que dura muy poco, la aplicación revisa la carpeta y crea o pone al día las instrucciones de los agentes, el espacio donde se dejan mensajes y el repositorio git. Después abre una ventana de terminal titulada con el agente y el proyecto, por ejemplo Claude 1 · Gestor Claude, ya situada en la carpeta del proyecto y con el agente en marcha. La barra de estado lo confirma con un texto como Claude 1 abierto en Gestor Claude.

Si durante la preparación ha pasado algo que conviene que sepas, antes de abrir el agente la aplicación te enseña una ventana titulada Preparación seguida del nombre del proyecto. Tiene hasta tres apartados: Errores, Avisos y Cambios. Léela con calma y pulsa Aceptar. El apartado Avisos y qué hacer con cada aviso se explican más abajo.

Lo primero que hace el agente al arrancar es presentarse con una frase como esta: He leído AGENTS.md y las instrucciones del proyecto. Soy Claude 1. Canal Claude 1 disponible y listo. Si no se presenta, algo ha ido mal con sus instrucciones; díselo y te explicará qué ha leído.

La primera vez que un agente abre una carpeta suele preguntar si confías en ella. Contesta antes de escribirle nada. Con Codex, la primera vez puede aparecer además el aviso No active thread is available. Es un mensaje del propio Codex, no de Gestor Agentes. Codex funciona igual y el aviso no vuelve a salir las siguientes veces.

Puedes cerrar Gestor Agentes cuando quieras: las ventanas de los agentes que hayas abierto siguen funcionando.


Qué deja la aplicación dentro de cada proyecto

En la carpeta principal del proyecto aparecen dos archivos. AGENTS.md es el archivo de instrucciones que leen todos los agentes. Contiene las normas comunes de trabajo, que explican cómo se verifica el código, cómo se documenta, cómo se reparten las tareas los agentes y qué no se puede hacer sin tu permiso. Lleva también una cabecera que le dice a cada agente quién es y, al final, las normas de tu equipo, de las que se habla más abajo. CLAUDE.md es un archivo de dos líneas que solo sirve para que Claude Code cargue AGENTS.md.

Esos dos archivos son de la aplicación, que los escribe enteros cada vez que abres el proyecto. No merece la pena editarlos a mano, porque lo que escribas se sustituirá la siguiente vez. No se pierde, porque antes se guarda una copia, pero no se queda en su sitio.

Lo propio de cada proyecto, como sus reglas, tus órdenes particulares para él o cómo se compila y se prueba, va en el archivo instrucciones-del-proyecto.md, dentro de la carpeta .gestor del proyecto. La aplicación lo crea la primera vez y no lo vuelve a tocar nunca. Los agentes lo leen al empezar.

La carpeta .gestor es el espacio de trabajo de los agentes. Como su nombre empieza con un punto, puede que no la veas si tu sistema oculta ese tipo de carpetas. Dentro está el estado del proyecto, con el reparto de tareas entre agentes; la carpeta canal, con un buzón para cada agente, que en este programa se llama bandeja; la carpeta propia de cada agente, con su diario de trabajo y sus actas de verificación; el registro de tus órdenes; y el archivo propio del proyecto del que se acaba de hablar. Esa carpeta no se guarda en el repositorio git.

Si el proyecto ya tenía instrucciones de una versión anterior, la aplicación no las pierde. Guarda una copia exacta de los archivos antiguos en la carpeta .gestor\respaldos, aparta los que ya no se usan a la carpeta .gestor\archivo y copia lo que era propio del proyecto en instrucciones-del-proyecto.md, dentro de un bloque pendiente de revisar. El primer agente que arranque en el proyecto revisa ese bloque por su cuenta antes de ponerse con otra cosa: conserva lo que siga valiendo, te pregunta lo que le plantee dudas y borra el bloque.


Trabajar con varios agentes a la vez

Puedes abrir el mismo proyecto con varios agentes, cada uno desde su botón. Cada uno trabaja en su propia ventana y se comunica con los demás dejando mensajes en su bandeja de la carpeta .gestor\canal, que los demás leen. No tienes que decirles quién hace qué: se reparten el trabajo entre ellos y lo dejan escrito en el estado del proyecto.

Ningún agente da por bueno su propio trabajo. Cuando uno termina algo, otro lo comprueba ejecutando las pruebas y deja constancia por escrito. Si solo trabaja un agente, la comprobación la hace otro agente al que se llame para ello, o tú mismo.

Si un agente agota su límite de uso, deja escrito en qué punto estaba. Abres el mismo proyecto con otro botón y el nuevo agente continúa desde ahí.

Cuando quieras dar una orden que valga para todos, dísela a cualquiera de ellos. La anotará en el registro de órdenes y se la pasará a los demás.


Quitar un proyecto de la lista

Cada fila de la lista tiene un botón Quitar. Al pulsarlo, el proyecto desaparece de la lista y nada más: su carpeta sigue intacta en el disco, y puedes volver a añadirla cuando quieras.


Eliminar un proyecto del disco

El botón Eliminar del disco, en la parte de abajo, borra de verdad la carpeta del proyecto seleccionado con todo lo que tenga dentro. Los archivos no pasan por la papelera, así que no se pueden recuperar. Antes de borrar, la aplicación abre una ventana titulada Eliminar proyecto del disco y te pide que confirmes con Sí. Úsalo solo si quieres destruir el proyecto. Si solo quieres dejar de verlo en la lista, el botón que buscas es Quitar. Si la carpeta ya no existía, la aplicación se limita a quitarla de la lista.


La pestaña Agentes

Esta pestaña tiene una fila por cada agente, con tres columnas: Agente, Estado y Ruta del ejecutable. El estado puede decir Encontrado, seguido de la ruta donde está el programa; No encontrado, si la aplicación no lo ha encontrado; o Falta la carpeta de su cuenta, si es una segunda cuenta cuya carpeta todavía no existe, como se explica en el apartado siguiente.

Solo los agentes encontrados tienen botón en la pestaña Proyectos. Si un agente sale como no encontrado y sabes que está instalado, puede ser por dos motivos. El primero es que lo instalaste con Gestor Agentes abierto: ciérralo y vuelve a abrirlo. El segundo es que está en una carpeta donde la aplicación no mira: escribe su ruta en la columna Ruta del ejecutable, o búscala con el botón Examinar, y pulsa Guardar rutas. Si la ruta que escribes no existe, la aplicación te lo dice en una ventana titulada Rutas no válidas y no guarda nada.

El botón Búsqueda automática borra la ruta que hubieras escrito, para que la aplicación vuelva a buscar el programa por su cuenta. El botón Buscar de nuevo repite la búsqueda de todos los agentes.

La aplicación busca cada programa por su cuenta en estos sitios, por este orden. En Windows, codex se busca primero en AppData\Local\Programs\OpenAI\Codex\bin y agy en AppData\Local\agy\bin, que es donde los dejan sus instaladores oficiales. Después, todos se buscan en la carpeta .local\bin de tu carpeta de usuario, donde lo dejan los instaladores de Claude Code y, en Linux, el de agy. En Linux se mira luego en /usr/local/bin y /usr/bin. Por último, en todas las carpetas del PATH del sistema. Programator nunca se busca solo: hay que escribir su ruta.

Si escribes la ruta de un agente, la aplicación usa esa y ninguna otra. Si un día esa ruta deja de existir, el agente aparecerá como no encontrado, para que te des cuenta, en lugar de abrir otro programa que encuentre por casualidad.


Dos cuentas del mismo agente

Si tienes dos cuentas de Claude Code, por ejemplo una personal y otra del trabajo, la aplicación muestra un botón para cada una: Claude 1 y Claude 2. Cada cuenta guarda su sesión en su propia carpeta dentro de tu carpeta de usuario. La primera usa la carpeta .claude, la que Claude Code usa siempre, y la segunda usa .claude2. Con Codex pasa lo mismo, con las carpetas .codex y .codex2. La aplicación reconoce como cuenta cualquier carpeta de esas que tenga una sesión iniciada dentro, y añade su botón ella sola.

Para crear la segunda cuenta de Claude Code, abre PowerShell y escribe esta orden, que arranca Claude Code usando la carpeta .claude2:

$env:CLAUDE_CONFIG_DIR = "$HOME\.claude2"; claude

Inicia sesión con la otra cuenta y cierra la ventana. Al volver a abrir Gestor Agentes aparecerá el botón Claude 2. Para una segunda cuenta de Codex se hace igual, cambiando la orden por esta:

$env:CODEX_HOME = "$HOME\.codex2"; codex

Gestor Agentes no inicia sesión ni guarda contraseñas. Solo reconoce las carpetas de cuenta que ya tienes y abre cada agente con la suya. Gemini y Programator no tienen cuentas separadas.


Los avisos y qué hacer con ellos

La ventana de preparación de un proyecto puede traer alguno de estos avisos. Ninguno impide trabajar.

Si habla de una copia exacta o de un archivo respaldado, es que AGENTS.md o CLAUDE.md tenían texto que no había escrito la aplicación. No se ha perdido nada: el original está en la carpeta .gestor\respaldos del proyecto. Lo que quieras que lean los agentes, escríbelo en instrucciones-del-proyecto.md.

Si habla de archivos llevados al archivo, es que el proyecto tenía archivos del sistema de trabajo anterior, que ahora están en .gestor\archivo. El agente que revise las instrucciones mira si alguno guardaba una tarea a medias.

Si dice que una bandeja mide más de lo debido, por ejemplo .gestor/canal/claude.md mide 62066 bytes, más de los 16384 de una bandeja podada, es que el buzón de ese agente ha crecido sin limpiarse. Díselo a ese agente para que pase lo ya resuelto a su histórico. El aviso sale con cualquier agente que abras, porque es del proyecto y no del agente, y deja de salir en cuanto la bandeja vuelve a su tamaño.

Si dice que las marcas estaban dañadas, un archivo de instrucciones antiguo estaba mal formado. La aplicación ha guardado una copia y ha pasado su texto entero al bloque pendiente de revisar, para que el agente quite lo que sobre.

Si dice que no se ha podido comprobar git, git no está instalado o no responde. El proyecto se ha preparado igualmente, pero sin repositorio. Instala git y vuelve a abrir el proyecto.

Hay otro aviso que no sale en esa ventana, sino en la barra de estado, al abrir Codex o Gemini. Empieza por la palabra AVISO y dice que AGENTS.md mide más de lo que lee el agente: Codex lee los primeros 32768 bytes y Gemini los primeros 24000. Significa que ese agente no verá el final de sus instrucciones. Normalmente no pasa, porque la aplicación escribe AGENTS.md por debajo de esos tamaños; si sale, suele ser porque las normas de tu equipo, de las que se habla a continuación, son muy largas.

Si al pulsar un botón el agente no se abre, la aplicación lo explica en una ventana. Si dice que el agente no está instalado, mira la pestaña Agentes. Si dice que el perfil está incompleto, falta algún dato del perfil, normalmente la ruta de Programator, y la ventana dice en qué archivo se arregla. En los dos casos, los archivos del proyecto ya se han preparado.


Las normas de tu equipo

Hay normas que valen para todos tus proyectos, como las máquinas donde se hacen las pruebas o las carpetas donde los agentes no deben escribir. No hace falta repetirlas en cada proyecto. Escríbelas una vez en un archivo CLAUDE.md dentro de la carpeta que contiene todos tus proyectos.

Cada vez que abres un proyecto, la aplicación sube desde su carpeta hasta encontrar ese archivo y copia su contenido al final de AGENTS.md, en el bloque de entorno, con una línea que dice de qué archivo sale. Si cambias el archivo, el cambio llega a cada proyecto la próxima vez que lo abras. Si la aplicación no encuentra ninguno, el bloque lo dice.


La pestaña Instrucciones de los agentes

Muestra las normas comunes que la aplicación escribe en todos los proyectos. Solo se pueden leer, porque van dentro del programa y cambian con cada versión. Lo que quieras añadir para un proyecto concreto se escribe en su instrucciones-del-proyecto.md, y lo que valga para todos tus proyectos, en el CLAUDE.md de la carpeta que los contiene.


Dónde guarda la aplicación tu configuración

Todo está en la carpeta datos, junto al ejecutable. Son cuatro archivos, todos de texto. config.json guarda la lista de proyectos. perfiles-agentes.json guarda los perfiles de agente: para cada uno, su tipo, su número, la carpeta de su cuenta y, si se la has puesto, la ruta de su programa. perfiles-agentes.version es un número que la aplicación usa para saber si tiene que poner al día los perfiles. ajustes.json guarda dos ajustes de funcionamiento: TiempoMaximoGitSegundos, los segundos que la aplicación espera a git antes de darlo por perdido, que son 30 si no lo cambias, y LimiteDeBandejaBytes, el tamaño a partir del cual avisa de una bandeja sin podar, que son 16384.

No necesitas tocar ninguno de estos archivos, porque todo se hace desde la ventana. Si los editas a mano y te equivocas, la aplicación te dice al arrancar que la configuración no es válida y no sobrescribe nada, para que puedas corregirlo.


Llevarte el programa a otro ordenador

El programa es portable de verdad: copia su carpeta entera, con la carpeta datos dentro, a una memoria USB o a otro ordenador, y úsala sin instalar nada. La configuración viaja con ella.

Los perfiles de agente funcionan igual en cualquier equipo, salvo las rutas que hayas escrito a mano, que solo valen si el programa está en el mismo sitio. La lista de proyectos guarda rutas de carpetas, así que solo sirve tal cual si los proyectos están en la misma ruta en los dos equipos, por ejemplo en una unidad compartida que se monta con la misma letra. Si no, quita de la lista los que no se puedan abrir y añade los de ese equipo.

Si prefieres empezar de cero en el otro ordenador, copia la carpeta del programa sin el contenido de la carpeta datos.


Desinstalar

No hay nada que desinstalar. Borra la carpeta del programa y listo; con ella se va tu configuración. Tus proyectos no se tocan.


Un día normal de trabajo

Abres Gestor Agentes y ves tu lista. Seleccionas el proyecto y pulsas el botón del agente con el que quieres empezar. Se abre la terminal, el agente se presenta y retoma lo que hubiera pendiente. Si quieres que otro agente revise lo hecho o ayude con otra parte, pulsas su botón y se incorpora al mismo proyecto. Al terminar, los agentes dejan escrito el estado del proyecto y el historial de cambios, de modo que la próxima vez todo sigue ahí, abras con el agente que abras.
