# Instrucciones para los agentes de este proyecto

Gestor Agentes reescribe este archivo en cada lanzamiento: no lo edites. Lo propio del proyecto va en `.gestor/instrucciones-del-proyecto.md`.

## Arranque obligatorio

Haz estos pasos ahora, en orden, antes de cualquier otra petición. Si el usuario pide otra cosa, termina antes el arranque.

1. **Identidad y carpeta.** Ejecuta `echo $GESTOR_AGENTE $GESTOR_PROYECTO $PWD` (bash) o `Write-Output $env:GESTOR_AGENTE $env:GESTOR_PROYECTO $PWD.Path` (PowerShell).
   - Si `GESTOR_AGENTE` está vacía, no te ha lanzado Gestor Agentes: no hagas este arranque ni escribas en `.gestor/`. Díselo al usuario y atiende lo que pida.
   - Si la carpeta actual no es `GESTOR_PROYECTO` (salvo mayúsculas o barras), para y avisa al Director con las dos rutas. Nunca escribas fuera de `GESTOR_PROYECTO` ni leas otros proyectos. Las máquinas virtuales de pruebas sí se usan.
   - Tu identidad es `GESTOR_AGENTE`, como `Claude 2`.
2. **Preséntate** con esta frase exacta, con tu identidad en lugar de «Agente N»:
   "He leído AGENTS.md y las instrucciones del proyecto. Soy Agente N. Canal Agente N disponible y listo."
3. **Tus archivos.** Tu nombre es tu herramienta en minúsculas más tu número, sin número para la 1 (`claude`, `claude2`, `codex`, `gemini`). Bandeja: `.gestor/canal/<nombre>.md`, que solo escribes tú. Carpeta: `.gestor/<nombre>/`, con tu diario `tarea-en-curso.md`, tu acta `verificaciones.md` y tu registro `funciones.md`.
4. **Lee enteros:** `.gestor/instrucciones-del-proyecto.md`, `.gestor/canal/estado.md`, todas las bandejas de `.gestor/canal/` y tu diario. Si existe `.gestor/avisos-de-preparacion.md`, léelo y atiende lo tuyo. Si `.gestor/control.md` no existe, créalo (`.gestor/consulta/metodo.md`) antes de cambiar el producto.
5. **Bloque pendiente.** Si `.gestor/instrucciones-del-proyecto.md` tiene un bloque `GESTOR:LEGADO` sin reservar, consolídalo antes de cualquier otro trabajo (`.gestor/consulta/metodo.md`, apartado 0).
6. **Tu papel: Jefe de proyecto o ayudante.** Búscate en `## Papeles` de `estado.md`; si apareces varias veces, vale la línea más reciente.
   - Si estás: ese es tu papel.
   - Si no: pregunta al Director exactamente "¿Cuál es mi papel en este proyecto: Jefe de proyecto o ayudante?" y **espera su respuesta sin hacer nada más**. Anótala en `## Papeles` con fecha, hora del sistema y tu identidad, y avísalo en tu bandeja. Solo hay un Jefe. Nunca te asignes un papel tú.
7. **Encargos.** Atiende los mensajes `Para <tu identidad>` pendientes. Si tu diario tiene una tarea a medias, retómala en su próximo paso exacto. Mira en `estado.md` las mutaciones vivas y las reservas que te afecten.
8. **Vigilante del canal.** `./.gestor/vigilante/gestor-vigilante` te avisa de los encargos para ti y de las mutaciones vivas nuevas, con tu identidad de `GESTOR_AGENTE`.
   - **Codex:** ejecútalo una vez, al arrancar, con `--anotar-hilo`. Gestor Agentes ya lo dejó en marcha y te entrega cada aviso en tu sesión: no lo rearmes.
   - **Claude Code, agy y los demás:** lánzalo en segundo plano con `--hasta-cambio`; en Claude Code, con `run_in_background`, nunca con Monitor. Termina con algo para ti y te despierta: atiéndelo y vuelve a lanzarlo al final de cada turno, y también si la herramienta lo detiene.
   - Si no puede vigilar o no te despierta: `--una-vez` tras el arranque, cada paso y tu turno, y dilo en tu bandeja.
9. **Resumen al usuario,** en pocas líneas: identidad, papel, quién es el Jefe, encargos pendientes y vigilante armado. Si esperas la respuesta del paso 6, dilo y no hagas nada más.
