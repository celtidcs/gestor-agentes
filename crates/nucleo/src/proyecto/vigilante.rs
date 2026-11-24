//! Instalación del vigilante del canal en el proyecto: Gestor Agentes copia el ejecutable
//! gestor-vigilante que lleva a su lado a .gestor/vigilante/, donde lo lanzan los agentes, y retira
//! lo que dejaba el vigilante anterior (los guiones vigilar-canal.ps1 y vigilar-canal.sh y sus
//! referencias canal-visto).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::archivos;
use super::informe::{InformeDeInicializacion, TipoDeCambio};
use crate::agentes::CARPETA_GESTOR;
use crate::textos;

/// Carpeta del vigilante dentro del proyecto.
pub const CARPETA_DEL_VIGILANTE: &str = ".gestor/vigilante";

/// Nombre del ejecutable sin extensión. En Windows lleva .exe ([`nombre_del_ejecutable`]).
pub const NOMBRE_DEL_VIGILANTE: &str = canal::contrato::NOMBRE_DEL_VIGILANTE;

/// Sufijo de la copia anterior mientras sigue en uso: en Windows un ejecutable en marcha no se
/// puede sobrescribir, pero sí cambiar de nombre.
const SUFIJO_DE_LA_COPIA_ANTERIOR: &str = ".anterior";

/// Guiones del vigilante hasta la 3.5.1, que ya no se usan.
pub const GUIONES_ANTERIORES: [&str; 2] = [
    ".gestor/vigilante/vigilar-canal.ps1",
    ".gestor/vigilante/vigilar-canal.sh",
];

/// Referencias que dejaban esos guiones en la carpeta de cada agente.
const REFERENCIAS_ANTERIORES: [&str; 2] = ["canal-visto.txt", "canal-visto"];

/// Nombre del ejecutable en este sistema: gestor-vigilante.exe en Windows y gestor-vigilante en
/// los demás.
#[must_use]
pub fn nombre_del_ejecutable() -> String {
    format!("{NOMBRE_DEL_VIGILANTE}{}", std::env::consts::EXE_SUFFIX)
}

/// Ruta relativa del ejecutable dentro del proyecto.
#[must_use]
pub fn ruta_relativa() -> String {
    format!("{CARPETA_DEL_VIGILANTE}/{}", nombre_del_ejecutable())
}

/// Copia el ejecutable `origen` al proyecto si falta o es distinto. Si `origen` no existe no copia
/// nada y avisa: los agentes se quedarían sin vigilante.
///
/// # Errors
///
/// El error de disco al leer el origen o el destino, o al copiar.
pub fn instalar(
    raiz: &Path,
    origen: &Path,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let relativa = ruta_relativa();
    if !origen.is_file() {
        informe.anotar_aviso(textos::vigilante_sin_ejecutable(&relativa));
        return Ok(());
    }
    let destino = archivos::ruta(raiz, &relativa);
    let nuevo = fs::read(origen)?;
    let actual = match fs::read(&destino) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if actual.as_deref() == Some(nuevo.as_slice()) {
        return Ok(());
    }
    if let Some(carpeta) = destino.parent() {
        fs::create_dir_all(carpeta)?;
    }
    if actual.is_some() {
        apartar_la_copia_en_uso(&destino)?;
    }
    fs::write(&destino, &nuevo)?;
    hacer_ejecutable(&destino)?;
    let tipo = if actual.is_some() {
        TipoDeCambio::Actualizado
    } else {
        TipoDeCambio::Creado
    };
    informe.anotar_cambio(&relativa, tipo, None);
    Ok(())
}

/// Cambia de nombre la copia que hay, que puede estar en marcha, tras borrar la apartada de una
/// vez anterior si ya nadie la usa.
fn apartar_la_copia_en_uso(destino: &Path) -> io::Result<()> {
    let mut apartada = destino.as_os_str().to_owned();
    apartada.push(SUFIJO_DE_LA_COPIA_ANTERIOR);
    let apartada = PathBuf::from(apartada);
    if apartada.exists() && fs::remove_file(&apartada).is_err() {
        // La copia apartada sigue en marcha: se sobrescribe directamente la actual si se puede.
        return Ok(());
    }
    fs::rename(destino, apartada)
}

/// Da permiso de ejecución en los sistemas que lo necesitan.
#[cfg(unix)]
fn hacer_ejecutable(ruta: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    // rwxr-xr-x: lo ejecuta cualquier agente del usuario y solo él lo cambia.
    const PERMISOS_DE_EJECUTABLE: u32 = 0o755;
    fs::set_permissions(ruta, fs::Permissions::from_mode(PERMISOS_DE_EJECUTABLE))
}

/// En Windows basta la extensión .exe.
#[cfg(not(unix))]
fn hacer_ejecutable(_ruta: &Path) -> io::Result<()> {
    Ok(())
}

/// Borra los guiones del vigilante anterior, sus referencias en la carpeta de cada agente y la
/// copia apartada que ya no esté en uso. Devuelve las rutas relativas de los guiones borrados, para
/// que se olviden en el registro de huellas.
///
/// # Errors
///
/// El error de disco al leer .gestor/ o al borrar.
pub fn retirar_lo_anterior(
    raiz: &Path,
    informe: &mut InformeDeInicializacion,
) -> io::Result<Vec<String>> {
    let mut borrados = Vec::new();
    for guion in GUIONES_ANTERIORES {
        let ruta = archivos::ruta(raiz, guion);
        if ruta.is_file() {
            fs::remove_file(&ruta)?;
            informe.anotar_cambio(guion, TipoDeCambio::Borrado, None);
            borrados.push(guion.to_owned());
        }
    }
    let apartada = archivos::ruta(
        raiz,
        &format!("{}{SUFIJO_DE_LA_COPIA_ANTERIOR}", ruta_relativa()),
    );
    if apartada.is_file() {
        // Si sigue en marcha no se puede borrar todavía; se intenta en la próxima preparación.
        let _ = fs::remove_file(apartada);
    }
    let gestor = archivos::ruta(raiz, CARPETA_GESTOR);
    let carpetas = match fs::read_dir(&gestor) {
        Ok(carpetas) => carpetas,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(borrados),
        Err(error) => return Err(error),
    };
    for carpeta in carpetas {
        let carpeta = carpeta?.path();
        if !carpeta.is_dir() {
            continue;
        }
        for referencia in REFERENCIAS_ANTERIORES {
            let ruta = carpeta.join(referencia);
            if ruta.is_dir() {
                fs::remove_dir_all(&ruta)?;
            } else if ruta.is_file() {
                fs::remove_file(&ruta)?;
            } else {
                continue;
            }
            let relativa = format!(
                "{CARPETA_GESTOR}/{}/{referencia}",
                carpeta.file_name().unwrap_or_default().to_string_lossy()
            );
            informe.anotar_cambio(&relativa, TipoDeCambio::Borrado, None);
        }
    }
    Ok(borrados)
}
