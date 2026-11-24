//! Búsqueda de cuentas en la carpeta personal y en las variables de entorno.

use crate::{Cuenta, Herramienta, OrigenDeLaCuenta};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Busca las cuentas de todas las herramientas.
///
/// Para cada herramienta cuenta su carpeta por defecto si existe; cada carpeta de `personal`
/// cuyo nombre es el de la carpeta por defecto seguido de algo más (por ejemplo `.claude2` o
/// `.claude-trabajo`) y que contiene los archivos de una cuenta; y la carpeta que señale su
/// variable de entorno, leída con `leer_variable`, si existe. Una misma carpeta solo aparece
/// una vez. El orden es estable: por herramienta, la cuenta por defecto primero y después por
/// carpeta.
pub fn buscar_cuentas(
    personal: &Path,
    leer_variable: impl Fn(&str) -> Option<OsString>,
) -> Vec<Cuenta> {
    let mut cuentas = Vec::new();
    for herramienta in Herramienta::TODAS {
        let por_defecto = herramienta.carpeta_por_defecto(personal);
        if herramienta.carpeta_que_delata(personal).exists()
            || herramienta_configurada(herramienta, personal)
        {
            anadir(
                &mut cuentas,
                herramienta,
                por_defecto,
                OrigenDeLaCuenta::PorDefecto,
            );
        }
        for carpeta in carpetas_con_sufijo(herramienta, personal) {
            anadir(
                &mut cuentas,
                herramienta,
                carpeta,
                OrigenDeLaCuenta::Busqueda,
            );
        }
        if let Some(carpeta) = carpeta_de_la_variable(herramienta, &leer_variable) {
            anadir(
                &mut cuentas,
                herramienta,
                carpeta,
                OrigenDeLaCuenta::Variable,
            );
        }
    }
    ordenar(&mut cuentas);
    cuentas
}

/// Añade a `cuentas` las dadas de alta a mano que no estén ya, con origen
/// [`OrigenDeLaCuenta::Manual`]. Se conservan aunque su carpeta no exista, para que la persona
/// vea que ya no está y pueda quitarla.
///
/// Descarta las de herramientas que no admiten alta manual
/// ([`Herramienta::admite_alta_manual`]): su consola no usaría esa carpeta, así que no serían
/// cuentas de verdad. El programa no tiene que filtrarlas antes.
pub fn con_cuentas_manuales(
    mut cuentas: Vec<Cuenta>,
    manuales: &[(Herramienta, PathBuf)],
) -> Vec<Cuenta> {
    for (herramienta, carpeta) in manuales
        .iter()
        .filter(|(herramienta, _)| herramienta.admite_alta_manual())
    {
        anadir(
            &mut cuentas,
            *herramienta,
            carpeta.clone(),
            OrigenDeLaCuenta::Manual,
        );
    }
    ordenar(&mut cuentas);
    cuentas
}

/// La cuenta por defecto de Claude Code guarda su configuración fuera de su carpeta
/// (`~/.claude.json`): basta con ese archivo para que exista.
fn herramienta_configurada(herramienta: Herramienta, personal: &Path) -> bool {
    let cuenta = Cuenta {
        herramienta,
        carpeta: herramienta.carpeta_por_defecto(personal),
        origen: OrigenDeLaCuenta::PorDefecto,
    };
    cuenta.archivo_de_configuracion(personal).is_file()
}

/// Carpetas de `personal` que se llaman como la carpeta por defecto más un sufijo y contienen
/// los archivos de una cuenta de `herramienta`.
fn carpetas_con_sufijo(herramienta: Herramienta, personal: &Path) -> Vec<PathBuf> {
    let archivos = herramienta.archivos_de_cuenta();
    if archivos.is_empty() {
        return Vec::new();
    }
    let por_defecto = herramienta.carpeta_por_defecto(personal);
    let Some(prefijo) = por_defecto.file_name().and_then(|nombre| nombre.to_str()) else {
        return Vec::new();
    };
    let Ok(entradas) = std::fs::read_dir(personal) else {
        return Vec::new();
    };
    entradas
        .flatten()
        .map(|entrada| entrada.path())
        .filter(|ruta| {
            ruta.file_name()
                .and_then(|nombre| nombre.to_str())
                .is_some_and(|nombre| nombre.len() > prefijo.len() && nombre.starts_with(prefijo))
        })
        .filter(|ruta| ruta.is_dir() && archivos.iter().any(|archivo| ruta.join(archivo).is_file()))
        .collect()
}

/// Carpeta que señala la variable de cuenta de `herramienta`, si está puesta y existe.
fn carpeta_de_la_variable(
    herramienta: Herramienta,
    leer_variable: &impl Fn(&str) -> Option<OsString>,
) -> Option<PathBuf> {
    let valor = leer_variable(herramienta.variable_de_cuenta()?)?;
    let carpeta = PathBuf::from(valor);
    (!carpeta.as_os_str().is_empty() && carpeta.is_dir()).then_some(carpeta)
}

/// Añade la cuenta salvo que su carpeta ya esté para esa herramienta.
fn anadir(
    cuentas: &mut Vec<Cuenta>,
    herramienta: Herramienta,
    carpeta: PathBuf,
    origen: OrigenDeLaCuenta,
) {
    let repetida = cuentas.iter().any(|cuenta| {
        cuenta.herramienta == herramienta && misma_carpeta(&cuenta.carpeta, &carpeta)
    });
    if !repetida {
        cuentas.push(Cuenta {
            herramienta,
            carpeta,
            origen,
        });
    }
}

/// Si dos rutas son la misma carpeta: por su forma canónica cuando existen, y en todo caso sin
/// distinguir la barra final ni, en Windows, el tipo de barra ni mayúsculas de minúsculas.
pub(crate) fn misma_carpeta(una: &Path, otra: &Path) -> bool {
    forma_comparable(una) == forma_comparable(otra)
}

/// Prefijo de ruta extendida que pone `canonicalize` en Windows, ya con barras normales.
const PREFIJO_DE_RUTA_EXTENDIDA: &str = "//?/";

/// Forma de una ruta para compararla con otra (ver [`misma_carpeta`]).
fn forma_comparable(ruta: &Path) -> String {
    let canonica = ruta.canonicalize().unwrap_or_else(|_| ruta.to_path_buf());
    // En Windows la barra invertida separa carpetas; en los demás sistemas es un carácter válido
    // de un nombre, y «a\b» no es la misma carpeta que «a/b».
    let texto = if cfg!(windows) {
        canonica.to_string_lossy().replace('\\', "/")
    } else {
        canonica.to_string_lossy().into_owned()
    };
    let texto = texto
        .strip_prefix(PREFIJO_DE_RUTA_EXTENDIDA)
        .unwrap_or(&texto);
    let texto = texto.trim_end_matches('/');
    if cfg!(windows) {
        texto.to_lowercase()
    } else {
        texto.to_owned()
    }
}

/// Orden estable: herramienta, cuenta por defecto primero, después por carpeta.
fn ordenar(cuentas: &mut [Cuenta]) {
    cuentas.sort_by(|a, b| {
        (a.herramienta, !a.es_la_por_defecto(), &a.carpeta).cmp(&(
            b.herramienta,
            !b.es_la_por_defecto(),
            &b.carpeta,
        ))
    });
}
