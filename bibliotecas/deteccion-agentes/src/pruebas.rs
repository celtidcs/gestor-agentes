//! Pruebas de la búsqueda de cuentas, siempre en carpetas temporales propias: nunca se mira
//! la carpeta personal ni las variables reales de quien ejecuta las pruebas.

use crate::{
    buscar_cuentas, con_cuentas_manuales, validar_alta_manual, AjusteDeEntorno, Cuenta,
    Herramienta, OrigenDeLaCuenta, RechazoDeAlta,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Carpeta personal falsa, distinta para cada prueba, que se borra al terminar.
struct Personal(PathBuf);

impl Personal {
    fn nueva(nombre: &str) -> Self {
        let ruta =
            std::env::temp_dir().join(format!("deteccion-agentes-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&ruta);
        std::fs::create_dir_all(&ruta).expect("crear la carpeta personal de prueba");
        Self(ruta)
    }

    /// Crea `relativa/archivo` dentro de la carpeta personal.
    fn archivo(&self, relativa: &str, archivo: &str) -> PathBuf {
        let carpeta = self.0.join(relativa);
        std::fs::create_dir_all(&carpeta).expect("crear carpeta");
        std::fs::write(carpeta.join(archivo), "{}").expect("crear archivo");
        carpeta
    }
}

impl Drop for Personal {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn sin_variables(_: &str) -> Option<OsString> {
    None
}

fn carpetas(cuentas: &[Cuenta], herramienta: Herramienta) -> Vec<(String, OrigenDeLaCuenta)> {
    cuentas
        .iter()
        .filter(|cuenta| cuenta.herramienta == herramienta)
        .map(|cuenta| {
            let nombre = cuenta
                .carpeta
                .file_name()
                .map(|nombre| nombre.to_string_lossy().into_owned())
                .unwrap_or_default();
            (nombre, cuenta.origen)
        })
        .collect()
}

/// La cuenta por defecto y las de sufijo cualquiera, sin ningún nombre fijado.
#[test]
fn encuentra_las_cuentas_con_cualquier_sufijo() {
    let personal = Personal::nueva("sufijos");
    personal.archivo(".claude", "settings.json");
    personal.archivo(".claude2", ".credentials.json");
    personal.archivo(".claude-trabajo", ".claude.json");
    personal.archivo(".codex", "config.toml");
    personal.archivo(".codex-personal", "auth.json");

    let cuentas = buscar_cuentas(&personal.0, sin_variables);

    assert_eq!(
        carpetas(&cuentas, Herramienta::ClaudeCode),
        vec![
            (".claude".to_owned(), OrigenDeLaCuenta::PorDefecto),
            (".claude-trabajo".to_owned(), OrigenDeLaCuenta::Busqueda),
            (".claude2".to_owned(), OrigenDeLaCuenta::Busqueda),
        ]
    );
    assert_eq!(
        carpetas(&cuentas, Herramienta::CodexCli),
        vec![
            (".codex".to_owned(), OrigenDeLaCuenta::PorDefecto),
            (".codex-personal".to_owned(), OrigenDeLaCuenta::Busqueda),
        ]
    );
}

/// Una carpeta con el nombre adecuado pero sin archivos de cuenta no es una cuenta, y un
/// archivo con ese nombre tampoco.
#[test]
fn una_carpeta_sin_archivos_de_cuenta_no_cuenta() {
    let personal = Personal::nueva("vacia");
    std::fs::create_dir_all(personal.0.join(".claude-copia")).expect("crear");
    personal.archivo(".claude-otra", "notas.txt");
    std::fs::write(personal.0.join(".claude.json"), "{}").expect("crear");

    let cuentas = buscar_cuentas(&personal.0, sin_variables);

    assert_eq!(
        carpetas(&cuentas, Herramienta::ClaudeCode),
        vec![(".claude".to_owned(), OrigenDeLaCuenta::PorDefecto)],
        "solo cuenta la por defecto, que existe por su ~/.claude.json"
    );
}

/// La variable de entorno aporta una cuenta fuera de la carpeta personal, y si apunta a
/// una cuenta ya encontrada no la repite.
#[test]
fn la_variable_aporta_su_carpeta_sin_repetir() {
    let personal = Personal::nueva("variable");
    let fuera = Personal::nueva("variable-fuera");
    personal.archivo(".claude2", "settings.json");
    let externa = fuera.archivo("cuenta-claude", "settings.json");
    let repetida = personal.0.join(".claude2");

    let con_externa = buscar_cuentas(&personal.0, |nombre| {
        (nombre == "CLAUDE_CONFIG_DIR").then(|| externa.clone().into_os_string())
    });
    let con_repetida = buscar_cuentas(&personal.0, |nombre| {
        (nombre == "CLAUDE_CONFIG_DIR").then(|| repetida.clone().into_os_string())
    });

    assert!(con_externa
        .iter()
        .any(|cuenta| cuenta.carpeta == externa && cuenta.origen == OrigenDeLaCuenta::Variable));
    assert_eq!(
        carpetas(&con_repetida, Herramienta::ClaudeCode),
        vec![(".claude2".to_owned(), OrigenDeLaCuenta::Busqueda)]
    );
}

/// Las cuentas a mano se añaden sin repetir las encontradas.
#[test]
fn las_cuentas_a_mano_se_anaden_sin_repetir() {
    let personal = Personal::nueva("manual");
    let encontrada = personal.archivo(".codex", "config.toml");
    let nueva = personal.0.join("otra-cuenta-codex");

    let cuentas = con_cuentas_manuales(
        buscar_cuentas(&personal.0, sin_variables),
        &[
            (Herramienta::CodexCli, encontrada.clone()),
            (Herramienta::CodexCli, nueva.clone()),
        ],
    );

    let codex: Vec<_> = cuentas
        .iter()
        .filter(|cuenta| cuenta.herramienta == Herramienta::CodexCli)
        .map(|cuenta| (cuenta.carpeta.clone(), cuenta.origen))
        .collect();
    assert_eq!(
        codex,
        vec![
            (encontrada, OrigenDeLaCuenta::PorDefecto),
            (nueva, OrigenDeLaCuenta::Manual)
        ]
    );
}

/// Una cuenta a mano de una herramienta sin variable de cuenta se descarta: su consola no
/// usaría esa carpeta. Las de las herramientas que sí la tienen se conservan.
#[test]
fn las_cuentas_a_mano_sin_variable_de_cuenta_se_descartan() {
    let personal = Personal::nueva("manual-sin-variable");
    let de_claude = personal.0.join("otra-cuenta-claude");
    let de_codex = personal.0.join("otra-cuenta-codex");
    let de_agy = personal.0.join("otra-cuenta-agy");

    let cuentas = con_cuentas_manuales(
        Vec::new(),
        &[
            (Herramienta::ClaudeCode, de_claude.clone()),
            (Herramienta::CodexCli, de_codex.clone()),
            (Herramienta::Agy, de_agy),
        ],
    );

    let manuales: Vec<_> = cuentas
        .iter()
        .map(|cuenta| (cuenta.herramienta, cuenta.carpeta.clone()))
        .collect();
    assert_eq!(
        manuales,
        vec![
            (Herramienta::ClaudeCode, de_claude),
            (Herramienta::CodexCli, de_codex)
        ]
    );
}

/// Solo admiten alta manual las herramientas con variable de cuenta, en el orden de TODAS.
#[test]
fn solo_admiten_alta_manual_las_herramientas_con_variable() {
    for herramienta in Herramienta::TODAS {
        assert_eq!(
            herramienta.admite_alta_manual(),
            herramienta.variable_de_cuenta().is_some(),
            "{herramienta:?}"
        );
    }
    assert_eq!(
        Herramienta::con_alta_manual(),
        vec![Herramienta::ClaudeCode, Herramienta::CodexCli]
    );
}

/// El alta a mano se valida antes de guardarla: agy no la admite, y una carpeta que ya usa otra
/// cuenta de la misma herramienta se rechaza con esa cuenta. La de otra herramienta no estorba.
#[test]
fn validar_el_alta_da_el_motivo_del_rechazo() {
    let usada = Cuenta {
        herramienta: Herramienta::CodexCli,
        carpeta: PathBuf::from("cuentas").join("una"),
        origen: OrigenDeLaCuenta::Manual,
    };
    let cuentas = [usada.clone()];

    assert_eq!(
        validar_alta_manual(Herramienta::Agy, Path::new("otra"), &cuentas),
        Err(RechazoDeAlta::SinVariableDeCuenta)
    );
    assert_eq!(
        validar_alta_manual(Herramienta::CodexCli, &usada.carpeta, &cuentas),
        Err(RechazoDeAlta::CarpetaYaUsada(usada.clone()))
    );
    assert_eq!(
        validar_alta_manual(Herramienta::ClaudeCode, &usada.carpeta, &cuentas),
        Ok(())
    );
    assert_eq!(
        validar_alta_manual(Herramienta::CodexCli, Path::new("cuentas/otra"), &cuentas),
        Ok(())
    );
}

/// Las carpetas se comparan sin distinguir la barra final, y en Windows tampoco el tipo de barra
/// ni mayúsculas de minúsculas, como se escriben a mano en una configuración. En los demás
/// sistemas la barra invertida es parte del nombre y no separa carpetas.
#[test]
fn la_misma_carpeta_escrita_de_otra_forma_es_la_misma() {
    let usada = Cuenta {
        herramienta: Herramienta::CodexCli,
        carpeta: PathBuf::from("cuentas/una"),
        origen: OrigenDeLaCuenta::Manual,
    };
    let cuentas = [usada.clone()];
    let rechazada = |carpeta: &str| {
        validar_alta_manual(Herramienta::CodexCli, Path::new(carpeta), &cuentas)
            == Err(RechazoDeAlta::CarpetaYaUsada(usada.clone()))
    };

    assert!(rechazada("cuentas/una/"), "barra final");
    assert_eq!(rechazada("cuentas\\una"), cfg!(windows), "barra invertida");
    assert_eq!(rechazada("CUENTAS/UNA"), cfg!(windows), "mayúsculas");

    let sin_repetir = con_cuentas_manuales(
        Vec::new(),
        &[
            (Herramienta::CodexCli, PathBuf::from("cuentas/una")),
            (Herramienta::CodexCli, PathBuf::from("cuentas/una/")),
        ],
    );
    assert_eq!(sin_repetir.len(), 1, "{sin_repetir:?}");
}

/// Cada cuenta registra en su propio archivo y abre la consola con su variable.
#[test]
fn cada_cuenta_tiene_su_archivo_y_su_variable() {
    let personal = Path::new("/casa");
    let cuenta = |herramienta, carpeta: &str, origen| Cuenta {
        herramienta,
        carpeta: personal.join(carpeta),
        origen,
    };
    let claude = cuenta(
        Herramienta::ClaudeCode,
        ".claude",
        OrigenDeLaCuenta::PorDefecto,
    );
    let claude2 = cuenta(
        Herramienta::ClaudeCode,
        ".claude2",
        OrigenDeLaCuenta::Busqueda,
    );
    let codex2 = cuenta(Herramienta::CodexCli, ".codex2", OrigenDeLaCuenta::Manual);
    let agy = cuenta(Herramienta::Agy, ".gemini", OrigenDeLaCuenta::PorDefecto);

    assert_eq!(
        claude.archivo_de_configuracion(personal),
        personal.join(".claude.json")
    );
    assert_eq!(
        claude2.archivo_de_configuracion(personal),
        personal.join(".claude2").join(".claude.json")
    );
    assert_eq!(
        codex2.archivo_de_configuracion(personal),
        personal.join(".codex2").join("config.toml")
    );
    assert_eq!(
        claude.ajuste_de_entorno(),
        Some(AjusteDeEntorno::Quitar {
            variable: "CLAUDE_CONFIG_DIR"
        })
    );
    assert_eq!(
        claude2.ajuste_de_entorno(),
        Some(AjusteDeEntorno::Poner {
            variable: "CLAUDE_CONFIG_DIR",
            valor: personal.join(".claude2")
        })
    );
    assert_eq!(
        codex2.ajuste_de_entorno(),
        Some(AjusteDeEntorno::Poner {
            variable: "CODEX_HOME",
            valor: personal.join(".codex2")
        })
    );
    assert_eq!(agy.ajuste_de_entorno(), None);
}

/// Archivos y carpetas de la biblioteca que se versionan y viajan con ella: los que revisa la
/// prueba de datos privados. Lo demás (lo compilado, el git, lo que nunca se versiona) no cuenta.
const LO_QUE_SE_VERSIONA: [&str; 9] = [
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "mejoras.md",
    "LICENSE",
    ".gitignore",
    "src",
    "examples",
    "documentacion",
];

/// Único dominio de correo admitido: el que asigna GitHub para no publicar el correo real.
const DOMINIO_DE_CORREO_ADMITIDO: &str = "users.noreply.github.com";

/// Lista privada de fragmentos prohibidos, en la carpeta que contiene la biblioteca. Solo existe
/// junto al original: en las copias de los programas no está, y la prueba se queda con las formas
/// genéricas. Nunca se copia dentro de la biblioteca.
const LISTA_PRIVADA: &str = "fragmentos-privados.txt";

/// Todos los archivos de una carpeta, recorrida entera.
fn archivos_de(carpeta: &Path, encontrados: &mut Vec<PathBuf>) {
    let Ok(entradas) = std::fs::read_dir(carpeta) else {
        return;
    };
    for entrada in entradas.flatten() {
        let ruta = entrada.path();
        if ruta.is_dir() {
            archivos_de(&ruta, encontrados);
        } else {
            encontrados.push(ruta);
        }
    }
}

/// Si el texto contiene una ruta con letra de unidad: una letra suelta, dos puntos y una barra.
/// La letra no puede ir pegada a otra, para no confundirla con el esquema de una dirección web.
fn tiene_letra_de_unidad(texto: &str) -> bool {
    let letras: Vec<char> = texto.chars().collect();
    letras.windows(3).enumerate().any(|(i, trozo)| {
        let suelta = i == 0 || !letras[i - 1].is_alphanumeric();
        suelta
            && trozo[0].is_ascii_alphabetic()
            && trozo[1] == ':'
            && matches!(trozo[2], '/' | '\\')
    })
}

/// Si el texto contiene una carpeta personal o una dirección IP de una red privada. Las formas se
/// componen aquí para que este archivo no las contenga tal cual.
fn tiene_ruta_personal_o_red_privada(texto: &str) -> bool {
    let formas = [
        format!("{0}home{0}", '/'),
        format!("{0}users{0}", '/'),
        format!("{0}users{0}", '\\'),
        format!("{}.{}.", 192, 168),
    ];
    formas.iter().any(|forma| texto.contains(forma.as_str()))
}

/// Correos del texto cuyo dominio no es el admitido.
fn correos_no_admitidos(texto: &str) -> Vec<String> {
    let de_correo = |c: char| c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '+');
    let mut correos = Vec::new();
    for (posicion, _) in texto.match_indices('@') {
        let antes: String = texto[..posicion]
            .chars()
            .rev()
            .take_while(|c| de_correo(*c))
            .collect();
        let despues: String = texto[posicion + 1..]
            .chars()
            .take_while(|c| de_correo(*c))
            .collect();
        let despues = despues.trim_end_matches('.');
        if !antes.is_empty() && despues.contains('.') && despues != DOMINIO_DE_CORREO_ADMITIDO {
            correos.push(format!(
                "{}@{despues}",
                antes.chars().rev().collect::<String>()
            ));
        }
    }
    correos
}

/// Si el fragmento aparece como tal: cuando empieza o termina en letra o cifra, no puede ir pegado
/// a otra, para que un fragmento no salte dentro de una palabra legítima más larga.
fn contiene_fragmento(texto: &str, fragmento: &str) -> bool {
    let pegado = |c: Option<char>| c.is_some_and(char::is_alphanumeric);
    texto.match_indices(fragmento).any(|(posicion, _)| {
        let antes = texto[..posicion].chars().next_back();
        let despues = texto[posicion + fragmento.len()..].chars().next();
        let primera = fragmento.chars().next();
        let ultima = fragmento.chars().next_back();
        !(pegado(primera) && pegado(antes)) && !(pegado(ultima) && pegado(despues))
    })
}

/// La biblioteca viaja con la copia a cada programa y puede publicarse con él: ningún archivo que
/// se versiona contiene rutas del equipo, carpetas personales, direcciones de redes privadas,
/// correos distintos del que asigna GitHub ni los fragmentos de la lista privada, si existe.
#[test]
fn la_biblioteca_no_contiene_datos_privados() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !raiz.join(LISTA_PRIVADA).exists(),
        "la lista privada no puede ir dentro de la biblioteca"
    );
    let privados: Vec<String> = raiz
        .parent()
        .and_then(|carpeta| std::fs::read_to_string(carpeta.join(LISTA_PRIVADA)).ok())
        .unwrap_or_default()
        .lines()
        .map(|linea| linea.trim().to_lowercase())
        .filter(|linea| !linea.is_empty() && !linea.starts_with('#'))
        .collect();
    let mut archivos = Vec::new();
    for nombre in LO_QUE_SE_VERSIONA {
        let ruta = raiz.join(nombre);
        if ruta.is_dir() {
            archivos_de(&ruta, &mut archivos);
        } else if ruta.is_file() {
            archivos.push(ruta);
        }
    }
    archivos.sort();
    assert!(archivos.len() >= 10, "faltan archivos: {archivos:?}");
    let mut quejas = Vec::new();
    for archivo in archivos {
        let Ok(texto) = std::fs::read_to_string(&archivo) else {
            continue;
        };
        for (indice, linea) in texto.lines().enumerate() {
            let minusculas = linea.to_lowercase();
            let mut motivos = Vec::new();
            if tiene_letra_de_unidad(&minusculas) {
                motivos.push("una ruta con letra de unidad".to_string());
            }
            if tiene_ruta_personal_o_red_privada(&minusculas) {
                motivos.push("una carpeta personal o una red privada".to_string());
            }
            for correo in correos_no_admitidos(&minusculas) {
                motivos.push(format!("el correo {correo}"));
            }
            for fragmento in privados
                .iter()
                .filter(|f| contiene_fragmento(&minusculas, f))
            {
                motivos.push(format!("un fragmento de la lista privada («{fragmento}»)"));
            }
            for motivo in motivos {
                quejas.push(format!("{}:{}: {motivo}", archivo.display(), indice + 1));
            }
        }
    }
    assert!(
        quejas.is_empty(),
        "la biblioteca no puede contener datos privados:\n  - {}",
        quejas.join("\n  - ")
    );
}

/// Las formas genéricas saltan con lo que tienen que saltar y no con lo legítimo.
#[test]
fn las_formas_genericas_de_datos_privados_distinguen_lo_legitimo() {
    let unidad = format!("{}:{}carpeta", 'm', '\\');
    assert!(tiene_letra_de_unidad(&format!("está en {unidad}")));
    assert!(!tiene_letra_de_unidad(
        "https://code.claude.com/docs/en/settings"
    ));
    assert!(tiene_ruta_personal_o_red_privada(&format!(
        "{0}home{0}x",
        '/'
    )));
    assert!(!tiene_ruta_personal_o_red_privada(
        "~/.claude/settings.json"
    ));
    let ajeno = format!("alguien{}ejemplo.com", '@');
    assert_eq!(correos_no_admitidos(&ajeno), vec![ajeno]);
    let admitido = format!("celtidcs{}{DOMINIO_DE_CORREO_ADMITIDO}", '@');
    assert!(correos_no_admitidos(&admitido).is_empty());
    assert!(contiene_fragmento("lo dijo el vecino.", "el vecino"));
    assert!(!contiene_fragmento("los vecinos", "vecino"));
}
