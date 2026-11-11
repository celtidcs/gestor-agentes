//! Puesta al día de los perfiles guardados con un esquema anterior, sin tocar el disco.

use serde::Deserialize;

use super::perfiles::{HerramientaDelPerfil, PerfilDeAgente};

/// Versión del esquema de perfiles de la 3.0.0. La 2 añadió Gemini; la 3, Programator; la 4
/// quita el campo Modo, porque todos los agentes se abren en terminal.
pub(super) const VERSION_PERFILES: u32 = 4;

/// Primera versión del esquema sin el campo Modo. Los archivos anteriores pueden traer el mismo
/// agente dos veces, una para su aplicación de escritorio y otra en terminal, y al migrar se
/// funden en un solo perfil.
const VERSION_SIN_MODO: u32 = 4;

/// Valor del campo Modo, en los archivos anteriores a [`VERSION_SIN_MODO`], de un perfil que se
/// abría en terminal.
const MODO_TERMINAL_ANTIGUO: &str = "Terminal";

/// Versión que se supone si el archivo de versión no existe o no se entiende.
pub(super) const VERSION_SIN_ARCHIVO: u32 = 1;

/// Perfil tal como puede venir del archivo, con el campo Modo de los esquemas anteriores al 4,
/// que solo se lee para fundir perfiles al migrar y ya no se escribe.
#[derive(Deserialize)]
pub(super) struct PerfilGuardado {
    #[serde(flatten)]
    perfil: PerfilDeAgente,
    #[serde(rename = "Modo", default)]
    modo: Option<String>,
}

impl PerfilGuardado {
    /// Indica si el perfil se abría en terminal.
    fn en_terminal(&self) -> bool {
        self.modo.as_deref() == Some(MODO_TERMINAL_ANTIGUO)
    }

    /// El perfil, listo para abrirse en terminal. En Gemini, el ejecutable fijado de un perfil que
    /// no era de terminal es el de Antigravity, la aplicación de escritorio, que ya no se abre:
    /// se quita, para que se busque agy. En Codex se conserva, porque el programa que abría la
    /// aplicación de escritorio, con `codex app`, es el mismo codex.
    fn para_terminal(self) -> PerfilDeAgente {
        let en_terminal = self.en_terminal();
        let mut perfil = self.perfil;
        if !en_terminal && perfil.tipo() == HerramientaDelPerfil::Gemini {
            perfil.quitar_ejecutable();
        }
        perfil
    }
}

/// Indica si un archivo de perfiles con esta versión ya tiene el esquema vigente.
pub(super) const fn esta_al_dia(version: u32) -> bool {
    version >= VERSION_PERFILES
}

/// Perfiles leídos de un archivo con la versión `version`, llevados al esquema vigente: antes
/// del 4 se funden los que solo se distinguían por su campo Modo (ver [`fundir_modos`]), y se
/// añaden una sola vez las herramientas que trajo cada esquema posterior, si faltan.
pub(super) fn poner_al_dia(guardados: Vec<PerfilGuardado>, version: u32) -> Vec<PerfilDeAgente> {
    let mut perfiles = if version < VERSION_SIN_MODO {
        fundir_modos(guardados)
    } else {
        guardados
            .into_iter()
            .map(|guardado| guardado.perfil)
            .collect()
    };
    if !esta_al_dia(version) {
        anadir_perfiles_nuevos(&mut perfiles, version);
    }
    perfiles
}

/// Funde los perfiles de un archivo anterior al esquema 4 que eran el mismo agente abierto de
/// dos formas, en su aplicación de escritorio y en terminal. Queda uno, en el sitio del primero,
/// con los datos del de terminal, que es el que se usará; la carpeta de configuración, si el de
/// terminal no la traía, se toma del otro, porque es la de la cuenta. Dos perfiles del mismo
/// agente y la misma forma no se funden: siguen repetidos y la validación lo dice.
fn fundir_modos(guardados: Vec<PerfilGuardado>) -> Vec<PerfilDeAgente> {
    let mut fundidos: Vec<(PerfilDeAgente, bool)> = Vec::new();
    for guardado in guardados {
        let en_terminal = guardado.en_terminal();
        let perfil = guardado.para_terminal();
        let pareja = fundidos.iter_mut().find(|(otro, otro_en_terminal)| {
            otro.nombre() == perfil.nombre() && *otro_en_terminal != en_terminal
        });
        let Some((existente, existente_en_terminal)) = pareja else {
            fundidos.push((perfil, en_terminal));
            continue;
        };
        let (terminal, escritorio) = if en_terminal {
            (perfil, existente.clone())
        } else {
            (existente.clone(), perfil)
        };
        *existente = terminal.con_directorio_de(escritorio);
        *existente_en_terminal = true;
    }
    fundidos.into_iter().map(|(perfil, _)| perfil).collect()
}

/// Añade las herramientas que trajo cada esquema posterior a `version`, si faltan.
fn anadir_perfiles_nuevos(perfiles: &mut Vec<PerfilDeAgente>, version: u32) {
    let nuevas = [
        (2, HerramientaDelPerfil::Gemini),
        (3, HerramientaDelPerfil::Programator),
    ];
    for (desde, herramienta) in nuevas {
        if version < desde && !perfiles.iter().any(|p| p.tipo() == herramienta) {
            perfiles.push(PerfilDeAgente::new(herramienta, 1));
        }
    }
}
