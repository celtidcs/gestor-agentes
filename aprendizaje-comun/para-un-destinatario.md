# Un encargo, un «Para» al principio de la línea

Estado: comprobada
Problema: los vigilantes de la 3.5.1 solo despertaban al primer destinatario de «Para A y B» y no reconocían «Para Gemini» como Gemini 1.
Cómo se reconoce: un agente nombrado en un encargo no se despierta, y otro sí.
Solución: escribir cada encargo en su línea, empezando por `Para <identidad>` con el número. Si hay varios destinatarios, `Para A y B` vale desde la 3.6.0, pero un encargo por destinatario es más claro de acusar.
Evidencia: un proyecto del equipo, 2026-11-18 y 2026-11-19.
Herramientas y versiones: gestor-vigilante de Gestor Agentes 3.5.1 y anteriores.
Aplica a: todos los agentes.
