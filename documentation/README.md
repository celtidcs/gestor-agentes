Gestor Agentes

Gestor Agentes es una aplicación de escritorio para Windows y Linux que guarda la lista de tus proyectos de programación y los abre, con un clic, con un agente de inteligencia artificial: Claude Code, Codex, Gemini a través de agy, o Programator. Cada agente se abre en su propia ventana de terminal, ya situado en la carpeta del proyecto y con la cuenta que le corresponde, de modo que puedes tener varias cuentas del mismo agente y varios agentes trabajando a la vez en el mismo proyecto.

Antes de abrir el agente, la aplicación deja el proyecto preparado para ese trabajo en común. Escribe AGENTS.md, el archivo de instrucciones que leen todos los agentes, y CLAUDE.md, con el que Claude Code lo carga. Crea una sola vez el archivo propio del proyecto, instrucciones-del-proyecto.md, que nunca vuelve a tocar. Prepara la carpeta .gestor, donde los agentes se dejan mensajes y llevan su diario, y crea el repositorio git si no existe.

La aplicación es portable: no se instala, y su configuración viaja con ella en la carpeta datos, junto al ejecutable. Antes se llamaba Gestor Claude, y la primera vez que se abre en un equipo donde se usó aquella versión copia su configuración sin tocar la original.


Para usarla

Descarga el paquete portable de tu sistema, descomprímelo donde quieras y abre gestor-agentes.exe en Windows o gestor-agentes en Linux. Necesitas además al menos uno de los agentes instalado, porque son programas aparte, y conviene tener git. El archivo LEEME.txt del paquete explica los primeros pasos, y el Manual de uso lo explica todo con detalle, incluido cómo instalar cada agente.


Para compilarla

Está escrita en Rust y organizada en un workspace de cargo con tres partes, cada una con una sola responsabilidad. La parte nucleo prepara los proyectos y guarda la configuración. La parte lanzador decide cómo se abre cada agente en cada sistema y lo abre. La parte interfaz es la ventana, hecha con eframe y egui, y genera el ejecutable gestor-agentes. La versión se declara en un solo sitio, el Cargo.toml de la raíz, y sigue la numeración de tres cifras de Semantic Versioning: mayor, menor y corrección.

Para generar el paquete portable desde el código se ejecuta infrastructure/build-portable.ps1 en Windows o infrastructure/build-portable.sh en Linux. Si además quieres comprobar que el equipo tiene todo lo necesario y pasar las comprobaciones de calidad, usa infrastructure/setup.ps1 o infrastructure/setup.sh. La guía infrastructure/clone-guide.md explica cómo preparar desde cero un equipo nuevo para desarrollar.


Documentos

El manual de uso, usage.md, explica la aplicación paso a paso para quien no la conoce; en el paquete portable se llama Manual de uso.

La referencia de funcionalidades, features.md, describe con detalle técnico todo lo que hace la aplicación; en el paquete se llama Referencia de funcionalidades.

La arquitectura, architecture.md, explica cómo está organizado el código y por qué se tomó cada decisión.

El historial de cambios, changelog.md, recoge lo que cambia en cada versión.

Los defectos conocidos, defectos-conocidos.md, recogen los fallos y límites que todavía no se han corregido.

En la carpeta infrastructure están la visión general de los componentes, overview.md; los requisitos y versiones de las herramientas, requirements.md; los programas externos con los que trabaja la aplicación, services.md; y la guía para preparar un equipo nuevo, clone-guide.md. Están también los guiones: setup prepara el entorno, build-portable genera el paquete, prueba-portable comprueba el paquete, puertas-rust pasa las comprobaciones de calidad y prueba-setup comprueba esos mismos guiones. Cada uno tiene versión para Windows, terminada en .ps1, y para Linux, terminada en .sh.
