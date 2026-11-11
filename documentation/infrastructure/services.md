Programas externos

Gestor Agentes no se conecta a ningún servicio. Trabaja con los programas siguientes, que se ejecutan como procesos locales y se ocupan por su cuenta de conectarse con sus proveedores. Cómo instalar cada agente con el método oficial de su fabricante se explica en el manual de uso, usage.md.


Claude Code, de Anthropic

Es el agente de programación de Anthropic. Su instalador oficial lo deja en .local/bin, dentro de la carpeta personal, que es el primer sitio donde lo busca la aplicación. La variable CLAUDE_CONFIG_DIR elige su carpeta de configuración, y Gestor Agentes la fija según el perfil, de modo que cada perfil puede usar una cuenta distinta. Las cuentas se inician y se gestionan dentro de Claude Code, no desde Gestor Agentes. Claude Code no lee AGENTS.md por su cuenta; por eso la aplicación escribe CLAUDE.md, que se lo indica.


Codex, de OpenAI

Es el agente de programación de OpenAI. Gestor Agentes lo abre en una terminal con su programa de línea de órdenes, codex, que busca en .local/bin de la carpeta personal y en el PATH. En Windows vale también el lanzador codex.cmd que instala npm, con el límite que se describe en defectos-conocidos.md. El programa codex.exe que lleva dentro la aplicación de escritorio de Codex no funciona por sí solo, así que hay que instalar la versión de terminal. Su carpeta de configuración se elige con CODEX_HOME. Lee sus instrucciones de AGENTS.md.


Gemini, con agy

Gemini trabaja con agy, el programa de línea de órdenes de Google Antigravity. Los perfiles de tipo Gemini abren agy en una terminal, en la carpeta del proyecto. En Windows se busca en %LOCALAPPDATA%\agy\bin, que es donde lo deja su instalador oficial, en .local/bin de la carpeta personal y en el PATH; en Linux, en los mismos sitios que claude. La aplicación de escritorio de Antigravity no se abre desde Gestor Agentes. agy lee sus instrucciones de AGENTS.md, como Codex, así que la aplicación ya no genera GEMINI.md.


Programator

Programator no tiene un sitio habitual de instalación, así que la aplicación no lo busca: se abre con la ruta del ejecutable que se indica en la pestaña Agentes.


Git

Gestor Agentes usa git para comprobar si cada proyecto es un repositorio y crearlo si no lo es. Solo ejecuta rev-parse e init, siempre con la opción safe.directory en la propia orden, y nunca modifica la configuración global de git. No hace commits ni envía nada a ningún repositorio remoto: eso lo deciden el usuario y sus agentes.
