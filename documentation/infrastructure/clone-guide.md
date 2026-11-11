Guía para preparar un equipo nuevo

Esta guía explica cómo compilar Gestor Agentes desde el código en un equipo nuevo, sin conocer el proyecto. Los pasos son los mismos en Windows y en Linux, salvo donde se indica.


Paso 1. Instalar lo necesario

Instala Rust con rustup, desde https://rustup.rs; después cargo-audit, con cargo install cargo-audit --locked; y git. Los detalles y las versiones están en requirements.md. Instala también los agentes que vayas a usar: Claude Code, Codex, agy para Gemini, o Programator. El manual de uso, usage.md, explica cómo instalar cada uno. Comprueba que todo responde con cargo --version, git --version y, por ejemplo, claude --version.


Paso 2. Obtener el código

Clona el repositorio con git clone seguido de su dirección, o descarga el código de una versión desde la página de versiones publicadas y descomprímelo. Si git se queja de que la carpeta tiene un propietario dudoso, lo que pasa en algunas unidades de red, puedes autorizarla con git config --global --add safe.directory seguido de la ruta. La aplicación no lo necesita, porque ya pasa ese ajuste en cada orden de git que ejecuta.


Paso 3. Preparar el entorno y generar el paquete portable

En Windows, abre PowerShell en la carpeta del código y ejecuta .\documentation\infrastructure\setup.ps1. En Linux, ejecuta ./documentation/infrastructure/setup.sh. Los dos guiones comprueban lo necesario, buscan los agentes instalados, pasan las cuatro comprobaciones de calidad y generan el paquete portable en la carpeta portable: gestor-agentes.exe en Windows o gestor-agentes en Linux, con la carpeta datos al lado. Con -SinPuertas en Windows o --sin-puertas en Linux se saltan las comprobaciones. Puedes ejecutarlos tantas veces como quieras, y al regenerar el paquete conservan lo que haya en datos.

La primera compilación descarga las dependencias y tarda unos minutos; las siguientes son mucho más rápidas.


Paso 4. Configurar los perfiles

Abre la aplicación una vez. Si en el equipo se usó Gestor Claude, la versión anterior, copia su configuración a la carpeta datos sin borrar la original y te dice qué ha copiado. Si no, crea allí perfiles-agentes.json con los perfiles de serie, que puedes ampliar si necesitas más cuentas. Si quieres que todos tus proyectos compartan unas normas comunes, escríbelas en un CLAUDE.md o un AGENTS.md en la carpeta que contiene los proyectos; la aplicación las copia sola en cada uno.

La configuración viaja con la carpeta del paquete: para llevarla a otro equipo basta con copiar la carpeta entera, con datos dentro.


Paso 5. Comprobar que funciona

Añade como proyecto una carpeta vacía de prueba, selecciónala y pulsa Abrir · Claude 1, o el botón del agente que tengas. Debe abrirse una terminal con el agente en esa carpeta, con el título Claude 1 y el nombre de la carpeta, y el agente debe presentarse diciendo que ha leído AGENTS.md y las instrucciones del proyecto y que es Claude 1. En la carpeta tienen que haber aparecido AGENTS.md, CLAUDE.md, .gitignore, un repositorio git y la carpeta .gestor, con el canal, instrucciones-del-proyecto.md y los documentos de consulta. Si escribiste normas comunes en la carpeta superior, deben aparecer al final de AGENTS.md. No tiene que aparecer GEMINI.md.
