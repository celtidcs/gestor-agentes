Requisitos


Para usar la aplicación

En Windows hace falta Windows 10 u 11 de 64 bits. El ejecutable gestor-agentes.exe no necesita ningún componente instalado aparte.

En Linux hace falta una distribución de 64 bits con escritorio gráfico, sea X11 o Wayland, y uno de estos emuladores de terminal: gnome-terminal, konsole, xfce4-terminal o xterm.

La ventana necesita OpenGL 2.0 o superior, que da el controlador de la tarjeta gráfica. Las distribuciones de Linux de escritorio lo traen siempre, porque Mesa lo da por software si no hay tarjeta gráfica. En un Windows sin aceleración gráfica, como algunas máquinas virtuales o escritorios remotos, la aplicación avisa de que falta OpenGL y no se abre; se arregla instalando el complemento gratuito de Microsoft OpenCL, OpenGL, and Vulkan Compatibility Pack, como se explica en defectos-conocidos.md.

En los dos sistemas hace falta al menos uno de los programas de agente en terminal para que aparezca algún botón: claude para Claude Code, codex para Codex, agy para Gemini, o Programator, cuyo ejecutable se indica en la pestaña Agentes. Las aplicaciones de escritorio de Codex y de Antigravity no sirven, porque la aplicación abre los agentes en terminal. Si un programa no está en su sitio habitual, su ruta se indica en esa misma pestaña. Conviene tener git, porque la aplicación crea con él el repositorio de los proyectos que no lo tienen.


Para compilar

Hace falta Rust con cargo, instalado con rustup desde https://rustup.rs. La versión con la que se ha comprobado es la 1.99.0, con clippy y rustfmt, que rustup instala con el perfil por defecto. Se comprueba con cargo --version.

Para la cuarta comprobación de calidad hace falta cargo-audit, que se instala con cargo install cargo-audit --locked.

En Windows hacen falta las herramientas de compilación de C++ de Visual Studio, que rustup ofrece instalar. El compilador de recursos rc.exe, del Windows SDK, es opcional: solo sirve para poner el icono en gestor-agentes.exe. Si no está, la compilación termina bien, avisa de ello y el ejecutable queda sin icono.

Las dependencias se descargan solas, con las versiones fijadas en el Cargo.lock de la raíz, que se respeta siempre con la opción --locked. Las directas son thiserror, serde, serde_json y chrono en el núcleo; thiserror en el lanzador; y eframe 0.37, con el motor glow, y rfd 0.18 en la interfaz.

El Cargo.toml de la raíz fija para todo el workspace la versión del producto y unas reglas que el compilador hace cumplir. Prohíbe el código unsafe y prohíbe en el código de producción unwrap, expect, panic, todo, unimplemented y dbg. Las comprobaciones tratan cualquier aviso de clippy como error.


Generar el paquete portable

En Windows, documentation\infrastructure\build-portable.ps1 genera la carpeta portable con gestor-agentes.exe, la carpeta datos vacía, el LEEME y la documentación. Admite -Destino para elegir otra carpeta de salida y -Objetivo para compilar para otro triple de Rust, como aarch64-pc-windows-msvc.

En Linux, documentation/infrastructure/build-portable.sh hace lo mismo con gestor-agentes, y admite --destino y --objetivo.

Los dos compilan en modo release con las versiones de Cargo.lock y, al regenerar el paquete, conservan lo que haya en datos, que es la configuración del usuario. Cada uno se ejecuta en su propio sistema, porque no hay compilación cruzada preparada. Después, prueba-portable.ps1 o prueba-portable.sh comprueban que el paquete tiene exactamente lo que debe.


Comprobaciones de calidad

Las cuatro comprobaciones se pasan con documentation/infrastructure/puertas-rust.sh en Linux o puertas-rust.ps1 en Windows. Son cargo fmt --all --check, cargo clippy --workspace --all-targets --locked -- -D warnings, cargo test --workspace --locked y cargo audit. El guion termina con código 0 solo si pasan las cuatro. Admite como argumento una carpeta de compilación, que pasa a cargo con --target-dir.


Varias cuentas por agente

Cada perfil de perfiles-agentes.json puede tener su carpeta de configuración, que la aplicación pasa a la herramienta con CLAUDE_CONFIG_DIR en Claude Code y con CODEX_HOME en Codex. Qué cuenta usa cada carpeta lo decide la propia herramienta cuando se inicia sesión en ella.
