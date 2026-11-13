# Consulta: cómo trabajar con Programator

Documento de consulta de `AGENTS.md`. Lo escribe Gestor Agentes en cada lanzamiento; no lo edites. Léelo entero antes de encargarle algo a Programator. Si existe `.gestor/canal/COMO-ENCARGAR-A-PROGRAMATOR.md`, léelo también: lo publica Programator al arrancar, con los datos de su instalación (contexto del modelo, desempeño medido y comprobadores) y manda sobre este documento si difieren.

## Qué es

- Un modelo local pequeño, bajo un arnés que lo confina. No es un agente autónomo: no ejecuta nada, no edita el proyecto, no navega ni consulta documentación y no recuerda encargos anteriores.
- Es lento: una respuesta de código tarda minutos.
- Solo trabaja si el Director lo ha abierto en este proyecto desde Gestor Agentes. Antes de encargarle algo, comprueba que está vivo: escribe su estado en `.gestor/programator/latido.json` (`reposo`, `atendiendo` o `error`) y anuncia en su terminal cuándo empieza a escuchar.

## Qué pedirle y qué no

| Sí | No |
|---|---|
| Una función o un archivo acotado, con los requisitos en lista | Diagnosticar la causa de un fallo |
| Auditar buscando formas: números sueltos, métodos largos, clases que hacen de todo | Auditar validaciones ausentes o errores de significado |
| Reescribir o traducir código siguiendo un patrón dado | Confirmar que un archivo está limpio (se inventa hallazgos) |
| | Preguntas sobre APIs, versiones o parámetros de bibliotecas (se los inventa con aplomo) |

Regla: pídele que **reconozca formas**, nunca que **entienda comportamientos**.

## Cómo se le encarga

1. Escribe el encargo en **tu propia bandeja**, en una intervención nueva, tras tu latido, con una línea que empiece exactamente por `Para Programator: <nombre corto>`, como cualquier encargo del canal. Mencionarlo en medio de una frase no es un encargo, y si va «con copia a Programator» tampoco. El encargo llega hasta el siguiente `LATIDO:`, la siguiente línea `Para …`, un encabezado `## ` o un `---`. No añadas texto a un encargo anterior: cada corrección es un encargo nuevo.
2. Pon los requisitos uno por línea, todos obligatorios.
3. Un encargo, un archivo. Lo que le quepa en su ventana es lo único con lo que trabaja.
4. Dile cómo entregar: «entrégalo con `escribir_propuesta`, nombre exacto `<archivo>`».
5. Pídele que, antes de entregar, escriba una línea por requisito diciendo cómo lo ha cumplido.
6. Si necesita un fragmento de un archivo grande, pégalo en el encargo con sus números de línea reales, o dale la ruta y el rango para `leer_fichero`.

Los encargos que empiezan por `revisión:` son de la revisión mensual de modelos de Programator. Se hacen solo en la carpeta de Programator, con su procedimiento (`plantillas/revision-mensual.md` de ese proyecto). No los pidas desde otro proyecto.

## Qué devuelve y quién lo verifica

- Responde en `.gestor/canal/programator.md`. Debajo, el arnés añade el resultado de comprobar sus propuestas; esa línea es del arnés, no del modelo.
- Sus archivos quedan en `.gestor/candidatos/programator/` hasta que alguien los revise. Nada suyo entra en el proyecto sin más.
- **Quien encarga, verifica**, con las reglas de `.gestor/consulta/verificacion.md`. Su declaración de cumplimiento no basta: comprueba contra el archivo real y verifica todo lo que diga de bibliotecas.

Fuente: `plantillas/como-encargar-a-programator.md` del proyecto Programator, con las medidas de su uso real.
