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

Arriba a la izquierda está el nombre de la aplicación, y a la derecha su versión, por ejemplo v3.9.0. Es la versión que tienes instalada, y la que tienes que dar si preguntas por un fallo. La ventana tiene tres pestañas en la parte de arriba, en este orden: Proyectos, Agentes e Instrucciones de los agentes. Abajo del todo está la barra de estado, una línea de texto donde la aplicación te cuenta lo que acaba de hacer: cuántos proyectos tienes, qué agente acaba de abrir o qué ha ido mal.

En la pestaña Proyectos pasarás casi todo el tiempo. En la pestaña Agentes ves qué agentes ha encontrado la aplicación y puedes decirle dónde están los que no encuentra. La pestaña Instrucciones de los agentes solo sirve para leer las normas que la aplicación da a los agentes, y al principio puedes ignorarla.


La pestaña Proyectos

Muestra tus proyectos en una lista con cuatro columnas: Nombre, Ruta, Última apertura y Estado. El nombre es el de la carpeta del proyecto, la ruta dice dónde está, la última apertura dice cuándo lo abriste por última vez desde la aplicación, y el estado dice cómo están sus instrucciones.

El estado puede ser uno de estos tres. Sin configurar quiere decir que el proyecto no tiene todavía archivos de instrucciones. Formato antiguo quiere decir que tiene instrucciones que no escribió esta versión, por ejemplo de una versión anterior o escritas a mano. Configurado quiere decir que está al día. No tienes que hacer nada con ninguno de ellos, porque los dos primeros se arreglan solos la primera vez que abres el proyecto.

Debajo de la lista aparece un botón por cada agente que la aplicación ha encontrado en tu ordenador, con un texto como Abrir · Claude 1 o Abrir · Codex 1. Cuando seleccionas un proyecto, encima de esos botones aparece además una línea con el estado de su equipo de agentes, que se explica en el apartado Cómo va el equipo de un proyecto, y otra para elegir el papel del próximo agente que abras, que se explica en el apartado Elegir el papel al abrir.


Añadir un proyecto

Pulsa el botón Agregar Proyecto. Se abre la ventana normal de tu sistema para elegir una carpeta. Elige la carpeta principal del proyecto, la que contiene todo lo demás, y no una subcarpeta. Es la decisión más importante que vas a tomar en la aplicación, porque esa carpeta es la que el agente tomará como su lugar de trabajo y la que recibirá los archivos de instrucciones.

El proyecto aparece en la lista; si la carpeta no tenía archivos de instrucciones, con el estado Sin configurar. Si eliges una carpeta que ya estaba en la lista, la aplicación no la repite y te dice que este proyecto ya está en la lista.


Abrir un proyecto con un agente

Haz un clic sobre el proyecto para seleccionarlo y pulsa el botón del agente con el que quieras trabajar. El doble clic sobre un proyecto no hace nada, y es a propósito: así no se abre ningún proyecto sin querer con el agente equivocado.

Al pulsar el botón, la barra de estado dice que está preparando el proyecto para ese agente. Durante esa preparación, que dura muy poco, la aplicación revisa la carpeta y crea o pone al día las instrucciones de los agentes, el espacio donde se dejan mensajes y el repositorio git. Después abre una ventana de terminal titulada con el agente y el proyecto, por ejemplo Claude 1 · Mi proyecto, ya situada en la carpeta del proyecto y con el agente en marcha. La barra de estado lo confirma con un texto como Claude 1 abierto en Mi proyecto.

Gemini se abre con agy dentro del proyecto de Antigravity que tiene esa carpeta, para que no trabaje en otro proyecto suyo. Si Antigravity todavía no tiene ningún proyecto con esa carpeta, la aplicación le pide a agy que cree uno nuevo para ella. Solo deja de abrirlo, y te lo dice, si no puede leer ningún proyecto de Antigravity, por ejemplo porque todavía no tienes ninguno: entonces abre una vez la carpeta como proyecto en Antigravity y vuelve a pulsar el botón. Codex se abre con su entorno aislado de escritura limitado al proyecto, con acceso a la red y sin pedir aprobación, así que lee sus instrucciones y el canal y trabaja en el proyecto sin pedirte permiso cada vez, como se explica en el apartado Permisos de los agentes al arrancar; si tu perfil de Codex ya fija el sandbox o la aprobación, manda el perfil.

Si durante la preparación algo ha fallado, o la aplicación ha tenido que guardar una copia de un archivo o llevar al archivo algo del sistema anterior, te enseña una ventana titulada Preparación seguida del nombre del proyecto. Tiene hasta tres apartados: Errores, Avisos y Cambios. Léela con calma y pulsa Aceptar. Los avisos que no van acompañados de nada de eso no abren ninguna ventana, porque los tienen que atender los agentes y no tú: la barra de estado dice cuántos hay, y la aplicación los deja en el archivo .gestor/avisos-de-preparacion.md del proyecto, que cada agente lee al arrancar. Ese archivo desaparece solo cuando ya no queda ningún aviso. Qué significa cada aviso se explica más abajo.

El agente empieza a trabajar solo, sin que le escribas nada. La aplicación le manda al abrirlo un primer mensaje que le pide seguir su arranque, que son nueve pasos fijos y siempre los mismos. Primero averigua quién es y dónde está, porque la aplicación le dice al abrirlo su identidad, en la variable GESTOR_AGENTE, y la carpeta del proyecto, en la variable GESTOR_PROYECTO. Si no está en esa carpeta, para y te avisa con las dos rutas, sin leer ni escribir nada más. Si no tiene identidad, porque lo has abierto por tu cuenta y no desde la aplicación, no hace este arranque ni escribe en la carpeta .gestor, y atiende lo que le pidas. Si todo cuadra, se presenta con una frase como esta: He leído AGENTS.md y las instrucciones del proyecto. Soy Claude 1. Canal Claude 1 disponible y listo. Después lee las instrucciones propias del proyecto, el estado del proyecto, los mensajes de todos los demás agentes y su propio diario, por si dejó algo a medias la vez anterior.

Luego busca su papel en el estado del proyecto. Si lo elegiste en la ventana al abrirlo, ya lo encuentra anotado. Si no, y es la primera vez que ese agente trabaja en el proyecto, no lo encontrará, y te preguntará exactamente esto: ¿Cuál es mi papel en este proyecto: Jefe de proyecto o ayudante? Hasta que contestes no hace nada más. Contesta con una frase normal, por ejemplo eres el Jefe, o eres ayudante y el Jefe es Claude 2. El agente lo anota con la fecha y la hora, y la siguiente vez que lo abras ya no te lo preguntará. Solo puede haber un Jefe por proyecto: si nombras Jefe a otro, el anterior pasa a ser ayudante y el cambio queda anotado.

Después atiende los encargos que tenga pendientes, retoma lo que hubiera dejado a medias y pone en marcha su vigilante del canal, que es lo que le avisa cuando otro agente le hace un encargo. Termina con un resumen de pocas líneas: quién es, qué papel tiene, quién es el Jefe, qué tiene pendiente y cómo vigila el canal.

Si un agente no se presenta, o se queda quieto sin hacer nada, algo ha ido mal con su arranque. Escríbele que haga el arranque obligatorio de AGENTS.md y te dirá qué ha leído. El documento Resolución de problemas explica las causas más habituales.

La primera vez que un agente abre una carpeta suele preguntar si confías en ella. Contesta antes de escribirle nada. Con Codex, la primera vez puede aparecer además el aviso No active thread is available. Es un mensaje del propio Codex, no de Gestor Agentes. Codex funciona igual y el aviso no vuelve a salir las siguientes veces.

Puedes cerrar Gestor Agentes cuando quieras: las ventanas de los agentes que hayas abierto siguen funcionando.


Elegir el papel al abrir

Encima de los botones de los agentes hay una línea que dice Papel del agente que abras, con tres opciones: que lo pregunte, Jefe de proyecto y ayudante. Si eliges Jefe de proyecto o ayudante antes de pulsar el botón de un agente, la aplicación anota ese papel para él en el estado del proyecto, con la fecha y la hora, y el agente lo encuentra al arrancar sin tener que preguntártelo. Si eliges Jefe, la anotación dice que sustituye a cualquier Jefe anotado antes. La elección vale para una sola apertura: después vuelve sola a que lo pregunte, que es también lo que pasa si no eliges nada, y entonces el agente te pregunta su papel al arrancar si no lo tiene anotado.


Qué deja la aplicación dentro de cada proyecto

En la carpeta principal del proyecto aparecen dos archivos. AGENTS.md es el archivo de instrucciones que leen todos los agentes. Empieza con el arranque en nueve pasos que se acaba de describir y sigue con las normas comunes de trabajo, que explican cómo se verifica el código, cómo se documenta, cómo se reparten las tareas los agentes y qué no se puede hacer sin tu permiso. Al final lleva las normas de tu equipo, de las que se habla más abajo. Está escrito para los agentes, no para ti: por eso usa listas, tablas y las órdenes exactas que tienen que ejecutar, y es corto, unos 14 KB, para que no les ocupe más de lo necesario. CLAUDE.md es un archivo pequeño que solo sirve para que Claude Code cargue AGENTS.md.

Esos dos archivos son de la aplicación, que los escribe enteros cada vez que abres el proyecto. No merece la pena editarlos a mano, porque lo que escribas se sustituirá la siguiente vez. No se pierde, porque antes se guarda una copia, pero no se queda en su sitio.

Lo propio de cada proyecto, como sus reglas, tus órdenes particulares para él o cómo se compila y se prueba, va en el archivo instrucciones-del-proyecto.md, dentro de la carpeta .gestor del proyecto. La aplicación lo crea la primera vez y no lo vuelve a tocar nunca. Los agentes lo leen al empezar.

La carpeta .gestor es el espacio de trabajo de los agentes. Como su nombre empieza con un punto, puede que no la veas si tu sistema oculta ese tipo de carpetas. Dentro está la carpeta canal, con el estado del proyecto, que recoge los papeles de cada agente, el reparto de tareas y los archivos que cada uno tiene reservados, y con un buzón para cada agente, que en este programa se llama bandeja. Está también la carpeta propia de cada agente, con su diario de trabajo, sus actas de verificación y su registro de funciones; el registro de tus órdenes; el archivo propio del proyecto del que se acaba de hablar; la carpeta consulta, con los documentos que los agentes leen solo cuando les hacen falta, sobre cómo se escribe el código, cómo se verifica un cambio, cómo se documenta y se publica, cómo se trabaja con Programator y qué soluciones comprobadas en otros proyectos conviene conocer, que es el aprendizaje común del que se habla más abajo; y la carpeta vigilante, con el programa gestor-vigilante, que vigila el canal. Esa carpeta no se guarda en el repositorio git, y no tienes que tocar nada de ella: la cuidan los agentes.

El archivo propio del proyecto tiene además un apartado llamado Reglas aprendidas. Cuando un trabajo falla por una trampa del proyecto que se puede repetir, por ejemplo una orden que en tu equipo necesita una opción especial, el agente que lo descubre la deja allí escrita en una línea, para que ningún otro vuelva a caer en ella.

Si una de esas reglas puede servir también en otros proyectos, el agente la marca terminándola con las palabras candidata a común, entre paréntesis. La próxima vez que abras el proyecto, la aplicación la copia, con el nombre del proyecto y la fecha, al archivo candidatas-comunes.md de su carpeta datos, y te avisa, para que decidas si pasa a ser común. Las soluciones que ya se han dado por comprobadas viajan dentro de la aplicación, y cada proyecto las recibe en el archivo aprendizaje-comun.md de su carpeta .gestor/consulta, que los agentes consultan cuando les hace falta.

Si el proyecto ya tenía instrucciones de una versión anterior, la aplicación no las pierde. Guarda una copia exacta de los archivos antiguos en la carpeta .gestor\respaldos, aparta a la carpeta .gestor\archivo los que ya no se usan, incluidas las reglas que agy leía por su cuenta desde la carpeta .agents, y copia lo que era propio del proyecto en instrucciones-del-proyecto.md, dentro de un bloque pendiente de revisar. El primer agente que arranque en el proyecto revisa ese bloque por su cuenta antes de ponerse con otra cosa: conserva lo que siga valiendo, te pregunta lo que le plantee dudas, rescata cualquier tarea que hubiera quedado a medias en los archivos apartados y, cuando ha terminado, borra el bloque. La siguiente vez que abres el proyecto, la aplicación ve que el bloque ya no está y borra ella misma las carpetas que había apartado, sistema-anterior y canal-gemini-antiguo dentro de .gestor\archivo, porque el agente no siempre tiene permiso para hacerlo. Así lo antiguo no se queda dando instrucciones contradictorias a los agentes.


Trabajar con varios agentes a la vez

Puedes abrir el mismo proyecto con varios agentes, cada uno desde su botón. Cada uno trabaja en su propia ventana y se comunica con los demás dejando mensajes en su bandeja de la carpeta .gestor\canal, que los demás leen. Un mensaje es un encargo cuando dice a quién va con la palabra Para, por ejemplo Para Gemini 1; si solo menciona a alguien, no lo es. Un encargo puede ir a varios agentes a la vez, como Para Claude 2 y Codex 1, y la primera cuenta de cada herramienta se puede nombrar sin número, como Para Codex. Cuando un agente lee un encargo para él, escribe en su bandeja una línea que empieza por RECIBIDO, con la hora, como prueba de que está vivo y de que lo ha leído. Esas líneas no se borran al podar la bandeja: pasan a su histórico.

El reparto lo hace el Jefe de proyecto. Tú hablas con el Jefe, y el Jefe habla con sus ayudantes: propone quién hace cada cosa, como programar, verificar o documentar, y lo deja escrito en el estado del proyecto. Las decisiones técnicas las acuerdan entre todos, y si no se ponen de acuerdo, el Jefe te pregunta a ti. Antes de cambiar un archivo, cada agente lo reserva en el estado del proyecto, para que dos no toquen lo mismo a la vez.

Ningún agente da por bueno su propio trabajo. Cuando uno termina algo, otro lo comprueba ejecutando las pruebas, rompe a propósito lo que vigila cada prueba para ver que de verdad falla, y deja constancia por escrito con un registro que se puede comprobar. Si solo trabaja un agente, la comprobación la hace otro agente al que se llame para ello, o tú mismo.

Si un agente agota su límite de uso, deja escrito en su diario en qué punto estaba y cuál es su próximo paso exacto. Abres el mismo proyecto con otro botón y el nuevo agente continúa desde ahí.

Cuando quieras dar una orden que valga para todos, dísela al Jefe, o a cualquiera de ellos. La anotará en el registro de órdenes con tus palabras literales y se la pasará a los demás por el canal. Lo que ya está decidido no te lo vuelven a preguntar.

Si trabajas con Programator, el modelo local, los demás agentes saben encargarle trabajo sin que se lo expliques: la aplicación deja en cada proyecto una guía para ellos que dice cómo se le hace un encargo, qué hace bien y qué no, y que quien le encarga algo comprueba después el resultado.


Cómo se enteran los agentes de los mensajes

Cada agente tiene un vigilante del canal, un programa pequeño llamado gestor-vigilante que va en el paquete junto a la aplicación y que esta deja en la carpeta .gestor/vigilante de cada proyecto. El vigilante mira cada pocos segundos los mensajes de los demás y solo avisa al agente de lo que le toca: un encargo para él, o código cambiado a propósito para una prueba que todavía no se ha restaurado, cuando eso cambia. Un encargo ya avisado que solo cambia de sitio en la bandeja no lo vuelve a despertar. No funciona igual en todas las herramientas. Claude Code y agy lo dejan en segundo plano esperando, y termina en cuanto hay algo para el agente; al terminar, la herramienta despierta al agente, que atiende el encargo y vuelve a lanzar el vigilante. Mientras no hay nada para él, no gasta nada. Para Codex lo deja en marcha la propia aplicación al abrirlo, desde su terminal y en modo persistente, porque dentro de su entorno aislado Codex no puede ver los procesos del equipo; Codex solo le anota su sesión al arrancar, y el vigilante le entrega cada aviso dentro de su propia sesión con la orden codex queue de Codex, así que Codex no tiene que montarse ninguna vigilancia propia. Nunca hay dos vigilantes del mismo agente a la vez, y ninguno sigue en marcha cuando se cierra la sesión que lo lanzó. Si su herramienta no les deja vigilar así, miran el canal al terminar cada paso de su trabajo y lo dicen en su bandeja; en ese caso, si el agente está parado esperando y otro le escribe, no se entera hasta que tú le digas algo, y basta con escribirle que mire el canal.


Cómo va el equipo de un proyecto

Al seleccionar un proyecto en la lista, encima de los botones aparece una línea con su equipo, por ejemplo: Equipo, Jefe: Claude 2, claude2 hace 5 min, gemini hace 3 h, 2 reservas. Te dice quién es el Jefe, cuánto hace que cada agente dio señales de vida y cuántos archivos tienen reservados. Junto al latido de cada agente, entre paréntesis, la línea dice vigila si su vigilante está en marcha, VIGILANTE CAÍDO si se cerró sin terminar bien y, si le han llegado encargos después de su último RECIBIDO, cuántos son, por ejemplo 2 encargos sin RECIBIDO. Un agente con el vigilante caído o con encargos sin acuse puede no estar atendiendo: escríbele en su ventana que mire el canal. Si alguno dejó código cambiado a propósito para una prueba y sin restaurar, la línea lo avisa con MUTACIONES VIVAS. La línea se pone al día al seleccionar el proyecto y cada vez que abres un agente.

Si el proyecto todavía no tiene estado del canal, porque no se ha abierto nunca con esta versión, la línea no aparece.

Al abrir un proyecto, la preparación también puede avisar de cosas del equipo que no cuadran: dos agentes que reservan el mismo archivo, la reserva de un agente que lleva más de doce horas sin dar señales, código cambiado a propósito sin restaurar o código del proyecto que depende de los archivos de instrucciones. El apartado Los avisos y qué hacer con ellos explica cada uno. Díselo al Jefe del proyecto, que es quien lo ordena.


Quitar un proyecto de la lista

Cada fila de la lista tiene un botón Quitar. Al pulsarlo, el proyecto desaparece de la lista y nada más: su carpeta sigue intacta en el disco, y puedes volver a añadirla cuando quieras.


Eliminar un proyecto del disco

El botón Eliminar del disco, en la parte de abajo, borra de verdad la carpeta del proyecto seleccionado con todo lo que tenga dentro. Los archivos no pasan por la papelera, así que no se pueden recuperar. Antes de borrar, la aplicación abre una ventana titulada Eliminar proyecto del disco y te pide que confirmes con Sí. Úsalo solo si quieres destruir el proyecto. Si solo quieres dejar de verlo en la lista, el botón que buscas es Quitar. Si la carpeta ya no existía, la aplicación se limita a quitarla de la lista.


La pestaña Agentes

Esta pestaña tiene una fila por cada agente, con tres columnas: Agente, Estado y Ruta del ejecutable. El nombre del agente aparece sobre un color que resume su estado, y la columna Estado lo dice además con palabras, para que no dependa solo del color. Verde y Localizado quieren decir que la aplicación ha encontrado el programa y, si es Claude Code o Codex, la carpeta de su cuenta, así que el agente se puede abrir. Ámbar y Falta la cuenta, que el programa está pero falta la carpeta de su cuenta, como se explica en el apartado siguiente. Rojo y No encontrado, que la aplicación no encuentra el programa. Gris y Sin comprobar, que lo mostrado todavía no vale: o la búsqueda está en marcha, o has cambiado la ruta del programa y aún no la has guardado. Después de la palabra, el estado dice la carpeta donde vive el agente, con la indicación no existe si falta, y la ruta del programa encontrado.

Mientras la aplicación busca los agentes, al arrancar, al pulsar Buscar de nuevo o al guardar las rutas, junto a esos botones gira un indicador con el texto Buscando agentes.

Solo los agentes encontrados tienen botón en la pestaña Proyectos. Si un agente sale como no encontrado y sabes que está instalado, puede ser por dos motivos. El primero es que lo instalaste con Gestor Agentes abierto: ciérralo y vuelve a abrirlo. El segundo es que está en una carpeta donde la aplicación no mira: escribe su ruta en la columna Ruta del ejecutable, o búscala con el botón Examinar, y pulsa Guardar rutas. Si la ruta que escribes no existe, la aplicación te lo dice en una ventana titulada Rutas no válidas y no guarda nada.

El botón Búsqueda automática borra la ruta que hubieras escrito, para que la aplicación vuelva a buscar el programa por su cuenta. El botón Buscar de nuevo repite la búsqueda de todos los agentes.

La aplicación busca cada programa por su cuenta en estos sitios, por este orden. En Windows, codex se busca primero en AppData\Local\Programs\OpenAI\Codex\bin y agy en AppData\Local\agy\bin, que es donde los dejan sus instaladores oficiales. Después, todos se buscan en la carpeta .local\bin de tu carpeta de usuario, donde lo dejan los instaladores de Claude Code y, en Linux, el de agy. En Linux se mira luego en /usr/local/bin y /usr/bin. Por último, en todas las carpetas del PATH del sistema. Programator nunca se busca solo: hay que escribir su ruta.

Si escribes la ruta de un agente, la aplicación usa esa y ninguna otra. Si un día esa ruta deja de existir, el agente aparecerá como no encontrado, para que te des cuenta, en lugar de abrir otro programa que encuentre por casualidad.


Dónde vive cada agente y cómo se nombran sus carpetas

Cada agente tiene dos cosas en tu equipo: el programa que se ejecuta y la carpeta donde vive, que es donde guarda su sesión, su configuración y lo que recuerda. Varios agentes de la misma herramienta comparten el mismo programa, así que lo que los distingue es su carpeta. Gestor Agentes localiza las cuentas por el nombre de esas carpetas, dentro de tu carpeta de usuario, C:\Users\tu-usuario en Windows o /home/tu-usuario en Linux, con la misma biblioteca que usan otros programas para encontrar las cuentas de los agentes.

Claude Code. La primera cuenta vive en la carpeta .claude, que es la que Claude Code usa siempre, y se llama Claude 1; tiene botón si existe esa carpeta, aunque esté vacía. Las demás se llaman .claude seguido de lo que quieras. Si es un número sin ceros delante, desde el 2, la cuenta toma ese número si está libre: .claude2 es Claude 2 y .claude3 es Claude 3. Si ese número ya lo tiene otro agente, o el nombre lleva cualquier otro añadido, como .claude1, .claude02 o .claude-trabajo, toma el número siguiente al mayor que ya tengan tus agentes de Claude. Una de esas carpetas se reconoce como cuenta si dentro tiene alguno de los archivos que Claude Code crea al usarla: .claude.json, .credentials.json o settings.json.

Codex. Igual, con la carpeta .codex para Codex 1 y .codex2, .codex3 o cualquier otro nombre que empiece por .codex para las demás. Basta con que tenga uno de estos archivos: config.toml o auth.json.

Una vez que una cuenta tiene su agente, conserva su número aunque aparezcan otras carpetas. Si quieres guardar una copia de una cuenta sin que la aplicación la use, ponle un nombre que no empiece por .claude ni por .codex, porque desde la versión 3.9.0 carpetas como .claude-old o .claude1 también son cuentas. La aplicación cuenta además la carpeta que señale CLAUDE_CONFIG_DIR o CODEX_HOME en el entorno desde el que la abres, aunque esté fuera de tu carpeta de usuario, siempre que esa carpeta exista. El agente que se crea para ella queda en tu lista aunque después abras la aplicación sin esa variable.

Para crear la segunda cuenta de Claude Code, abre PowerShell y escribe esta orden, que arranca Claude Code usando la carpeta .claude2:

$env:CLAUDE_CONFIG_DIR = "$HOME\.claude2"; claude

Inicia sesión con la otra cuenta y cierra la ventana. Al volver a abrir Gestor Agentes, o al pulsar Buscar de nuevo, aparecerá el botón Claude 2. Para una tercera cuenta se cambia .claude2 por .claude3. Si la carpeta de una cuenta no está en tu carpeta de usuario con ese nombre, pulsa en la pestaña Agentes Añadir cuenta de Claude o Añadir cuenta de Codex y elige su carpeta: la aplicación crea el agente siguiente de esa herramienta con ella, y te avisa si esa carpeta ya es la de otro agente. Con Codex se hace igual, con esta orden:

$env:CODEX_HOME = "$HOME\.codex2"; codex

agy, la herramienta de Gemini, vive en la carpeta .gemini de tu carpeta de usuario, y guarda sus ajustes en .gemini\antigravity-cli\settings.json según su documentación oficial, https://antigravity.google/docs/cli/reference y https://antigravity.google/docs/settings?tab=cli. Esa documentación no describe ninguna variable ni opción para usar otra carpeta, así que solo puede haber una cuenta de Gemini, Gemini 1.

Programator no tiene cuenta. Vive en la carpeta donde lo instalaste, junto a su programa, y la aplicación no lo busca sola: hay que escribir la ruta de su programa en la pestaña Agentes. Si instalas una versión nueva en otra carpeta, cambia allí la ruta, porque la aplicación seguirá abriendo la que tenga escrita. Para que no se te pase, en Windows la pestaña Agentes muestra la versión del Programator de la ruta y te avisa si hay otra más nueva publicada.

Gestor Agentes no inicia sesión ni guarda contraseñas. Solo reconoce las carpetas de cuenta que ya tienes y abre cada agente con la suya: a Claude Code le pasa su carpeta con la variable CLAUDE_CONFIG_DIR y a Codex con CODEX_HOME. La primera cuenta de cada herramienta se abre quitando esa variable, como la abrirías tú desde una terminal sin nada especial, para que use su cuenta por defecto. Claude Code guarda el estado de su primera cuenta en el archivo .claude.json de tu carpeta de usuario, y no dentro de .claude. Hasta la versión 3.8, Gestor Agentes abría Claude 1 con la variable puesta, y entonces Claude Code lo guardaba en .claude\.claude.json. La primera vez que abras Claude 1 con la versión 3.9.0, si existe ese archivo, la aplicación lo copia sobre el de tu carpeta de usuario, para que Claude 1 conserve sus proyectos y ajustes, y guarda antes el que hubiera en la carpeta datos, con el nombre estado-de-claude-antes-de-3.9.0.json. Después cambia el nombre de .claude\.claude.json a .claude.json.migrado-3.9.0, y eso es lo que indica que ya está hecho: no se repite, ni siquiera si abres otra copia de la aplicación con otra carpeta datos. Te lo cuenta en la ventana de preparación del proyecto que se abre al terminar. Si no existe .claude\.claude.json, no toca nada. Si algo falla, te lo dice en esa misma ventana, abre Claude 1 igualmente y lo vuelve a intentar la próxima vez, sin sobrescribir nunca la copia guardada en datos, que sigue siendo la de tu archivo original. Si borras o renombras la carpeta de cualquier cuenta, también la primera, .claude o .codex, la pestaña Agentes la muestra en ámbar con Falta la cuenta y su botón desaparece, para que la herramienta no cree una carpeta vacía y te pida iniciar sesión de nuevo. Si acabas de instalar Claude Code o Codex y todavía no lo has abierto nunca, ábrelo una vez por tu cuenta e inicia sesión: así se crea su carpeta y, al pulsar Buscar de nuevo, aparece su botón.


Si guardas copias de cuentas: aviso para quien viene de la versión 3.8 o anterior

Hasta la versión 3.8, la aplicación solo reconocía las carpetas de cuenta con un número detrás, como .claude2, y solo si tenían el archivo de la sesión. Por eso era habitual guardar una copia de una cuenta, o una cuenta que ya no se usaba, con un nombre como .claude-old, .claude-copia, .claude1 o .codex-antigua: la aplicación la ignoraba.

Desde la versión 3.9.0 eso ya no es así. La búsqueda de cuentas la hace ahora una biblioteca común que comparten varios programas, para que todos vean las mismas cuentas de la misma forma, y para ella cualquier carpeta de tu carpeta de usuario cuyo nombre empiece por .claude o por .codex es una cuenta si contiene alguno de los archivos que la herramienta crea al usarla: .claude.json, .credentials.json o settings.json en Claude Code, y config.toml o auth.json en Codex. Una copia de una cuenta tiene esos archivos, así que ahora cuenta como una cuenta más.

Qué pasa si no haces nada. La próxima vez que abras la aplicación, o que pulses Buscar de nuevo, cada una de esas copias aparece como un agente nuevo, con el número siguiente al mayor que ya tengas, por ejemplo Claude 4, y con su botón para abrirlo. Si lo abres, Claude Code o Codex trabajan con esa copia como si fuera una cuenta en uso: pueden pedirte iniciar sesión si su sesión ha caducado y escriben dentro de ella, así que la copia deja de ser lo que guardaste. Además, el agente nuevo queda anotado en tu lista de agentes y conserva su número aunque después cambies el nombre de la carpeta: a partir de ese momento aparece en ámbar con Falta la cuenta, hasta que lo quites de la lista. Nada de esto toca tus cuentas de verdad, Claude 1, Claude 2 y las demás, que siguen igual.

Cómo se soluciona. Antes de abrir la versión 3.9.0 por primera vez, cambia el nombre de esas copias por otro que no empiece por .claude ni por .codex, o sácalas de tu carpeta de usuario. Por ejemplo, .claude-old puede pasar a llamarse copia-claude-old. En Windows, con PowerShell:

Rename-Item "$HOME\.claude-old" "copia-claude-old"

y en Linux, en una terminal:

mv ~/.claude-old ~/copia-claude-old

Si alguna vez necesitas volver a usar esa copia, basta con devolverle su nombre, o darla de alta con Añadir cuenta de Claude o Añadir cuenta de Codex en la pestaña Agentes.

Si ya ha aparecido un agente que no querías. La ventana no tiene todavía un botón para quitar un agente de la lista. Cambia primero el nombre de la carpeta como se explica arriba y, con la aplicación cerrada, abre el archivo perfiles-agentes.json de la carpeta datos con un editor de texto y borra el bloque de ese agente, el que lleva su número en Instancia y la carpeta de la copia en DirectorioConfiguracion, con cuidado de no dejar una coma de más ni de menos entre los bloques. Al volver a abrir la aplicación ya no aparece. Si el archivo queda mal escrito, la aplicación no lo usa: muestra solo las cuentas de Claude Code y Codex que encuentra en tu carpeta de usuario, sin Gemini, sin Programator y sin las rutas que hubieras puesto. En la barra de estado te dice que no guarda la lista de agentes porque perfiles-agentes.json no se pudo leer, y no escribe nada en ese archivo durante esa sesión, para no borrar lo que tenías en él. Ciérrala, corrige el archivo y vuelve a abrirla. Antes de editarlo, guarda una copia del archivo en otra carpeta: así podrás volver atrás si algo sale mal.

Por qué se ha hecho así. Antes, cada programa decidía por su cuenta qué carpeta era una cuenta, y dos programas podían ver cuentas distintas en el mismo equipo. Con la biblioteca común, la regla es una sola y es más amplia: reconoce también las cuentas con un nombre elegido por ti, como .claude-trabajo, sin obligarte a numerarlas. La contrapartida es que una copia ya no se distingue de una cuenta por su nombre, y por eso las copias tienen que guardarse con un nombre que no empiece por .claude ni por .codex.


Modelo y esfuerzo de cada agente

Gestor Agentes no elige el modelo ni el esfuerzo de los agentes: lo hace cada herramienta, que es la única que sabe qué modelos tiene tu cuenta. Basta con hacerlo una vez, la primera vez que abras el agente, y vale para todos tus proyectos.

En Claude Code, escribe /model dentro de la sesión, elige el modelo y pulsa Enter; después, /effort, elige el nivel y pulsa Enter. Claude Code guarda las dos cosas en la carpeta de esa cuenta y las usa en las sesiones siguientes, en cualquier proyecto. Si en lugar de Enter pulsas s, el cambio vale solo para esa sesión. El nivel máximo de esfuerzo nunca se guarda: hay que elegirlo cada vez. Si tienes dos cuentas de Claude Code, cada una guarda su propia elección, así que se hace una vez en cada una. Si un proyecto fija su propio modelo en su configuración de Claude Code, ese manda sobre tu elección.

En agy, escribe /model dentro de la sesión y elige el modelo; después, /effort, elige el nivel y pulsa Enter. agy guarda las dos cosas en su configuración general y las usa en las sesiones siguientes, en cualquier proyecto. No todos los modelos tienen todos los niveles: los dos más altos solo aparecen en los modelos que los ofrecen.

Para Codex, consulta en su propia documentación cómo se elige y se guarda el modelo.


Permisos de los agentes al arrancar

Al arrancar, cada agente hace siempre lo mismo: mira quién es y en qué carpeta está, la fecha y la hora, lista archivos, consulta el estado de git, escribe en la carpeta .gestor y pone en marcha su vigilante del canal. Para que no te pida permiso para eso en cada sesión, la aplicación deja esas acciones autorizadas, y solo esas, en el archivo de permisos del proyecto de cada herramienta que lo admite. No autoriza nada más: ni subir cambios, ni borrar, ni conectarse a internet.

En Claude Code el archivo es .claude/settings.local.json, dentro del proyecto. Las reglas que añade la aplicación son once. Cinco son para la terminal de bash: la orden exacta con la que el agente muestra su identidad, la carpeta del proyecto y la carpeta actual, echo $GESTOR_AGENTE $GESTOR_PROYECTO $PWD, y las órdenes date, ls, git status y la del vigilante, ./.gestor/vigilante/gestor-vigilante, cada una con cualquier argumento. Otras cinco son las mismas para PowerShell: Write-Output $env:GESTOR_AGENTE $env:GESTOR_PROYECTO $PWD.Path tal cual, y Get-Date, Get-ChildItem, git status y ./.gestor/vigilante/gestor-vigilante con cualquier argumento. La última permite crear y editar archivos dentro de la carpeta .gestor del proyecto. Si ya lo tenías, la aplicación conserva todo lo que hubiera y solo añade las reglas que falten. Si el archivo está mal formado no lo toca y te lo dice en los avisos. Claude Code solo aplica este archivo sin preguntar mientras no esté guardado en git, así que no lo añadas al repositorio.

Codex no usa un archivo de permisos del proyecto. Para que pueda trabajar solo, sin pedirte aprobación en cada tarea, la aplicación lo abre con su entorno aislado de escritura en el proyecto, con acceso a la red, que le hace falta para las máquinas virtuales de pruebas, y sin pedir nunca aprobación: las opciones -s workspace-write y -a never y la clave sandbox_workspace_write.network_access. En Windows le pide además que prefiera MXC, el entorno aislado que OpenAI recomienda allí, con la clave features.prefer_mxc; si tu equipo no lo admite, Codex usa su modo de respaldo. Dentro del entorno aislado, Codex no tiene permiso para escribir fuera de la carpeta del proyecto, y con este modo tampoco te pide aprobación para saltárselo. Si tu perfil de Codex ya lleva en sus argumentos una opción de sandbox o de aprobación, o una de esas claves con -c, la aplicación no añade nada y manda lo tuyo.

En agy los permisos se guardan en tu configuración general, fuera del proyecto, y la aplicación no la toca. Puedes añadir tú las reglas una sola vez con la orden /permissions de agy. Son estas: command(date), command(ls) y command(git status); para el vigilante, command(regex:\./\.gestor/vigilante/gestor-vigilante( .+)?); y, para escribir en la carpeta de los agentes, write_file(.gestor/).


Los avisos y qué hacer con ellos

La preparación de un proyecto puede dejar alguno de estos avisos, en la ventana de preparación si se abre o, si no, en el archivo .gestor/avisos-de-preparacion.md, con un resumen en la barra de estado. Ninguno impide trabajar.

Si habla de una copia exacta o de un archivo respaldado, es que AGENTS.md o CLAUDE.md tenían texto que no había escrito la aplicación. No se ha perdido nada: el original está en la carpeta .gestor\respaldos del proyecto. Lo que quieras que lean los agentes, escríbelo en instrucciones-del-proyecto.md.

Si habla de archivos llevados al archivo, es que el proyecto tenía archivos del sistema de trabajo anterior, que ahora están en .gestor\archivo. El agente que revise las instrucciones mira si alguno guardaba una tarea a medias.

Si dice que una bandeja mide más de lo debido, por ejemplo .gestor/canal/claude.md mide 62066 bytes, más de los 16384 de una bandeja podada, es que el buzón de ese agente ha crecido sin limpiarse. Díselo a ese agente para que pase lo ya resuelto a su histórico. El aviso sale con cualquier agente que abras, porque es del proyecto y no del agente, y deja de salir en cuanto la bandeja vuelve a su tamaño. Lo mismo pasa con el diario de un agente, su archivo tarea-en-curso.md, y con el estado del proyecto, .gestor/canal/estado.md: los agentes los leen enteros cada vez que arrancan, así que, si pasan de ese tamaño, lo ya cerrado tiene que ir a su histórico o al archivo.

Si dice que las marcas estaban dañadas, un archivo de instrucciones antiguo estaba mal formado. La aplicación ha guardado una copia y ha pasado su texto entero al bloque pendiente de revisar, para que el agente quite lo que sobre.

Si dice que no se ha podido comprobar git, git no está instalado o no responde. El proyecto se ha preparado igualmente, pero sin repositorio. Instala git y vuelve a abrir el proyecto.

Los avisos siguientes son del estado del equipo de agentes, que la aplicación revisa cada vez que abres el proyecto. Si al estado del proyecto le falta alguno de los apartados Papeles, Reservas activas o Mutaciones vivas, porque viene de una versión anterior o alguien lo ha editado, la aplicación lo añade al final sin borrar nada y lo anota entre los cambios; si aun así avisa de que falta uno, es que no ha podido escribir el archivo. Si dice que hay mutaciones vivas, algún agente cambió código a propósito para comprobar una prueba y todavía no lo ha restaurado; mientras siga así, el proyecto no está en su estado normal, y su dueño tiene que restaurarlo antes de seguir. Si dice que dos agentes tienen reservada la misma ruta, los dos podrían estar cambiando el mismo archivo; uno de los dos tiene que liberarla. Si dice que la reserva de un agente es de alguien sin latido reciente, ese agente lleva más de doce horas sin dar señales y puede que ya no esté trabajando; el Jefe comprueba si la reserva sigue valiendo. Si dice que una reserva no sigue el formato, está mal escrita y la aplicación no puede comprobarla. Las doce horas se cambian en ajustes.json, con LimiteDeLatidoHoras.

Si dice que el código del proyecto cita AGENTS.md, CLAUDE.md o GEMINI.md, nombra hasta diez archivos de código que mencionan alguno de ellos. La aplicación reescribe esos archivos cada vez que abres el proyecto, así que una prueba o un programa del proyecto que dependa de su contenido se romperá solo, sin que nadie haya tocado el proyecto. Pide a los agentes que quiten esa dependencia.

Si dice que no se ha encontrado gestor-vigilante junto a Gestor Agentes, el programa del vigilante no está en la carpeta de la aplicación y no se ha podido copiar al proyecto, así que los agentes no tendrán vigilante. Descarga de nuevo el paquete completo y sustituye los dos programas.

Si dice que se han anotado reglas candidatas a aprendizaje común, un agente ha marcado alguna regla del proyecto como candidata a común y la aplicación la ha copiado a candidatas-comunes.md, en su carpeta datos. Léela cuando puedas y decide si vale para todos tus proyectos.

Hay otro aviso que no sale en esa ventana, sino en la barra de estado, al abrir Codex o Gemini. Empieza por la palabra AVISO y dice que AGENTS.md mide más de lo que lee el agente: Codex lee los primeros 32.768 bytes y Gemini los primeros 24.000. Significa que ese agente no verá el final de sus instrucciones. Normalmente no pasa, porque la aplicación escribe AGENTS.md por debajo de esos tamaños; si sale, suele ser porque las normas de tu equipo, de las que se habla a continuación, son muy largas.

Si al pulsar un botón el agente no se abre, la aplicación lo explica en una ventana. Si dice que el agente no está instalado, mira la pestaña Agentes. Si dice que el perfil está incompleto, falta algún dato del perfil, normalmente la ruta de Programator, y la ventana dice en qué archivo se arregla. En los dos casos, los archivos del proyecto ya se han preparado.


Las normas de tu equipo

Hay normas que valen para todos tus proyectos, como las máquinas donde se hacen las pruebas o las carpetas donde los agentes no deben escribir. No hace falta repetirlas en cada proyecto. Escríbelas una vez en un archivo CLAUDE.md dentro de la carpeta que contiene todos tus proyectos.

Cada vez que abres un proyecto, la aplicación sube desde su carpeta hasta encontrar ese archivo y copia su contenido al final de AGENTS.md, en el bloque de entorno, con una línea que dice de qué archivo sale. Si cambias el archivo, el cambio llega a cada proyecto la próxima vez que lo abras. Si la aplicación no encuentra ninguno, el bloque lo dice.


La pestaña Instrucciones de los agentes

Muestra las normas comunes que la aplicación escribe en todos los proyectos. Solo se pueden leer, porque van dentro del programa y cambian con cada versión. Lo que quieras añadir para un proyecto concreto se escribe en su instrucciones-del-proyecto.md, y lo que valga para todos tus proyectos, en el CLAUDE.md de la carpeta que los contiene.


Dónde guarda la aplicación tu configuración

Todo está en la carpeta datos, junto al ejecutable. Son cuatro archivos, todos de texto, más candidatas-comunes.md, que solo aparece cuando algún proyecto propone una regla como candidata a común, y estado-de-claude-antes-de-3.9.0.json, que aparece la primera vez que abres Claude 1 con la versión 3.9.0 y guarda el estado que Claude Code tenía en tu carpeta de usuario antes de esa versión. Puedes borrarlo cuando compruebes que Claude 1 conserva sus proyectos y ajustes. config.json guarda la lista de proyectos. perfiles-agentes.json guarda los perfiles de agente: para cada uno, su tipo, su número, la carpeta de su cuenta y, si se la has puesto, la ruta de su programa. perfiles-agentes.version es un número que la aplicación usa para saber si tiene que poner al día los perfiles. ajustes.json guarda los ajustes de funcionamiento: TiempoMaximoGitSegundos, los segundos que la aplicación espera a git antes de darlo por perdido, que son 30 si no lo cambias; LimiteDeBandejaBytes, el tamaño a partir del cual avisa de una bandeja sin podar, que son 16384; ComprobarActualizaciones, que vale true para que la aplicación mire al arrancar si hay una versión nueva, y false para que no lo mire; RepositorioDeActualizaciones, el repositorio de GitHub donde lo mira, que es celtidcs/gestor-agentes; RepositorioDeProgramator, el repositorio donde mira la última versión de Programator, que es celtidcs/Programator; TiempoMaximoActualizacionSegundos, los segundos que espera la respuesta, que son 5; y LimiteDeLatidoHoras, las horas sin noticias de un agente a partir de las cuales la aplicación avisa de que sus reservas pueden estar abandonadas, que son 12.

No necesitas tocar ninguno de estos archivos, porque todo se hace desde la ventana. Si los editas a mano y te equivocas, la aplicación te dice al arrancar que la configuración no es válida y no sobrescribe nada, para que puedas corregirlo.


Cuando hay una versión nueva

Cada vez que abres la aplicación, esta mira en GitHub, sin hacerte esperar, si se ha publicado una versión más nueva que la tuya. Si la hay, te enseña una ventana titulada Actualización disponible, con la versión nueva, la que tienes y la dirección de la página donde se descarga. Descarga allí el paquete de tu sistema y sustituye los dos programas, gestor-agentes y gestor-vigilante, y la carpeta documentacion, pero conserva tu carpeta datos, que es la que guarda tu lista de proyectos y tus perfiles. La dirección de esa página la compone siempre la aplicación a partir del repositorio configurado, nunca con lo que conteste GitHub. La aplicación no descarga ni instala nada por su cuenta, y no envía ningún dato tuyo: solo pregunta cuál es la última versión. Si sales a internet a través de un proxy, lo respeta: el de las variables HTTPS_PROXY y similares y, en Windows, el de las opciones de internet. Si no tienes conexión, o GitHub no contesta en los segundos de TiempoMaximoActualizacionSegundos, simplemente no sale nada. Si no quieres que lo mire, pon ComprobarActualizaciones en false en ajustes.json.


Llevarte el programa a otro ordenador

El programa es portable de verdad: copia su carpeta entera, con la carpeta datos dentro, a una memoria USB o a otro ordenador, y úsala sin instalar nada. La configuración viaja con ella.

Los perfiles de agente funcionan igual en cualquier equipo, salvo las rutas que hayas escrito a mano, que solo valen si el programa está en el mismo sitio. La lista de proyectos guarda rutas de carpetas, así que solo sirve tal cual si los proyectos están en la misma ruta en los dos equipos, por ejemplo en una unidad compartida que se monta con la misma letra. Si no, quita de la lista los que no se puedan abrir y añade los de ese equipo.

Si prefieres empezar de cero en el otro ordenador, copia la carpeta del programa sin el contenido de la carpeta datos.


Desinstalar

No hay nada que desinstalar. Borra la carpeta del programa y listo; con ella se va tu configuración. Tus proyectos no se tocan.


Un día normal de trabajo

Abres Gestor Agentes y ves tu lista. Seleccionas el proyecto y, encima de los botones, ves cómo está su equipo: quién es el Jefe y cuándo dio señales cada agente. Pulsas el botón del Jefe y se abre su terminal: el agente se presenta, lee lo que dejaron los demás y retoma lo que hubiera pendiente. Le cuentas lo que quieres y él reparte el trabajo. Si quieres que otro agente revise lo hecho o ayude con otra parte, pulsas su botón y se incorpora al mismo proyecto como ayudante; el Jefe le hace los encargos por el canal y otro agente comprueba cada trabajo antes de darlo por bueno. Al terminar, los agentes dejan escrito el estado del proyecto, su diario con el próximo paso y el historial de cambios, de modo que la próxima vez todo sigue ahí, abras con el agente que abras.
