Núcleo común de gestión y verificación

Este archivo, AGENTS.md, es el único archivo de reglas de todos los agentes del proyecto (Claude, Codex, Gemini y los demás). Lo escribe entero Gestor Agentes cada vez que lanza un agente, así que nadie lo edita a mano: lo que se escribiera aquí se perdería en el siguiente lanzamiento (quedaría solo en .gestor/respaldos/). CLAUDE.md es solo un puntero que hace que Claude Code cargue este archivo. Lo propio de este proyecto vive en .gestor/instrucciones-del-proyecto.md, que obliga igual que este archivo, se lee entero al empezar cada sesión y Gestor Agentes no toca nunca una vez creado. Lo que solo hace falta en momentos concretos está en .gestor/consulta/ y se lee entero cuando llega ese momento, como se indica en cada apartado.

Si .gestor/instrucciones-del-proyecto.md contiene un bloque pendiente de consolidar (entre marcas GESTOR:LEGADO), el primer agente que arranque lo consolida antes de cualquier otro trabajo. Lo reserva en estado.md; si ya lo ha reservado otro, no lo toca. Revisa el bloque regla por regla: lo que ya dicen este archivo o las normas de la carpeta proyecto se descarta; lo particular del proyecto que siga vigente se queda en ese archivo, redactado como una regla para todos los agentes; lo que contradiga este núcleo no se descarta por cuenta propia, porque pudo ser una orden del Director, y se le pregunta a través del Jefe de proyecto. Si el bloque cita archivos del sistema anterior llevados a .gestor/archivo/, mira si guardan una tarea viva y la traslada a su diario y a estado.md. Al terminar borra el bloque con sus marcas, libera la reserva y anota en su diario qué conservó y qué descartó. La copia exacta de los archivos anteriores está en .gestor/respaldos/.


1. Norma suprema de verificación y seguridad

Esta norma va antes que todo lo demás. La seguridad del código y de la plataforma es prioritaria y máxima: ningún plazo, cuota o comodidad justifica relajarla ni aceptar código sin verificar.

1.1 Nada se da por bueno porque lo diga quien lo hizo

Todo lo que entra en el repositorio pasa un criterio de aceptación mecánico, ejecutado por un agente o persona distinta de quien lo escribió. Son cuatro puertas, en su forma equivalente para el lenguaje del proyecto: la suite de pruebas completa en verde; el linter al máximo nivel de exigencia razonable, sin un solo aviso; el formateador, sin una sola diferencia; y la auditoría de dependencias, sin vulnerabilidades conocidas ni excepciones. Se pasan en todas las plataformas del proyecto, Windows y Linux salvo que sea de una sola, antes de fijar cualquier candidato de entrega. Ninguna prueba desaparece por el camino: si una comprobación deja de ser posible se declara en el canal; borrarla o relajarla en silencio está prohibido. Nadie verifica su propio trabajo; si solo interviene un agente, verifica otro convocado para ello o el Director.

1.2 Se ejecuta el código; no se lee y se opina

Leer código y opinar es lo que un modelo de lenguaje hace peor. Está prohibido dar por hallazgo lo que no se ha reproducido mecánicamente. Antes de afirmar que algo está roto o arreglado, se ejecuta. Después se rompe a propósito y se comprueba que la prueba se pone en rojo, porque una prueba que nunca se ha visto fallar no se distingue de un adorno. En una reescritura o refactorización se compara la salida de antes y de después. Lo no determinista (identificadores aleatorios, marcas de tiempo, orden de tablas hash) se fija antes de comparar. Las listas de reglas no se aplican leyendo código: son criterios de aceptación y se aplican ejecutando.

1.3 Separación de barreras y criterios previos

Las pruebas barrera viven en un archivo distinto del código que vigilan, para que sobrevivan si la implementación se daña o se revierte. Los criterios de aceptación de cualquier cambio se publican en el canal antes de escribir código, para que nadie acomode el éxito a la solución que le haya salido.

1.4 Mutaciones de control y restauraciones

Antes de aplicar una mutación, el archivo se copia en .gestor/respaldos/ y se restaura solo desde esa copia o con un parche inverso exacto; con cambios sin confirmar de varios autores, git checkout está prohibido porque destruye el trabajo ajeno. Mientras una mutación siga aplicada figura en el apartado de mutaciones vivas de estado.md, y nadie corrige un rojo ajeno sin mirar antes ese apartado y la bandeja del dueño del archivo.

1.5 Calibrar las herramientas de medida

Antes de creer un grep, un diff o cualquier medición, se contrasta con un caso de resultado conocido. Si la salida sorprende, la primera hipótesis es que la medición responde a otra pregunta. Las marcas se reconocen como líneas completas, nunca como subcadenas.

1.6 Registros que no se pueden inventar

Toda verificación deja un registro en la carpeta del agente que verifica, con su suma sha256 publicada en el canal. El registro empieza con el nombre de la máquina, el sistema (uname o la versión de Windows), la fecha y hora, y lo que se verifica: el commit, si ya existe, verificado sobre el objeto del commit (git archive) y no sobre el árbol de trabajo; o, en una entrega congelada antes del commit, el commit base y la suma de la lista de la entrega, comprobada en la máquina con sha256sum -c o Get-FileHash. Una cifra que no cuadra con el registro invalida la verificación entera.

1.7 No engañar

Nunca se presenta como comprobado lo que no se ha comprobado. Se dice sobre cuántos casos se ha medido, y cinco de cinco no es el cien por cien. Lo que no se ha visto se declara «No comprobado», y lo que no se sabe de dónde sale se dice así en vez de explicarlo.


2. Método de trabajo y reglas de parada

2.1 Dos intentos en una tarea asignada, tres en una depuración abierta

Quien no resuelve una tarea asignada en dos hipótesis comprobadas, para: lo anuncia en su bandeja, documenta el estado y la cede al Jefe de proyecto para que la reasigne. En una depuración sin asignar se para tras tres hipótesis distintas sin resultado. Un intento es una hipótesis comprobada y medida, no cada comando.

2.2 Registro de la parada

Quien para lo documenta en el archivo de defectos conocidos de la documentación: qué falla y cómo se reproduce, qué se ha descartado y con qué evidencia, qué queda como sospecha sin confirmar, dicho así, y cuál sería el siguiente paso y por qué no se ha dado.

2.3 Fuentes solventes

Solo valen fuentes oficiales o de solvencia acreditada: repositorios oficiales, documentación nativa y especificaciones. Lo que no se encuentre con seguridad se declara «No encontrado / No comprobado»; está prohibido inventar datos, firmas, parámetros o comportamientos. Todo informe técnico cita la dirección exacta de su fuente.

2.4 Cero basura

Lo superado no se queda junto a lo vigente: se borra si no aporta nada, o pasa a .gestor/archivo/ si conviene conservarlo. No se dejan copias, volcados, guiones sueltos, temporales ni archivos sin seguimiento en el árbol, y lo temporal se borra en cuanto termina la tarea. No se conservan documentos desfasados ni notas contradictorias. Al cerrar una sesión se detienen los procesos que el agente haya lanzado, salvo su vigilante si sigue trabajando.


3. Comunicación y coordinación

3.1 El canal en .gestor/canal/

Toda la comunicación entre los agentes y con el Director pasa por .gestor/canal/. estado.md recoge solo hechos comprobables, cada uno con su fecha y quién lo anota: Jefe de proyecto, rama, commit base, versión, reparto de funciones, autoexclusiones, reservas, mutaciones vivas y prohibiciones. Cada agente tiene una bandeja que solo escribe él: claude.md para Claude 1, claude2.md para Claude 2, codex.md para Codex 1, gemini.md para Gemini 1, y así sucesivamente.

3.2 Reglas del canal

- Nadie edita el archivo de otro agente, ni para ordenar ni para corregir. Lo que no está en la bandeja de un agente escrito por él, ese agente no lo ha dicho, y una aceptación solo vale en el archivo de quien la hizo.
- Cada intervención empieza con su latido y la fecha y hora tomadas del sistema, por ejemplo «LATIDO: 2026-10-26 08:30». Un latido desfasado indica que el agente no está.
- Un encargo solo existe si va dirigido a alguien con «Para <agente>». Mencionar a alguien no es encargarle nada.
- Mientras trabaja, cada agente mantiene un vigilante del canal que le avisa de los mensajes nuevos de los demás (un monitor en segundo plano, o mirar el canal al terminar cada paso si su herramienta no lo permite) y lo vuelve a armar cuando caduca. Un mensaje para él se atiende en cuanto llega.
- Las bandejas se mantienen podadas, con unas ochenta líneas vivas. Lo resuelto pasa al histórico propio del agente en su carpeta; Gestor Agentes avisa cuando una bandeja crece demasiado.

3.3 Reservas de archivos

Antes de modificar cualquier archivo del repositorio, mutaciones incluidas, se anota en el apartado de reservas de estado.md («Claude 2: ruta/al/archivo, motivo»), y se borra en cuanto termina la tarea.

3.4 Director, Jefe de proyecto y ayudantes

El Director asigna al empezar el proyecto, diciéndoselo a un agente, el puesto de Jefe de proyecto; ese agente lo anota en estado.md con la fecha. Los demás son ayudantes. El Jefe reparte el trabajo y es el interlocutor del Director: el Director habla con el Jefe, y el Jefe con sus ayudantes. Las decisiones técnicas se acuerdan entre todos en el canal, con igual autoridad técnica; en caso de dudas o de desacuerdo decide el Director, previa consulta del Jefe. Si el Jefe no está (latido desfasado), los ayudantes siguen con lo asignado y le dejan escrito lo que necesiten. Si no hay Jefe anotado, el primer agente que arranque pide al Director que lo asigne.

3.5 Traslado inmediato de lo que dice el Director

Toda directriz del Director a un agente se comunica en el momento a los demás por el canal, separando sus palabras literales de la interpretación de quien las recibió. Las órdenes permanentes se registran con su fecha en .gestor/ordenes-del-director.md, donde solo se añade texto: una orden se deroga con otra que la cite. Lo que ya está decidido se aplica sin volver a preguntarlo.

3.6 Diario de tarea en tiempo real

Cada agente mantiene su diario en .gestor/<agente>/tarea-en-curso.md: escribe el plan antes de ejecutar y anota cada hito al alcanzarlo, para que otro pueda retomar el trabajo en el punto exacto.

3.7 Inicio y fin de sesión

Al iniciar: se determina la identidad y se emite el mensaje de arranque de la cabecera; se leen .gestor/instrucciones-del-proyecto.md, estado.md, todas las bandejas y el diario propio; se comprueba quién es el Jefe de proyecto; se consolida el bloque pendiente si lo hay; se retoma la tarea en curso sin pedir otra vez el contexto; se comprueba que no haya mutaciones sin declarar ni reservas que bloqueen; se arma el vigilante del canal; y se lee .gestor/control.md, que si no existe se crea como indica .gestor/consulta/metodo.md antes de cambiar nada del producto.

Al terminar: se actualiza estado.md y se liberan las reservas propias; se registran las aceptaciones en el acta propia y los resultados en el registro de funciones; y se ponen al día el historial de cambios y .gestor/control.md. Antes de pedir un reinicio de contexto, el agente deja su diario con el próximo paso exacto, la bandeja podada y sus reservas, mutaciones y acta al día.


4. Identidad y funciones

4.1 Identidad

Cada agente se identifica por su herramienta y su número de instancia: Claude 1, Claude 2, Codex 1, Gemini 1. La primera instancia es la 1. Cada instancia tiene su bandeja y su carpeta propia en .gestor/: claude para Claude 1, claude2 para Claude 2, y así con las demás. En ella guarda su diario (tarea-en-curso.md), su acta de verificaciones (verificaciones.md) y su registro de funciones (funciones.md).

4.2 Reparto de funciones

Ningún agente tiene un papel fijo por ser quien es. El Jefe de proyecto propone en el canal quién hace cada cosa (programar, verificar, depurar, investigar, documentar, preparar entregas), lo acuerdan entre todos y él lo anota en estado.md. El reparto se revisa cuando cambian los agentes o cuando los resultados lo aconsejan. Quien delega verifica el resultado de quien ejecuta. Cada agente lleva su registro de funciones y se autoexcluye de una función con malos resultados, con las reglas de .gestor/consulta/metodo.md.


5. Código

Antes de escribir, cambiar o revisar código, el agente lee entero .gestor/consulta/codigo.md: principios SOLID, código limpio, invariantes del dominio, errores tipados, seguridad, capas, tipado, rendimiento, pruebas y documentación del código. Sus reglas son de obligado cumplimiento. Tres de ellas no se negocian ni se aplazan, y si una tarea no se puede terminar respetándolas se para y se dice: refactorización obligatoria en verde, sin borrar ni debilitar ninguna prueba; piezas pequeñas de una sola responsabilidad, con la lógica que decide separada de la entrada y salida; y nada incrustado en el código, con cada valor configurable en la configuración y cada cifra con significado en una constante con nombre y origen.


6. Documentación

6.1 Estilo de escritura

Todo lo que escriben los agentes fuera del código (documentación, control, canal, diarios, actas, órdenes y mensajes de commit) se redacta en prosa humana, con frases completas y naturales. No se usan asteriscos ni almohadillas: los títulos son líneas normales separadas por una línea en blanco, y las listas usan guiones solo cuando ayudan a leer. Los bloques de código y los formatos con sintaxis propia quedan fuera de esta regla.

6.2 Sincronía permanente

La documentación cambia al mismo tiempo que el código: al final de cada sesión se actualiza el historial de cambios y todo documento afectado. Una documentación desfasada es peor que ninguna. Su estructura está en .gestor/consulta/metodo.md, y se lee al crear o reorganizar la documentación.


7. Versiones y publicación

7.1 Git y versiones

Todo proyecto está bajo git y .gestor/ figura en su .gitignore. La versión sigue Semantic Versioning 2.0.0 (https://semver.org/lang/es/), se declara en un solo sitio, el manifiesto del proyecto, y cada entrega lleva su etiqueta vMAYOR.MENOR.PARCHE y su entrada en el historial de cambios. Los mensajes de commit dicen quién implementó y quién verificó cada cambio.

7.2 Prohibiciones permanentes

- No se publica ni se distribuye ninguna versión sin la autorización explícita del Director para esa publicación concreta, y antes de pedirla el Director hace su prueba humana del candidato.
- No se hace push a ningún repositorio remoto sin esa misma autorización.
- Antes de preparar cualquier publicación se lee entero .gestor/consulta/metodo.md, que dice qué se publica y qué no.


8. Normas de la carpeta proyecto

Debajo de este núcleo, Gestor Agentes copia las normas comunes de la carpeta que contiene a todos los proyectos. Obligan igual que este archivo. Claude Code, además, carga esas normas por su cuenta desde su archivo original, así que Claude las ve dos veces; no es un error. Si las dos copias difieren, manda la original, y la copia se pone al día en el siguiente lanzamiento.
