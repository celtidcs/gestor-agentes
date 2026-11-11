Defectos conocidos

Este documento recoge los fallos y los límites conocidos que todavía no se han corregido, con lo que se sabe de cada uno y cómo evitarlo. Cuando uno se resuelve se quita de aquí y se anota en changelog.md.


La ventana no se abre en un Windows sin OpenGL 2.0

La ventana se dibuja con OpenGL y necesita la versión 2.0 o superior. El controlador de cualquier tarjeta gráfica actual la da, pero el adaptador básico de Windows, el que usan muchas máquinas virtuales y algunos escritorios remotos, solo da OpenGL 1.1. En ese caso la aplicación muestra el mensaje «egui_glow: OpenGL: egui_glow requires opengl 2.0+.» y termina.

Se arregla instalando el complemento gratuito de Microsoft OpenCL, OpenGL, and Vulkan Compatibility Pack, desde la Microsoft Store o con la orden winget install --id 9NQPSL29BFFF --source msstore. Se ha comprobado en una máquina virtual de Hyper-V con el adaptador «Microsoft Hyper-V Video»: sin el complemento sale el mensaje, y con él la ventana se abre y funciona.

En Linux no pasa, porque Mesa da OpenGL por software cuando no hay tarjeta gráfica.

Se probó también dibujar con wgpu sobre Direct3D 12, y en ese mismo equipo el proceso quedaba en marcha sin mostrar ninguna ventana ni ningún error, que es peor. Por eso se mantiene OpenGL, que al menos falla con un mensaje claro.


La terminal de Claude Code hereda una salida redirigida en Windows

Si Gestor Agentes se arranca desde un guion con su salida redirigida a un archivo, la terminal de PowerShell que abre escribe en ese archivo y no en su ventana. Arrancada de la forma normal, desde el Explorador o desde el menú Inicio, no pasa: se ha comprobado que la terminal abre con la entrada y la salida propias. Corregirlo exigiría abrir la consola sin heredar los manejadores del proceso padre, y hoy solo se sabe hacer con código unsafe, que el proyecto prohíbe. Queda como límite: no arranques la aplicación con la salida redirigida.


Codex instalado con npm en Windows

Si el programa codex que se encuentra es codex.cmd, el que instala npm, PowerShell lo ejecuta a través de cmd.exe, y cmd.exe interpreta caracteres como & o % en los argumentos que añade el perfil. La ruta del proyecto no corre ese riesgo, porque no va en los argumentos. Pero un argumento del perfil con esos caracteres no llegaría intacto a codex. Si necesitas pasar argumentos así, instala codex con su instalador oficial, que deja un codex.exe, o fija en el perfil la ruta de un codex.exe.


Claude Code lee dos veces las normas comunes

Claude Code carga por su cuenta el CLAUDE.md de las carpetas superiores, y además la aplicación copia esas normas al final de AGENTS.md, que Claude Code carga a través de su CLAUDE.md. No causa contradicciones, porque el texto es el mismo, pero ocupa algo más de contexto. Se copian igualmente porque Codex y Gemini no las leerían de otro modo. Si las dos copias llegan a diferir, manda la original, y la copia se pone al día en la siguiente apertura.


Los proyectos que vienen de una versión anterior reciben un bloque pendiente

Casi todos los proyectos preparados con Gestor Claude recibirán, la primera vez que se abran, un bloque pendiente de consolidar en su .gestor/instrucciones-del-proyecto.md. Sus archivos de instrucciones venían de plantillas que fueron cambiando y no coinciden exactamente con las que la aplicación reconoce como suyas, así que se recoge su texto en vez de descartarlo. No se pierde nada, porque hay copia en .gestor/respaldos, y el primer agente que arranque en cada proyecto lo consolida por su cuenta.
