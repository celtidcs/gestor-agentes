Programas externos

Gestor Agentes solo se conecta a un servicio, la API de GitHub, y solo para saber si hay una versión nueva, como se explica en el último apartado. Trabaja además con los programas siguientes, que se ejecutan como procesos locales y se ocupan por su cuenta de conectarse con sus proveedores. Cómo instalar cada agente con el método oficial de su fabricante se explica en el manual de uso, usage.md.


Claude Code, de Anthropic

Es el agente de programación de Anthropic. Su instalador oficial lo deja en .local/bin, dentro de la carpeta personal, que es el primer sitio donde lo busca la aplicación. La variable CLAUDE_CONFIG_DIR elige su carpeta de configuración, y Gestor Agentes la fija según el perfil, de modo que cada perfil puede usar una cuenta distinta. Las cuentas se inician y se gestionan dentro de Claude Code, no desde Gestor Agentes. Claude Code no lee AGENTS.md por su cuenta; por eso la aplicación escribe CLAUDE.md, que se lo indica.


Codex, de OpenAI

Es el agente de programación de OpenAI. Gestor Agentes lo abre en una terminal con su programa de línea de órdenes, codex, que en Windows busca primero en %LOCALAPPDATA%\Programs\OpenAI\Codex\bin, donde lo deja su instalador oficial, y después, en los dos sistemas, en .local/bin de la carpeta personal, en Linux luego en /usr/local/bin y /usr/bin, y por último en el PATH. En Windows vale también el lanzador codex.cmd que instala npm, con el límite que se describe en defectos-conocidos.md. El programa codex.exe que lleva dentro la aplicación de escritorio de Codex no funciona por sí solo, así que hay que instalar la versión de terminal. Su carpeta de configuración se elige con CODEX_HOME. Lee sus instrucciones de AGENTS.md.


Gemini, con agy

Gemini trabaja con agy, el programa de línea de órdenes de Google Antigravity. Los perfiles de tipo Gemini abren agy en una terminal, en la carpeta del proyecto. En Windows se busca en %LOCALAPPDATA%\agy\bin, que es donde lo deja su instalador oficial, en .local/bin de la carpeta personal y en el PATH; en Linux, en los mismos sitios que claude. La aplicación de escritorio de Antigravity no se abre desde Gestor Agentes. agy lee sus instrucciones de AGENTS.md, como Codex, así que la aplicación ya no genera GEMINI.md.


Programator

Programator no tiene un sitio habitual de instalación, así que la aplicación no lo busca: se abre con la ruta del ejecutable que se indica en la pestaña Agentes.


Git

Gestor Agentes usa git para comprobar si cada proyecto es un repositorio y crearlo si no lo es. Solo ejecuta rev-parse e init, siempre con la opción safe.directory en la propia orden, y nunca modifica la configuración global de git. No hace commits ni envía nada a ningún repositorio remoto: eso lo deciden el usuario y sus agentes.


API de GitHub

Al arrancar, si ComprobarActualizaciones vale true en datos/ajustes.json, la aplicación hace dos peticiones GET, sin autenticación, a https://api.github.com/repos/<repositorio>/releases/latest: una para Gestor Agentes, donde el repositorio es RepositorioDeActualizaciones, celtidcs/gestor-agentes por defecto, y otra para Programator, donde es RepositorioDeProgramator, celtidcs/Programator por defecto, para comparar su última versión con la del Programator de su perfil. Es la operación pública Get the latest release de la API de releases (https://docs.github.com/en/rest/releases/releases#get-the-latest-release). La hace la biblioteca aviso-version-github, en su versión 0.2.3, incluida en bibliotecas/. Aparte de las que exige el propio protocolo HTTP, cada petición lleva solo dos cabeceras: User-Agent, que la API exige, con el nombre y la versión de Gestor Agentes seguidos de los de la biblioteca, por ejemplo gestor-agentes/3.8.2 aviso-version-github/0.2.3, y Accept con el tipo de contenido que recomienda GitHub. No lleva ningún dato del usuario ni de su equipo. Va cifrada con TLS, solo admite direcciones https, lee como mucho un mebibyte de respuesta y espera como mucho TiempoMaximoActualizacionSegundos. Sale por el proxy del sistema si lo hay: el de las variables ALL_PROXY, HTTPS_PROXY o HTTP_PROXY, miradas por este orden, con las excepciones de NO_PROXY, y, en Windows y sin esas variables, el de las opciones de internet. No interpreta archivos PAC ni se autentica ante el proxy. Sin autenticación, GitHub admite un número limitado de peticiones por hora desde cada dirección, de sobra para dos por arranque. Si falla, porque no hay red, se agota el tiempo, la respuesta no se entiende o la etiqueta publicada no tiene la forma X.Y.Z, como una versión previa del tipo v3.2.0-rc.1, la aplicación no avisa de nada y sigue igual, pero escribe el motivo en su salida de errores, en una línea con la fecha y la hora locales en formato ISO 8601, como 2026-11-23T16:30:00.0000000+01:00, seguida de Gestor Agentes, comprobación de versión nueva, de qué programa era la comprobación, Gestor Agentes o Programator, y su repositorio entre paréntesis, y después el tipo de fallo y de su detalle técnico, que en el caso de una etiqueta rechazada es la propia etiqueta. No se registra nada más, ni cuando la comprobación sale bien.
