Consulta: control del proyecto, registro de funciones, documentación y publicación

Documento de consulta de AGENTS.md. Lo escribe Gestor Agentes en cada lanzamiento, así que no se edita a mano. Cada apartado dice cuándo se lee; sus reglas obligan igual que AGENTS.md.


1. Contenido de .gestor/control.md

Se lee al crear .gestor/control.md o al ponerlo al día. El control de proyecto recoge la descripción y la forma de ejecutar el proyecto, el estado actual, el historial de sesiones, las tareas y defectos priorizados, la arquitectura y las decisiones con su justificación, y las notas o advertencias que necesite un relevo. En una primera sesión, la exploración termina solo cuando esas seis áreas permiten retomar el trabajo sin conocimiento previo.


2. Registro de funciones

Se lee al anotar un encargo, una aceptación, una parada o un rechazo. Cada agente lleva en su funciones.md, función por función, cuántos encargos ha asumido, cuántos le han aceptado, cuántas veces ha parado por la regla de los dos intentos cediendo la tarea, y cuántos rechazos ha recibido en revisión. Cada rechazo cita la entrada del acta del revisor donde consta su motivo, porque un rechazo que no está en el acta de quien revisó no cuenta. El registro se anota con honestidad, también cuando perjudica a quien lo escribe.


3. Autoexclusión

Se lee al anotar un resultado negativo. Un agente se autoexcluye de una función cuando acumula en ella tres resultados negativos, sumando paradas por la regla de los dos intentos y rechazos de revisión, y esos resultados negativos igualan o superan a sus encargos aceptados. Por ejemplo, si un agente ha intentado escribir código cinco veces y en tres ha tenido que ceder la tarea o le han rechazado el resultado, deja de programar. Al autoexcluirse lo anuncia en su bandeja, lo anota en el apartado de autoexclusiones vigentes de estado.md y, desde ese momento, cede esa función a quien la haga mejor. La exclusión solo se levanta por orden expresa del Director.


4. Estructura de la documentación

Se lee al crear o reorganizar la documentación. La documentación del proyecto vive en documentacion/, con estos archivos:

- README.md: propósito, tecnologías, instalación y arranque, con enlaces al resto.
- funcionalidades.md: catálogo completo del comportamiento y de los casos de uso.
- manual-de-uso.md: flujos paso a paso y configuración, para el usuario final.
- arquitectura.md: módulos, cómo se comunican y decisiones justificadas.
- historial-de-cambios.md: registro por versión y por sesión.
- defectos-conocidos.md: defectos abiertos e investigaciones detenidas.
- infraestructura/vision-general.md: mapa de componentes, qué corre en local y qué fuera, y variables de entorno.
- infraestructura/requisitos.md: sistemas, versiones exactas y herramientas.
- infraestructura/servicios.md: servicios y protocolos externos, con su autenticación descrita sin secretos.
- infraestructura/guia-de-clonado.md: reproducción del entorno desde cero, que pueda seguir alguien que no conoce el proyecto.
- infraestructura/preparar-entorno.ps1 y infraestructura/preparar-entorno.sh: preparación automática en Windows y en Linux o macOS. Solo uno de los dos si el proyecto es exclusivo de un sistema.

Los guiones de preparación son idempotentes, están comentados en español y verifican sistema, entorno de ejecución y herramientas, instalan dependencias, crean la configuración o sus plantillas sin secretos, preparan datos o migraciones si hacen falta, compilan, hacen una comprobación final e indican cómo arrancar. Se ejecutan en su plataforma antes de darlos por completos.

Si el proyecto tiene la estructura anterior (documentation/ con nombres en inglés, o guiones setup.ps1 y setup.sh), se migra a la nueva solo si el cambio no rompe nada. Antes de mover un archivo se busca mecánicamente toda referencia a su ruta en guiones, compilación, configuración, integración continua y enlaces de la documentación. Si no hay ninguna, se renombra con git mv. Si la hay, o el cambio puede provocar fallos, se deja el nombre antiguo, se trata como equivalente del nuevo y se anota la decisión en .gestor/instrucciones-del-proyecto.md.


5. Qué se publica cuando hay autorización

Se lee antes de preparar cualquier publicación. El repositorio público es un producto de software, no el registro de cómo se construyó. Se sube lo que un desarrollador ajeno necesita para clonar, compilar y seguir el desarrollo por su cuenta, o lo que un usuario necesita para usar la aplicación. No se sube nada del método de trabajo interno: ni .gestor/, ni AGENTS.md o CLAUDE.md, ni actas, diarios, órdenes del Director o documentos que narren auditorías, mutaciones o acuerdos entre agentes, ni ganchos de git que solo tengan sentido con .gestor/ presente, ni la infraestructura personal del Director. Los nombres de los agentes pueden aparecer como funcionalidad del producto, nunca para contar quién hizo qué. El historial de cambios y los defectos conocidos se publican solo en una redacción para el usuario, sin atribuciones internas. El historial de git interno no se publica sin filtrar: se construye un árbol curado con historia propia.

Antes de pedir la autorización, el candidato se fija con su huella (la suma sha256 de lo que se publicaría y el commit del que sale) y el Director hace su prueba humana: lo usa como lo usaría alguien que lo recibe. Después se le presenta la lista exacta de lo que se incluye y lo que se excluye, y se espera su confirmación, porque lo que se expone puede quedar indexado o clonado aunque se borre después.
