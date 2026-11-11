Consulta: estándares de código, arquitectura e ingeniería

Documento de consulta de AGENTS.md. Lo escribe Gestor Agentes en cada lanzamiento, así que no se edita a mano. Se lee entero antes de escribir, cambiar o revisar código, y sus reglas son de obligado cumplimiento en cualquier cambio.


1. Principios SOLID

Cada módulo, estructura o función tiene una sola razón para cambiar, sin mezclar lógica de negocio con presentación, acceso a disco o gestión de eventos. Los componentes se extienden por composición, rasgos o interfaces, sin modificar código probado para meter casos nuevos. Las implementaciones cumplen estrictamente los contratos que implementan. Las interfaces son pequeñas y específicas. Los módulos de alto nivel dependen de abstracciones y reciben inyectadas sus dependencias.


2. Código limpio

Los nombres dicen su intención sin necesidad de descifrarlos. Cada función hace una sola cosa. La lógica repetida se extrae a una función o módulo común. Se elige siempre el camino más simple y robusto, sin abstracciones para necesidades hipotéticas. Los casos límite y los errores se tratan primero, con retornos tempranos, para no anidar condiciones. Las variables son inmutables salvo que la mutación sea imprescindible, y entonces se confina al ámbito más estrecho posible.


3. Estado e invariantes del dominio

El estado nuevo se incorpora al módulo que conserva su invariante y se modifica mediante una operación con significado. Está prohibido hacer público un campo o mutarlo desde la interfaz por comodidad: si la interfaz necesita cambiar el estado, llama a un método del dominio que preserve su coherencia.


4. Errores centralizados y tipados

No se dispersan capturas genéricas con mensajes ambiguos. Los errores son tipos ricos, con contexto, origen y gravedad, y se propagan de forma controlada hasta la capa exterior, que decide cómo presentarlos. Las entradas inválidas se rechazan en la frontera del sistema, antes de llegar a la lógica interna, y se validan con contratos o esquemas reutilizables en lugar de validaciones manuales repetidas. Los fallos se registran con fecha, origen, contexto y cadena causal suficientes para reproducirlos.


5. Seguridad en el código

Ninguna contraseña, clave, token o credencial figura en el código ni en el historial de git. Todo dato externo se sanea para impedir inyecciones de órdenes, secuencias de escape o recorridos de rutas. Cada componente recibe solo los permisos que necesita. Las consultas y las órdenes de consola se construyen con parámetros, nunca concatenando texto sin escapar. Las dependencias se mantienen al día y se auditan con la herramienta oficial de su ecosistema (cargo audit, npm audit, pip-audit, dotnet list package con la opción de vulnerables, o la que corresponda).


6. Separación de capas

La interfaz, el dominio, la persistencia y los conectores externos o de red se mantienen aislados, y la capa de presentación nunca llega a la persistencia ni a la red sin pasar por los contratos de servicio. La arquitectura real se comprueba en el árbol; no se inventan rutas para aparentar una separación. La configuración vive fuera del código. Cada archivo tiene una responsabilidad coherente.


7. Tipado fuerte

Se aprovecha al máximo el sistema de tipos del lenguaje: enumeraciones exhaustivas, tipos algebraicos, interfaces cerradas y tipos envoltorio donde aporten. Se evitan las conversiones sin verificar y los descartes silenciosos de errores, como capturas vacías, desenvolturas forzadas o aserciones en código de producción.


8. Rendimiento

Nunca se bloquea el hilo de la interfaz o del bucle de eventos con entrada y salida o cálculos pesados; se usan hilos de trabajo o mecanismos asíncronos. No se lee el disco una y otra vez cuando basta una caché coherente que se invalide con precisión. Los datos se cargan cuando hacen falta. Con base de datos se pagina, se evitan las consultas en cascada, los índices se crean a partir de mediciones y las conexiones se reutilizan con un pool.


9. Pruebas

El código se escribe para poder probarlo: funciones puras, dependencias inyectadas e interfaces que se puedan sustituir. Cada funcionalidad nueva lleva pruebas unitarias de su lógica si el proyecto tiene marco de pruebas; si no lo tiene y nadie lo ha pedido, el código queda al menos estructurado para poder probarlo en el futuro. Las pruebas unitarias no dependen de red, GPU, modelos ni del reloj real.


10. Tres órdenes permanentes, no sugerencias

Estas tres no se negocian ni se aplazan a la siguiente versión. Si una tarea no se puede terminar respetándolas, se para y se dice, en vez de entregar deuda.

Refactorización obligatoria. El código queda mejor de lo que estaba en cada tanda: la duplicación, el nombre que miente, la función que hace dos cosas o la abstracción que se quedó corta se arreglan en el momento, no se apuntan para nunca. Se refactoriza en verde: primero pasa la suite, luego se reestructura y después vuelve a pasar, y nunca se refactoriza y se añade funcionalidad en el mismo paso. Ninguna prueba existente se borra ni se debilita para que entre un cambio; si estorba, o el cambio está mal o la prueba describe otro contrato, y eso se discute. Si refactorizar cambia lo que hace el programa, ya no es refactorizar y necesita su propia prueba.

Piezas pequeñas de una sola responsabilidad. Cada módulo trata un asunto y cada función toma una decisión. Lo que obliga a partir un módulo es que asuma más de una responsabilidad, nunca su longitud: no hay tope de líneas, y un módulo de mil líneas que cumple los principios por necesidad real del diseño está bien así. La lógica que decide se separa de la entrada y salida, para probarla sin tocar disco, red ni reloj. Una función con banderas booleanas por parámetro son dos funciones metidas en una. Ningún módulo aborta en producción: todo error se propaga con tipos de resultado o excepciones tipadas.

Nada incrustado en el código. Ningún valor que el Director pueda querer cambiar vive dentro del código: rutas, puertos, tiempos de espera, límites, modelos y versiones de servicios van al archivo de configuración del proyecto, con su valor por defecto declarado y documentado. Toda cifra con significado lleva una constante con nombre en español y un comentario que diga de dónde sale. No hay rutas absolutas de una máquina concreta; la configuración local del Director vive en archivos de configuración ignorados por git. Los textos que ve el usuario se componen en un solo sitio.


11. Documentación del código

Todo el código se documenta en español, con rigor ortográfico y terminológico y con el formato nativo del lenguaje (comentarios de documentación, docstrings, JSDoc o el equivalente). Toda estructura, enumeración o interfaz pública explica su propósito, sus invariantes y su responsabilidad. Toda función pública explica su cometido, sus parámetros, lo que devuelve, sus condiciones de error y sus efectos secundarios. Los bloques no evidentes explican el razonamiento, y las constantes, por qué tienen ese valor.
