deteccion-agentes

Biblioteca en Rust que busca en el equipo las cuentas de los agentes de programación de consola y dice, para cada una, en qué archivo se registra un servidor MCP y cómo abrir su consola con esa cuenta. Sabe de tres herramientas: Claude Code, Codex CLI y Antigravity CLI (agy). No conoce ninguna cuenta concreta ni ningún nombre de carpeta inventado: las encuentra. No escribe nada ni lanza nada: eso lo decide el programa que la usa.

Qué hace

La biblioteca hace cuatro cosas, cada una en su propio módulo. Describe cada herramienta: su carpeta por defecto, la variable de entorno con que elige otra cuenta, los archivos que delatan una cuenta suya y el archivo donde lee sus servidores MCP. Busca las cuentas en la carpeta personal y en las variables de entorno, y añade las que la persona haya dado de alta a mano, descartando las que su herramienta no podría usar. Valida, antes de guardarla, una cuenta que la persona quiera dar de alta a mano. Y, para cada cuenta encontrada, da el archivo de registro MCP y el ajuste de entorno con que abrir su consola. La lectura de las variables de entorno se le pasa como una función, para poder probar la búsqueda en carpetas temporales sin tocar el equipo real.

Cómo se usa

Cada programa lleva su propia copia en la carpeta bibliotecas/deteccion-agentes y la declara como dependencia por ruta en su Cargo.toml, con esta línea:

deteccion-agentes = { path = "bibliotecas/deteccion-agentes" }

Después, el programa llama a buscar_cuentas con la carpeta personal y una función que lea variables de entorno, normalmente |nombre| std::env::var_os(nombre), y pasa el resultado a con_cuentas_manuales junto con las cuentas que guarde en sus preferencias; con_cuentas_manuales descarta sola las de herramientas que no eligen la cuenta con una variable de entorno, como Antigravity CLI. Antes de guardar una cuenta nueva dada de alta a mano, validar_alta_manual dice si se puede y, si no, por qué: porque la herramienta no admite cuentas a mano, o porque otra cuenta suya ya usa esa carpeta. El formulario de alta ofrece las herramientas de Herramienta::con_alta_manual. De cada cuenta, archivo_de_configuracion da el archivo donde registrar el servidor MCP, y ajuste_de_entorno dice qué variable poner, o quitar, en la orden con que se abre su consola; aplicar_a la aplica. La búsqueda no falla nunca: una carpeta que no se puede leer simplemente no aporta cuentas.

La carpeta documentacion reúne todo lo necesario para conectar la biblioteca de la misma forma en cualquier programa. Su punto de entrada es indice.md. La norma de integración fija la forma única de hacerlo, y el ejemplo integracion_de_referencia de la carpeta examples la aplica en código: Cargo lo compila y cargo test ejecuta sus pruebas. La carpeta también tiene la guía de la API y los problemas frecuentes. Qué programas usan la biblioteca, y los fallos que encuentran en ella, constan en el registro común de proyectos y bibliotecas, en la carpeta que contiene este repositorio.

Qué es una cuenta

Una misma herramienta puede tener varias cuentas en un equipo. Cada una vive en su carpeta de configuración, y la herramienta elige cuál usar por una variable de entorno: CLAUDE_CONFIG_DIR en Claude Code (https://code.claude.com/docs/en/settings) y CODEX_HOME en Codex CLI (https://learn.chatgpt.com/docs/config-file/environment-variables). Antigravity CLI no documenta ninguna variable así (https://antigravity.google/docs/cli/reference) y siempre lee ~/.gemini/config/mcp_config.json (https://antigravity.google/docs/mcp), de modo que de ella solo se conoce la cuenta por defecto. Por eso Herramienta::Agy no tiene variable de cuenta y Cuenta::ajuste_de_entorno devuelve None: una carpeta de agy dada de alta a mano no la usaría la consola, y un programa no debe ofrecerla.

Cómo busca

Para cada herramienta cuenta su carpeta por defecto (~/.claude, ~/.codex o ~/.gemini) si la herramienta ha dejado rastro en ella; cada carpeta de la carpeta personal que se llama como la por defecto seguida de algo más, como .claude2 o .codex-trabajo, y que contiene los archivos que deja una cuenta (en Claude Code .claude.json, .credentials.json o settings.json; en Codex CLI config.toml o auth.json); y la carpeta que señale la variable de la herramienta, esté donde esté. Además, con_cuentas_manuales añade las carpetas que la persona haya dado de alta a mano. Una misma carpeta solo aparece una vez, aunque se llegue a ella por dos caminos, y el orden es estable: por herramienta, la cuenta por defecto primero y después por carpeta.

Seguridad

La biblioteca solo comprueba si existen carpetas y archivos: no lee su contenido, no escribe en ningún sitio y no lanza ningún programa. Tampoco cambia el entorno del proceso que la usa: el ajuste de una cuenta se aplica solo a la orden con que se abrirá su consola. En la cuenta por defecto ese ajuste quita la variable, para que la consola no herede la que tenga puesta el programa que la abre y acabe usando otra cuenta. Ningún fallo provoca un pánico: lo que no se puede leer simplemente no aporta cuentas. No necesita red y solo usa la biblioteca estándar de Rust.

Cómo se compila y se prueba

Se ha probado con Rust 1.99 en Windows y en Linux; no se ha medido con versiones anteriores. Desde la carpeta de la biblioteca se compila con la orden

cargo build

y se prueba con la orden

cargo test

Ninguna prueba necesita red ni toca la carpeta personal real ni las variables de entorno del proceso: cada una trabaja en su propia carpeta temporal. Antes de dar por bueno un cambio conviene pasar también las demás comprobaciones, cada una con su orden:

cargo clippy --all-targets -- -D warnings

cargo fmt --check

cargo doc --no-deps

cargo audit

Licencia

GPL-3.0 o posterior, la misma que el programa del que se separó. El texto íntegro está en el archivo LICENSE.

Historial de cambios

Versión 0.2.0, del 23 de noviembre de 2026. La biblioteca aplica sola la regla del alta manual, que antes tenía que repetir cada programa: con_cuentas_manuales descarta las cuentas de herramientas sin variable de cuenta, como Antigravity CLI, en lugar de admitirlas. Añade Herramienta::admite_alta_manual y Herramienta::con_alta_manual, y validar_alta_manual con su tipo RechazoDeAlta, para rechazar con su motivo un alta antes de guardarla. Las carpetas se comparan ahora sin distinguir la barra final, y en Windows tampoco el tipo de barra ni mayúsculas, así que una cuenta escrita de otra forma ya no aparece dos veces. Rompe la compatibilidad solo en un caso: un programa que contara con recibir cuentas manuales de agy deja de recibirlas.

Versión 0.1.0, primera versión como biblioteca independiente, separada de la detección de agentes del programa en que nació. Busca las cuentas de Claude Code y Codex CLI por su carpeta por defecto, por el nombre de su carpeta y por su variable de entorno, admite cuentas dadas de alta a mano y da, para cada cuenta, el archivo de registro MCP y el ajuste de entorno de su consola. De Antigravity CLI solo conoce la cuenta por defecto.
