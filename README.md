Gestor Agentes

Gestor Agentes es una aplicación de escritorio para Windows y Linux que guarda la lista de tus proyectos de programación y los abre, con un clic, con un agente de inteligencia artificial: Claude Code, Codex, Gemini a través de agy, o Programator. Cada agente se abre en su propia ventana de terminal, ya situado en la carpeta del proyecto y con la cuenta que le corresponde, de modo que puedes tener varias cuentas del mismo agente y varios agentes trabajando a la vez en el mismo proyecto.

Antes de abrir el agente, la aplicación deja el proyecto preparado para ese trabajo en común. Escribe AGENTS.md, el archivo de instrucciones que leen todos los agentes, y CLAUDE.md, con el que Claude Code lo carga. Crea una sola vez el archivo propio del proyecto, instrucciones-del-proyecto.md, que nunca vuelve a tocar. Prepara la carpeta .gestor, donde los agentes se dejan mensajes y llevan su diario, y crea el repositorio git si no existe.

Cada agente empieza a trabajar solo en cuanto se abre: se presenta, lee el estado del proyecto y los mensajes de los demás, te pregunta si es el Jefe de proyecto o un ayudante la primera vez y deja preparado un vigilante que le avisa cuando otro agente le escribe. Al seleccionar un proyecto, la ventana muestra cómo va su equipo, y al abrirlo avisa de lo que no cuadra, como dos agentes que reservan el mismo archivo. La versión se ve arriba a la derecha, y al arrancar la aplicación te avisa si hay una más nueva.

La aplicación es portable: no se instala, y su configuración viaja con ella en la carpeta datos, junto al ejecutable. Antes se llamaba Gestor Claude, y la primera vez que se abre en un equipo donde se usó aquella versión copia su configuración sin tocar la original.

![Ventana de Gestor Agentes con la lista de proyectos](documentation/imagenes/gestor-agentes.png)


Gestor Agentes y la ingeniería de arneses

Un modelo de lenguaje no trabaja de forma fiable por sí solo. Lo que lo convierte en un agente útil es lo que se construye a su alrededor: las instrucciones que recibe, el estado que conserva entre sesiones, las comprobaciones que tiene que superar, los límites de lo que puede tocar y cómo empieza y acaba cada sesión. A ese conjunto se le llama arnés, y a su diseño, ingeniería de arneses. Gestor Agentes es un arnés para varios agentes a la vez: no sustituye a Claude Code, a Codex ni a agy, sino que añade en cada proyecto lo que esas herramientas no traen. Cumple cada parte de un arnés con una funcionalidad concreta.

Instrucciones. Un solo archivo de reglas, AGENTS.md, corto y con un arranque en pasos numerados. Lo que solo hace falta a veces, como los estándares de código, el detalle de la verificación o la guía de Programator, va en documentos de consulta que el agente lee cuando le toca. Lo propio de cada proyecto, en un archivo aparte que la aplicación no vuelve a tocar. Las instrucciones antiguas que daban órdenes contradictorias se apartan al abrir el proyecto.

Estado. Todo lo que importa se guarda en disco, en la carpeta .gestor del proyecto, y no en la memoria de los agentes: el estado del canal con los papeles, la versión, el reparto, las reservas y las mutaciones de control, el diario de cada agente con su próximo paso exacto y el registro de las órdenes del Director. Si un agente se queda sin cuota, otro retoma su trabajo en el mismo punto.

Verificación. Nada se acepta porque lo diga quien lo hizo: lo comprueba otro agente o el Director, con cuatro comprobaciones mecánicas en Windows y Linux, que son las pruebas, el linter sin avisos, el formateador sin diferencias y la auditoría de dependencias. Cada prueba se rompe a propósito para ver que se pone en rojo, y cada verificación deja un registro con la máquina, el sistema, la hora y su suma, para que se pueda comprobar después.

Alcance. Antes de cambiar un archivo, el agente lo reserva en el estado del canal y lo libera al terminar. Cada agente tiene su bandeja y su carpeta, que solo escribe él. Un encargo solo existe si va dirigido a alguien por su nombre. Al abrir un proyecto, la aplicación avisa si dos agentes reservan el mismo archivo o si una reserva es de un agente que ya no da señales.

Ciclo de la sesión. La aplicación manda a cada agente un primer mensaje al abrirlo para que haga su arranque solo: se identifica, lee las reglas, el estado y los mensajes, pregunta su papel si no lo tiene, retoma lo pendiente y deja vigilando el canal. Al terminar, actualiza el estado, libera sus reservas y deja su diario al día.

Reglas de parada y reglas aprendidas. Quien no resuelve una tarea en dos hipótesis comprobadas para, documenta lo que sabe y la cede al Jefe de proyecto, y quien acumula malos resultados en una función se aparta de ella. Cuando un fallo se debe a una trampa que se puede repetir, queda escrito como una regla de una línea que todos los agentes leen al empezar.

Aprobación humana. El Director, que eres tú, es el nodo de aprobación: nada se publica ni se sube a un repositorio remoto sin tu autorización expresa, y lo que ordenas se traslada a los demás agentes con tus palabras literales.

Varios agentes trabajando como un grafo. Cada agente es un nodo con su identidad, como Claude 2 o Codex 1. El canal los conecta, y un vigilante avisa a cada uno de lo que le escriben y le marca los encargos. El estado compartido se revisa al abrir el proyecto y se resume en la ventana, que enseña quién es el Jefe, cuándo dio señales cada agente, cuántas reservas hay y si queda alguna mutación viva. El reparto lo hace el Jefe de proyecto que tú eliges.

Lo que todavía no hace. La revisión del canal comprueba la forma, no lo que dicen los mensajes. agy no recibe aviso automático y mira el canal al terminar cada paso, y el aviso de Codex depende de que él monte su vigilancia al arrancar. Las reservas y la prohibición de escribir fuera del proyecto las hacen cumplir las instrucciones y la verificación de otro agente, no la aplicación.

El documento documentation/harness-engineering.md lo explica con detalle, con las fuentes en que se basa: el curso Learn Harness Engineering de WalkingLabs, la revisión Awesome Agent Harness, Harness² de Google Research y el arnés de referencia de TrishamBP.


Sobre Celtilander

Soy un usuario novel en el mundo de la programación y la IA. He «ayudado a crear» esta app y espero que a alguien le pueda servir para algo... sin más.

Lo que no funciona no lo escondo: está escrito en los defectos conocidos (https://github.com/celtidcs/gestor-agentes/blob/main/documentation/defectos-conocidos.md). Si encuentras algo que no esté ahí, cuéntamelo.

Para realizar esta app he montado un sistema colaborativo y de consenso entre ChatGPT Codex, Claude Code y Gemini Antigravity. Yo puse la idea y dirigía el proyecto, mientras que ellos diseñaban, repartían y verificaban el trabajo. Ningún código fue revisado por el agente que lo había escrito, y una vez terminado todo debía ser aprobado por unanimidad. Hasta que no se conseguía el consenso, no se daba una tarea por concluida. Cuando por algún motivo no se llegaba a un consenso, era yo quien tomaba la decisión final, así que los fallos que pueda haber serán más míos que de ellos.

Saludos a todo el mundo.


Para usarla

Descarga el paquete portable de tu sistema desde la página de versiones publicadas del repositorio, descomprímelo donde quieras y abre gestor-agentes.exe en Windows o gestor-agentes en Linux. Necesitas además al menos uno de los agentes instalado, porque son programas aparte, y conviene tener git. El archivo LEEME.txt del paquete explica los primeros pasos, y el manual de uso, documentation/usage.md, lo explica todo con detalle, incluido cómo instalar cada agente.


Para compilarla

Está escrita en Rust y organizada en un workspace de cargo con tres partes, cada una con una sola responsabilidad. La parte nucleo prepara los proyectos y guarda la configuración. La parte lanzador decide cómo se abre cada agente en cada sistema y lo abre. La parte interfaz es la ventana, hecha con eframe y egui, y genera el ejecutable gestor-agentes.

La guía documentation/infrastructure/clone-guide.md explica paso a paso cómo preparar un equipo y compilar. En resumen, con Rust instalado, el guion documentation/infrastructure/setup.ps1 en Windows o documentation/infrastructure/setup.sh en Linux comprueba lo necesario, pasa las comprobaciones de calidad y genera el paquete portable en la carpeta portable.


Documentación

Toda la documentación está en la carpeta documentation, y su README.md explica qué contiene cada documento: el manual de uso, la referencia de funcionalidades, la arquitectura, el historial de cambios, los defectos conocidos y los documentos de infraestructura.


Licencia

Gestor Agentes es software libre, publicado con la licencia GPL-3.0 o posterior. El texto íntegro está en el archivo LICENSE. Puedes usarlo, estudiarlo, modificarlo y redistribuirlo; si distribuyes una versión modificada, tienes que publicar también su código, con esta misma licencia.
