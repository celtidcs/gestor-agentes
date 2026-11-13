# Consulta: control del proyecto, registro de funciones, documentación y publicación

Documento de consulta de `AGENTS.md`. Lo escribe Gestor Agentes en cada lanzamiento; no lo edites. Cada apartado dice cuándo se lee. Sus reglas obligan igual que `AGENTS.md`.

## 0. Consolidar el bloque pendiente

**Cuándo:** al arrancar, si `.gestor/instrucciones-del-proyecto.md` tiene un bloque entre marcas `GESTOR:LEGADO`. Lo hace el primer agente que arranque, antes de cualquier otro trabajo.

1. Resérvalo en `estado.md`. Si ya lo ha reservado otro, no lo toques.
2. Revísalo regla por regla:
   - Lo que ya dicen `AGENTS.md` o las normas de la carpeta proyecto: se descarta.
   - Lo particular del proyecto que siga vigente: se queda en ese archivo, redactado como una regla para todos los agentes.
   - Lo que contradiga el núcleo: no lo descartes por tu cuenta, porque pudo ser una orden del Director. Pregúntaselo a través del Jefe.
3. Si el bloque cita archivos del sistema anterior llevados a `.gestor/archivo/`, mira si guardan una tarea viva y trasládala a tu diario y a `estado.md`.
4. Al terminar, ya rescatado lo vivo:
   - borra el bloque con sus marcas;
   - borra `.gestor/archivo/sistema-anterior/` y borra `.gestor/archivo/canal-gemini-antiguo/`, los que llevó allí Gestor Agentes, para que no quede lastre;
   - libera la reserva y anota en tu diario qué conservaste, qué descartaste y qué borraste.

La copia exacta de los CLAUDE.md y AGENTS.md anteriores sigue en `.gestor/respaldos/`.

## 1. Contenido de `.gestor/control.md`

**Cuándo:** al crear `.gestor/control.md` o al ponerlo al día.

El control de proyecto recoge seis áreas:
1. La descripción y la forma de ejecutar el proyecto.
2. El estado actual.
3. El historial de sesiones.
4. Las tareas y los defectos, priorizados.
5. La arquitectura y las decisiones, con su justificación.
6. Las notas o advertencias que necesite un relevo.

En una primera sesión, la exploración termina solo cuando esas seis áreas permiten retomar el trabajo sin conocimiento previo.

## 2. Registro de funciones

**Cuándo:** al anotar un encargo, una aceptación, una parada o un rechazo.

- Cada agente lleva en su `funciones.md`, función por función:
  - cuántos encargos ha asumido;
  - cuántos le han aceptado;
  - cuántas veces ha parado por la regla de los dos intentos, cediendo la tarea;
  - cuántos rechazos ha recibido en revisión.
- Cada rechazo cita la entrada del acta del revisor donde consta su motivo. Un rechazo que no está en el acta de quien revisó no cuenta.
- Se anota con honestidad, también cuando perjudica a quien lo escribe.

## 3. Reglas aprendidas

**Cuándo:** al registrar un rechazo de verificación o una parada por la regla de los dos intentos.

Si la causa es una **trampa reproducible** del proyecto o de su entorno (una orden que en esta máquina necesita una opción, una ruta que engaña, una herramienta que mide otra cosa), conviértela en una regla para que nadie vuelva a caer:

1. Busca antes en `## Reglas aprendidas` de `.gestor/instrucciones-del-proyecto.md` si ya está. Si está, no la repitas.
2. Añádela allí en **una sola línea**: `- <regla>. (<fecha>, <tu identidad>, <origen: acta o diario>)`.
3. Escribe la regla como una orden para cualquier agente, no como el relato del fallo.
4. Si el apartado pasa de 30 reglas, el Jefe las agrupa y quita las que ya no se aplican.

No es una trampa reproducible un error de razonamiento propio sin causa en el entorno: eso va a tu registro de funciones, no a las reglas.

## 4. Autoexclusión

**Cuándo:** al anotar un resultado negativo.

- Te autoexcluyes de una función cuando se cumplen las dos condiciones:
  1. acumulas en ella tres resultados negativos, sumando paradas por la regla de los dos intentos y rechazos de revisión; y
  2. esos resultados negativos igualan o superan a tus encargos aceptados.
- Ejemplo: si has intentado escribir código cinco veces y en tres has tenido que ceder la tarea o te han rechazado el resultado, dejas de programar.
- Al autoexcluirte: lo anuncias en tu bandeja, lo anotas en el apartado de autoexclusiones vigentes de `estado.md` y cedes esa función a quien la haga mejor.
- La exclusión solo se levanta por orden expresa del Director.

## 5. Estructura de la documentación

**Cuándo:** al crear o reorganizar la documentación.

La documentación del proyecto vive en `documentacion/`:

| Archivo | Contenido |
|---|---|
| `README.md` | Propósito, tecnologías, instalación y arranque, con enlaces al resto |
| `funcionalidades.md` | Catálogo completo del comportamiento y de los casos de uso |
| `manual-de-uso.md` | Flujos paso a paso y configuración, para el usuario final |
| `arquitectura.md` | Módulos, cómo se comunican y decisiones justificadas |
| `historial-de-cambios.md` | Registro por versión y por sesión |
| `defectos-conocidos.md` | Defectos abiertos e investigaciones detenidas |
| `infraestructura/vision-general.md` | Mapa de componentes, qué corre en local y qué fuera, y variables de entorno |
| `infraestructura/requisitos.md` | Sistemas, versiones exactas y herramientas |
| `infraestructura/servicios.md` | Servicios y protocolos externos, con su autenticación descrita sin secretos |
| `infraestructura/guia-de-clonado.md` | Reproducción del entorno desde cero, que pueda seguir alguien que no conoce el proyecto |
| `infraestructura/preparar-entorno.ps1` y `.sh` | Preparación automática en Windows y en Linux o macOS. Solo uno si el proyecto es de un único sistema |

Los guiones de preparación:
- son idempotentes y están comentados en español;
- verifican sistema, entorno de ejecución y herramientas;
- instalan dependencias, crean la configuración o sus plantillas sin secretos y preparan datos o migraciones si hacen falta;
- compilan, hacen una comprobación final e indican cómo arrancar;
- se ejecutan en su plataforma antes de darlos por completos.

Si el proyecto tiene la estructura anterior (`documentation/` con nombres en inglés, o guiones `setup.ps1` y `setup.sh`), se migra a la nueva solo si el cambio no rompe nada:
1. Antes de mover un archivo, busca mecánicamente toda referencia a su ruta en guiones, compilación, configuración, integración continua y enlaces de la documentación.
2. Si no hay ninguna, renómbralo con `git mv`.
3. Si la hay, o el cambio puede provocar fallos, deja el nombre antiguo, trátalo como equivalente del nuevo y anota la decisión en `.gestor/instrucciones-del-proyecto.md`.

## 6. Qué se publica cuando hay autorización

**Cuándo:** antes de preparar cualquier publicación.

- El repositorio público es un producto de software, no el registro de cómo se construyó.
- Se sube lo que un desarrollador ajeno necesita para clonar, compilar y seguir el desarrollo por su cuenta, o lo que un usuario necesita para usar la aplicación.
- No se sube nada del método de trabajo interno:
  - ni `.gestor/`, ni `AGENTS.md` o `CLAUDE.md`;
  - ni actas, diarios, órdenes del Director o documentos que narren auditorías, mutaciones o acuerdos entre agentes;
  - ni ganchos de git que solo tengan sentido con `.gestor/` presente;
  - ni la infraestructura personal del Director.
- Los nombres de los agentes pueden aparecer como funcionalidad del producto, nunca para contar quién hizo qué.
- El historial de cambios y los defectos conocidos se publican solo en una redacción para el usuario, sin atribuciones internas.
- El historial de git interno no se publica sin filtrar: se construye un árbol curado con historia propia.

Antes de pedir la autorización:
1. Fija el candidato con su huella: la suma sha256 de lo que se publicaría y el commit del que sale.
2. El Director hace su prueba humana: lo usa como lo usaría alguien que lo recibe.
3. Preséntale la lista exacta de lo que se incluye y lo que se excluye, y espera su confirmación, porque lo que se expone puede quedar indexado o clonado aunque se borre después.
