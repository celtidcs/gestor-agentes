Infraestructura: visión general


Todo se ejecuta en local

Gestor Agentes no tiene servicios en la nube, contenedores ni interfaces de programación propias. Es un ejecutable que lee y escribe archivos en su carpeta datos y en las carpetas de los proyectos, y que lanza procesos locales: git, el vigilante del canal, gestor-vigilante, y el programa del agente elegido. Su única salida a la red son dos peticiones HTTPS al arrancar a la API de GitHub, para saber si hay una versión nueva de Gestor Agentes y de Programator, que se desactivan con ComprobarActualizaciones en false en ajustes.json y que services.md describe con detalle. El resto del tráfico de red son las conexiones que los propios agentes hacen por su cuenta con sus proveedores.


Componentes

El ejecutable es gestor-agentes.exe en Windows y gestor-agentes en Linux. A su lado va el vigilante del canal, gestor-vigilante.exe en Windows y gestor-vigilante en Linux, que la aplicación copia a cada proyecto. Los dos los generan build-portable.ps1 y build-portable.sh dentro de la carpeta portable, junto con LEEME.txt y la carpeta documentacion para el usuario, que lleva el Manual de uso, Resolucion de problemas, la Referencia de funcionalidades y Licencia.txt.

La configuración del usuario está en la carpeta datos, junto al ejecutable. config.json guarda la lista de proyectos; perfiles-agentes.json, los perfiles de agente con su tipo, su instancia y su carpeta de configuración; perfiles-agentes.version, el número con que la aplicación sabe si tiene que poner al día los perfiles; y ajustes.json, los ajustes de funcionamiento: el tiempo máximo de git, el límite de tamaño de una bandeja, las horas sin latido a partir de las cuales una reserva puede estar abandonada, si se comprueba al arrancar si hay versión nueva, los repositorios de GitHub de Gestor Agentes y de Programator y el tiempo máximo de esa comprobación. Si algún proyecto propone una regla como aprendizaje común, aparece además candidatas-comunes.md. La primera vez que se abre, si la carpeta datos no tiene todavía config.json ni perfiles-agentes.json, la aplicación copia allí la configuración de Gestor Claude, la versión anterior, que estaba en %APPDATA%\GestorClaude en Windows y en ~/.config/GestorClaude en Linux. La copia original no se toca.

Las normas del espacio de trabajo son el CLAUDE.md, AGENTS.md o GEMINI.md de la carpeta que contiene los proyectos, si existe. Recogen las normas comunes a todos ellos, y la aplicación las copia al final del AGENTS.md de cada proyecto al abrir un agente. Puede haber también un claude-instructions.txt de versiones anteriores, que solo se lee.

Claude Code se abre en una terminal con el programa claude, que la aplicación busca primero en la carpeta .local/bin de la carpeta personal, en Linux luego en /usr/local/bin y /usr/bin, y por último en el PATH. En Windows prueba las extensiones .exe y .cmd. Codex se abre igual, con el programa codex, que se busca en los mismos sitios, y en Windows antes que en ninguno en %LOCALAPPDATA%\Programs\OpenAI\Codex\bin. Gemini se abre con agy, que en Windows se busca antes que en ningún otro sitio en %LOCALAPPDATA%\agy\bin. Programator se abre con la ruta de su perfil. Si un perfil lleva la ruta de su programa, se usa solo esa. La aplicación hace esta misma búsqueda al arrancar para decidir qué botones muestra, y reconoce las cuentas instaladas por sus carpetas: .claude, .claude2, .codex, .codex2 y las demás con el mismo patrón. Cada perfil puede apuntar a su propia carpeta de configuración, por ejemplo .claude y .claude2 para dos cuentas de Claude.

Git debe estar instalado para que la aplicación cree repositorios en los proyectos. Sin git la aplicación sigue funcionando y lo avisa en el informe de preparación.

La aplicación no abre las aplicaciones de escritorio de Codex ni de Antigravity, solo los programas de terminal. Si un programa no se encuentra, su botón no aparece, y la pestaña Agentes dice por qué.

En cada proyecto, la aplicación mantiene en la raíz AGENTS.md, el único archivo de reglas, y CLAUDE.md, el puntero con el que Claude Code lo carga; añade la entrada .gestor/ al .gitignore y crea la carpeta .gestor. En ella están el canal de los agentes; el archivo propio del proyecto, instrucciones-del-proyecto.md, que la aplicación solo crea si falta; los documentos de consulta, consulta/codigo.md, consulta/verificacion.md, consulta/metodo.md, consulta/programator.md y consulta/aprendizaje-comun.md; la carpeta vigilante, con la copia de gestor-vigilante; los avisos de la última preparación para los agentes, avisos-de-preparacion.md, mientras quede alguno; los respaldos; el archivo de lo que usaba el sistema anterior; y el registro de huellas, regiones-gestionadas.json. GEMINI.md ya no se genera.


Variables de entorno que fija la aplicación al abrir un agente

GESTOR_AGENTE lleva la identidad del agente, por ejemplo Claude 2, Codex 1 o Gemini 1. Las instrucciones piden a cada agente que la lea para saber quién es y qué bandeja del canal le corresponde.

GESTOR_PROYECTO, para Claude Code, Codex y agy, lleva la carpeta del proyecto. Cada agente comprueba al arrancar que está en ella y, si no, para sin leer ni escribir nada. A Codex se le pasan además las dos, GESTOR_AGENTE y GESTOR_PROYECTO, como configuración de su sesión, para que lleguen a las órdenes que ejecuta.

CLAUDE_CONFIG_DIR, solo para Claude Code, apunta a la carpeta de configuración del perfil.

CODEX_HOME hace lo mismo para Codex, cuando el perfil tiene carpeta propia.

CLAUDECODE se deja vacía, para que Claude Code no crea que se ejecuta dentro de otra sesión suya si Gestor Agentes se abrió desde una.

Ninguna de estas variables hay que configurarla en el sistema: la aplicación las fija solo para el proceso que abre.


Sistema de ventanas en Linux

La interfaz se compila con soporte para X11 y para Wayland, y elige al arrancar el que haya, sin decidir nada al compilar. El detalle está en architecture.md.
