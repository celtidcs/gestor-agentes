Resolución de problemas

Este documento recoge lo que suele salir mal al usar Gestor Agentes y qué hacer en cada caso. Está pensado para leerlo solo cuando algo no funciona. Cada apartado empieza por lo que ves en pantalla.


Windows dice que ha protegido mi equipo y no me deja abrir el programa

Es el aviso habitual de Windows ante un programa que no lleva firma digital de una empresa registrada en Microsoft. No significa que el archivo tenga nada malo, solo que Windows no conoce a quien lo hizo. En ese aviso azul, pulsa en Más información y después en Ejecutar de todas formas. Windows recordará tu decisión.


El programa tarda mucho en abrirse la primera vez

Es normal. El antivirus revisa entero un ejecutable nuevo la primera vez que se abre. Las siguientes veces arranca mucho más rápido.


Sale un mensaje que dice que egui_glow necesita OpenGL 2.0 y el programa no se abre

El programa dibuja su ventana con OpenGL, y en ese ordenador no hay OpenGL 2.0 o superior. Pasa sobre todo en máquinas virtuales, cuyo adaptador gráfico virtual no lo ofrece, y en ordenadores sin el controlador de su tarjeta gráfica.

En Windows tiene un arreglo sencillo y gratuito: instala desde la Microsoft Store el complemento de Microsoft llamado OpenCL, OpenGL, and Vulkan Compatibility Pack, que añade OpenGL moderno a Windows. También se puede instalar desde PowerShell con esta orden:

winget install --id 9NQPSL29BFFF --source msstore

Después vuelve a abrir Gestor Agentes. Se ha comprobado en una máquina virtual de Hyper-V sin tarjeta gráfica: sin el complemento sale este mensaje, y con él la ventana se abre con normalidad. En un ordenador con tarjeta gráfica, lo primero es instalar o actualizar su controlador desde la página del fabricante.


Al arrancar dice que la configuración no es válida

La barra de estado dice Configuración no válida, seguido del motivo. Casi siempre es porque se ha editado a mano uno de los archivos de la carpeta datos, que está junto al ejecutable. La aplicación no lo sobrescribe, para no perder lo que hubieras escrito.

Abre el archivo que nombra el mensaje con un editor de texto, como el Bloc de notas, y busca el error. En perfiles-agentes.json suele ser una coma de más o de menos, una comilla sin cerrar, o un tipo de agente que no es Claude, Codex, Gemini ni Programator. En ajustes.json, el mensaje dice qué clave tiene un cero, y ninguna de las dos lo admite. Si no encuentras el error, cambia el nombre del archivo, por ejemplo añadiéndole .viejo al final, y la aplicación creará uno nuevo con los valores de serie: cinco perfiles, que son Claude 1, Claude 2, Codex 1, Gemini 1 y Programator 1, y los dos ajustes con sus valores por defecto.


No aparece ningún botón para abrir

La barra de estado dice: No se ha encontrado ningún agente. Indica la ruta de sus programas en la pestaña Agentes. Significa que la aplicación no ha encontrado instalado ninguno de los agentes. La lista de proyectos funciona igual, pero sin un agente no hay nada que abrir. Instala al menos uno, como explica el Manual de uso en su apartado Cómo instalar cada agente, cierra Gestor Agentes y vuelve a abrirlo.


No aparece el botón de un agente

La aplicación solo muestra los botones de los agentes que encuentra. Mira la pestaña Agentes: dice, para cada uno, si se ha encontrado y dónde, si no se ha encontrado, o si falta la carpeta de su cuenta.

Si acabas de instalar el agente con Gestor Agentes abierto, ciérralo y vuelve a abrirlo. Los instaladores añaden su carpeta a las rutas de búsqueda del sistema, el PATH, pero un programa que ya estaba en marcha no se entera del cambio. El botón Buscar de nuevo no basta en ese caso, porque busca con las rutas que el programa tenía al arrancar. La excepción son codex y agy en Windows instalados con sus instaladores oficiales: como la aplicación mira siempre en sus carpetas, basta con pulsar Buscar de nuevo.

Si el agente está instalado en una carpeta donde la aplicación no mira, escribe su ruta en la pestaña Agentes, o búscala con Examinar, y pulsa Guardar rutas. En Windows, la aplicación mira primero en las carpetas de los instaladores oficiales: AppData\Local\Programs\OpenAI\Codex\bin para codex y AppData\Local\agy\bin para agy. Después mira, para todos, en la carpeta .local\bin de tu carpeta de usuario; en Linux, luego en /usr/local/bin y /usr/bin; y por último en todas las carpetas del PATH. En Windows acepta los programas terminados en .exe y en .cmd. Programator no se busca nunca: hay que escribir siempre su ruta.

Si el estado dice Falta la carpeta de su cuenta, se trata de una segunda cuenta, como Claude 2, cuya carpeta todavía no existe. El Manual de uso explica cómo crearla en el apartado Dos cuentas del mismo agente.

Para comprobar a mano si un agente está instalado, abre una ventana de PowerShell o una terminal nueva y escribe su nombre seguido de --version, por ejemplo codex --version. Si contesta con un número de versión, está instalado y en el PATH. Si el sistema no reconoce la orden, no lo está, o su carpeta no está en el PATH.


Codex aparece como no encontrado aunque tengo Codex instalado

Lo más probable es que tengas la aplicación de escritorio de Codex, la que se descarga de la Microsoft Store. Gestor Agentes abre la versión de terminal de Codex, que es otro programa. La aplicación de la Store lleva dentro un archivo codex.exe, pero no funciona por su cuenta en una terminal: si lo ejecutas, responde que no tiene un paquete local completo y que instales la versión de terminal. Instala Codex de terminal con el instalador oficial de OpenAI, como explica el Manual de uso, y después cierra y vuelve a abrir Gestor Agentes.


Codex instalado con npm no recibe bien un argumento

Si instalaste Codex con npm en Windows, el programa que se ejecuta es un archivo intermedio llamado codex.cmd, y Windows lo pasa por su intérprete de órdenes clásico. Ese intérprete da un significado especial a algunos caracteres, como el ampersand y el signo de porcentaje. Solo te afecta si has añadido al perfil de Codex argumentos extra que lleven esos caracteres: en ese caso no le llegarían intactos a Codex. La carpeta del proyecto no corre ese riesgo. Si lo necesitas, instala Codex con el instalador oficial de OpenAI, que deja un codex.exe sin ese intermediario.


Al abrir Codex aparece el mensaje No active thread is available

Es un mensaje del propio Codex, no de Gestor Agentes. Puede salir la primera vez que Codex se abre en una carpeta. Codex funciona igual, puedes escribirle con normalidad, y el mensaje no vuelve a salir las siguientes veces. Para evitarlo, espera a que Codex termine de arrancar y conteste a su pregunta sobre si confías en la carpeta antes de escribirle nada.


Al pulsar un botón sale una ventana que dice que el agente no está instalado

El programa del agente estaba cuando arrancó Gestor Agentes, pero ya no está donde se encontró, por ejemplo porque se ha desinstalado o actualizado mientras tanto. Los archivos del proyecto se han preparado igualmente. Cierra y vuelve a abrir Gestor Agentes para que lo busque de nuevo, y mira la pestaña Agentes.


Al pulsar un botón sale una ventana titulada Perfil incompleto

Al perfil de ese agente le falta un dato imprescindible. Casi siempre es la ruta de Programator, que no se busca solo. Escríbela en la pestaña Agentes y pulsa Guardar rutas. La ventana dice también en qué archivo de la carpeta datos está el perfil, por si prefieres corregirlo a mano.


Las instrucciones de un proyecto no incluyen las normas de mi equipo

El bloque de entorno de cada proyecto se copia del archivo CLAUDE.md, AGENTS.md o GEMINI.md de la carpeta superior más cercana que no sea de otro proyecto gestionado. Si el bloque dice que no se ha encontrado nada, crea un CLAUDE.md en la carpeta que contiene tus proyectos y vuelve a abrir el proyecto. Si se ha copiado otro archivo, es que hay uno más cerca del proyecto: el encabezado del bloque dice de cuál se ha copiado.


Al abrir un proyecto aparece una ventana con un informe

La ventana se titula Preparación, seguido del nombre del proyecto, y sale cuando la aplicación ha hecho algo que conviene que sepas. Tiene hasta tres apartados: Errores, Avisos y Cambios. Ninguno impide trabajar.

Si habla de una copia exacta o de un archivo respaldado, es que AGENTS.md o CLAUDE.md tenían texto que no había escrito la aplicación: instrucciones de una versión anterior, o algo que alguien escribió a mano. No se ha perdido nada: el original está en la carpeta .gestor\respaldos del proyecto. Si el proyecto venía de una versión anterior, lo propio del proyecto está además en .gestor\instrucciones-del-proyecto.md, dentro de un bloque pendiente de revisar. No tienes que hacer nada: el primer agente que arranque en el proyecto lo revisa por su cuenta antes de ponerse con otra cosa, y te preguntará si alguna regla le plantea dudas. Lo que quieras añadir a las instrucciones de un proyecto, escríbelo en ese archivo, no en AGENTS.md.

Si habla de archivos llevados al archivo, es que el proyecto tenía archivos del sistema de trabajo anterior, como GEMINI.md o la memoria compartida. Están ahora en la carpeta .gestor\archivo del proyecto, con la ruta que dice el informe. El agente que revise las instrucciones mira si alguno guardaba una tarea a medias y la recoge.

Si dice que una bandeja mide más de lo debido, es que el buzón de algún agente en .gestor\canal ha crecido sin limpiarse. El aviso dice qué archivo es, cuánto mide y el límite, que son 16384 bytes si no lo has cambiado. Díselo a ese agente para que pase lo ya resuelto a su histórico. El aviso sale abras el agente que abras, porque es del proyecto, y deja de salir en cuanto la bandeja vuelve a su tamaño.

Si dice que las marcas estaban dañadas, un archivo de instrucciones antiguo tenía las líneas de marca mal puestas. Como la aplicación no sabía con seguridad qué parte era suya, ha guardado la copia del original en .gestor\respaldos y ha pasado el texto entero al bloque pendiente de revisar, sin descartar nada. El agente que revise el bloque quitará lo que sobre.

Si dice que no se ha podido comprobar git, el proyecto se ha preparado igualmente, pero sin repositorio. Instala git y vuelve a abrir el proyecto.


La barra de estado dice AVISO y que AGENTS.md mide más de lo que lee el agente

Sale al abrir Codex o Gemini, que solo leen el principio de su archivo de instrucciones: Codex los primeros 32768 bytes y Gemini los primeros 24000. Significa que ese agente no verá el final de sus instrucciones, que es donde van las normas de tu equipo. Normalmente no pasa, porque la aplicación escribe AGENTS.md por debajo de esos tamaños. Si sale, acorta el CLAUDE.md de la carpeta que contiene tus proyectos, que es lo que se copia al final.


Git dice que la carpeta tiene un propietario dudoso

Ocurre con proyectos en algunas unidades, por ejemplo las de red, cuyo sistema de archivos no informa del propietario como git espera. La aplicación ya lo tiene en cuenta en sus propias órdenes, pero si quieres usar git a mano en esa carpeta, autorízala una vez. La orden es git config --global --add safe.directory, seguida de un espacio y la ruta de la carpeta.


Quiero abrir la aplicación de escritorio de Codex o de Antigravity

Gestor Agentes abre todos los agentes en una terminal: Codex con su programa codex y Gemini con agy. Las aplicaciones de escritorio se abren desde su propio acceso directo.


He copiado el programa a otro ordenador y la lista de proyectos está vacía

Se ha copiado la carpeta del programa sin el contenido de datos, que es donde viaja la configuración. Copia también esa carpeta desde el ordenador de origen. Si ese ordenador nuevo tenía Gestor Claude, la lista que ves es la suya, copiada la primera vez que abriste Gestor Agentes.


En Linux no se abre la ventana o dice que falta una biblioteca

La ventana necesita un escritorio gráfico, X11 o Wayland, con OpenGL. Si al ejecutarlo desde una terminal dice que no tiene permiso, dáselo una vez con chmod +x gestor-agentes. Si dice que falta una biblioteca, la orden ldd gestor-agentes muestra cuál; instálala con el gestor de paquetes de tu distribución.


La lista muestra mis proyectos pero al abrirlos dice que la carpeta no existe

Comprueba que las carpetas siguen existiendo en las rutas que muestra la columna Ruta. Pasa a menudo después de llevarte la carpeta datos a otro ordenador, o si has movido o renombrado una carpeta. Quita la entrada que ya no vale con el botón Quitar, que no borra nada del disco, y vuelve a añadir el proyecto desde donde esté ahora.


Un proyecto aparece como Sin configurar o Formato antiguo

No es un fallo. Sin configurar indica que el proyecto no tiene todavía archivos de instrucciones; Formato antiguo, que tiene instrucciones que no escribió esta versión, por ejemplo de una versión anterior o escritas a mano. Las dos cosas se arreglan solas la primera vez que lo abras con cualquier botón.


La barra de estado dice que el hilo de trabajo se ha detenido

La aplicación hace su trabajo pesado, como preparar un proyecto o buscar agentes, en segundo plano, para que la ventana no se congele. Si esa parte deja de responder, la barra lo dice y pide reiniciar la aplicación. Ciérrala y vuelve a abrirla. Tus datos no se pierden, porque se guardan en cuanto cambian.


Quiero desinstalar el programa

No hay nada que desinstalar. Basta con borrar la carpeta del programa; con ella se va tu configuración, que está en datos. Si piensas volver a usarlo, guarda antes una copia de esa carpeta. Nada de esto afecta a tus proyectos.
