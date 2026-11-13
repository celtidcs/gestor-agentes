Gestor Agentes y la ingeniería de arneses

Un modelo de lenguaje no es, por sí solo, un agente que trabaje de forma fiable. Lo que lo convierte en uno es lo que se construye a su alrededor: las instrucciones que recibe, el estado que conserva entre sesiones, las comprobaciones que tiene que superar antes de dar algo por terminado, los límites de lo que puede tocar y la forma en que empieza y acaba cada sesión. A ese conjunto se le llama arnés, y a su diseño, ingeniería de arneses. La idea de fondo la resume la revisión de Gloriaameng citada al final: la capacidad de un agente surge de la combinación de modelo, arnés y entorno, no del modelo aislado.

Gestor Agentes es, en esa terminología, un arnés para varios agentes a la vez. No sustituye a Claude Code, a Codex ni a agy, que traen su propio arnés de herramientas, sino que añade encima, en cada proyecto que abre, lo que esas herramientas no traen: unas reglas comunes, un estado compartido, un canal entre agentes y unas comprobaciones que nadie puede saltarse. Este documento explica qué ideas de la ingeniería de arneses aplica la aplicación y con qué funcionalidad, con la fuente de cada una.


Las cinco partes de un arnés

El curso Learn Harness Engineering, de WalkingLabs, divide un arnés en cinco partes, cada una con una sola tarea: las instrucciones, el estado, la verificación, el alcance y el ciclo de la sesión. Gestor Agentes tiene una pieza para cada una.


Instrucciones

Según el curso, las instrucciones dicen al agente qué hacer y en qué orden, y se dan por revelación progresiva, no en un único archivo grande.

Gestor Agentes escribe en cada proyecto un solo archivo de reglas, AGENTS.md, corto y escrito para que un agente lo siga con facilidad: un arranque en pasos numerados y las normas que se aplican siempre. Lo que solo hace falta en momentos concretos va aparte, en los documentos de consulta de .gestor/consulta: los estándares de código, el detalle de cómo se verifica un cambio, el método de documentación y publicación y la guía para trabajar con Programator. AGENTS.md dice en una tabla cuándo se lee cada uno, así que el agente solo los carga cuando le hacen falta. Lo propio de cada proyecto vive en su propio archivo, .gestor/instrucciones-del-proyecto.md, que la aplicación crea una vez y no vuelve a tocar.

Las instrucciones de las versiones anteriores, que se habían quedado a la vista de los agentes y les daban órdenes contradictorias, se apartan al abrir el proyecto, incluidas las reglas que agy carga por su cuenta desde la carpeta .agents. Su texto propio pasa a un bloque pendiente que el primer agente ordena, y lo apartado se borra cuando ya no hace falta.


Estado

El curso pide que el trabajo hecho, el que está a medias y el que viene se guarden en disco, para que la siguiente sesión pueda continuar. El proyecto Harness² de Google Research va en la misma línea: guarda cada ejecución con sus resúmenes, sus entregables y sus notas, y para retomarla basta con repetir la orden.

En Gestor Agentes, el estado vive en la carpeta .gestor de cada proyecto, fuera de la memoria de los agentes. El archivo de estado del canal recoge solo hechos comprobables, con su fecha y quién los anota: los papeles de cada agente, la rama y la versión, el reparto de funciones, las reservas de archivos y las mutaciones de control que siguen aplicadas. Cada agente lleva además su diario, con el plan y el próximo paso exacto, para que otro pueda retomar su trabajo en el mismo punto si se queda sin cuota, y el registro de las órdenes del Director, al que solo se añade texto.


Verificación

El curso exige una prueba ejecutable, como pruebas, linter o comprobación de tipos, antes de que el agente pueda dar algo por terminado. La revisión de Gloriaameng va más allá: la parte del arnés que evalúa el trabajo debe exportar eventos y trayectorias, no limitarse a poner una nota.

Las reglas de Gestor Agentes fijan las dos cosas. Nada se acepta porque lo diga quien lo hizo: lo comprueba otro agente o el Director, con cuatro comprobaciones mecánicas en todas las plataformas del proyecto, que son las pruebas, el linter sin avisos, el formateador sin diferencias y la auditoría de dependencias. Toda prueba se rompe a propósito para ver que se pone en rojo, porque una prueba que nunca ha fallado no demuestra nada. Y cada verificación deja un registro con la máquina, el sistema, la hora y lo verificado, con su suma publicada en el canal, de modo que el resultado se puede comprobar después.


Alcance

El curso limita al agente a una tarea cada vez. El arnés de referencia de TrishamBP describe además los ámbitos de recursos declarados: cada acción dice qué recursos toca, y las que tocan el mismo recurso se ejecutan una detrás de otra.

En Gestor Agentes, antes de cambiar un archivo el agente lo reserva en el estado del canal, con su nombre y el motivo, y borra la reserva al terminar. Cada agente tiene una bandeja del canal que solo escribe él y una carpeta propia. Al abrir un proyecto, la aplicación revisa las reservas y avisa si dos agentes tienen la misma ruta o si una reserva es de un agente que ya no da señales. Un encargo solo existe si va dirigido a alguien por su nombre, y cada agente trabaja en lo que tiene encargado.


Ciclo de la sesión

La quinta parte del curso cubre el principio de la sesión, la limpieza al terminar y el relevo para la siguiente.

Gestor Agentes manda un primer mensaje a cada agente al abrirlo, para que haga su arranque sin esperar a que el usuario escriba: se identifica, lee las reglas, el estado y los mensajes de los demás, pregunta al Director su papel si no está anotado, retoma lo que tuviera a medias y deja vigilando el canal. Al terminar, el agente actualiza el estado, libera sus reservas, pone al día su diario con el próximo paso exacto y detiene lo que haya lanzado.


Ciclo de trabajo y reglas de parada

El arnés de referencia de TrishamBP sostiene que el bucle del agente solo debe decidir el orden de los pasos y cuándo parar. La revisión de Gloriaameng incluye en el bucle la terminación, los reintentos, la recuperación y la delegación.

Las reglas de Gestor Agentes fijan cuándo parar: quien no resuelve una tarea en dos hipótesis comprobadas, o una depuración abierta en tres, para, documenta lo que sabe en los defectos conocidos y cede la tarea al Jefe de proyecto. Un agente que acumula malos resultados en una función se aparta de ella hasta que el Director diga otra cosa.

Los fallos se convierten además en guías, que es lo que el curso de WalkingLabs y Harness² piden de un arnés que aprende: cuando un rechazo o una parada se deben a una trampa reproducible del proyecto, quien la detecta deja una regla de una línea en el apartado Reglas aprendidas del archivo propio del proyecto, que todos los agentes leen al empezar.


Permisos y aprobación humana

La revisión de Gloriaameng sitúa en los ganchos del ciclo de vida la autorización, la aprobación y las políticas. El curso de WalkingLabs incluye, entre las piezas de un grafo de agentes, un nodo de aprobación humana.

En Gestor Agentes ese nodo es el Director. Nada se publica ni se sube a un repositorio remoto sin su autorización expresa para esa publicación concreta, después de su prueba humana. Los agentes no escriben fuera de la carpeta del proyecto, nadie verifica su propio trabajo y nadie edita el archivo de otro agente. Lo que el Director ordena se traslada a los demás por el canal, con sus palabras literales separadas de la interpretación de quien lo recibió.


Varios agentes: el grafo

El curso de WalkingLabs dedica una lección a pasar de un solo bucle a un grafo de agentes, con cuatro piezas: los nodos, las conexiones entre ellos, el estado compartido y el reparto del trabajo. Su idea clave es que un bucle es un grafo de un solo nodo.

Gestor Agentes organiza así a sus agentes. Los nodos son los agentes, cada uno con su identidad, como Claude 2 o Codex 1, que la aplicación le fija al abrirlo. Las conexiones son el canal, donde cada agente deja sus mensajes y sus encargos; el vigilante marca como encargo la línea nueva dirigida a cada uno, que es la señal que reparte el trabajo. El estado compartido es el archivo de estado del canal, que la aplicación revisa al abrir el proyecto y resume en la ventana: quién es el Jefe, cuándo dio señales cada agente, cuántas reservas hay y si queda alguna mutación viva. Es la observabilidad del grafo, que la revisión de Gloriaameng sitúa en la parte de evaluación del arnés. Y el reparto lo hace el Jefe de proyecto, que el Director elige y que propone en el canal quién hace cada cosa. Cada agente vigila el canal con los guiones que la aplicación deja en .gestor/vigilante: Claude Code recibe un aviso en cuanto otro escribe; Codex se monta al arrancar su propio sistema de vigilancia, que lo avisa aunque esté esperando; y agy, que no tiene aviso automático, lo comprueba al terminar cada paso.


Lo que todavía no hace

La revisión del estado del canal es solo de la forma: comprueba apartados, mutaciones vivas, reservas repetidas y reservas sin latido, pero no entiende lo que dicen los mensajes. agy no recibe aviso automático de los mensajes nuevos: solo los ve cuando ejecuta el vigilante al terminar cada paso. El aviso de Codex depende de que él mismo monte su vigilancia al arrancar. Y la aplicación no hace cumplir por sí misma las reservas ni la prohibición de escribir fuera del proyecto: lo hacen las instrucciones y la verificación de otro agente.


Fuentes

Learn Harness Engineering, de WalkingLabs: https://github.com/walkinglabs/learn-harness-engineering

Agent Harness for Large Language Model Agents: A Survey, repositorio de Gloriaameng: https://github.com/Gloriaameng/Awesome-Agent-Harness

Harness², de Google Research: https://github.com/google-research/harness2 y su artículo, Harness²: Recursive Agent Harnessing for an Open World, de Ruiyao Xu y otros: https://arxiv.org/abs/2610.14224

Harness Engineering, el arnés de referencia de TrishamBP: https://github.com/TrishamBP/harness-engineering

Todas se consultaron el 13 de noviembre de 2026.
