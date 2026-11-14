Gestor Agentes

Gestor Agentes es una aplicación de escritorio portable, escrita en Rust para Windows y Linux, que guarda la lista de tus proyectos de programación y los abre, con un clic, con un agente de inteligencia artificial: Claude Code, Codex, Gemini a través de agy, o Programator.

Cada agente se abre en su propia ventana de terminal, ya situado en la carpeta del proyecto y con la cuenta que le corresponde, de modo que puedes tener varias cuentas del mismo agente y varios agentes trabajando a la vez en el mismo proyecto, coordinados a través de un canal de archivos de texto que la propia aplicación prepara.


Principio rector de diseño

Todo Gestor Agentes responde a una premisa: nada se da por bueno porque lo diga el agente que lo hizo.

Un modelo de lenguaje no trabaja de forma fiable por sí solo, y varios a la vez todavía menos si nadie pone orden. Por eso la aplicación no se limita a abrir terminales: antes de lanzar a cada agente deja en el proyecto unas reglas comunes, un estado compartido y un canal entre agentes, y esas reglas obligan a que el trabajo de uno lo compruebe otro, o tú. Y por encima de todos estás tú, el Director: nada se publica ni se sube a un repositorio remoto sin tu autorización expresa.


Para qué sirve

Gestor Agentes tiene sentido cuando trabajas con más de un agente, o con más de una cuenta del mismo agente, y quieres que colaboren en lugar de pisarse.

El primer uso es abrir cualquier proyecto con cualquier agente en un clic. La aplicación recuerda tus proyectos, sabe qué agentes y qué cuentas tienes, y abre cada uno en su terminal, en la carpeta correcta y con la cuenta que toca. No se instala: su configuración viaja con ella en la carpeta datos, junto al ejecutable.

El segundo es dejar el proyecto preparado para trabajar en equipo. Antes de abrir el agente escribe AGENTS.md, el archivo de instrucciones que leen todos los agentes, y CLAUDE.md, con el que Claude Code lo carga. Crea una sola vez el archivo propio del proyecto, instrucciones-del-proyecto.md, que nunca vuelve a tocar. Prepara la carpeta .gestor, donde los agentes se dejan mensajes y llevan su diario, y crea el repositorio git si no existe.

El tercero es que cada agente arranque solo y sepa dónde está. En cuanto se abre, el agente se presenta, lee el estado del proyecto y los mensajes de los demás, te pregunta si es el Jefe de proyecto o un ayudante la primera vez, retoma lo pendiente y deja preparado un vigilante que le avisa cuando otro agente le escribe. Si un agente se queda sin cuota, otro retoma su trabajo en el mismo punto, porque todo lo que importa está en disco y no en su memoria.

El cuarto es ver de un vistazo cómo va el equipo. Al seleccionar un proyecto, la ventana muestra quién es el Jefe, cuándo dio señales cada agente y cuántas reservas de archivos hay. Al abrirlo avisa de lo que no cuadra, como dos agentes que reservan el mismo archivo o una reserva de un agente que ya no da señales. La versión se ve arriba a la derecha, y al arrancar la aplicación te avisa si hay una más nueva.


Para qué no sirve

Contar solo lo que hace bien sería venderla por encima de lo que da, así que aquí va la otra mitad.

No sustituye a los agentes. Claude Code, Codex, agy y Programator son programas aparte que tienes que instalar tú; Gestor Agentes los abre y les da las reglas, pero no trae ninguno.

No obliga por la fuerza a cumplir las reglas. Las reservas de archivos y la prohibición de escribir fuera del proyecto las hacen cumplir las instrucciones y la verificación de otro agente, no la aplicación. La revisión del canal comprueba la forma de los archivos, no lo que dicen los mensajes.

No avisa igual a todos los agentes. Claude Code recibe un aviso cada vez que otro agente le escribe. Codex se monta él mismo su vigilancia al arrancar, y si no lo consigue se queda sin aviso. agy no tiene esa posibilidad: solo ve los mensajes nuevos cuando mira el canal al terminar cada paso, así que si está parado esperando, díselo tú.

No abre la ventana en un Windows sin OpenGL 2.0, como el que tienen muchas máquinas virtuales con el adaptador básico de Windows. Se arregla instalando el complemento gratuito de Microsoft OpenCL, OpenGL, and Vulkan Compatibility Pack. Este y los demás límites están contados, con su solución cuando la tienen, en documentation/defectos-conocidos.md.


Cómo se ve

![Ventana de Gestor Agentes con la lista de proyectos](documentation/imagenes/gestor-agentes.png)

Arriba están las tres pestañas de la aplicación: Proyectos, Agentes e Instrucciones de los agentes, y a la derecha la versión. En la pestaña Proyectos, la lista muestra cada proyecto con su nombre, su ruta, la última vez que se abrió y su estado. Debajo, la línea del equipo resume el proyecto seleccionado: quién es el Jefe y desde cuándo, hace cuánto dio señales cada agente y cuántas reservas hay. Y al pie, un botón para agregar proyectos y un botón por cada agente y cuenta para abrirlo en su terminal.


Gestor Agentes y la ingeniería de arneses

Lo que convierte a un modelo de lenguaje en un agente útil es lo que se construye a su alrededor: las instrucciones que recibe, el estado que conserva entre sesiones, las comprobaciones que tiene que superar, los límites de lo que puede tocar y cómo empieza y acaba cada sesión. A ese conjunto se le llama arnés, y a su diseño, ingeniería de arneses. Gestor Agentes es un arnés para varios agentes a la vez: no sustituye a Claude Code, a Codex ni a agy, sino que añade en cada proyecto lo que esas herramientas no traen. Cumple cada parte de un arnés con una funcionalidad concreta.

Instrucciones. Un solo archivo de reglas, AGENTS.md, corto y con un arranque en pasos numerados. Lo que solo hace falta a veces, como los estándares de código, el detalle de la verificación o la guía de Programator, va en documentos de consulta que el agente lee cuando le toca. Lo propio de cada proyecto, en un archivo aparte que la aplicación no vuelve a tocar. Las instrucciones antiguas que daban órdenes contradictorias se apartan al abrir el proyecto.

Estado. Todo lo que importa se guarda en disco, en la carpeta .gestor del proyecto, y no en la memoria de los agentes: el estado del canal con los papeles, la versión, el reparto, las reservas y las mutaciones de control, el diario de cada agente con su próximo paso exacto y el registro de las órdenes del Director.

Verificación. Nada se acepta porque lo diga quien lo hizo: lo comprueba otro agente o el Director, con cuatro comprobaciones mecánicas en Windows y Linux, que son las pruebas, el linter sin avisos, el formateador sin diferencias y la auditoría de dependencias. Cada prueba se rompe a propósito para ver que se pone en rojo, y cada verificación deja un registro con la máquina, el sistema, la hora y su suma, para que se pueda comprobar después.

Alcance. Antes de cambiar un archivo, el agente lo reserva en el estado del canal y lo libera al terminar. Cada agente tiene su bandeja y su carpeta, que solo escribe él. Un encargo solo existe si va dirigido a alguien por su nombre.

Ciclo de la sesión. La aplicación manda a cada agente un primer mensaje al abrirlo para que haga su arranque solo: se identifica, lee las reglas, el estado y los mensajes, pregunta su papel si no lo tiene, retoma lo pendiente y deja vigilando el canal. Al terminar, actualiza el estado, libera sus reservas y deja su diario al día.

Reglas de parada y reglas aprendidas. Quien no resuelve una tarea en dos hipótesis comprobadas para, documenta lo que sabe y la cede al Jefe de proyecto, y quien acumula malos resultados en una función se aparta de ella. Cuando un fallo se debe a una trampa que se puede repetir, queda escrito como una regla de una línea que todos los agentes leen al empezar.

Aprobación humana. El Director, que eres tú, es el nodo de aprobación: nada se publica ni se sube a un repositorio remoto sin tu autorización expresa, y lo que ordenas se traslada a los demás agentes con tus palabras literales.

Varios agentes trabajando como un grafo. Cada agente es un nodo con su identidad, como Claude 2 o Codex 1. El canal los conecta, y un vigilante avisa a cada uno de lo que le escriben y le marca los encargos. El reparto lo hace el Jefe de proyecto que tú eliges.

El documento documentation/harness-engineering.md lo explica con detalle, con las fuentes en que se basa: el curso Learn Harness Engineering de WalkingLabs, la revisión Awesome Agent Harness, Harness² de Google Research y el arnés de referencia de TrishamBP.


Puesta en marcha

Descarga el paquete portable de tu sistema desde la página de versiones publicadas del repositorio, descomprímelo donde quieras y abre gestor-agentes.exe en Windows o gestor-agentes en Linux. Necesitas además al menos uno de los agentes instalado, porque son programas aparte, y conviene tener git.

El archivo LEEME.txt del paquete explica los primeros pasos, y el manual de uso, documentation/usage.md, lo explica todo con detalle, incluido cómo instalar cada agente.

Antes se llamaba Gestor Claude. La primera vez que se abre en un equipo donde se usó aquella versión, copia su configuración sin tocar la original.


Compilación y verificación mecánica

El proyecto se rige por un criterio de verificación estricto: ninguna entrega se acepta sin superar, en Windows y en Linux, cuatro comprobaciones mecánicas. La primera es el formato, que no puede dejar ni una diferencia. La segunda es el linter clippy en todo el workspace, con los avisos tratados como errores. La tercera son las pruebas, todas en verde. Y la cuarta es la auditoría de dependencias, sin vulnerabilidades conocidas. Los guiones puertas-rust.ps1 y puertas-rust.sh, de la carpeta documentation/infrastructure, las pasan en ese orden.

Para preparar un equipo y compilar, la guía documentation/infrastructure/clone-guide.md lo explica paso a paso. En resumen, con Rust instalado, el guion setup.ps1 en Windows o setup.sh en Linux, de esa misma carpeta, comprueba lo necesario, pasa las comprobaciones de calidad y genera el paquete portable en la carpeta portable.


Estructura del código fuente

El código está organizado en un workspace de cargo con tres partes, cada una con una sola responsabilidad.

La parte nucleo prepara los proyectos, escribe las instrucciones de los agentes, revisa el canal y guarda la configuración, los agentes y las cuentas.

La parte lanzador decide cómo se abre cada agente en cada sistema, con qué terminal y con qué entorno, y lo abre.

La parte interfaz es la ventana, hecha con eframe y egui, y genera el ejecutable gestor-agentes.

La versión se declara en un solo sitio, el Cargo.toml de la raíz, y sigue Semantic Versioning.


Guía de documentación

La carpeta documentation contiene lo que hace falta saber para usar la aplicación y para seguir trabajando en ella, y su README.md explica qué contiene cada documento.

El manual de uso, usage.md, explica la aplicación paso a paso para quien no la conoce.

La referencia de funcionalidades, features.md, describe con detalle técnico todo lo que hace.

La arquitectura, architecture.md, explica cómo está organizado el código y por qué se tomó cada decisión.

Gestor Agentes y la ingeniería de arneses, harness-engineering.md, explica qué ideas de la ingeniería de arneses aplica y con qué funcionalidad, con sus fuentes.

El historial de cambios, changelog.md, recoge lo que cambia en cada versión.

Los defectos conocidos, defectos-conocidos.md, recogen los fallos y límites que todavía no se han corregido.

En la carpeta infrastructure están la visión general de los componentes, los requisitos, los programas externos con los que trabaja la aplicación, la guía para preparar un equipo y los guiones de preparación, compilación y comprobación, cada uno en versión para Windows y para Linux.


Sobre Celtilander

Soy un usuario novel en el mundo de la programación y la IA. He «ayudado a crear» esta app y espero que a alguien le pueda servir para algo... sin más.

Lo que no funciona no lo escondo: está escrito en los defectos conocidos (https://github.com/celtidcs/gestor-agentes/blob/main/documentation/defectos-conocidos.md). Si encuentras algo que no esté ahí, cuéntamelo.

Para realizar esta app he montado un sistema colaborativo y de consenso entre ChatGPT Codex, Claude Code y Gemini Antigravity. Yo puse la idea y dirigía el proyecto, mientras que ellos diseñaban, repartían y verificaban el trabajo. Ningún código fue revisado por el agente que lo había escrito, y una vez terminado todo debía ser aprobado por unanimidad. Hasta que no se conseguía el consenso, no se daba una tarea por concluida. Cuando por algún motivo no se llegaba a un consenso, era yo quien tomaba la decisión final, así que los fallos que pueda haber serán más míos que de ellos.

Saludos a todo el mundo.


Licencia

Gestor Agentes es software libre, publicado con la licencia GPL-3.0 o posterior. El texto íntegro está en el archivo LICENSE.

En corto: puedes usar el programa para lo que quieras, estudiar cómo funciona por dentro, modificarlo y repartirlo. La única condición es que si repartes una versión modificada, publiques también su código, con esta misma licencia.
