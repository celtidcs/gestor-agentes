# Norma de integración de deteccion-agentes

Obligatoria para todo programa que use esta biblioteca. Todos los programas la conectan **de la misma forma**, para que un agente que conoce una integración las conozca todas. La referencia ejecutable es `examples/integracion_de_referencia.rs`: Cargo la compila, clippy la revisa y `cargo test` ejecuta sus pruebas.

- Para qué es cada función y cada tipo: `guia-de-uso-para-agentes.md`.
- Dudas y fallos habituales: `problemas-frecuentes.md`.
- Qué programas la usan y qué fallos han encontrado: el registro común, `registro de proyectos_bibliotecas.md`, en la carpeta que contiene el repositorio de la biblioteca (no va con las copias).

Si un programa necesita apartarse de una regla, la desviación se anota en su apartado del registro común, con su motivo. Si no está anotada, la desviación es un defecto.

## N1. La copia de la biblioteca

1. Va en `bibliotecas/deteccion-agentes/`, en la raíz del programa. Ninguna otra ruta.
2. Es una copia íntegra de una versión etiquetada de este repositorio (`vX.Y.Z`): `Cargo.toml`, `Cargo.lock`, `LICENSE`, `README.md`, `src/`, `examples/` y `documentacion/`. Sin `target/`, sin `.gitignore` y sin `mejoras.md`.
3. Se copian los bytes de la etiqueta con `git cat-file blob vX.Y.Z:<ruta>`, archivo por archivo. No uses `git archive` en Windows: con `core.autocrlf=true` entrega los archivos con CRLF y la copia deja de ser idéntica.
4. No se edita dentro del programa. Un fallo o una necesidad se anota en el registro común; el cambio se hace en este repositorio, se etiqueta y se vuelve a copiar.
5. Para saber qué versión lleva un programa, mira el `version` de `bibliotecas/deteccion-agentes/Cargo.toml`.

## N2. La dependencia

1. Se declara por ruta, en el `Cargo.toml` del crate que contiene el módulo de N3:

```toml
deteccion-agentes = { path = "bibliotecas/deteccion-agentes" }
```

   (en un workspace, la ruta relativa desde ese crate, como `../../bibliotecas/deteccion-agentes`).
2. Si el programa es un workspace, la copia es miembro del workspace. Así sus pruebas corren con las del programa.
3. Si no lo es, las puertas del programa y su CI añaden `cargo test --manifest-path bibliotecas/deteccion-agentes/Cargo.toml --target-dir <carpeta fuera de la copia>`. El `--target-dir` evita que quede un `target/` dentro de la copia.

## N3. Un solo módulo conecta la biblioteca

1. El programa tiene **un** módulo, llamado `cuentas_de_agentes`, que es el único que llama a `buscar_cuentas` y `con_cuentas_manuales`. Se copia de `examples/integracion_de_referencia.rs`.
2. Fuera de ese módulo solo se usan los tipos `Cuenta`, `Herramienta`, `OrigenDeLaCuenta` y `AjusteDeEntorno`, y los métodos de `Cuenta` y `Herramienta` (para guardar, mostrar o lanzar una cuenta ya encontrada).
3. Ningún archivo del programa contiene una carpeta de cuenta concreta (`.claude2`, `.codex-trabajo`...) ni la ruta de un archivo de configuración de una herramienta. Las pruebas sí, dentro de carpetas temporales.

## N4. La carpeta personal y las variables

1. La carpeta personal sale de `USERPROFILE` en Windows y de `HOME` en los demás sistemas, como en `carpeta_personal` de la referencia. Si falta o está vacía, no se busca nada y el programa dice que no ha encontrado agentes; no inventa una carpeta.
2. La función de variables real es `|nombre| std::env::var_os(nombre)`. Se inyecta: el código que busca no lee el entorno por su cuenta.
3. No se fija ninguna variable de cuenta en el proceso del programa (`std::env::set_var`). La variable se pone o se quita solo en la orden de la consola (N8).

## N5. Cuándo se busca

1. Se busca cada vez que el programa va a mostrar o usar las cuentas: al abrir su ventana de conexiones y antes de registrar un servidor o abrir una consola. Así aparece una cuenta creada después sin reiniciar el programa.
2. Nunca en cada fotograma ni en un bucle: la búsqueda lee el disco. Si la interfaz necesita la lista en cada repintado, guarda la última búsqueda y la renueva en los momentos de N5.1.
3. La búsqueda no devuelve errores. Una carpeta ilegible no aporta cuentas; no hace falta registrar nada.

## N6. Alta manual

1. El programa deja dar de alta a mano una cuenta eligiendo la herramienta y su carpeta, y la guarda en sus preferencias (herramienta y carpeta). Se puede quitar.
2. Solo se ofrecen las herramientas de `Herramienta::con_alta_manual()`. Hoy son Claude Code y Codex CLI. Una cuenta manual de una herramienta sin variable que venga de unas preferencias antiguas o editadas a mano la descarta `con_cuentas_manuales`; el programa no la filtra por su cuenta.
3. Antes de guardar un alta manual, el programa llama a `validar_alta_manual` (`validar_alta` de la referencia) y, si la rechaza, no la guarda y dice a la persona el motivo con sus propios textos.
4. Las manuales se pasan siempre por `con_cuentas_manuales`, nunca se añaden a la lista a mano. Una manual cuya carpeta ya no existe se conserva y se muestra, para que la persona vea que falta y la quite.
5. Si el programa guarda la herramienta como texto, usa nombres propios y estables, no el `Debug` de `Herramienta`.

## N7. Registrar un servidor MCP

1. El archivo donde se registra el servidor de una cuenta es siempre `cuenta.archivo_de_configuracion(&personal)`. No se compone a mano: la cuenta por defecto de Claude Code lo tiene fuera de su carpeta (`~/.claude.json`).
2. Registrar en una cuenta no toca el archivo de ninguna otra.
3. El formato de cada archivo (JSON en Claude Code y agy, TOML en Codex CLI) y la forma de escribirlo sin romperlo son cosa del programa; la biblioteca solo da la ruta.

## N8. Abrir la consola de una cuenta

1. Antes de lanzar la consola se aplica `cuenta.ajuste_de_entorno()` a la orden con `aplicar_a`, como en `orden_de_consola` de la referencia.
2. Se aplica siempre, también en la cuenta por defecto: allí el ajuste **quita** la variable, para que la consola no herede la que tenga puesta el programa que la abre.
3. Con `None` (agy) no se toca el entorno.

## N9. Presentar las cuentas

1. Cada cuenta se presenta con el nombre de la herramienta y, si no es la por defecto, con `nombre_visible()` (el nombre de su carpeta), como en `nombre_para_mostrar` de la referencia.
2. Las cuentas manuales se distinguen de las encontradas (`origen == OrigenDeLaCuenta::Manual`), porque son las únicas que se pueden quitar.
3. Si agy aparece, el programa dice que solo se usa su cuenta por defecto.

## N10. Pruebas mínimas del programa

En carpetas temporales propias y con una función de variables propia, sin tocar la carpeta personal real ni el entorno del proceso, el programa prueba como mínimo estos seis casos, que son los de la referencia:
1. La cuenta por defecto se encuentra, su archivo MCP es el correcto y su consola quita la variable.
2. Una segunda cuenta (`.<herramienta><sufijo>` con sus archivos) se encuentra, su archivo MCP está en su carpeta y su consola pone la variable con esa carpeta.
3. Una carpeta con el nombre pero sin archivos de cuenta no es una cuenta.
4. La carpeta de la variable de entorno cuenta aunque esté fuera de la carpeta personal.
5. Una cuenta manual se añade y se conserva aunque su carpeta no exista; una de agy no se admite.
6. La carpeta personal sale de la variable del sistema, y una vacía no vale.

Además, si el programa registra servidores MCP: registrar en la segunda cuenta no cambia el archivo de la primera.

## N11. Actualizar la copia

1. Copia la versión etiquetada nueva sobre `bibliotecas/deteccion-agentes/` (N1).
2. Lee su entrada en el historial de cambios del `README.md` y, si rompe la compatibilidad, el apartado de migración de `guia-de-uso-para-agentes.md`.
3. Adapta el módulo `cuentas_de_agentes` comparándolo con el nuevo `examples/integracion_de_referencia.rs`.
4. Pasa las puertas del programa en Linux y Windows, incluidas las pruebas de la copia (N2).
5. Pon al día el apartado del programa en el registro común: versión de la copia, fecha, desviaciones y puertas pasadas.
6. Revisa la documentación del programa que hable de los agentes y sus cuentas: manual, ayuda, configuración e historial de cambios.

## Lista de comprobación para aceptar una integración

Quien verifica marca cada punto ejecutando o mirando el archivo, nunca de memoria:

- [ ] N1: `bibliotecas/deteccion-agentes/` es idéntica, archivo por archivo, a la etiqueta que dice su `Cargo.toml` (compara con `sha256sum` contra `git cat-file blob`).
- [ ] N2: dependencia por ruta; las pruebas de la copia corren en las puertas y en la CI.
- [ ] N3: un `grep` de `buscar_cuentas` y `con_cuentas_manuales` solo los encuentra en `cuentas_de_agentes`; un `grep` de `.claude2`, `.codex-` y `mcp_config.json` fuera de las pruebas no encuentra nada.
- [ ] N4: la carpeta personal sale de `USERPROFILE`/`HOME`; ningún `set_var` de `CLAUDE_CONFIG_DIR` ni `CODEX_HOME`.
- [ ] N5: ninguna búsqueda dentro del bucle de dibujado.
- [ ] N6: el alta manual solo ofrece `Herramienta::con_alta_manual()`, se valida con `validar_alta_manual` antes de guardarse y las manuales pasan por `con_cuentas_manuales` sin filtro propio.
- [ ] N7: el registro MCP usa `archivo_de_configuracion`.
- [ ] N8: la consola aplica siempre `ajuste_de_entorno` (una mutación que no lo aplique en la cuenta por defecto da rojo).
- [ ] N9: las cuentas se presentan con `nombre_visible` y las manuales se distinguen.
- [ ] N10: las seis pruebas existen y fallan si se rompe lo que vigilan.
- [ ] N11: el apartado del programa en el registro común está al día.
