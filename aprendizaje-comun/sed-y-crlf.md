# Mutaciones con sed en archivos con finales CRLF

Estado: comprobada
Problema: una mutación aplicada con sed no cambia nada si el archivo tiene finales de línea CRLF y el patrón termina en `$`, o si las comillas anidadas rompen la orden. La prueba sigue en verde y parece que la barrera no vigila.
Cómo se reconoce: la mutación sale en verde y `git diff` del archivo mutado está vacío.
Solución: antes de leer el resultado de una mutación, comprobar que se ha aplicado (diferencia del archivo o compilación nueva). Con CRLF, el patrón admite `\r?$`, o se aplica la mutación con un parche o por SFTP.
Evidencia: un proyecto del equipo, 2026-11-18: una mutación no se aplicó por CRLF y otra tanda no mutó nada por las comillas; las dos se detectaron al ver el verde y se repitieron.
Herramientas y versiones: GNU sed, bash, archivos editados en Windows.
Aplica a: todos los agentes que apliquen mutaciones.
