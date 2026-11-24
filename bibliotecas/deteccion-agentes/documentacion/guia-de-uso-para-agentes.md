# Guía de uso de deteccion-agentes para agentes

Referencia de la API para el agente que integra esta biblioteca en un programa en Rust o mantiene esa integración. Describe la versión 0.2.0. **Cómo** se conecta en un programa lo fija `norma-de-integracion.md`, que es obligatoria; el punto de entrada de toda la documentación es `indice.md`. La referencia de cada función está también en los comentarios de documentación del código (`cargo doc --no-deps --open`).

## 1. Qué hace y qué no

- Busca en el equipo las cuentas de tres herramientas de consola: Claude Code, Codex CLI y Antigravity CLI (agy).
- Para cada cuenta dice en qué archivo se registra un servidor MCP y qué variable de entorno hay que poner o quitar para abrir su consola con ella.
- Añade las cuentas que la persona haya dado de alta a mano.
- No escribe nada, no lanza nada y no lee el contenido de los archivos de configuración: solo mira si existen carpetas y archivos.
- No busca los programas (`claude`, `codex`, `agy`) en el `PATH`: eso es del programa que la usa.
- No conoce ninguna cuenta concreta. Nada de lo que encuentra está escrito en su código salvo las carpetas por defecto de cada herramienta, que son las documentadas.
- No falla: no devuelve errores ni provoca pánicos. Lo que no puede leer no aporta cuentas.
- Solo usa la biblioteca estándar. Funciona igual en Windows, Linux y macOS.

## 2. Cómo se incorpora

- La forma es única y obligatoria: `norma-de-integracion.md` (N1 a N3). Resumen: una copia en `bibliotecas/deteccion-agentes/` y una dependencia por ruta:

```toml
[dependencies]
deteccion-agentes = { path = "bibliotecas/deteccion-agentes" }
```

- No está en crates.io (`publish = false`). Cada programa lleva su propia copia.
- El original vive en el repositorio de la biblioteca. Qué programas la usan y con qué versión: el registro común, `registro de proyectos_bibliotecas.md`, en la carpeta que contiene el repositorio de la biblioteca. Un cambio aquí no les llega solo: cada programa vuelve a copiar la etiqueta y pasa sus puertas (N11).

## 3. Uso mínimo

```rust
use deteccion_agentes::{buscar_cuentas, con_cuentas_manuales, Herramienta};
use std::path::PathBuf;

fn cuentas(personal: &std::path::Path, manuales: &[(Herramienta, PathBuf)]) {
    let cuentas = con_cuentas_manuales(
        buscar_cuentas(personal, |nombre| std::env::var_os(nombre)),
        manuales,
    );
    for cuenta in &cuentas {
        let archivo_mcp = cuenta.archivo_de_configuracion(personal);
        let mut orden = std::process::Command::new("claude");
        if let Some(ajuste) = cuenta.ajuste_de_entorno() {
            ajuste.aplicar_a(&mut orden);
        }
        // registrar el servidor en archivo_mcp; lanzar orden cuando la persona lo pida
    }
}
```

En un programa real esto va dentro del módulo `cuentas_de_agentes` de la referencia (N3).

## 4. Reglas que no se saltan

- **Sin red ni escritura.** La biblioteca solo mira si existen carpetas y archivos. Registrar el servidor y lanzar la consola son del programa.
- **Sin pánicos.** Nada en la biblioteca hace `panic!`, y la búsqueda no devuelve error; solo la validación del alta devuelve un `Result`, con el motivo del rechazo. No conviertas tú un resultado vacío en un error: «ninguna cuenta» es una respuesta válida.
- **Carpeta personal y variables inyectadas.** El programa pasa la carpeta personal y la función que lee variables; la biblioteca no lee el entorno por su cuenta. Así las pruebas no tocan el equipo real.
- **El archivo MCP y la variable salen de la cuenta.** Nunca los compongas tú: la cuenta por defecto de Claude Code tiene su archivo fuera de su carpeta.
- **El ajuste se aplica siempre.** También en la cuenta por defecto, donde quita la variable.
- **No edites la copia dentro del programa.** Los cambios se hacen en este repositorio y se vuelven a copiar.

## 5. API pública

### `buscar_cuentas(personal: &Path, leer_variable: impl Fn(&str) -> Option<OsString>) -> Vec<Cuenta>`

- `personal`: la carpeta personal (N4).
- `leer_variable`: lee una variable de entorno por su nombre. Real: `|nombre| std::env::var_os(nombre)`. Pruebas: una función propia.
- Devuelve las cuentas de las tres herramientas, sin repetir carpetas, en orden estable: por herramienta (Claude Code, Codex CLI, agy), la por defecto primero y después por carpeta.
- Reglas exactas en el apartado 6.

### `con_cuentas_manuales(cuentas: Vec<Cuenta>, manuales: &[(Herramienta, PathBuf)]) -> Vec<Cuenta>`

- Añade cada manual con origen `Manual`, salvo que su carpeta ya esté para esa herramienta.
- Conserva las manuales aunque su carpeta no exista.
- Descarta las de herramientas sin alta manual (`Herramienta::admite_alta_manual`), como agy: su consola no usaría esa carpeta. El programa no las filtra antes.
- Una carpeta escrita de otra forma (barra final; en Windows, también otra barra o mayúsculas) cuenta como la misma. En Linux y macOS la barra invertida es parte del nombre, no un separador.
- Vuelve a ordenar.

### `validar_alta_manual(herramienta: Herramienta, carpeta: &Path, cuentas: &[Cuenta]) -> Result<(), RechazoDeAlta>`

- Se llama antes de guardar una cuenta nueva dada de alta a mano, con las cuentas que ya tiene el programa (las de `con_cuentas_manuales`).
- `Err(RechazoDeAlta::SinVariableDeCuenta)` si la herramienta no admite alta manual (agy).
- `Err(RechazoDeAlta::CarpetaYaUsada(cuenta))` si otra cuenta de la misma herramienta ya usa esa carpeta, con esa cuenta. Una carpeta de otra herramienta no estorba.
- Las carpetas se comparan como en `con_cuentas_manuales`.
- El programa muestra el motivo con sus propios textos; el `Display` de `RechazoDeAlta` es para el registro.

### `RechazoDeAlta`

- `SinVariableDeCuenta`: la herramienta no elige la cuenta con una variable de entorno.
- `CarpetaYaUsada(Cuenta)`: la cuenta que ya usa la carpeta.
- Implementa `Display` y `std::error::Error`.

### `Cuenta`

Campos públicos:
- `herramienta: Herramienta`.
- `carpeta: PathBuf`: la carpeta de configuración de la cuenta.
- `origen: OrigenDeLaCuenta`.

Métodos:
- `es_la_por_defecto() -> bool`: `true` si `origen == PorDefecto`.
- `nombre_visible() -> Option<String>`: el nombre de la carpeta (`.claude2`), o `None` en la cuenta por defecto, que se presenta solo con el nombre de la herramienta.
- `archivo_de_configuracion(personal: &Path) -> PathBuf`: dónde registrar un servidor MCP para esta cuenta (tabla del apartado 7).
- `ajuste_de_entorno() -> Option<AjusteDeEntorno>`: `Quitar` en la cuenta por defecto, `Poner` con la carpeta en las demás, `None` si la herramienta no tiene variable (agy).

Una `Cuenta` se puede construir a mano (todos sus campos son públicos). Úsalo solo para casos como «la herramienta está instalada pero aún no tiene carpeta»: entonces se construye su cuenta por defecto con `carpeta_por_defecto` y origen `PorDefecto`.

### `OrigenDeLaCuenta`

- `PorDefecto`: la carpeta que usa la herramienta sin variable.
- `Busqueda`: una carpeta de la carpeta personal con nombre y archivos de cuenta.
- `Variable`: la carpeta que señala la variable de la herramienta en el entorno de quien busca.
- `Manual`: dada de alta a mano.

Si una carpeta aparece por varias vías, gana la primera en este orden: por defecto, búsqueda, variable, manual.

### `AjusteDeEntorno`

- `Poner { variable, valor }`: poner `variable` con la carpeta de la cuenta.
- `Quitar { variable }`: quitar `variable`.
- `aplicar_a(&self, orden: &mut std::process::Command)`: aplica el ajuste a la orden con que se abrirá la consola. No toca el entorno del proceso del programa.

### `Herramienta`

- Variantes: `ClaudeCode`, `CodexCli`, `Agy`. Constante `TODAS` con las tres en orden.
- `carpeta_por_defecto(personal: &Path) -> PathBuf`: `~/.claude`, `~/.codex` o `~/.gemini`.
- `variable_de_cuenta() -> Option<&'static str>`: `CLAUDE_CONFIG_DIR`, `CODEX_HOME` o `None`.
- `admite_alta_manual() -> bool`: si se le pueden dar de alta cuentas a mano; solo a las que tienen variable de cuenta.
- `con_alta_manual() -> Vec<Herramienta>`: las que admiten alta manual, en el orden de `TODAS`. Son las que ofrece el formulario de alta.
- No implementa `serde`. Para guardarla en preferencias, el programa usa su propio nombre estable para cada variante (N6.5).

## 6. Reglas de búsqueda, una por una

Para cada herramienta, en este orden:

1. **Cuenta por defecto.** Cuenta si existe la carpeta que delata a la herramienta o su archivo de registro MCP:
   - Claude Code: existe `~/.claude` o existe el archivo `~/.claude.json`.
   - Codex CLI: existe `~/.codex` o existe `~/.codex/config.toml`.
   - agy: existe `~/.gemini/antigravity-cli` o existe el archivo `~/.gemini/config/mcp_config.json`. No basta `~/.gemini`, que también crean otras herramientas de Google.
2. **Búsqueda por nombre.** Cada entrada de la carpeta personal (sin bajar a subcarpetas) que:
   - es una carpeta;
   - se llama como la por defecto seguida de al menos un carácter más (`.claude2`, `.claude-trabajo`, `.codexB`); la comparación distingue mayúsculas;
   - contiene, directamente, al menos uno de los archivos de cuenta de la herramienta:
     - Claude Code: `.claude.json`, `.credentials.json` o `settings.json`;
     - Codex CLI: `config.toml` o `auth.json`.
   - agy no tiene búsqueda por nombre.
3. **Variable de entorno.** Si `leer_variable` da un valor no vacío para la variable de la herramienta y es una carpeta que existe, cuenta, esté donde esté. agy no tiene variable.

Después se quitan las carpetas repetidas por herramienta. Dos rutas son la misma carpeta si coinciden tras `canonicalize` (resuelve enlaces simbólicos y, en Windows, mayúsculas y la forma de la ruta); si una no existe, se comparan tal cual.

## 7. Cada herramienta

| Herramienta | Carpeta por defecto | Variable de cuenta | Archivo de registro MCP | Archivos que delatan una cuenta |
|---|---|---|---|---|
| Claude Code | `~/.claude` | `CLAUDE_CONFIG_DIR` | por defecto: `~/.claude.json`; otras: `<carpeta>/.claude.json` | `.claude.json`, `.credentials.json`, `settings.json` |
| Codex CLI | `~/.codex` | `CODEX_HOME` | `<carpeta>/config.toml` | `config.toml`, `auth.json` |
| agy | `~/.gemini` | ninguna | `<carpeta>/config/mcp_config.json` | (sin búsqueda) |

Fuentes oficiales:
- Claude Code, `CLAUDE_CONFIG_DIR`: https://code.claude.com/docs/en/settings («To keep the home-directory files somewhere else, set CLAUDE_CONFIG_DIR») y https://code.claude.com/docs/en/env-vars.
- Claude Code, servidores MCP de ámbito de usuario en `~/.claude.json`: https://code.claude.com/docs/en/mcp. Que con `CLAUDE_CONFIG_DIR` pase a `<carpeta>/.claude.json`: **No encontrado** en la documentación; se ha observado en cuentas reales. Si una versión de Claude Code lo cambia, es un fallo que se anota en el registro común.
- Codex CLI, `CODEX_HOME`: https://learn.chatgpt.com/docs/config-file/environment-variables («Sets the root for Codex state, including config, auth…», por defecto `~/.codex`).
- agy, configuración MCP en `~/.gemini/config/mcp_config.json`: https://antigravity.google/docs/mcp. Variable para otra carpeta: **No encontrado** en https://antigravity.google/docs/cli/reference.

## 8. Agy y las cuentas manuales

- agy no documenta ninguna variable para usar otra carpeta, así que su consola siempre usa `~/.gemini`. Por eso `variable_de_cuenta()` es `None` y `ajuste_de_entorno()` también.
- Una cuenta manual de agy sería engañosa: el programa registraría su servidor MCP en una carpeta que la consola no lee. La biblioteca la descarta al leerla y la rechaza al validarla; la norma prohíbe ofrecerla (N6.2) y obliga a decir que de agy solo se usa la cuenta por defecto (N9.3).
- Si agy documenta una variable en el futuro, se añade a `variable_de_cuenta` y a la búsqueda en una versión nueva de la biblioteca, no en los programas.

## 9. Fallos

La búsqueda no devuelve errores. El único `Result` es el de `validar_alta_manual`, cuyo rechazo no es un fallo sino un motivo que se explica a la persona.

| Situación | Qué pasa |
|---|---|
| La carpeta personal no se puede leer | No hay búsqueda por nombre; las demás vías siguen |
| Una carpeta candidata no se puede leer | No aporta cuenta |
| La variable apunta a algo que no existe o no es una carpeta | Se ignora |
| La variable está vacía | Se ignora |
| Una cuenta manual apunta a una carpeta que no existe | Se conserva, para que la persona la vea y la quite |

- El programa decide qué decir cuando no hay ninguna cuenta (por ejemplo, «no se ha encontrado ningún agente»).
- Lo que sí puede fallar es lo que hace el programa con la cuenta: escribir el archivo MCP o lanzar la consola. Esos fallos son del programa y se tratan con su sistema de errores.

## 10. Pruebas en el programa que la usa

- Cada prueba crea su propia carpeta temporal que hace de carpeta personal, con las subcarpetas y archivos que necesite, y la borra al terminar (`CarpetaTemporal` de la referencia).
- La función de variables es propia: `|_| None` para «ninguna puesta», o una que devuelva una ruta solo para el nombre que se prueba.
- Nunca se usa la carpeta personal real ni `std::env::set_var`: las pruebas corren en paralelo y se pisarían.
- Para comprobar el ajuste de una consola sin lanzarla, se mira `Command::get_envs()`: `Some(valor)` es «poner» y `None` es «quitar» (función `variable_de` de la referencia).
- Las seis pruebas mínimas están en N10.

## 11. Pasar de una versión a otra

De la 0.1.0 a la 0.2.0:

- Si el programa filtraba él las cuentas manuales de herramientas sin variable antes de `con_cuentas_manuales`, el filtro sobra: quítalo.
- Si definía sus propias `admite_alta_manual` o `herramientas_con_alta_manual`, usa `Herramienta::admite_alta_manual` y `Herramienta::con_alta_manual`.
- Antes de guardar un alta manual, llama a `validar_alta_manual` y muestra el motivo del rechazo.
- Nada más cambia: el resto de la API es igual.

Al actualizar la copia de un programa a una versión futura:
1. Sustituye su carpeta `bibliotecas/deteccion-agentes/` por la versión nueva, sin `target/` (N1).
2. Lee su entrada en el historial de cambios del `README.md`. Si rompe la compatibilidad, aquí habrá un apartado con los cambios de firma y cómo adaptarlos.
3. Adapta el módulo `cuentas_de_agentes` comparándolo con el nuevo `examples/integracion_de_referencia.rs`.
4. Pasa las puertas del programa en Linux y Windows: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` y `cargo audit`, más las pruebas de la copia.
5. Si el programa tiene su propio `Cargo.lock`, revisa que solo cambian las dependencias de esta biblioteca (hoy no tiene ninguna).
6. Pon al día el apartado del programa en el registro común y la documentación del programa que hable de los agentes y sus cuentas.
