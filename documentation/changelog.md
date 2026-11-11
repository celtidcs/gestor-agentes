Historial de cambios

Este historial recoge lo que cambia en cada versión, contado para quien usa la aplicación. Las versiones siguen la numeración de Semantic Versioning: mayor, menor y corrección.


3.0.0, del 11 de noviembre de 2026. Gestor Agentes

La aplicación se llamaba Gestor Claude y estaba escrita en C Sharp con .NET. Desde esta versión se llama Gestor Agentes y está escrita en Rust. El ejecutable es uno solo, gestor-agentes.exe en Windows o gestor-agentes en Linux, no necesita .NET ni ningún otro componente, y la ventana es la misma en los dos sistemas.

Portable de verdad. La configuración ya no está en la carpeta del usuario, sino en la carpeta datos, junto al ejecutable, y viaja con él. La primera vez que se abre sin lista de proyectos ni perfiles en datos, copia la configuración de Gestor Claude sin tocar la original. Desaparecen los guiones de guardar y restaurar la configuración, que ya no hacen falta. Los ajustes nuevos, el tiempo máximo de git y el límite de tamaño de una bandeja, van en datos/ajustes.json.

Todos los agentes en terminal. Claude Code, Codex, Gemini con agy y Programator se abren en una terminal, situada en la carpeta del proyecto. Ya no se abren las aplicaciones de escritorio de Codex ni de Antigravity, y con ello desaparece el aviso de que Antigravity ya estaba abierto. Programator es un tipo de agente nuevo, que se abre con la ruta de su ejecutable.

Solo los botones que sirven. Al arrancar, la aplicación busca el programa de cada perfil y muestra solo los botones de los agentes instalados, con la misma búsqueda que se usa al abrirlos. En Windows, codex se busca primero donde lo deja su instalador oficial. También reconoce las cuentas instaladas, como ~/.claude, ~/.claude2 o ~/.codex, y añade un perfil por cada una que falte, sin tocar los que ya había. La pestaña nueva Agentes muestra el estado de cada perfil y permite fijar la ruta de su programa, con Examinar o con Búsqueda automática.

Un solo archivo de instrucciones. AGENTS.md pasa a ser el único archivo de reglas de todos los agentes, y la aplicación lo escribe entero cada vez que abre uno. CLAUDE.md queda como un puntero que lo carga, y GEMINI.md deja de generarse, porque Antigravity lee AGENTS.md. Lo propio de cada proyecto va en .gestor/instrucciones-del-proyecto.md, que la aplicación crea una vez y no vuelve a tocar. En un proyecto preparado con una versión anterior, recoge el texto propio de sus archivos antiguos en un bloque pendiente de consolidar, que el primer agente que arranque ordena por su cuenta, y lleva a .gestor/archivo lo que ya no se usa. Lo que solo se consulta a veces sale a .gestor/consulta, y así AGENTS.md cabe entero en lo que leen Codex y Gemini.

Avisos nuevos. Si AGENTS.md mide más de lo que lee Codex o Gemini, la barra de estado lo avisa al abrirlos, y un perfil puede fijar otro límite. El informe de preparación avisa de las bandejas del canal que han crecido demasiado. Si falta OpenGL 2.0, la aplicación lo dice con un mensaje claro en vez de cerrarse sin más.

Correcciones. En Windows, un nombre de carpeta con comillas tipográficas ya no puede romper la orden con la que se abre la terminal: se escapan las cinco variantes de la comilla simple que reconoce PowerShell. Los archivos de instrucciones ya no llegan cortados a Codex ni a Gemini.

Para desarrollar. El código se organiza en tres partes, nucleo, lanzador e interfaz, con sus pruebas aparte. Los guiones setup, build-portable, prueba-portable y puertas-rust, en versión para Windows y para Linux, preparan el entorno, generan y comprueban el paquete portable y pasan las cuatro comprobaciones de calidad.


Versiones anteriores

Las versiones 1.x y 2.x se publicaron con el nombre Gestor Claude y estaban escritas en C Sharp, primero con WPF y después con Avalonia. La 3.0.0 las sustituye por completo y conserva su configuración.
