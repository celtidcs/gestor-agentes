# Despertar a Codex con codex queue

Estado: comprobada
Problema: Codex CLI no ve los encargos nuevos del canal mientras espera al usuario; el agente se queda dormido aunque tenga trabajo.
Cómo se reconoce: un encargo «Para Codex N» sin RECIBIDO durante minutos, con la sesión de Codex abierta y sin turno en curso.
Solución: entregar el aviso en la sesión abierta con `codex queue --thread <sesión> --message <aviso> --resume -C <raíz del proyecto>`, donde la sesión es la variable CODEX_THREAD_ID de la propia sesión. Desde Gestor Agentes 3.6.0 lo hace `gestor-vigilante --persistente --entrega codex`. Un resume fallido termina con código 0 y una advertencia: la salida se lee, no solo el código.
Evidencia: un proyecto del equipo, 2026-11-18. Con Codex ocupado, el aviso se encoló 1,3 s después del encargo y el turno siguiente lo atendió; con Codex ocioso, turno nuevo a los 16 s y respuesta en el canal a los 45 s. Antes, en otro proyecto, el 2026-11-12.
Herramientas y versiones: Codex CLI 0.167.1, daemon 0.170.0, Windows.
Aplica a: Codex CLI.
