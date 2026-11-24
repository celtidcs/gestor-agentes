# Problemas frecuentes

Síntoma, causa y qué hacer. Las reglas exactas de búsqueda están en `guia-de-uso-para-agentes.md`, apartado 6.

## No aparece ninguna cuenta

- ¿El programa ha encontrado la carpeta personal? Sin `USERPROFILE` (Windows) o `HOME` (los demás) no se busca nada (N4.1).
- ¿La herramienta se ha usado alguna vez en este equipo? Hasta que crea su carpeta o su archivo de configuración no hay cuenta por defecto. El programa puede mostrar la herramienta con su cuenta por defecto si encuentra su ejecutable (guía, apartado 5, `Cuenta`).
- ¿Las pruebas usan la carpeta personal real? Deben usar una temporal (guía, apartado 10).

## No aparece una segunda cuenta

- ¿Su carpeta está directamente en la carpeta personal? La búsqueda no baja a subcarpetas. Si está en otro sitio, la persona la da de alta a mano o la señala con la variable.
- ¿Su nombre empieza exactamente por `.claude` o `.codex` y lleva algo más detrás? `claude2` (sin punto) o `.Claude2` (con mayúscula) no cuentan.
- ¿Tiene dentro alguno de los archivos de cuenta (`.claude.json`, `.credentials.json` o `settings.json` en Claude Code; `config.toml` o `auth.json` en Codex CLI)? Una carpeta recién creada y vacía no es una cuenta hasta que la herramienta la usa.
- ¿Es de agy? De agy solo existe la cuenta por defecto (guía, apartado 8).

## Aparece una cuenta que no es de verdad

- Una carpeta como `.claude-copia` con un `settings.json` dentro es una cuenta para la biblioteca. Si la persona no la quiere, la borra o la renombra; el programa no debe esconderla con una lista de nombres (N3.3).

## La cuenta de la variable aparece en un equipo y no en otro

- La variable se lee en el entorno del programa que busca. Si la persona la puso solo en una terminal, el programa abierto desde el menú del sistema no la ve. Es correcto: para que sea fija, la persona la da de alta a mano.

## El servidor MCP está registrado pero la herramienta no lo ve

- ¿Se registró en `cuenta.archivo_de_configuracion(&personal)`? En la cuenta por defecto de Claude Code el archivo es `~/.claude.json`, fuera de `~/.claude`. Componerlo a mano es el error típico (N7.1).
- ¿La consola se abrió con la cuenta en la que se registró? Mira el siguiente apartado.

## La consola usa otra cuenta

- ¿Se aplicó `ajuste_de_entorno` a la orden? Sin él, la consola usa la cuenta que diga el entorno del programa (N8.1).
- ¿Se aplicó también en la cuenta por defecto? Si el programa se abrió con `CLAUDE_CONFIG_DIR` o `CODEX_HOME` puestos, la consola los hereda y usa esa cuenta en vez de la por defecto. El ajuste `Quitar` lo evita (N8.2).
- ¿Se lanza a través de otra terminal o de un intérprete que carga un perfil? Un perfil que fija la variable puede volver a ponerla. Lanza el programa directamente, sin intérprete.

## Una cuenta manual aparece aunque su carpeta ya no existe

- Es lo esperado: se conserva para que la persona vea que falta y la quite (N6.4). El programa puede marcarla como «no encontrada».

## La misma cuenta aparece dos veces

- No debería: las carpetas se comparan tras `canonicalize` y sin distinguir la barra final ni, en Windows, el tipo de barra ni mayúsculas. Si pasa, anota el caso en el registro común con las dos rutas exactas y el sistema operativo.

## La copia de la biblioteca no coincide con la etiqueta

- Casi siempre es por copiar con `git archive` en Windows con `core.autocrlf=true`, que convierte los finales de línea. Copia los bytes con `git cat-file blob vX.Y.Z:<ruta>` (N1.3).
- Si alguien editó la copia, deshaz el cambio y anótalo como fallo o mejora en el registro común (N1.4).

## Error de compilación al actualizar a una versión nueva

- Una firma ha cambiado. Lee el historial de cambios del `README.md` y el apartado 11 de `guia-de-uso-para-agentes.md`, y compara tu módulo `cuentas_de_agentes` con el nuevo `examples/integracion_de_referencia.rs` (N11.3).

## Las pruebas del programa tocan el equipo real

- Alguna prueba pasa la carpeta personal real o `|nombre| std::env::var_os(nombre)`, o usa `std::env::set_var`. Las pruebas usan siempre una carpeta temporal propia y una función de variables propia (N10; guía, apartado 10).

## Ninguna de estas causas

- Anota el fallo en tu apartado del registro común, `registro de proyectos_bibliotecas.md`, con cómo reproducirlo. No lo arregles en la copia de tu programa (N1.4).
