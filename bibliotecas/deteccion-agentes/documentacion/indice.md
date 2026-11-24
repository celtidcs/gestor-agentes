# Base de conocimiento de deteccion-agentes

Punto de entrada para cualquier agente que tenga que usar, conectar, actualizar o revisar esta biblioteca en un programa. Describe la versión 0.2.0.

## Qué leer según lo que vayas a hacer

| Vas a... | Lee entero |
|---|---|
| Integrar la biblioteca en un programa por primera vez | `norma-de-integracion.md` y `../examples/integracion_de_referencia.rs` |
| Actualizar la copia de un programa a una versión nueva | `norma-de-integracion.md` (N11) y `guia-de-uso-para-agentes.md` (apartado 11) |
| Saber qué hace una función, un tipo o una herramienta | `guia-de-uso-para-agentes.md` |
| Entender por qué no aparece una cuenta o por qué la consola usa otra | `problemas-frecuentes.md` |
| Saber qué programas la usan y con qué versión, o anotar un fallo de la biblioteca | `registro de proyectos_bibliotecas.md`, el registro común (ver abajo) |
| Verificar la integración de otro agente | La lista de comprobación al final de `norma-de-integracion.md` |
| Cambiar la propia biblioteca | `../README.md`, el código de `../src/` y `../mejoras.md` |

## Documentos

- `norma-de-integracion.md`: **obligatoria**. La forma única de conectar la biblioteca (N1 a N11) y la lista de comprobación para aceptar una integración.
- `../examples/integracion_de_referencia.rs`: la norma hecha código. El módulo `cuentas_de_agentes` se copia tal cual en el programa. Cargo lo compila y `cargo test` ejecuta sus pruebas, así que siempre está al día con la biblioteca.
- `guia-de-uso-para-agentes.md`: referencia de la API, con cada herramienta (carpetas, archivos, variables y fuentes oficiales), las reglas de búsqueda, las pruebas sin tocar el equipo real y el paso de una versión a otra.
- `problemas-frecuentes.md`: síntoma, causa y solución de los fallos habituales.
- `registro de proyectos_bibliotecas.md`: el registro común. Está fuera de este repositorio, en la carpeta que lo contiene, no va con las copias y es común a todas las bibliotecas. Cada programa se da de alta en él, anota con qué versión de la copia trabaja y sus desviaciones de la norma, y deja los fallos que encuentre en la biblioteca. Quien mantiene la biblioteca lo lee al empezar cada sesión.

## Reglas que no se olvidan

- El original es este repositorio, con su git y sus etiquetas `vX.Y.Z`. Es el único sitio donde se cambia su código.
- Una copia por programa, en `bibliotecas/deteccion-agentes/`, idéntica a una etiqueta de este repositorio. No se edita allí, y no se refresca sola ni al compilar: se actualiza a propósito (N11).
- Un solo módulo `cuentas_de_agentes` por programa.
- Ninguna cuenta, carpeta ni sufijo escrito en el código del programa: se buscan.
- El archivo de registro MCP y la variable de la consola salen siempre de la cuenta, nunca se componen a mano.
- Al abrir una consola se aplica siempre el ajuste de entorno de la cuenta, también en la cuenta por defecto.
- Sin alta manual para herramientas sin variable de cuenta (hoy, agy).
- Las pruebas usan carpetas temporales y una función de variables propia; nunca la carpeta personal real.
- Después de integrar o actualizar, se pone al día el apartado del programa en el registro común y la documentación del programa.
- Un fallo encontrado en la biblioteca se anota en ese registro, no se arregla en la copia.
