//! Guiones y órdenes de la terminal en la que se abren Claude, codex, agy y, en Linux,
//! Programator.
//!
//! Traducción de las terminales de PowerShell y de Linux del lanzador de la 2.3.2. Sin
//! argumentos, el texto que se genera es el mismo, carácter
//! a carácter; cada argumento se añade tras el programa, escapado igual que las demás cadenas.

use std::ffi::OsString;

use crate::base64::orden_codificada_powershell;
use crate::escapes::{comilla_simple_bash, comilla_simple_powershell};

/// Programa de la terminal de Windows.
pub(crate) const POWERSHELL: &str = "powershell.exe";

/// Cada cuántos milisegundos restaura el guion de PowerShell el título de la ventana. Las
/// interfaces de terminal de los agentes lo cambian al arrancar y mientras trabajan; un segundo es
/// lo que usaba la 2.3.2, lo bastante corto para que el
/// título vuelva enseguida y lo bastante largo para no cargar la consola.
const INTERVALO_DE_RESTAURACION_DEL_TITULO_MS: u32 = 1000;

/// Emuladores de terminal de Linux, por orden de preferencia: GNOME, KDE, XFCE y X11 genérico.
/// xterm es el último recurso, porque está en casi cualquier sistema con X11.
pub(crate) const TERMINALES_LINUX: [TerminalLinux; 4] = [
    TerminalLinux::Gnome,
    TerminalLinux::Konsole,
    TerminalLinux::Xfce,
    TerminalLinux::Xterm,
];

/// Emulador de terminal de Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalLinux {
    /// gnome-terminal.
    Gnome,
    /// konsole.
    Konsole,
    /// xfce4-terminal.
    Xfce,
    /// xterm.
    Xterm,
}

impl TerminalLinux {
    /// Nombre del programa.
    pub(crate) const fn programa(self) -> &'static str {
        match self {
            Self::Gnome => "gnome-terminal",
            Self::Konsole => "konsole",
            Self::Xfce => "xfce4-terminal",
            Self::Xterm => "xterm",
        }
    }

    /// Argumentos que abren `guion` con bash en una ventana titulada `titulo`.
    ///
    /// Cada dato va en su propio argumento. En xfce4-terminal se usa -x y no -e, porque -e vuelve
    /// a interpretar su texto como una orden.
    pub(crate) fn argumentos(self, titulo: &str, guion: &str) -> Vec<OsString> {
        let textos: Vec<String> = match self {
            Self::Gnome => vec![format!("--title={titulo}"), "--".into()],
            Self::Konsole => vec!["--title".into(), titulo.into(), "-e".into()],
            Self::Xfce => vec![format!("--title={titulo}"), "-x".into()],
            Self::Xterm => vec!["-T".into(), titulo.into(), "-e".into()],
        };
        textos
            .into_iter()
            .chain(["bash".into(), "-c".into(), guion.into()])
            .map(OsString::from)
            .collect()
    }
}

/// Variable de entorno que se fija dentro del guion de la terminal.
pub(crate) struct Asignacion<'a> {
    /// Nombre de la variable; lo fija el programa, nunca el usuario.
    pub(crate) nombre: &'a str,
    /// Valor, que se escapa.
    pub(crate) valor: &'a str,
}

/// Guion de PowerShell que fija el título (y lo restaura cada segundo con un temporizador, porque
/// las interfaces de terminal de los agentes lo cambian), asigna las variables, entra en la
/// carpeta y ejecuta el programa con sus argumentos.
pub(crate) fn guion_powershell(
    titulo: &str,
    variables: &[Asignacion<'_>],
    carpeta: &str,
    ejecutable: &str,
    argumentos: &[&str],
) -> String {
    let mut partes = vec![
        format!("$t = '{}'", comilla_simple_powershell(titulo)),
        "[Console]::Title = $t".to_owned(),
        "$cb = [System.Threading.TimerCallback]{param($s) [Console]::Title = $s}".to_owned(),
        format!(
            "$tmr = New-Object System.Threading.Timer($cb, $t, 0, {INTERVALO_DE_RESTAURACION_DEL_TITULO_MS})"
        ),
    ];
    partes.extend(variables.iter().map(|v| {
        format!(
            "$env:{} = '{}'",
            v.nombre,
            comilla_simple_powershell(v.valor)
        )
    }));
    partes.push(format!(
        "Set-Location -LiteralPath '{}'",
        comilla_simple_powershell(carpeta)
    ));
    let mut llamada = format!("& '{}'", comilla_simple_powershell(ejecutable));
    for argumento in argumentos {
        anadir_entre_comillas(&mut llamada, " ", &comilla_simple_powershell(argumento), "");
    }
    partes.push(llamada);
    partes.push("$tmr.Dispose()".to_owned());
    partes.push("[Console]::Title = $t".to_owned());
    partes.join("; ")
}

/// Argumentos de powershell.exe que ejecutan `guion` sin cerrar la ventana al terminar.
pub(crate) fn argumentos_powershell(guion: &str) -> Vec<OsString> {
    vec![
        OsString::from("-NoExit"),
        OsString::from("-EncodedCommand"),
        OsString::from(orden_codificada_powershell(guion)),
    ]
}

/// Guion de bash que exporta las variables, entra en la carpeta, ejecuta el programa con sus
/// argumentos y, al terminar, deja abierta la ventana con `exec bash` (como -NoExit en
/// PowerShell).
pub(crate) fn guion_bash(
    variables: &[Asignacion<'_>],
    carpeta: &str,
    ejecutable: &str,
    argumentos: &[&str],
) -> String {
    let mut guion: String = variables
        .iter()
        .map(|v| format!("export {}='{}'; ", v.nombre, comilla_simple_bash(v.valor)))
        .collect();
    anadir_entre_comillas(&mut guion, "cd ", &comilla_simple_bash(carpeta), "; ");
    anadir_entre_comillas(&mut guion, "", &comilla_simple_bash(ejecutable), "");
    for argumento in argumentos {
        anadir_entre_comillas(&mut guion, " ", &comilla_simple_bash(argumento), "");
    }
    guion.push_str("; ");
    guion.push_str("exec bash");
    guion
}

/// Añade a `destino` el texto `antes`, después `escapado` entre comillas simples y por último
/// `despues`. `escapado` ya tiene que venir escapado para el intérprete al que va el guion.
fn anadir_entre_comillas(destino: &mut String, antes: &str, escapado: &str, despues: &str) {
    destino.push_str(antes);
    destino.push('\'');
    destino.push_str(escapado);
    destino.push('\'');
    destino.push_str(despues);
}
