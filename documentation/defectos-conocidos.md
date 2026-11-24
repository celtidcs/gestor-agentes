Defectos conocidos

Este documento recoge los fallos y los límites conocidos que todavía no se han corregido, con lo que se sabe de cada uno y cómo evitarlo. Cuando uno se resuelve se quita de aquí y se anota en changelog.md.


La ventana no se abre en un Windows sin OpenGL 2.0

La ventana se dibuja con OpenGL y necesita la versión 2.0 o superior. El controlador de cualquier tarjeta gráfica actual la da, pero el adaptador básico de Windows, el que usan muchas máquinas virtuales y algunos escritorios remotos, solo da OpenGL 1.1. En ese caso la aplicación muestra el mensaje «egui_glow: OpenGL: egui_glow requires opengl 2.0+.» y termina.

Se arregla instalando el complemento gratuito de Microsoft OpenCL, OpenGL, and Vulkan Compatibility Pack, desde la Microsoft Store o con la orden winget install --id 9NQPSL29BFFF --source msstore. Se ha comprobado en una máquina virtual de Hyper-V con el adaptador «Microsoft Hyper-V Video»: sin el complemento sale el mensaje, y con él la ventana se abre y funciona.

En Linux no pasa, porque Mesa da OpenGL por software cuando no hay tarjeta gráfica.

Se probó también dibujar con wgpu sobre Direct3D 12, y en ese mismo equipo el proceso quedaba en marcha sin mostrar ninguna ventana ni ningún error, que es peor. Por eso se mantiene OpenGL, que al menos falla con un mensaje claro.


La terminal de Claude Code hereda una salida redirigida en Windows

Si Gestor Agentes se arranca desde un guion con su salida redirigida a un archivo, la terminal de PowerShell que abre escribe en ese archivo y no en su ventana. Arrancada de la forma normal, desde el Explorador o desde el menú Inicio, no pasa: se ha comprobado que la terminal abre con la entrada y la salida propias. Corregirlo exigiría abrir la consola sin heredar los manejadores del proceso padre, y hoy solo se sabe hacer con código unsafe, que el proyecto prohíbe. Queda como límite: no arranques la aplicación con la salida redirigida.


agy no se abre si no se entienden sus proyectos

Antigravity guarda sus proyectos en archivos de la carpeta .gemini/config/projects de tu carpeta personal, y la aplicación busca ahí el que tiene la carpeta del proyecto para abrir agy con él. Si ninguno la tiene, abre agy con su opción --new-project, que crea un proyecto para esa carpeta, como se ha comprobado con agy 1.5.14. Ese formato no está documentado: es el que se ha visto con agy 1.5.12 y 1.5.14. Si una versión futura de agy lo cambia y la aplicación no entiende ninguno de esos archivos, no abre agy y lo dice, en lugar de crear un proyecto nuevo cada vez o dejarlo trabajar en otro. Se resuelve abriendo una vez la carpeta como proyecto en Antigravity.


La entrega de avisos a Codex depende de codex queue

El vigilante entrega cada aviso a Codex con la orden codex queue, que existe en Codex CLI 0.167.1 y que la ayuda del propio programa describe como experimental en parte. La sesión se reconoce por la variable CODEX_THREAD_ID que Codex pone a las órdenes que ejecuta, que no aparece en su documentación. Si una versión de Codex cambia cualquiera de las dos cosas, el vigilante lo anota en .gestor/<agente>/vigilante/avisos.log como entrega no acreditada, y el agente tiene que mirar el canal por su cuenta con la opción --una-vez. Con Codex CLI 0.171.0, la ayuda de codex queue es idéntica a la de la 0.167.1 y sus notas de versión no anuncian cambios en esa orden; la variable de la sesión solo se puede comprobar con una sesión abierta, al usar la aplicación.

Con Codex CLI 0.167.1, al encolar el aviso Codex intenta reanudar la sesión, y como la sesión ya está abierta avisa de que no puede. El aviso queda en la cola y la sesión lo recoge sola al terminar su turno, como se ha comprobado en vivo varias veces. El vigilante lo anota entonces como entregado en la cola de la sesión abierta, con la advertencia de Codex entre paréntesis, y solo como no acreditado si falta la confirmación de la cola o aparece cualquier otra advertencia.


Codex en Windows muestra un error de PowerShell en cada orden

Con su entorno aislado de Windows, cada orden de PowerShell que ejecuta Codex CLI 0.167.1 escribe al empezar un error de la operación InitializeDefaultDrives. En las pruebas no le impidió leer, escribir ni usar la red, pero ensucia la salida de cada orden. Tampoco se ha podido comprobar desde fuera si Codex usó MXC o su modo de respaldo, y el vigilante que lanza la aplicación desde la terminal de Codex está probado con pruebas automáticas, pero no todavía en una sesión real. Con el modo anterior, que pedía aprobación cuando hacía falta, la preparación del entorno aislado fallaba a veces con el mensaje «setup refresh had errors» y Codex pedía permiso para repetir la orden; con el modo actual no se ha visto ese fallo.


Codex ejecuta las órdenes en un servicio compartido

Codex CLI 0.167.1 no ejecuta las órdenes de una sesión desde la terminal que la abrió, sino desde un servicio de fondo que comparten todas sus sesiones del equipo, y ese servicio conserva las variables de la sesión que lo arrancó. Por eso la aplicación pasa la identidad y la carpeta del proyecto a cada sesión de Codex como configuración de la propia sesión, que sí llega a sus órdenes. La carpeta de cada cuenta de Codex sigue yendo solo como variable de la terminal, CODEX_HOME, que se pone con su carpeta o, en Codex 1, se quita: no está comprobado qué hace el servicio compartido con dos sesiones de cuentas distintas.


Codex instalado con npm en Windows

Si el programa codex que se encuentra es codex.cmd, el que instala npm, PowerShell lo ejecuta a través de cmd.exe, y cmd.exe interpreta caracteres como & o % en los argumentos. Desde la 3.6.0 la ruta del proyecto también va en los argumentos de Codex, para que llegue a sus órdenes, así que una carpeta de proyecto con & o % en el nombre, igual que un argumento del perfil con esos caracteres, no llegaría intacta a codex. En ese caso, instala codex con su instalador oficial, que deja un codex.exe, o fija en el perfil la ruta de un codex.exe.


Un agente cuya herramienta no le deja vigilar no recibe aviso de los mensajes nuevos

El vigilante del canal es el programa gestor-vigilante, que la aplicación deja en la carpeta .gestor/vigilante de cada proyecto. Claude Code y agy lo dejan en segundo plano con la opción --hasta-cambio, y el vigilante los despierta cuando hay un encargo para ellos o una mutación viva nueva; después de atenderlo lo vuelven a lanzar. Para Codex lo deja en marcha la aplicación al abrirlo, desde su terminal; Codex solo le anota su sesión al arrancar, con --anotar-hilo, y el vigilante le entrega cada aviso dentro de ella. Si la herramienta de un agente no le deja vigilar así, o no lo despierta, el agente usa el modo de una pasada, --una-vez, al terminar cada paso, y lo dice en su bandeja. Entonces, si está parado esperando, no se entera hasta que le escribes. Dile que mire el canal. El panel del equipo ayuda a verlo: un agente sin la nota vigila junto a su latido no tiene en ese momento ningún vigilante en marcha.


Las sesiones abiertas con una versión anterior siguen vigilando a la antigua

Cada agente lee sus instrucciones al arrancar, así que una sesión que se abrió con una versión anterior a la 3.6.0 sigue intentando lanzar los guiones del vigilante anterior, aunque la aplicación ya los haya retirado del proyecto, y no recibe los avisos del vigilante nuevo. Después de actualizar, cierra del todo las sesiones de agente abiertas, también las que creas paradas, y vuelve a abrirlas desde la aplicación. Para comprobar que no queda ningún vigilante de más, en PowerShell, la orden (Get-Process -Name 'gestor-vigilante' -ErrorAction SilentlyContinue).Count debe dar como mucho uno por cada agente que esté trabajando.


En Windows, Claude Code toma la bandeja claude.md por un archivo de instrucciones

La bandeja de Claude 1 se llama claude.md. Windows no distingue mayúsculas de minúsculas, y Claude Code carga como instrucciones el CLAUDE.md de las carpetas cuyos archivos lee, así que al leer el canal puede tomar esa bandeja por un archivo de instrucciones de la carpeta .gestor/canal. Se ha visto en una sesión de Claude Code en Windows. No cambia lo que hace el agente, porque la bandeja solo contiene sus propios mensajes, pero los mete en su contexto una vez más. En Linux no pasa.


Claude Code lee dos veces las normas comunes

Claude Code carga por su cuenta el CLAUDE.md de las carpetas superiores, y además la aplicación copia esas normas al final de AGENTS.md, que Claude Code carga a través de su CLAUDE.md. No causa contradicciones, porque el texto es el mismo, pero ocupa algo más de contexto. Se copian igualmente porque Codex y Gemini no las leerían de otro modo. Si las dos copias llegan a diferir, manda la original, y la copia se pone al día en la siguiente apertura.


Los proyectos que vienen de una versión anterior reciben un bloque pendiente

Casi todos los proyectos preparados con Gestor Claude recibirán, la primera vez que se abran, un bloque pendiente de consolidar en su .gestor/instrucciones-del-proyecto.md. Sus archivos de instrucciones venían de plantillas que fueron cambiando y no coinciden exactamente con las que la aplicación reconoce como suyas, así que se recoge su texto en vez de descartarlo. No se pierde nada, porque hay copia en .gestor/respaldos, y el primer agente que arranque en cada proyecto lo consolida por su cuenta.


Las respuestas de Programator 0.12 no despiertan a quien le encargó el trabajo

Gestor Agentes despierta al agente que hizo un encargo a Programator cuando la respuesta empieza con una línea Para dirigida a él, pero Programator 0.12 y las versiones anteriores todavía no escriben esa línea. Hasta que Programator la incorpore, el agente que le encarga algo no recibe aviso de la respuesta. Mientras tanto, la guía que Gestor Agentes deja en cada proyecto le indica que espere mirando el estado de Programator, que vuelve a reposo al terminar, y la línea con la que Programator confirma que leyó el encargo.


En Linux no se ve la versión de Programator

La versión del Programator de la ruta se lee del recurso de versión de su ejecutable, que solo llevan los programas de Windows. En Linux la pestaña Agentes no puede decir qué versión tienes ni avisar de que hay otra más nueva; para saberlo, mira la página de versiones de Programator en GitHub.


No se puede quitar un agente desde la ventana

La pestaña Agentes no tiene un botón para quitar un agente de la lista. Desde la versión 3.9.0 esto se nota más, porque una copia de una cuenta guardada con un nombre que empieza por .claude o por .codex aparece como un agente nuevo y se queda en la lista. Mientras no exista ese botón, el agente se quita a mano: con la aplicación cerrada, se borra su bloque en perfiles-agentes.json, en la carpeta datos. El manual de uso lo explica paso a paso en el apartado Si guardas copias de cuentas.
