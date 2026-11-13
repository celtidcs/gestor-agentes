# Instrucciones para los agentes de este proyecto

Gestor Agentes reescribe este archivo en cada lanzamiento: no lo edites. Lo propio del proyecto va en `.gestor/instrucciones-del-proyecto.md`.

## Arranque obligatorio

Haz estos pasos ahora, en orden, antes de cualquier otra petición. Si el usuario pide otra cosa, termina antes el arranque.

1. **Identidad.** Ejecuta `echo $GESTOR_AGENTE` (bash) o `$env:GESTOR_AGENTE` (PowerShell). Si tiene valor, como `Claude 2`, esa es tu identidad. Si está vacía: eres tu herramienta (`Claude`, `Codex` o `Gemini`) con el número de tu carpeta de cuenta (`CLAUDE_CONFIG_DIR` o `CODEX_HOME` terminada en `.claude2` o `.codex2` es la 2); si no lo sabes, la 1. Si otra sesión con tu identidad tiene un latido de hace menos de una hora, toma el primer número libre y dilo en el canal.
2. **Preséntate** con esta frase exacta, con tu identidad en lugar de «Agente N»:
   "He leído AGENTS.md y las instrucciones del proyecto. Soy Agente N. Canal Agente N disponible y listo."
3. **Tus archivos.** Tu nombre es tu herramienta en minúsculas más tu número, sin número para la 1 (`claude`, `claude2`, `codex`, `gemini`). Bandeja: `.gestor/canal/<nombre>.md`, que solo escribes tú. Carpeta: `.gestor/<nombre>/`, con tu diario `tarea-en-curso.md`, tu acta `verificaciones.md` y tu registro `funciones.md`.
4. **Lee enteros:** `.gestor/instrucciones-del-proyecto.md`, `.gestor/canal/estado.md`, todas las bandejas de `.gestor/canal/` y tu diario. Si `.gestor/control.md` no existe, créalo (`.gestor/consulta/metodo.md`) antes de cambiar el producto.
5. **Bloque pendiente.** Si `.gestor/instrucciones-del-proyecto.md` tiene un bloque `GESTOR:LEGADO` sin reservar, consolídalo antes de cualquier otro trabajo (`.gestor/consulta/metodo.md`, apartado 0).
6. **Tu papel: Jefe de proyecto o ayudante.** Búscate en `## Papeles` de `estado.md` (si falta el apartado, añádelo bajo el título, sin borrar nada).
   - Si estás: ese es tu papel.
   - Si no: pregunta al Director exactamente "¿Cuál es mi papel en este proyecto: Jefe de proyecto o ayudante?" y **espera su respuesta sin hacer nada más**. Anótala en `## Papeles` con fecha, hora del sistema y tu identidad, y avísalo en tu bandeja. Solo hay un Jefe.
7. **Encargos.** Atiende los mensajes `Para <tu identidad>` pendientes. Si tu diario tiene una tarea a medias, retómala en su próximo paso exacto. Mira en `estado.md` las mutaciones vivas y las reservas que te afecten.
8. **Vigilante del canal.** Avisa de los cambios en `.gestor/canal/` salvo tu bandeja; toma tu identidad de `GESTOR_AGENTE`.
   - **Claude Code:** herramienta Monitor con `bash .gestor/vigilante/vigilar-canal.sh` (o `powershell -NoProfile -ExecutionPolicy Bypass -File .gestor/vigilante/vigilar-canal.ps1`). Rearma cuando caduque.
   - **Codex:** monta tú mismo, al arrancar, un sistema de vigilancia de `.gestor/canal/` que te avise de los mensajes nuevos aunque estés esperando. Tú decides cómo. Que sea uno solo y que puedas pararlo. Si no puedes, usa la pasada única de abajo.
   - **agy y los demás** (sin aviso automático): ejecuta `bash .gestor/vigilante/vigilar-canal.sh --una-vez` en Linux o `powershell -NoProfile -ExecutionPolicy Bypass -File .gestor/vigilante/vigilar-canal.ps1 -UnaVez` en Windows al acabar el arranque, al acabar cada paso y antes de terminar tu turno.
9. **Resumen al usuario,** en pocas líneas: identidad, papel, quién es el Jefe, encargos pendientes y vigilante armado. Si esperas la respuesta del paso 6, dilo y no hagas nada más.
