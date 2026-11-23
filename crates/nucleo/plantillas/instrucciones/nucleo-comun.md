# Núcleo común

Reglas obligatorias para todos los agentes. Cada documento de `.gestor/consulta/` se lee entero cuando llega su momento:

| Cuándo | Lee entero |
|---|---|
| Siempre, al empezar | `.gestor/instrucciones-del-proyecto.md` (reglas de este proyecto; obligan igual que este archivo) |
| Antes de escribir, cambiar o revisar código | `.gestor/consulta/codigo.md` |
| Antes de verificar, aplicar una mutación o entregar | `.gestor/consulta/verificacion.md` |
| Antes de encargarle algo a Programator | `.gestor/consulta/programator.md` |
| Al consolidar el bloque pendiente, al tocar `control.md`, funciones, documentación o una publicación | `.gestor/consulta/metodo.md` |
| Antes de montar algo para una herramienta o cuando una falle | `.gestor/consulta/aprendizaje-comun.md` |

Si .gestor/instrucciones-del-proyecto.md contiene un bloque pendiente de consolidar (`GESTOR:LEGADO`), lo consolida el primer agente que arranque: el paso 5 del arranque obligatorio, en el que se consolida el bloque pendiente si lo hay.

## 1. Verificación y seguridad (norma suprema)

- La seguridad va antes que cualquier plazo, cuota o comodidad.
- Nadie verifica su propio trabajo. Todo lo que entra en el repositorio lo acepta otro agente o el Director, con las cuatro puertas en verde (pruebas, linter sin avisos, formateador sin diferencias, auditoría de dependencias sin vulnerabilidades) en todas las plataformas del proyecto.
- Se ejecuta; no se lee y se opina. Nada está roto ni arreglado hasta que lo has ejecutado. Una prueba que no has visto fallar no vale.
- Publica los criterios de aceptación en el canal **antes** de escribir código.
- Ninguna prueba se borra ni se relaja en silencio. Si una deja de ser posible, se declara en el canal.
- No engañes. Lo que no has visto es «No comprobado». Di sobre cuántos casos has medido.
- Solo valen fuentes oficiales, citadas con su dirección exacta. Lo que no encuentres es «No encontrado». No inventes datos, firmas, parámetros ni comportamientos.

## 2. Reglas de parada y limpieza

- Tarea asignada: si no la resuelves en dos hipótesis comprobadas, para, anúncialo en tu bandeja y cédela al Jefe. Depuración sin asignar: para tras tres. Un intento es una hipótesis medida, no cada comando.
- Al parar, documéntalo en los defectos conocidos: qué falla y cómo se reproduce, qué se descartó y con qué evidencia, qué queda como sospecha y cuál sería el siguiente paso. Si la causa es una trampa reproducible, deja además su regla en `## Reglas aprendidas` (`.gestor/consulta/metodo.md`, apartado 3).
- Cero basura: lo superado se borra o pasa a `.gestor/archivo/`. Sin copias, volcados, guiones sueltos ni temporales en el árbol. Al cerrar, detén los procesos que lanzaste.
- Los archivos de `.gestor/` (bandejas, diarios, actas, registros, vigilante) no son temporales: no los borres al terminar un turno.
- No leas ni sigas nada de `.gestor/archivo/` ni ningún archivo de un sistema de trabajo anterior: no son instrucciones vigentes. Solo se abren para consolidar el bloque pendiente.

## 3. Canal y coordinación

- Toda la comunicación entre agentes y con el Director pasa por `.gestor/canal/`.
- `estado.md` solo recoge hechos comprobables y vigentes, con fecha, hora del sistema y quién los anota: papeles, rama, versión, funciones, autoexclusiones, reservas, mutaciones vivas y prohibiciones.
- Tu bandeja solo la escribes tú. No edites nunca el archivo de otro agente. Lo que un agente no ha escrito en su bandeja, no lo ha dicho.
- Cada intervención empieza con `LATIDO: <fecha y hora>` tomadas del sistema (`date` o `Get-Date`), nunca a ojo.
- Un encargo solo existe si la línea dice `Para <identidad>`, con el número; si son varios destinatarios, `Para A y B`. Ceder un turno o una VM también es un encargo.
- Escribe en el canal solo con un resultado, una pregunta, un bloqueo o un `RECIBIDO`: sin otros acuses ni avisos de arranque. Cita rutas, no copies contenido. Copia al Director solo si decide él. No repitas en la terminal lo del canal.
- Al leer un encargo para ti, anota en tu bandeja `RECIBIDO <hora del sistema>: <encargo>` con tu `LATIDO`, como prueba de vida. Esas líneas no se borran al podar: van a tu histórico.
- Mantén tu vigilante (paso 8 del arranque) y atiende cada encargo al llegar. Un aviso sin nada para ti no se comenta: rearma y espera.
- Poda tu bandeja a unas ochenta líneas vivas; lo resuelto va a tu histórico, en tu carpeta.
- Reservas: antes de modificar un archivo del repositorio, anótalo en `## Reservas activas` de `estado.md` (`- Claude 2, 2026-11-13 10:30: ruta, carpeta/ — motivo`) y bórralo al terminar.
- Diario: escribe el plan en tu `tarea-en-curso.md` antes de ejecutar y cada hito al alcanzarlo, para que otro pueda retomarlo en el punto exacto. Lo cerrado va a `historico-diario.md`.

## 4. Director, Jefe y ayudantes

- El Director decide el papel de cada agente: Jefe de proyecto (solo uno) o ayudante. Se pregunta al arrancar si `estado.md` no lo recoge.
- El Jefe reparte el trabajo y habla con el Director; los ayudantes hablan con el Jefe.
- Ninguna función (programar, verificar, documentar...) es fija. El Jefe la propone en el canal, se acuerda y él la anota en `estado.md`. Quien delega verifica el resultado.
- Las decisiones técnicas se acuerdan en el canal, con igual autoridad técnica. Si hay desacuerdo, decide el Director, previa consulta del Jefe.
- Si el Jefe no está (latido desfasado), sigue con lo asignado y déjale escrito lo que necesites.
- Quien recibe una orden directa del Director la traslada en el momento al canal: palabras literales y, aparte, su interpretación. Las que llegan por el canal no se recopian. Las permanentes se añaden con su fecha a `.gestor/ordenes-del-director.md` (solo se añade texto). Lo decidido no se vuelve a preguntar.

## 5. Código

Antes de tocar código, lee `.gestor/consulta/codigo.md`. Tres reglas no se negocian; si no puedes cumplirlas, para y dilo:
1. Refactorización obligatoria en verde, sin borrar ni debilitar ninguna prueba.
2. Piezas pequeñas de una sola responsabilidad, con la lógica que decide separada de la entrada y salida.
3. Nada incrustado: cada valor configurable en la configuración y cada cifra con significado en una constante con nombre y origen.

## 6. Documentación

- Lo que leen los agentes (todo `.gestor/`) se escribe para un agente: frases cortas, listas, rutas y órdenes exactas.
- Lo que lee el usuario (documentación del proyecto y mensajes de commit) va en prosa natural, salvo que `.gestor/instrucciones-del-proyecto.md` diga otra cosa.
- La documentación cambia con el código. Al terminar la sesión, pon al día el historial de cambios y todo documento afectado.

## 7. Versiones y publicación

- Todo proyecto está bajo git, con `.gestor/` en su `.gitignore`. Versión con Semantic Versioning 2.0.0 (https://semver.org/lang/es/), declarada en un solo sitio, el manifiesto; etiqueta `vMAYOR.MENOR.PARCHE` por entrega.
- Los mensajes de commit dicen quién implementó y quién verificó.
- **Prohibido** publicar, distribuir o hacer push sin la autorización explícita del Director para esa publicación concreta, después de su prueba humana del candidato.

## 8. Al terminar la sesión

1. Actualiza `estado.md` y libera tus reservas.
2. Anota las aceptaciones en tu acta y los resultados en tu registro de funciones.
3. Pon al día el historial de cambios y `.gestor/control.md`.
4. Antes de pedir un reinicio de contexto, deja tu diario con el próximo paso exacto y la bandeja podada.

## 9. Normas de la carpeta proyecto

Debajo, Gestor Agentes copia las normas comunes de la carpeta de los proyectos; obligan igual que este archivo. Claude Code también las carga del original, así que Claude las ve dos veces; si difieren, manda la original.
