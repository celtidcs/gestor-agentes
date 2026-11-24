//! Migración única del estado de la cuenta por defecto de Claude Code al pasar a la 3.9.0.
//!
//! Hasta la 3.8, Gestor abría la cuenta por defecto de Claude Code con CLAUDE_CONFIG_DIR puesta en
//! su propia carpeta, y con ella Claude Code guarda su estado (proyectos, servidores MCP, ajustes de
//! la sesión) dentro de esa carpeta. Desde la 3.9.0 la abre quitando la variable, como manda la
//! biblioteca deteccion-agentes, y entonces Claude Code lee el archivo de la carpeta personal,
//! que puede ser otro y más antiguo. Por decisión del Director (2026-11-23), antes de abrir por
//! primera vez esa cuenta con la 3.9.0, Gestor guarda en su carpeta de datos una copia del archivo
//! de la carpeta personal y pone en su lugar el que usaba la cuenta. Si ese archivo no existe, no
//! toca nada.
//!
//! La marca de que ya se hizo va con lo migrado, no con Gestor: al terminar, el archivo de origen se
//! renombra ([`SUFIJO_DEL_ORIGEN_MIGRADO`]). Así otra instalación de Gestor, con otra carpeta de
//! datos, no vuelve a migrar ni pisa un estado que Claude Code ya lleva tiempo usando. La copia de
//! seguridad no se sobrescribe nunca: si una migración se corta y se repite, la copia que queda es
//! la del archivo original.
//!
//! Las rutas de los dos archivos de estado salen de la biblioteca (`Cuenta::archivo_de_configuracion`),
//! no se componen aquí. La decisión ([`decidir`]) es pura; [`migrar`] hace la entrada y salida a
//! través de [`DiscoDeLaMigracion`], que se inyecta, y [`DiscoReal`] es la del sistema.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use deteccion_agentes::{Cuenta, Herramienta, OrigenDeLaCuenta};

/// Copia del archivo de estado de la carpeta personal tal como estaba antes de la migración, en la
/// carpeta de datos de Gestor.
pub const ARCHIVO_DE_COPIA: &str = "estado-de-claude-antes-de-3.9.0.json";

/// Lo que se añade al nombre del archivo de origen al terminar la migración: el archivo renombrado
/// es la marca de que ya se hizo, en el mismo equipo y para cualquier instalación de Gestor.
pub const SUFIJO_DEL_ORIGEN_MIGRADO: &str = ".migrado-3.9.0";

/// Lo que hay en el disco y decide la migración.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // Son cuatro hechos independientes del disco, no modos.
pub struct EstadoDelDisco {
    /// Si existe el archivo de estado que usaba la cuenta con la variable puesta.
    pub existe_el_origen: bool,
    /// Si existe el origen ya renombrado: la migración está hecha.
    pub existe_el_migrado: bool,
    /// Si existe el archivo de estado que la cuenta usa sin la variable.
    pub existe_el_destino: bool,
    /// Si ya hay una copia de seguridad del destino, de un intento anterior.
    pub existe_la_copia: bool,
}

/// Lo que hay que hacer, según lo que hay en el disco.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Ya se migró, o no hay nada que migrar.
    Nada,
    /// Hay que migrar.
    Migrar {
        /// Si hay que copiar antes el destino a la carpeta de datos: existe y aún no tiene copia.
        copiar_antes: bool,
    },
}

/// Decide la migración: nada si ya está hecha o si no existe el archivo que usaba la cuenta. La
/// copia de seguridad solo se hace si hay destino y no queda ya una de un intento anterior, que
/// es la del archivo original.
#[must_use]
pub const fn decidir(estado: EstadoDelDisco) -> Decision {
    if estado.existe_el_migrado || !estado.existe_el_origen {
        return Decision::Nada;
    }
    Decision::Migrar {
        copiar_antes: estado.existe_el_destino && !estado.existe_la_copia,
    }
}

/// Las rutas que toca la migración.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RutasDeLaMigracion {
    /// Archivo de estado que usaba la cuenta con la variable puesta en su carpeta por defecto.
    pub origen: PathBuf,
    /// El origen renombrado al terminar: la marca de migración hecha.
    pub migrado: PathBuf,
    /// Archivo de estado que usa la cuenta por defecto sin la variable.
    pub destino: PathBuf,
    /// Copia del destino antes de sustituirlo, en la carpeta de datos.
    pub copia: PathBuf,
}

impl RutasDeLaMigracion {
    /// Rutas para la carpeta personal `personal` y la carpeta de datos de Gestor `datos`.
    #[must_use]
    pub fn new(personal: &Path, datos: &Path) -> Self {
        let carpeta = Herramienta::ClaudeCode.carpeta_por_defecto(personal);
        let cuenta = |origen| Cuenta {
            herramienta: Herramienta::ClaudeCode,
            carpeta: carpeta.clone(),
            origen,
        };
        // Con la variable puesta en su carpeta, la cuenta no es la por defecto para Claude Code: su
        // estado va dentro de la carpeta. Sin ella, va en la carpeta personal.
        let origen = cuenta(OrigenDeLaCuenta::Manual).archivo_de_configuracion(personal);
        let mut migrado = OsString::from(origen.as_os_str());
        migrado.push(SUFIJO_DEL_ORIGEN_MIGRADO);
        Self {
            migrado: PathBuf::from(migrado),
            origen,
            destino: cuenta(OrigenDeLaCuenta::PorDefecto).archivo_de_configuracion(personal),
            copia: datos.join(ARCHIVO_DE_COPIA),
        }
    }
}

/// Entrada y salida que necesita la migración.
pub trait DiscoDeLaMigracion {
    /// Indica si existe el archivo.
    fn existe(&self, ruta: &Path) -> bool;
    /// Copia `origen` sobre `destino`.
    ///
    /// # Errors
    ///
    /// El error del sistema.
    fn copiar(&self, origen: &Path, destino: &Path) -> io::Result<()>;
    /// Cambia el nombre de `origen` por `destino`.
    ///
    /// # Errors
    ///
    /// El error del sistema.
    fn renombrar(&self, origen: &Path, destino: &Path) -> io::Result<()>;
}

/// La entrada y salida del sistema de archivos.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiscoReal;

impl DiscoDeLaMigracion for DiscoReal {
    fn existe(&self, ruta: &Path) -> bool {
        ruta.is_file()
    }

    fn copiar(&self, origen: &Path, destino: &Path) -> io::Result<()> {
        fs::copy(origen, destino).map(|_| ())
    }

    fn renombrar(&self, origen: &Path, destino: &Path) -> io::Result<()> {
        fs::rename(origen, destino)
    }
}

/// Lo que ha hecho una migración, para avisar al Director.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigracionHecha {
    /// Las rutas que ha tocado.
    pub rutas: RutasDeLaMigracion,
    /// Si hay copia del destino anterior en la carpeta de datos, de esta vez o de un intento
    /// anterior.
    pub con_copia: bool,
}

/// Fallo de la migración, con la ruta en que falló. Si falla, el origen no se renombra y se vuelve
/// a intentar en el siguiente lanzamiento.
#[derive(Debug, thiserror::Error)]
#[error("{}: {origen}", ruta.display())]
pub struct ErrorDeMigracion {
    /// Archivo que no se pudo copiar o renombrar.
    pub ruta: PathBuf,
    /// Error del sistema.
    #[source]
    pub origen: io::Error,
}

/// Hace la migración si toca (ver [`decidir`]): copia el destino a la carpeta de datos si hace
/// falta, copia el origen sobre el destino y renombra el origen. Devuelve lo hecho, o nada si no
/// tocaba.
///
/// # Errors
///
/// [`ErrorDeMigracion`] con el archivo que falló. Si falla la copia de seguridad, no se toca el
/// destino; si falla después, la copia de seguridad se conserva para el siguiente intento.
pub fn migrar(
    rutas: RutasDeLaMigracion,
    disco: &dyn DiscoDeLaMigracion,
) -> Result<Option<MigracionHecha>, ErrorDeMigracion> {
    let existe_la_copia = disco.existe(&rutas.copia);
    let decision = decidir(EstadoDelDisco {
        existe_el_origen: disco.existe(&rutas.origen),
        existe_el_migrado: disco.existe(&rutas.migrado),
        existe_el_destino: disco.existe(&rutas.destino),
        existe_la_copia,
    });
    let Decision::Migrar { copiar_antes } = decision else {
        return Ok(None);
    };
    let fallo = |ruta: &Path| {
        let ruta = ruta.to_path_buf();
        move |origen| ErrorDeMigracion { ruta, origen }
    };
    if copiar_antes {
        disco
            .copiar(&rutas.destino, &rutas.copia)
            .map_err(fallo(&rutas.copia))?;
    }
    disco
        .copiar(&rutas.origen, &rutas.destino)
        .map_err(fallo(&rutas.destino))?;
    disco
        .renombrar(&rutas.origen, &rutas.migrado)
        .map_err(fallo(&rutas.origen))?;
    Ok(Some(MigracionHecha {
        rutas,
        con_copia: copiar_antes || existe_la_copia,
    }))
}
