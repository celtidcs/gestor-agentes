Infraestructura: visión general


Todo se ejecuta en local

Gestor Agentes no tiene servicios en la nube, contenedores ni interfaces de programación propias. Es un ejecutable que lee y escribe archivos en su carpeta datos y en las carpetas de los proyectos, y que lanza procesos locales: git y el programa del agente elegido. Lo único que sale a la red son las conexiones que esos agentes hacen por su cuenta con sus proveedores.


Componentes

El ejecutable es gestor-agentes.exe en Windows y gestor-agentes en Linux. Lo generan build-portable.ps1 y build-portable.sh dentro de la carpeta portable, junto con la documentación para el usuario.

La configuración del usuario está en la carpeta datos, junto al ejecutable. config.json guarda la lista de proyectos; perfiles-agentes.json, los perfiles de agente con su tipo, su instancia y su carpeta de configuración; y ajustes.json, el tiempo máximo de git y el límite de tamaño de una bandeja. La primera vez que se abre, si la carpeta datos no tiene todavía config.json ni perfiles-agentes.json, la aplicación copia allí la configuración de Gestor Claude, la versión anterior, que estaba en %APPDATA%\GestorClaude en Windows y en ~/.config/GestorClaude en Linux. La copia original no se toca.

Las normas del espacio de trabajo son el CLAUDE.md, AGENTS.md o GEMINI.md de la carpeta que contiene los proyectos, si existe. Recogen las normas comunes a todos ellos, y la aplicación las copia al final del AGENTS.md de cada proyecto al abrir un agente. Puede haber también un claude-instructions.txt de versiones anteriores, que solo se lee.

Claude Code se abre en una terminal con el programa claude, que la aplicación busca primero en la carpeta .local/bin de la carpeta personal, en Linux luego en /usr/local/bin y /usr/bin, y por último en el PATH. En Windows prueba las extensiones .exe y .cmd. Codex se abre igual, con el programa codex, que se busca en los mismos sitios, y en Windows antes que en ninguno en %LOCALAPPDATA%\Programs\OpenAI\Codex\bin. Gemini se abre con agy, que en Windows se busca antes que en ningún otro sitio en %LOCALAPPDATA%\agy\bin. Programator se abre con la ruta de su perfil. Si un perfil lleva la ruta de su programa, se usa solo esa. La aplicación hace esta misma búsqueda al arrancar para decidir qué botones muestra, y reconoce las cuentas instaladas por sus carpetas: .claude, .claude2, .codex, .codex2 y las demás con el mismo patrón. Cada perfil puede apuntar a su propia carpeta de configuración, por ejemplo .claude y .claude2 para dos cuentas de Claude.

Git debe estar instalado para que la aplicación cree repositorios en los proyectos. Sin git la aplicación sigue funcionando y lo avisa en el informe de preparación.

La aplicación no abre las aplicaciones de escritorio de Codex ni de Antigravity, solo los programas de terminal. Si un programa no se encuentra, su botón no aparece, y la pestaña Agentes dice por qué.

En cada proyecto, la aplicación mantiene en la raíz AGENTS.md, el único archivo de reglas, y CLAUDE.md, el puntero con el que Claude Code lo carga; añade la entrada .gestor/ al .gitignore y crea la carpeta .gestor. En ella están el canal de los agentes; el archivo propio del proyecto, instrucciones-del-proyecto.md, que la aplicación solo crea si falta; los documentos de consulta, consulta/codigo.md y consulta/metodo.md; los respaldos; el archivo de lo que usaba el sistema anterior; y el registro de huellas, regiones-gestionadas.json. GEMINI.md ya no se genera.


Variables de entorno que fija la aplicación al abrir un agente

GESTOR_AGENTE lleva la identidad del agente, por ejemplo Claude 2, Codex 1 o Gemini 1. Las instrucciones piden a cada agente que la lea para saber quién es y qué bandeja del canal le corresponde.

CLAUDE_CONFIG_DIR, solo para Claude Code, apunta a la carpeta de configuración del perfil.

CODEX_HOME hace lo mismo para Codex, cuando el perfil tiene carpeta propia.

CLAUDECODE se deja vacía, para que Claude Code no crea que se ejecuta dentro de otra sesión suya si Gestor Agentes se abrió desde una.

Ninguna de estas variables hay que configurarla en el sistema: la aplicación las fija solo para el proceso que abre.


Sistema de ventanas en Linux

La interfaz se compila con soporte para X11 y para Wayland, y elige al arrancar el que haya, sin decidir nada al compilar. El detalle está en architecture.md.
