//! Barreras de inyección en los guiones de la terminal.
//!
//! Dos niveles. Primero, el texto exacto del guion con datos hostiles (comillas simples, rectas y
//! tipográficas, dobles, punto y coma, $, acentos y espacios). Después, en el sistema en que se
//! ejecutan las pruebas, se ejecuta de verdad el guion, como hacía la prueba de
//! lanzamiento de la 2.3.2: tiene que entrar en la carpeta hostil y no crear el archivo testigo.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default,
    missing_docs
)]

mod comun;

use std::path::PathBuf;

use comun::{
    con_su_ejecutable, guion_bash, guion_powershell, peticion, plan, CarpetaTemporal,
    EntornoDeEnsayo,
};
use lanzador::{Herramienta, Sistema};

/// Comillas que PowerShell toma por comilla simple.
const COMILLAS_POWERSHELL: [char; 5] = ['\'', '\u{2018}', '\u{2019}', '\u{201A}', '\u{201B}'];

/// Nombres de carpeta hostiles. `{q}` se sustituye por cada comilla y `{t}` por el testigo.
const PLANTILLAS_HOSTILES: [&str; 6] = [
    "a{q}; touch {t}; echo {q}",
    "b\\\"; touch {t}; echo \"",
    "c$(touch {t})`touch {t}`",
    "d$env:PATH ${{t}} \"$t\" {q}+{q}",
    "é ñ ü — Proyecto {q}con espacios{q}",
    "f{q}{q}{q}; & touch {t}",
];

fn hostiles(testigo: &str) -> Vec<String> {
    let mut nombres = Vec::new();
    for plantilla in PLANTILLAS_HOSTILES {
        for q in COMILLAS_POWERSHELL {
            nombres.push(
                plantilla
                    .replace("{q}", &q.to_string())
                    .replace("{t}", testigo),
            );
        }
    }
    nombres.dedup();
    nombres
}

/// Comprueba que en `guion`, todo lo que va entre comillas simples de PowerShell se lee igual que
/// el valor original: recorre el texto como lo haría el analizador de PowerShell.
fn literales_powershell(guion: &str) -> Vec<String> {
    let mut literales = Vec::new();
    let mut caracteres = guion.chars().peekable();
    while let Some(c) = caracteres.next() {
        if !COMILLAS_POWERSHELL.contains(&c) {
            continue;
        }
        let mut literal = String::new();
        loop {
            let Some(d) = caracteres.next() else {
                panic!("cadena sin cerrar en el guion: {guion}");
            };
            if COMILLAS_POWERSHELL.contains(&d) {
                if caracteres
                    .peek()
                    .is_some_and(|e| COMILLAS_POWERSHELL.contains(e))
                {
                    literal.push(caracteres.next().unwrap());
                } else {
                    break;
                }
            } else {
                literal.push(d);
            }
        }
        literales.push(literal);
    }
    literales
}

#[test]
fn powershell_cada_dato_hostil_queda_en_un_solo_literal_intacto() {
    for nombre in hostiles("PWNED") {
        let carpeta = format!(r"C:\proyectos\{nombre}");
        let mut p = peticion(Herramienta::Claude);
        p.carpeta_proyecto = PathBuf::from(&carpeta);
        p.nombre_proyecto = nombre.clone();
        p.identidad = format!("Claude {nombre}");
        p.directorio_configuracion = Some(PathBuf::from(format!(r"C:\cfg\{nombre}")));
        p.ejecutable = Some(PathBuf::from(format!(r"C:\bin\{nombre}\claude.exe")));
        let guion = guion_powershell(&plan(&p, Sistema::Windows, &con_su_ejecutable(&p)));

        let literales = literales_powershell(&guion);
        let esperados = vec![
            format!("Claude {nombre} · {nombre}"),
            format!("Claude {nombre}"),
            String::new(),
            format!(r"C:\cfg\{nombre}"),
            carpeta.clone(),
            format!(r"C:\bin\{nombre}\claude.exe"),
        ];
        assert_eq!(literales, esperados, "guion: {guion}");
    }
}

/// Lo que queda fuera de los literales es siempre el mismo código: ningún dato lo cambia.
#[test]
fn powershell_el_codigo_fuera_de_los_literales_no_depende_de_los_datos() {
    let esqueleto = |nombre: &str| {
        let mut p = peticion(Herramienta::Claude);
        p.carpeta_proyecto = PathBuf::from(format!(r"C:\{nombre}"));
        p.nombre_proyecto = nombre.to_owned();
        p.ejecutable = Some(PathBuf::from("claude"));
        let guion = guion_powershell(&plan(&p, Sistema::Windows, &con_su_ejecutable(&p)));
        let mut fuera = String::new();
        let mut dentro = false;
        let mut caracteres = guion.chars().peekable();
        while let Some(c) = caracteres.next() {
            if COMILLAS_POWERSHELL.contains(&c) {
                if dentro
                    && caracteres
                        .peek()
                        .is_some_and(|e| COMILLAS_POWERSHELL.contains(e))
                {
                    caracteres.next();
                    continue;
                }
                dentro = !dentro;
                fuera.push('\'');
            } else if !dentro {
                fuera.push(c);
            }
        }
        fuera
    };
    let referencia = esqueleto("normal");
    for nombre in hostiles("PWNED") {
        assert_eq!(esqueleto(&nombre), referencia, "nombre: {nombre}");
    }
}

#[test]
fn bash_cada_dato_hostil_queda_en_un_literal_intacto() {
    for nombre in hostiles("/tmp/PWNED") {
        let carpeta = format!("/proyectos/{nombre}");
        let mut p = peticion(Herramienta::Claude);
        p.carpeta_proyecto = PathBuf::from(&carpeta);
        p.ejecutable = Some(PathBuf::from("claude"));
        let guion = guion_bash(&plan(&p, Sistema::Linux, &con_su_ejecutable(&p)));
        let escapada = carpeta.replace('\'', "'\\''");
        assert!(
            guion.contains(&format!("cd '{escapada}'; 'claude'; exec bash")),
            "{guion}"
        );
    }
}

/// El título va en su propio argumento del emulador, entero.
#[test]
fn linux_el_titulo_hostil_va_entero_en_su_argumento() {
    for nombre in hostiles("/tmp/PWNED") {
        let mut p = peticion(Herramienta::Claude);
        p.nombre_proyecto = nombre.clone();
        p.ejecutable = Some(PathBuf::from("claude"));
        let plan = plan(&p, Sistema::Linux, &con_su_ejecutable(&p));
        let args = comun::textos(&plan.argumentos);
        assert_eq!(args[0], "-T");
        assert_eq!(args[1], format!("Claude 2 · {nombre}"));
    }
}

/// En Linux una ruta que no es UTF-8 se rechaza en lugar de escribirse cambiada.
#[cfg(unix)]
#[test]
fn linux_una_ruta_no_unicode_se_rechaza() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    use lanzador::ErrorDeLanzamiento;
    let mut p = peticion(Herramienta::Claude);
    p.carpeta_proyecto = PathBuf::from(OsStr::from_bytes(b"/proyectos/\xFF"));
    p.ejecutable = Some(PathBuf::from("/bin/claude"));
    let resultado = lanzador::planificar(&p, Sistema::Linux, &con_su_ejecutable(&p));
    assert!(
        matches!(resultado, Err(ErrorDeLanzamiento::RutaNoUnicode { .. })),
        "{resultado:?}"
    );
}

/// Nombre del archivo testigo, sin barras para que quepa en un nombre de carpeta. Si algo llega a
/// ejecutarse, se crea en la carpeta de trabajo del intérprete o en la carpeta hostil.
const TESTIGO: &str = "PWNED";

/// Comprueba que el testigo no apareció en ninguna de las dos carpetas posibles.
fn sin_testigo(raiz: &std::path::Path, carpeta: &std::path::Path) -> bool {
    !raiz.join(TESTIGO).exists() && !carpeta.join(TESTIGO).exists()
}

/// Ejecución real con bash: el guion entra en cada carpeta hostil y no crea el testigo.
#[cfg(unix)]
#[test]
fn bash_ejecutado_entra_en_la_carpeta_hostil_sin_inyectar() {
    use std::process::{Command, Stdio};
    let temporal = CarpetaTemporal::nueva("lanzador-inyeccion");
    let raiz = temporal.ruta().to_path_buf();
    for nombre in hostiles(TESTIGO) {
        let carpeta = raiz.join(&nombre);
        std::fs::create_dir_all(&carpeta).unwrap();
        let mut p = peticion(Herramienta::Claude);
        p.carpeta_proyecto = carpeta.clone();
        p.nombre_proyecto = nombre.clone();
        p.ejecutable = Some(PathBuf::from("/bin/pwd"));
        let guion = guion_bash(&plan(&p, Sistema::Linux, &con_su_ejecutable(&p)));
        let salida = Command::new("bash")
            .args(["-c", &guion])
            .current_dir(&raiz)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let pwd = String::from_utf8(salida.stdout).unwrap();
        assert_eq!(
            pwd.lines().next(),
            Some(carpeta.to_str().unwrap()),
            "nombre: {nombre}"
        );
        assert!(
            sin_testigo(&raiz, &carpeta),
            "inyección con el nombre: {nombre}"
        );
    }
}

/// Ejecución real con PowerShell: el guion entra en cada carpeta hostil y no crea el testigo.
/// Se quita -NoExit para que powershell.exe termine al acabar el guion, y el «ejecutable» es un
/// guion .ps1 que escribe la carpeta actual.
#[cfg(windows)]
#[test]
fn powershell_ejecutado_entra_en_la_carpeta_hostil_sin_inyectar() {
    use std::process::{Command, Stdio};
    let temporal = CarpetaTemporal::nueva("lanzador-inyeccion");
    let raiz = temporal.ruta().to_path_buf();
    // La carpeta se escribe en un archivo UTF-8 y no en la consola, cuya página de códigos
    // convertiría las comillas tipográficas en rectas.
    let escribir = raiz.join("escribir-carpeta.ps1");
    let anotada = raiz.join("carpeta.txt");
    std::fs::write(
        &escribir,
        format!(
            "[IO.File]::WriteAllText('{}', (Get-Location).ProviderPath)",
            anotada.display()
        ),
    )
    .unwrap();
    // En Windows «touch» no existe; New-Item crea el testigo si algo llega a ejecutarse.
    let orden = format!("New-Item -ItemType File -Path {TESTIGO}");
    for nombre in hostiles(&orden) {
        if nombre.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
            continue; // Caracteres que no admite un nombre de carpeta en Windows.
        }
        let carpeta = raiz.join(&nombre);
        std::fs::create_dir_all(&carpeta).unwrap();
        let mut p = peticion(Herramienta::Claude);
        p.carpeta_proyecto = carpeta.clone();
        p.nombre_proyecto = nombre.clone();
        p.ejecutable = Some(escribir.clone());
        let plan = plan(&p, Sistema::Windows, &con_su_ejecutable(&p));
        let salida = Command::new(&plan.programa)
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass"])
            .args(&plan.argumentos[1..])
            .current_dir(&raiz)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let texto = std::fs::read_to_string(&anotada).unwrap_or_else(|_| {
            panic!("el guion no llegó a ejecutar el programa; nombre: {nombre}; {salida:?}")
        });
        std::fs::remove_file(&anotada).unwrap();
        assert_eq!(texto, carpeta.to_str().unwrap(), "nombre: {nombre}");
        assert!(
            sin_testigo(&raiz, &carpeta),
            "inyección con el nombre: {nombre}"
        );
    }
}

// ── Argumentos del perfil ─────────

/// Programa de prueba de codex y agy en el doble del entorno, fijado en el perfil.
const PROGRAMA_FIJADO: &str = "/opt/fijado/programa";

/// Petición de codex o agy en terminal con datos hostiles en la carpeta y en los dos argumentos.
fn peticion_con_argumentos(
    herramienta: Herramienta,
    nombre: &str,
    carpeta: &str,
) -> lanzador::PeticionDeLanzamiento {
    let mut p = peticion(herramienta);
    p.carpeta_proyecto = PathBuf::from(carpeta);
    p.ejecutable = Some(PathBuf::from(PROGRAMA_FIJADO));
    p.argumentos_extra = vec![nombre.to_owned(), format!("--opcion={nombre}")];
    p
}

#[test]
fn powershell_cada_argumento_hostil_del_perfil_queda_en_su_literal_intacto() {
    let entorno = EntornoDeEnsayo::default().con_archivo(PROGRAMA_FIJADO);
    for herramienta in [Herramienta::Codex, Herramienta::Gemini] {
        for nombre in hostiles("PWNED") {
            let carpeta = format!(r"C:\proyectos\{nombre}");
            let p = peticion_con_argumentos(herramienta, &nombre, &carpeta);
            let guion = guion_powershell(&plan(&p, Sistema::Windows, &entorno));
            let esperados = vec![
                "Claude 2 · Demo".to_owned(),
                "Claude 2".to_owned(),
                carpeta.clone(),
                PROGRAMA_FIJADO.to_owned(),
                nombre.clone(),
                format!("--opcion={nombre}"),
            ];
            assert_eq!(literales_powershell(&guion), esperados, "guion: {guion}");
        }
    }
}

#[test]
fn bash_cada_argumento_hostil_del_perfil_queda_en_su_literal_intacto() {
    let entorno = EntornoDeEnsayo::default().con_archivo(PROGRAMA_FIJADO);
    for herramienta in [Herramienta::Codex, Herramienta::Gemini] {
        for nombre in hostiles("/tmp/PWNED") {
            let carpeta = format!("/proyectos/{nombre}");
            let p = peticion_con_argumentos(herramienta, &nombre, &carpeta);
            let guion = guion_bash(&plan(&p, Sistema::Linux, &entorno));
            let escapar = |texto: &str| texto.replace('\'', "'\\''");
            let esperado = format!(
                "cd '{}'; '{PROGRAMA_FIJADO}' '{}' '--opcion={}'; exec bash",
                escapar(&carpeta),
                escapar(&nombre),
                escapar(&nombre)
            );
            assert!(guion.ends_with(&esperado), "guion: {guion}");
        }
    }
}

/// Ejecución real con bash: printf recibe cada argumento hostil intacto y no se crea el testigo.
#[cfg(unix)]
#[test]
fn bash_ejecutado_pasa_los_argumentos_hostiles_intactos_sin_inyectar() {
    use std::process::{Command, Stdio};
    let temporal = CarpetaTemporal::nueva("lanzador-argumentos");
    let raiz = temporal.ruta().to_path_buf();
    let printf = PathBuf::from("/usr/bin/printf");
    let entorno = EntornoDeEnsayo::default().con_archivo(printf.clone());
    for nombre in hostiles(TESTIGO) {
        let mut p = peticion(Herramienta::Gemini);
        p.carpeta_proyecto = raiz.clone();
        p.ejecutable = Some(printf.clone());
        p.argumentos_extra = vec!["%s\\n".to_owned(), nombre.clone()];
        let guion = guion_bash(&plan(&p, Sistema::Linux, &entorno));
        let salida = Command::new("bash")
            .args(["-c", &guion])
            .current_dir(&raiz)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let impreso = String::from_utf8(salida.stdout).unwrap();
        assert_eq!(
            impreso.lines().next(),
            Some(nombre.as_str()),
            "nombre: {nombre}"
        );
        assert!(
            !raiz.join(TESTIGO).exists(),
            "inyección con el nombre: {nombre}"
        );
    }
}

/// Ejecución real con PowerShell: un guion .ps1 recibe cada argumento hostil intacto y no se
/// crea el testigo. Se quita -NoExit, como en la prueba de la carpeta.
#[cfg(windows)]
#[test]
fn powershell_ejecutado_pasa_los_argumentos_hostiles_intactos_sin_inyectar() {
    use std::process::{Command, Stdio};
    let temporal = CarpetaTemporal::nueva("lanzador-argumentos");
    let raiz = temporal.ruta().to_path_buf();
    let escribir = raiz.join("escribir-argumento.ps1");
    let anotado = raiz.join("argumento.txt");
    std::fs::write(
        &escribir,
        format!("[IO.File]::WriteAllText('{}', $args[0])", anotado.display()),
    )
    .unwrap();
    let entorno = EntornoDeEnsayo::default().con_archivo(escribir.clone());
    let orden = format!("New-Item -ItemType File -Path {TESTIGO}");
    for nombre in hostiles(&orden) {
        let mut p = peticion(Herramienta::Codex);
        p.carpeta_proyecto = raiz.clone();
        p.ejecutable = Some(escribir.clone());
        p.argumentos_extra = vec![nombre.clone()];
        let plan = plan(&p, Sistema::Windows, &entorno);
        let salida = Command::new(&plan.programa)
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass"])
            .args(&plan.argumentos[1..])
            .current_dir(&raiz)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let texto = std::fs::read_to_string(&anotado).unwrap_or_else(|_| {
            panic!("el guion no llegó a ejecutar el programa; nombre: {nombre}; {salida:?}")
        });
        std::fs::remove_file(&anotado).unwrap();
        assert_eq!(texto, nombre, "nombre: {nombre}");
        assert!(
            !raiz.join(TESTIGO).exists(),
            "inyección con el nombre: {nombre}"
        );
    }
}
