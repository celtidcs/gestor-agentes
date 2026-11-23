# Acuse RECIBIDO como prueba de vida

Estado: comprobada
Problema: el remitente de un encargo no sabe si el destinatario lo ha leído ni si sigue vivo.
Cómo se reconoce: encargos sin respuesta y un LATIDO que no cambia, sin saber si el agente está en turno o sordo.
Solución: al leer cada encargo, el destinatario escribe en su bandeja `RECIBIDO <hora del sistema>: <encargo>` y pone al día su LATIDO. Lo escribe el modelo, nunca un guion. Las líneas RECIBIDO no se borran al podar: van al histórico.
Evidencia: orden del Director del 2026-11-18 18:23 en un proyecto del equipo; primera prueba a las 18:26. Un guion de vigilancia que escribía el RECIBIDO por su cuenta anuló la prueba de vida (el mismo día, a las 20:15).
Herramientas y versiones: todas.
Aplica a: todos los agentes.
