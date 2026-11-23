Arquitectura


Visión general

Gestor Agentes es una aplicación de escritorio de una sola ventana, sin servidor ni base de datos, escrita en Rust. Guarda todo en unos pocos archivos de texto dentro de la carpeta datos, junto al ejecutable, y por eso es portable. El workspace de cargo tiene cinco crates.

El crate canal es una biblioteca pequeña, sin disco, sin reloj y sin dependencias, que lee el canal de los agentes: la identidad de un agente, los encargos dirigidos a él, el apartado de mutaciones vivas de estado.md y las huellas con que se reconoce un texto ya visto. La comparten el vigilante y el núcleo, para que los dos entiendan el canal igual.

El crate nucleo es una biblioteca sin interfaz ni dependencias de plataforma. Contiene la lógica que escribe en los proyectos del usuario: las identidades de los agentes, la configuración y los ajustes, las instrucciones del proyecto, la estructura de la carpeta .gestor, el archivo de lo que usaba el sistema anterior, git y la detección de cuentas. Es la parte crítica y la más cubierta por pruebas.

El crate lanzador es una biblioteca que decide cómo se abre cada agente en una terminal, sea Claude Code, codex, Gemini con agy o Programator, en Windows y en Linux, y después lo abre. La decisión es una función pura que recibe el sistema a través de un rasgo inyectado, de modo que se prueba sin abrir nada.

El crate vigilante genera el segundo ejecutable, gestor-vigilante, que vigila el canal por un agente. Va en el paquete junto a gestor-agentes, y la preparación lo copia a cada proyecto.

El crate interfaz, cuyo paquete se llama gestor-agentes, es el ejecutable principal. Dibuja la ventana con egui a través de eframe, compone las bibliotecas, guarda el estado de la ventana y lleva los trabajos largos a un hilo aparte. Su lógica no depende de egui y se prueba sin abrir ninguna ventana.

El ejemplo crates/nucleo/examples/oraculo.rs prepara un árbol de proyectos de prueba con una hora fija. Se usó para comparar byte a byte la salida de la 3.0.0 con la de la versión anterior, y sigue sirviendo para comparar la salida de dos versiones cualesquiera.


Estructura de carpetas

En la raíz están Cargo.toml, que declara el workspace, la versión común y los niveles de los avisos del compilador; Cargo.lock, con las versiones exactas de las dependencias; y .gitignore.

En crates/nucleo/src, agentes.rs define los tipos de agente y su identidad, de la que se derivan el nombre visible, la bandeja del canal, la carpeta propia y el archivo de reglas que lee cada herramienta. La carpeta configuracion guarda y lee la configuración portable de la carpeta datos: la lista de proyectos, los perfiles de agente con su esquema y su puesta al día, y la copia desde la versión anterior. ajustes.rs lee ajustes.json, con el tiempo máximo de git y el límite de tamaño de una bandeja. cuentas.rs reconoce las cuentas de Claude Code y de Codex instaladas en la carpeta personal y los perfiles que les corresponden. git.rs es el servicio de git, con el ejecutor de procesos inyectado y un tiempo máximo por orden. plantillas.rs incluye en el binario los textos canónicos de crates/nucleo/plantillas. aprendizaje.rs lee las fichas de aprendizaje común, que se guardan en la carpeta aprendizaje-comun de la raíz y viajan dentro del binario, compone con las comprobadas el documento de consulta de los proyectos y recoge las reglas candidatas. textos.rs reúne en un solo sitio los textos del núcleo que ve el usuario.

La carpeta instrucciones de nucleo es lógica pura, sin disco ni reloj. Contiene las marcas de región, el analizador, el formato de los archivos, que respeta la marca BOM y los saltos de línea, y la huella SHA-256 según FIPS 180-4. Contiene también la decisión de si un archivo que la aplicación escribe entero se deja, se crea o se sustituye con copia previa, y la recogida del texto propio de los archivos de instrucciones anteriores.

La carpeta proyecto de nucleo es la parte que toca el disco. El inicializador orquesta la preparación; el catálogo dice qué archivos escribe la aplicación y con qué texto; el localizador busca las normas del espacio de trabajo; la estructura crea lo que falta de .gestor; el módulo del sistema anterior archiva lo que ya no se usa; el de archivos lee, escribe de forma atómica y respalda; el registro de huellas guarda la huella de lo escrito; el informe recoge cambios, avisos y errores; y el de estado calcula la etiqueta de cada proyecto en la lista. papeles.rs añade a estado.md los apartados que le faltan y anota el papel elegido en la ventana; vigilante.rs copia gestor-vigilante al proyecto y retira lo del vigilante anterior; vigilancia.rs lee lo que deja cada vigilante para el panel del equipo; y citas_de_reglas.rs busca el código del proyecto que cita los archivos de reglas. La hora llega siempre de un reloj inyectado.

En crates/vigilante/src, opciones.rs interpreta la línea de órdenes sin tocar nada; pasada.rs decide, sin disco, qué ha cambiado y con qué etiquetas; recuerdo.rs guarda lo visto en la carpeta del agente; procesos.rs consulta el árbol de procesos para terminar cuando muere quien lo lanzó; entrega.rs hace llegar el aviso a la sesión de Codex; vivo.rs deja el rastro que lee la ventana; y ejecucion.rs junta todo en los tres modos. main.rs solo lee las opciones, la identidad y la carpeta .gestor.

En crates/lanzador/src, lib.rs publica el contrato: planificar, detectar_ejecutable, ejecutar y sus tipos. planificacion.rs decide; agy.rs busca el proyecto de Antigravity que tiene la carpeta; terminal.rs compone las órdenes y los guiones de PowerShell y de bash; escapes.rs y base64.rs protegen los datos que van dentro de esos guiones; entorno_real.rs es la única parte que consulta el sistema; ejecucion.rs lanza el plan; y textos.rs reúne sus mensajes.

En crates/interfaz/src, aplicacion.rs guarda el estado y las reglas de la ventana, con los servicios inyectados. trabajos.rs reúne lo que corre fuera del hilo de la ventana. sistema.rs contiene las implementaciones reales sobre el disco, el reloj, el núcleo y el lanzador. arranque.rs y rutas.rs resuelven la carpeta de datos y la copia desde la versión anterior. perfiles.rs, orden.rs, fecha.rs y textos.rs son piezas de apoyo, y ventana.rs, tema.rs e icono.rs son las tres que dibujan. main.rs solo compone.

Cada crate tiene sus pruebas en su carpeta tests, aparte del código que vigilan, para que sigan intactas aunque se dañe un archivo de implementación.

La carpeta documentation guarda esta documentación, los guiones de preparación del entorno, los de las comprobaciones de calidad y los de la compilación portable. La carpeta portable es la salida de build-portable y no se versiona, igual que target.


Cómo se prepara y se abre un proyecto

Al pulsar el botón de un perfil, la ventana encarga un trabajo al hilo aparte. Allí, MotorReal llama a InicializadorDeProyecto::inicializar con la carpeta del proyecto y la identidad del perfil, y después al lanzador. El inicializador da diez pasos, y si uno falla lo anota en el informe y sigue con los demás.

En el primer paso se ocupa de git. Si la carpeta no es un repositorio ejecuta git init, y se asegura de que .gestor esté en .gitignore. Cada orden de git lleva la opción -c safe.directory, así que nunca toca la configuración global de git, y se detiene si pasa del tiempo máximo de ajustes.json.

En el segundo, si falta el archivo propio del proyecto, lee el texto propio que haya en los CLAUDE.md, AGENTS.md y GEMINI.md anteriores. Va antes del tercero porque el tercero se lleva GEMINI.md.

En el tercero lleva a .gestor/archivo lo del sistema anterior: el canal antiguo de Gemini, GEMINI.md y la memoria compartida. Nunca borra nada.

En el cuarto crea lo que falte de .gestor para la instancia 1 de cada herramienta y para la identidad que se va a abrir, y añade a estado.md los apartados Papeles, Reservas activas y Mutaciones vivas que le falten. No sobrescribe nada de lo que ya existe en .gestor.

En el quinto crea .gestor/instrucciones-del-proyecto.md si falta, con lo recogido en el segundo paso dentro de un bloque pendiente de consolidar. Si ya existe, no lo toca nunca. Lleva además la cuenta del bloque pendiente en el registro de huellas: cuando un agente lo ha consolidado y en esa preparación no se ha archivado nada, borra las carpetas del archivo que dejó la aplicación. En este mismo paso copia a candidatas-comunes.md, en la carpeta de datos, las reglas aprendidas que el proyecto marca como candidatas a común.

En el sexto escribe enteros AGENTS.md, con la cabecera común, el núcleo común y las normas del espacio de trabajo que encuentra el localizador; CLAUDE.md, el puntero que carga AGENTS.md y el archivo propio; y los documentos de .gestor/consulta, entre ellos el de aprendizaje común. Si lo que había no es lo último que escribió la aplicación, antes lo copia en .gestor/respaldos. Después copia gestor-vigilante a .gestor/vigilante, si falta o es distinto, y retira los guiones del vigilante anterior y sus referencias canal-visto.

En el séptimo añade al archivo de permisos de Claude Code las reglas del arranque que le falten.

En el octavo avisa del código del proyecto que cita AGENTS.md, CLAUDE.md o GEMINI.md, de las bandejas, los diarios y el estado del canal que pasan del límite de ajustes.json y de lo que no cuadra en el estado del canal.

En el noveno guarda en .gestor/regiones-gestionadas.json la huella de lo que ha escrito.

En el décimo deja los avisos en .gestor/avisos-de-preparacion.md, para los agentes, o borra ese archivo si no queda ninguno.

Si el usuario ha elegido un papel en la ventana, la interfaz lo anota en estado.md después de la preparación y antes de lanzar el agente, con papeles.rs, y lo añade al informe como un cambio.

Con el informe ya devuelto, el lanzador planifica la apertura y la ejecuta. Claude Code recibe las variables GESTOR_AGENTE, GESTOR_PROYECTO y CLAUDE_CONFIG_DIR; codex, GESTOR_AGENTE, GESTOR_PROYECTO y CODEX_HOME, y las opciones -s workspace-write y -a on-request si el perfil no fija las suyas; agy, GESTOR_AGENTE y GESTOR_PROYECTO, y la opción --project con el proyecto de Antigravity que tiene la carpeta, sin el cual no se abre; y Programator, solo GESTOR_AGENTE. Todos se abren en una terminal nueva. Claude Code, codex y agy reciben además el mensaje inicial del núcleo, que la interfaz pone en la petición: los dos primeros como último argumento y agy detrás de su opción -i. Al arrancar, la interfaz usa detectar_ejecutable, que es la misma búsqueda que hace el lanzamiento, para mostrar solo los botones de los agentes que hay en el equipo.


Decisiones técnicas

Cinco crates. La lógica que escribe en proyectos ajenos y la que abre procesos se prueban sin ventanas, y la interfaz solo las compone. Las dependencias van en un solo sentido: interfaz depende de nucleo y de lanzador, y también de vigilante, cuya consulta de procesos usa para saber si el vigilante de un agente sigue en marcha; nucleo y lanzador no se conocen; nucleo y vigilante dependen de canal, que no depende de ningún otro crate.

Un solo archivo de reglas. Desde la 3.0.0 todos los agentes leen AGENTS.md; Claude Code lo hace a través del puntero CLAUDE.md, porque no lee AGENTS.md por su cuenta. Mantener tres copias del mismo núcleo, una por herramienta, acababa en copias distintas, y GEMINI.md sobra porque Antigravity lee AGENTS.md.

Archivos escritos enteros y un archivo propio aparte. La aplicación es dueña de AGENTS.md, CLAUDE.md y .gestor/consulta, y los reescribe enteros. Lo que aprende cada proyecto va en .gestor/instrucciones-del-proyecto.md, que no toca nunca. Así no hace falta mezclar texto de la aplicación con texto del usuario en un mismo archivo, y nada se pierde, porque lo que no escribió la aplicación se respalda antes de sustituirlo.

Huellas en lugar de comparar con la plantilla. Comparar un archivo solo con la plantilla vigente no distingue una plantilla vieja de una edición a mano. La huella de lo último que se escribió sí lo distingue, y decide cuándo hace falta respaldo.

Documentos de consulta. Lo que solo se necesita en momentos concretos, como los estándares de código, el detalle de la verificación o el método de documentación y publicación, va en .gestor/consulta, para que AGENTS.md sea corto y quepa holgado en lo que leen las herramientas: 24.000 bytes Antigravity y 32.768 Codex.

Mensaje inicial. Las herramientas no hacen nada hasta recibir un mensaje, y las instrucciones que cargan al abrirse las tratan como contexto, no como una tarea. Sin un primer mensaje, el agente no ejecutaba su arranque hasta que el usuario le escribía, y entonces atendía lo que le pedían antes que el arranque. El mensaje inicial convierte el arranque en la primera tarea de la sesión.

Vigilante en un ejecutable propio. Hasta la 3.5.1 el vigilante eran dos guiones, uno de bash y otro de PowerShell, que había que mantener iguales, y el de bash lanzaba programas en cada comprobación. gestor-vigilante es un solo proceso pequeño, sin intérprete, que entiende el canal con la misma biblioteca que el núcleo. Cada herramienta tiene una forma distinta de enterarse, así que tiene tres modos: uno que espera hasta que hay algo para el agente, para Claude Code y agy, que lo despiertan al terminar una tarea en segundo plano; uno persistente para Codex, que no se despierta así y recibe cada aviso dentro de su sesión con codex queue. Al persistente lo lanza la terminal que abre Codex y no el propio Codex, porque el entorno aislado de Codex no le deja ver los procesos del equipo ni, por tanto, saber qué sesión servir y cuándo termina; Codex solo le deja anotada su sesión en la carpeta del proyecto; y uno de una pasada para las herramientas que no pueden esperar. Los modos que esperan se cuidan solos: un solo vigilante por agente, fin cuando se cierra quien los lanzó y, en el que espera al cambio, una vida máxima, para que nunca queden procesos en marcha sin nadie que los escuche. Toma la identidad de GESTOR_AGENTE, como el resto del sistema, y deja en la carpeta del agente un rastro, vivo.json y avisos.log, que la ventana lee para el panel del equipo.

La carpeta del proyecto en GESTOR_PROYECTO. Una herramienta puede abrir su sesión en otro proyecto que el de la terminal, como agy sin su opción --project. Con la ruta en una variable, el agente comprueba en el primer paso del arranque que está donde debe, y para si no lo está, en lugar de escribir en un proyecto ajeno.

Plantillas incluidas en el binario. El núcleo común viaja dentro del ejecutable y cambia con cada versión, así que todos los proyectos reciben el mismo texto.

Aviso de versión. La decisión es pura y vive en el núcleo, en actualizaciones.rs: construye la dirección de la API de releases, lee la respuesta, valida que el enlace sea del propio repositorio en GitHub y compara las versiones con SemVer. La petición la hace la interfaz, en actualizaciones.rs y sistema.rs, con ureq y TLS de rustls para no depender del OpenSSL del sistema, en un hilo propio y no en el de los trabajos, para que una red lenta no retrase la apertura de un proyecto. El consultor se inyecta y las pruebas no usan la red; el ejemplo ultima_publicacion consulta la API de verdad.

Revisión del canal. proyecto/revision_del_canal.rs es lógica pura: recibe el texto de estado.md, el de las bandejas y la hora, y devuelve avisos y el resumen del equipo. Solo comprueba la forma, porque esos archivos los escriben modelos de lenguaje y un análisis más fino sería frágil. El inicializador pone sus avisos en el informe, como el de las bandejas sin podar, y la interfaz usa el resumen para la línea del equipo, leída al seleccionar un proyecto y al terminar un trabajo, no en cada fotograma.

Permisos del arranque. proyecto/permisos.rs es lógica pura: recibe el texto del archivo de permisos de Claude Code, si existe, y las reglas del arranque, y devuelve el texto nuevo, que no hace falta tocarlo o que el archivo no se entiende. Conserva el resto del archivo y el orden de sus claves. El inicializador lo lee y lo escribe. Codex no tiene un archivo de permisos del proyecto, así que sus permisos van en las opciones con que lo abre el lanzador.

Avisos para los agentes. proyecto/avisos_de_preparacion.rs compone el archivo .gestor/avisos-de-preparacion.md con los avisos de la preparación, o lo borra si no hay ninguno. La interfaz solo abre el diálogo del informe cuando InformeDeInicializacion::requiere_ventana es verdadero, es decir, con errores, respaldos, archivados o migraciones; los avisos solos van a la barra de estado.

Lo antiguo se aparta y se borra al consolidar. La experiencia de los primeros proyectos migrados enseñó que los archivos del sistema anterior que se dejaban a la vista acababan confundiendo a los agentes. La aplicación los aparta, incluidas las reglas que agy carga desde .agents, y las instrucciones prohíben seguirlos; el agente que consolida rescata antes lo que siga vivo. Las dos carpetas del archivo que deja la propia aplicación, sistema-anterior y canal-gemini-antiguo, las borra ella cuando ve que el bloque pendiente ya se ha consolidado, porque el permiso de Claude Code no deja borrarlas al agente. Solo borra lo que dejó anotado, y nunca en la misma preparación en que archiva algo, para que nada se pierda por un error suyo.

Normas del espacio de trabajo copiadas. El localizador busca el CLAUDE.md, AGENTS.md o GEMINI.md más cercano por encima del proyecto que no sea un archivo gestionado, y su texto va al final de AGENTS.md. Se copia porque Codex y Gemini no leen nada por encima de la raíz del repositorio.

Configuración portable en la carpeta datos. Viaja con el ejecutable. config.json y los perfiles conservan el esquema de la versión anterior, para que la migración sea una copia. Los ajustes nuevos van en ajustes.json porque config.json es una lista de proyectos y no admite claves sueltas. Un ajuste sin sentido, como un cero, se rechaza al arrancar en vez de corregirse en silencio.

La identidad en GESTOR_AGENTE. El lanzador sabe con certeza qué perfil ha abierto, mientras que adivinarlo por la carpeta de configuración solo funcionaba con Claude.

safe.directory en cada orden. En algunas unidades, como las de red, git rechaza los repositorios por propiedad dudosa. Pasar la opción en cada llamada lo resuelve sin escribir en la configuración global del usuario.

Sin aplicaciones de escritorio. Todos los agentes se abren en terminal. Las aplicaciones de escritorio de Codex y de Antigravity tienen sus propios accesos directos.

Argumentos como lista. Las órdenes se construyen con listas de argumentos. Donde la plataforma obliga a un guion, como la terminal de PowerShell o de bash, cada dato va entre comillas simples escapadas, y PowerShell recibe la orden codificada con -EncodedCommand.

Dibujo con OpenGL. La ventana usa el motor glow de eframe, que necesita OpenGL 2.0 o superior. Cuando falta, la aplicación muestra un mensaje claro y termina. Se probó el motor wgpu, sobre Direct3D 12, y en un Windows sin aceleración gráfica el proceso quedaba en marcha sin mostrar ninguna ventana ni ningún error, que es peor. En ese caso, el paquete de compatibilidad OpenGL de Microsoft da a Windows el OpenGL que falta.

Sistema de ventanas en Linux. eframe se compila con soporte para X11 y para Wayland, y winit elige al arrancar el que haya, sin decidir nada al compilar.


Verificación

El código se comprueba con cuatro puertas mecánicas, que documentation/infrastructure/puertas-rust.ps1 y puertas-rust.sh ejecutan sobre todo el workspace. La primera es cargo fmt, sin ninguna diferencia. La segunda es cargo clippy en todos los destinos, con los avisos tratados como errores. La tercera es cargo test con las versiones exactas de Cargo.lock. La cuarta es cargo audit, sin vulnerabilidades conocidas. Las cuatro se pasan en Windows y en Linux.
