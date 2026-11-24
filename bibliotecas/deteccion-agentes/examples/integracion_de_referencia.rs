//! Integración de referencia de deteccion-agentes en un programa.
//!
//! Es la forma única de conectar la biblioteca que describe documentacion/norma-de-integracion.md.
//! El módulo cuentas_de_agentes se copia en el programa y se adapta el tipo de las cuentas
//! manuales a las preferencias del programa. Cargo compila este ejemplo, lo revisa con clippy y
//! ejecuta sus pruebas con cargo test, así que la plantilla no puede quedarse atrás respecto de
//! la biblioteca sin que algo falle.
//!
//! Al ejecutarlo con cargo run --example integracion_de_referencia lee el equipo real: lista las
//! cuentas encontradas en la carpeta personal, con la orden que abriría la consola de cada una,
//! y las herramientas que admiten alta manual, sin escribir ni lanzar nada.

/// El único módulo del programa que habla con la biblioteca.
mod cuentas_de_agentes {
    use deteccion_agentes::{
        buscar_cuentas, con_cuentas_manuales, validar_alta_manual, Cuenta, Herramienta,
        RechazoDeAlta,
    };
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// Variable que guarda la carpeta personal en cada sistema.
    fn variable_de_la_carpeta_personal() -> &'static str {
        if cfg!(windows) {
            "USERPROFILE"
        } else {
            "HOME"
        }
    }

    /// Una cuenta dada de alta a mano, tal como la guarda el programa en sus preferencias.
    ///
    /// El programa la adapta a su formato (JSON, TOML...). Si guarda la herramienta como texto,
    /// usa un nombre propio y estable, no el Debug de Herramienta.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CuentaManual {
        /// Herramienta de la cuenta. Solo una de Herramienta::con_alta_manual().
        pub herramienta: Herramienta,
        /// Carpeta de configuración que eligió la persona.
        pub carpeta: PathBuf,
    }

    /// La carpeta personal de quien ejecuta el programa, o None si el sistema no la da.
    ///
    /// Sin carpeta personal no se busca nada: el programa dice que no ha encontrado agentes.
    pub fn carpeta_personal(leer_variable: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
        leer_variable(variable_de_la_carpeta_personal())
            .filter(|valor| !valor.is_empty())
            .map(PathBuf::from)
    }

    /// Todas las cuentas del equipo: las que encuentra la biblioteca y las dadas de alta a mano.
    ///
    /// Se llama cada vez que el programa va a mostrar o usar las cuentas (al abrir su ventana de
    /// conexiones, antes de lanzar una consola), nunca en cada fotograma. El programa real pasa
    /// |nombre| std::env::var_os(nombre) como leer_variable; las pruebas pasan una función propia.
    /// La biblioteca descarta sola las manuales que no se pueden usar: aquí no se filtra nada.
    pub fn cuentas_del_equipo(
        personal: &Path,
        leer_variable: impl Fn(&str) -> Option<OsString>,
        manuales: &[CuentaManual],
    ) -> Vec<Cuenta> {
        let manuales: Vec<(Herramienta, PathBuf)> = manuales
            .iter()
            .map(|manual| (manual.herramienta, manual.carpeta.clone()))
            .collect();
        con_cuentas_manuales(buscar_cuentas(personal, leer_variable), &manuales)
    }

    /// Comprueba una cuenta que la persona quiere dar de alta a mano, antes de guardarla.
    ///
    /// cuentas son las que el programa ya tiene (las de cuentas_del_equipo). Con el rechazo, el
    /// programa dice a la persona el motivo con sus propios textos.
    pub fn validar_alta(nueva: &CuentaManual, cuentas: &[Cuenta]) -> Result<(), RechazoDeAlta> {
        validar_alta_manual(nueva.herramienta, &nueva.carpeta, cuentas)
    }

    /// Archivo donde el programa registra su servidor MCP para esta cuenta.
    ///
    /// Nunca se compone a mano: la cuenta por defecto de Claude Code lo tiene fuera de su
    /// carpeta.
    pub fn archivo_para_registrar_mcp(cuenta: &Cuenta, personal: &Path) -> PathBuf {
        cuenta.archivo_de_configuracion(personal)
    }

    /// La orden con que se abre la consola de la herramienta con esta cuenta.
    ///
    /// Aplica siempre el ajuste de entorno de la cuenta: en una cuenta que no es la por defecto
    /// pone la variable con su carpeta, y en la por defecto la quita, por si el programa que
    /// abre la consola la tiene puesta.
    pub fn orden_de_consola(cuenta: &Cuenta, programa: &Path) -> Command {
        let mut orden = Command::new(programa);
        if let Some(ajuste) = cuenta.ajuste_de_entorno() {
            ajuste.aplicar_a(&mut orden);
        }
        orden
    }

    /// Nombre con que el programa presenta la cuenta: el de la herramienta y, si no es la por
    /// defecto, el de su carpeta entre paréntesis.
    pub fn nombre_para_mostrar(cuenta: &Cuenta, nombre_de_la_herramienta: &str) -> String {
        match cuenta.nombre_visible() {
            Some(carpeta) => format!("{nombre_de_la_herramienta} ({carpeta})"),
            None => nombre_de_la_herramienta.to_string(),
        }
    }
}

fn main() {
    let leer = |nombre: &str| std::env::var_os(nombre);
    let Some(personal) = cuentas_de_agentes::carpeta_personal(leer) else {
        println!("No se conoce la carpeta personal: no se busca ninguna cuenta.");
        return;
    };
    let cuentas = cuentas_de_agentes::cuentas_del_equipo(&personal, leer, &[]);
    for cuenta in &cuentas {
        let herramienta = format!("{:?}", cuenta.herramienta);
        let nombre = cuentas_de_agentes::nombre_para_mostrar(cuenta, &herramienta);
        println!(
            "{nombre}: {} ({:?}); servidor MCP en {}",
            cuenta.carpeta.display(),
            cuenta.origen,
            cuentas_de_agentes::archivo_para_registrar_mcp(cuenta, &personal).display()
        );
        // La orden se prepara pero no se lanza: el ejemplo no abre ninguna consola.
        let ejecutable = match cuenta.herramienta {
            deteccion_agentes::Herramienta::ClaudeCode => "claude",
            deteccion_agentes::Herramienta::CodexCli => "codex",
            deteccion_agentes::Herramienta::Agy => "agy",
        };
        let orden = cuentas_de_agentes::orden_de_consola(cuenta, std::path::Path::new(ejecutable));
        println!("  consola: {orden:?}");
    }
    println!(
        "Admiten alta manual: {:?}",
        deteccion_agentes::Herramienta::con_alta_manual()
    );
    // Lo que diría el formulario de alta si la persona eligiera la carpeta por defecto de Codex
    // CLI: no se guarda nada.
    let prueba = cuentas_de_agentes::CuentaManual {
        herramienta: deteccion_agentes::Herramienta::CodexCli,
        carpeta: deteccion_agentes::Herramienta::CodexCli.carpeta_por_defecto(&personal),
    };
    println!(
        "Alta a mano de {}: {:?}",
        prueba.carpeta.display(),
        cuentas_de_agentes::validar_alta(&prueba, &cuentas)
    );
}

/// Las pruebas mínimas que la norma exige a toda integración. Trabajan en carpetas temporales
/// propias y no leen la carpeta personal ni las variables reales.
#[cfg(test)]
mod pruebas {
    use super::cuentas_de_agentes::{
        archivo_para_registrar_mcp, carpeta_personal, cuentas_del_equipo, nombre_para_mostrar,
        orden_de_consola, validar_alta, CuentaManual,
    };
    use deteccion_agentes::{Herramienta, OrigenDeLaCuenta, RechazoDeAlta};
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};

    /// Carpeta temporal que se borra al terminar la prueba.
    struct CarpetaTemporal(PathBuf);

    impl CarpetaTemporal {
        fn nueva(nombre: &str) -> Self {
            let ruta = std::env::temp_dir().join(format!(
                "deteccion-agentes-referencia-{}-{nombre}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&ruta);
            std::fs::create_dir_all(&ruta).expect("crear la carpeta temporal");
            Self(ruta)
        }

        /// Crea una carpeta de cuenta con un archivo dentro.
        fn cuenta(&self, carpeta: &str, archivo: &str) -> PathBuf {
            let ruta = self.0.join(carpeta);
            std::fs::create_dir_all(&ruta).expect("crear la cuenta");
            std::fs::write(ruta.join(archivo), "{}").expect("crear el archivo de la cuenta");
            ruta
        }
    }

    impl Drop for CarpetaTemporal {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Ninguna variable puesta.
    fn sin_variables(_: &str) -> Option<OsString> {
        None
    }

    /// Valor de la variable de la orden, Some(None) si la orden la quita y None si no la toca.
    fn variable_de(orden: &std::process::Command, variable: &str) -> Option<Option<OsString>> {
        orden
            .get_envs()
            .find(|(nombre, _)| *nombre == OsStr::new(variable))
            .map(|(_, valor)| valor.map(OsStr::to_os_string))
    }

    /// 1. La cuenta por defecto se encuentra y abre su consola quitando la variable.
    #[test]
    fn la_cuenta_por_defecto_se_encuentra_y_quita_la_variable() {
        let personal = CarpetaTemporal::nueva("por-defecto");
        personal.cuenta(".claude", "settings.json");
        let cuentas = cuentas_del_equipo(&personal.0, sin_variables, &[]);
        assert_eq!(cuentas.len(), 1, "{cuentas:?}");
        assert_eq!(cuentas[0].origen, OrigenDeLaCuenta::PorDefecto);
        assert_eq!(
            archivo_para_registrar_mcp(&cuentas[0], &personal.0),
            personal.0.join(".claude.json")
        );
        let orden = orden_de_consola(&cuentas[0], Path::new("claude"));
        assert_eq!(variable_de(&orden, "CLAUDE_CONFIG_DIR"), Some(None));
        assert_eq!(
            nombre_para_mostrar(&cuentas[0], "Claude Code"),
            "Claude Code"
        );
    }

    /// 2. Una segunda cuenta se encuentra por su nombre y abre su consola con la variable puesta.
    #[test]
    fn una_segunda_cuenta_se_encuentra_y_pone_la_variable() {
        let personal = CarpetaTemporal::nueva("segunda");
        let segunda = personal.cuenta(".codex-trabajo", "config.toml");
        let cuentas = cuentas_del_equipo(&personal.0, sin_variables, &[]);
        let cuenta = cuentas
            .iter()
            .find(|cuenta| cuenta.carpeta == segunda)
            .expect("la segunda cuenta tiene que encontrarse");
        assert_eq!(cuenta.origen, OrigenDeLaCuenta::Busqueda);
        assert_eq!(
            archivo_para_registrar_mcp(cuenta, &personal.0),
            segunda.join("config.toml")
        );
        let orden = orden_de_consola(cuenta, Path::new("codex"));
        assert_eq!(
            variable_de(&orden, "CODEX_HOME"),
            Some(Some(segunda.clone().into_os_string()))
        );
        assert_eq!(
            nombre_para_mostrar(cuenta, "Codex CLI"),
            "Codex CLI (.codex-trabajo)"
        );
    }

    /// 3. Una carpeta con el nombre pero sin archivos de cuenta no es una cuenta.
    #[test]
    fn una_carpeta_sin_archivos_de_cuenta_no_cuenta() {
        let personal = CarpetaTemporal::nueva("vacia");
        std::fs::create_dir_all(personal.0.join(".claude-vacia")).expect("crear la carpeta");
        assert!(cuentas_del_equipo(&personal.0, sin_variables, &[]).is_empty());
    }

    /// 4. La carpeta de la variable de entorno cuenta aunque esté fuera de la carpeta personal.
    #[test]
    fn la_carpeta_de_la_variable_cuenta_aunque_este_fuera() {
        let personal = CarpetaTemporal::nueva("variable-personal");
        let fuera = CarpetaTemporal::nueva("variable-fuera");
        let leer = |nombre: &str| {
            (nombre == "CLAUDE_CONFIG_DIR").then(|| fuera.0.clone().into_os_string())
        };
        let cuentas = cuentas_del_equipo(&personal.0, leer, &[]);
        let de_la_variable = |cuenta: &&deteccion_agentes::Cuenta| {
            cuenta.carpeta == fuera.0 && cuenta.origen == OrigenDeLaCuenta::Variable
        };
        assert!(cuentas.iter().any(|c| de_la_variable(&c)), "{cuentas:?}");
    }

    /// 5. Una cuenta manual se añade y se conserva aunque su carpeta ya no exista.
    ///
    /// Las de una herramienta sin variable de cuenta no se admiten: la biblioteca las descarta
    /// al leerlas, y validar_alta las rechaza con su motivo antes de guardarlas, igual que una
    /// carpeta que ya es de otra cuenta.
    #[test]
    fn las_cuentas_manuales_se_anaden_y_las_de_agy_no() {
        let personal = CarpetaTemporal::nueva("manual");
        let manuales = [
            CuentaManual {
                herramienta: Herramienta::ClaudeCode,
                carpeta: personal.0.join("ya-no-existe"),
            },
            CuentaManual {
                herramienta: Herramienta::Agy,
                carpeta: personal.0.join("otra-de-agy"),
            },
        ];
        let cuentas = cuentas_del_equipo(&personal.0, sin_variables, &manuales);
        assert_eq!(cuentas.len(), 1, "{cuentas:?}");
        assert_eq!(cuentas[0].origen, OrigenDeLaCuenta::Manual);
        assert_eq!(cuentas[0].herramienta, Herramienta::ClaudeCode);
        assert_eq!(
            Herramienta::con_alta_manual(),
            vec![Herramienta::ClaudeCode, Herramienta::CodexCli]
        );
        assert_eq!(
            validar_alta(&manuales[1], &cuentas),
            Err(RechazoDeAlta::SinVariableDeCuenta)
        );
        assert_eq!(
            validar_alta(&manuales[0], &cuentas),
            Err(RechazoDeAlta::CarpetaYaUsada(cuentas[0].clone()))
        );
    }

    /// 6. La carpeta personal sale de la variable del sistema, y vacía no vale.
    #[test]
    fn la_carpeta_personal_sale_de_la_variable_del_sistema() {
        let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        let leer = |nombre: &str| (nombre == variable).then(|| OsString::from("/casa/persona"));
        assert_eq!(carpeta_personal(leer), Some(PathBuf::from("/casa/persona")));
        assert_eq!(carpeta_personal(|_| Some(OsString::new())), None);
        assert_eq!(carpeta_personal(sin_variables), None);
    }
}
