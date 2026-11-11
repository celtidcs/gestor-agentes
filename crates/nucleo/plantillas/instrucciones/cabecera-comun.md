Instrucciones para los agentes de este proyecto

Acción obligatoria al iniciar

Antes de cualquier otra cosa, determina tu identidad y comunícasela al usuario con exactamente este mensaje, poniendo tu identidad en lugar de «Agente N»:

"He leído AGENTS.md y las instrucciones del proyecto. Soy Agente N. Canal Agente N disponible y listo."

Así se confirma que la carga de instrucciones funciona y que sabes qué archivos son tuyos.

Cómo determinas tu identidad

Consulta la variable de entorno GESTOR_AGENTE (echo $GESTOR_AGENTE en bash, $env:GESTOR_AGENTE en PowerShell). Si tiene un valor, como «Claude 2» o «Codex 1», esa es tu identidad: te la ha fijado Gestor Agentes al lanzarte. Si está vacía, tu herramienta es tu nombre (Claude si trabajas en Claude Code, Codex en Codex, Gemini en Antigravity o en Gemini CLI) y tu número sale de la carpeta de tu cuenta: en Claude, la variable CLAUDE_CONFIG_DIR termina en .claude para Claude 1 o en .claude seguido de un número, como .claude2, para Claude con ese número; en Codex, CODEX_HOME hace lo mismo con .codex. Si no puedes saberlo, eres la instancia 1. Si otra sesión con tu misma identidad tiene un latido reciente en su bandeja, toma el primer número libre y dilo en el canal.

Tus archivos

Tu bandeja es .gestor/canal/<nombre><número>.md, sin número para la instancia 1: claude.md para Claude 1, claude2.md para Claude 2, codex.md para Codex 1, gemini.md para Gemini 1. Tu carpeta propia es .gestor/<nombre><número>/ con la misma regla, y en ella están tu diario (tarea-en-curso.md), tu acta (verificaciones.md) y tu registro de funciones (funciones.md).
