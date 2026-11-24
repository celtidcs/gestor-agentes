//! Prepara un proyecto cada vez que la aplicación lanza un agente en él.
//!
//! Pasos, en este orden:
//!
//! 1. Repositorio git y exclusión de .gestor/ en .gitignore.
//! 2. Si falta el archivo propio del proyecto (.gestor/instrucciones-del-proyecto.md), lectura del
//!    texto propio que haya en los archivos de instrucciones anteriores. Va antes del paso 3
//!    porque ese paso se lleva GEMINI.md.
//! 3. Archivo de los archivos del sistema anterior (canal antiguo de Gemini, GEMINI.md, memoria
//!    compartida y reglas de agy en .agents/).
//! 4. Estructura de .gestor/ (canal, bandejas, diarios, actas, registros y órdenes).
//!    Además, se añaden a estado.md los apartados obligatorios que le falten ([`papeles`]).
//! 5. Archivo propio del proyecto: se crea si falta, con lo recogido en el paso 2 y la lista de
//!    lo archivado en el paso 3. Si ya existe no se toca nunca.
//!    Seguimiento del bloque de legado: cuando un agente lo ha consolidado, se borran las carpetas
//!    del archivo que dejó la aplicación.
//! 6. Archivos que la aplicación escribe enteros: AGENTS.md, CLAUDE.md y los documentos de
//!    consulta, con copia previa de lo que no hubiera escrito ella.
//!    Vigilante del canal: copia del ejecutable gestor-vigilante a .gestor/vigilante/ y
//!    retirada de los guiones y referencias del vigilante anterior ([`vigilante`]).
//! 7. Permisos del arranque en el archivo de permisos del proyecto de cada herramienta
//!    ([`permisos`]), sin quitar nada de lo que ya tenga.
//! 8. Aviso del código del proyecto que cita AGENTS.md, CLAUDE.md o GEMINI.md
//!    ([`citas_de_reglas`]).
//!    Aviso de las bandejas del canal que han crecido demasiado y de lo que no cuadra en el estado
//!    del canal (apartados, mutaciones vivas y reservas), con [`revision_del_canal`].
//!    Además, aviso de los diarios y del estado del canal que pasan del mismo límite.
//! 9. Registro de huellas de lo escrito.
//! 10. Avisos de esta preparación en .gestor/avisos-de-preparacion.md, para los agentes, o borrado
//!     de ese archivo si no queda ninguno ([`avisos_de_preparacion`]).
//!
//! Un paso que falla se anota en el informe y no impide los siguientes. Nada se sustituye en
//! silencio: todo cambio queda en el informe y todo texto desplazado, en un respaldo o en el
//! archivo de .gestor/.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::archivos;
use super::avisos_de_preparacion;
use super::catalogo::{self, ArchivoGestionado};
use super::citas_de_reglas;
use super::estructura;
use super::informe::{InformeDeInicializacion, TipoDeCambio};
use super::localizador::FuenteDeEntorno;
use super::momento::{Momento, Reloj};
use super::papeles;
use super::permisos::{self, Fusion};
use super::registro_de_huellas::{self, RegistroDeHuellas};
use super::revision_del_canal;
use super::sistema_anterior;
use super::vigilante;
use crate::agentes::{IdentidadAgente, TipoAgente, CARPETA_CANAL, CARPETA_GESTOR};
use crate::aprendizaje;
use crate::git::{self, ServicioGit};
use crate::instrucciones::formato::FormatoDeTexto;
use crate::instrucciones::huella;
use crate::instrucciones::marcas;
use crate::instrucciones::migracion::{self, Archivado, ArchivoAnterior, TextoPropio};
use crate::instrucciones::sustitucion::{self, Sustitucion};
use crate::plantillas;
use crate::textos;

/// Marcador de la fecha en el encabezado del bloque pendiente de consolidar.
const MARCADOR_FECHA: &str = "{{FECHA}}";

pub use canal::contrato::es_bandeja;

/// Ruta de estado.md, relativa al proyecto.
fn ruta_del_estado() -> String {
    canal::contrato::ruta_relativa_del_estado()
}

/// Configuración del usuario que necesita la inicialización.
pub trait ConfiguracionDeInstrucciones {
    /// Plantilla de CLAUDE.md que el usuario guardó con la versión anterior de la aplicación, si
    /// existe. Una implementación que no puede leerla devuelve nada.
    fn leer_instrucciones_antiguas(&self) -> Option<String>;

    /// Tamaño en bytes a partir del cual se avisa de que una bandeja del canal está sin podar. Por
    /// defecto, el de [`crate::ajustes::LIMITE_DE_BANDEJA_POR_DEFECTO`].
    fn limite_de_bandeja(&self) -> u64 {
        crate::ajustes::LIMITE_DE_BANDEJA_POR_DEFECTO
    }

    /// Horas sin latido tras las que una reserva se da por huérfana. Por defecto, las de
    /// [`crate::ajustes::LIMITE_DE_LATIDO_POR_DEFECTO`].
    fn limite_de_latido_horas(&self) -> u64 {
        crate::ajustes::LIMITE_DE_LATIDO_POR_DEFECTO
    }

    /// Ejecutable gestor-vigilante que se copia a cada proyecto ([`vigilante`]). Sin él no se
    /// copia nada; la aplicación da el que lleva a su lado, y avisa si no está.
    fn ejecutable_del_vigilante(&self) -> Option<PathBuf> {
        None
    }

    /// Carpeta de datos de la aplicación, donde se juntan las reglas candidatas a aprendizaje común
    /// de todos los proyectos ([`crate::aprendizaje`]). Sin ella no se recogen.
    fn carpeta_de_datos(&self) -> Option<PathBuf> {
        None
    }
}

/// Inicializador con sus dependencias inyectadas.
pub struct InicializadorDeProyecto<'a> {
    configuracion: &'a dyn ConfiguracionDeInstrucciones,
    fuente_de_entorno: &'a dyn FuenteDeEntorno,
    git: ServicioGit<'a>,
    reloj: &'a dyn Reloj,
}

impl<'a> InicializadorDeProyecto<'a> {
    /// Crea el inicializador.
    #[must_use]
    pub const fn new(
        configuracion: &'a dyn ConfiguracionDeInstrucciones,
        fuente_de_entorno: &'a dyn FuenteDeEntorno,
        git: ServicioGit<'a>,
        reloj: &'a dyn Reloj,
    ) -> Self {
        Self {
            configuracion,
            fuente_de_entorno,
            git,
            reloj,
        }
    }

    /// Prepara el proyecto para la identidad que se va a lanzar, o solo actualiza sus archivos si
    /// no se lanza ninguna. Además de la lanzada, siempre se aseguran los archivos de la primera
    /// instancia de cada herramienta. Devuelve el informe de lo hecho.
    #[must_use]
    pub fn inicializar(
        &self,
        raiz: &Path,
        lanzada: Option<IdentidadAgente>,
    ) -> InformeDeInicializacion {
        let mut informe = InformeDeInicializacion::new();
        if !raiz.is_dir() {
            informe.anotar_error(textos::carpeta_inexistente(&raiz.display().to_string()));
            return informe;
        }

        let momento = self.reloj.ahora();
        let mut identidades: Vec<IdentidadAgente> = TipoAgente::TODOS
            .iter()
            .map(|tipo| IdentidadAgente::primera(*tipo))
            .collect();
        identidades.extend(lanzada);

        self.asegurar_git(raiz, &mut informe);
        let mut registro = RegistroDeHuellas::cargar(raiz, &mut informe);
        self.migrar_y_archivar(raiz, &identidades, momento, &mut registro, &mut informe);
        self.recoger_aprendizaje(raiz, momento, &mut informe);
        self.actualizar_archivos_gestionados(raiz, momento, &mut registro, &mut informe);
        self.revisar_y_avisar(raiz, momento, &mut informe);
        guardar_registros(raiz, &registro, momento, &mut informe);
        informe
    }

    /// Repositorio git y .gitignore con .gestor/.
    fn asegurar_git(&self, raiz: &Path, informe: &mut InformeDeInicializacion) {
        self.git.asegurar_repositorio(raiz, informe);
        paso(informe, git::ARCHIVO_IGNORADOS, |informe| {
            git::asegurar_ignorado(raiz, informe)
        });
    }

    /// Archiva el sistema de trabajo anterior, asegura la estructura de .gestor/ y estado.md, y crea
    /// las instrucciones propias del proyecto con el texto propio recogido si faltan.
    fn migrar_y_archivar(
        &self,
        raiz: &Path,
        identidades: &[IdentidadAgente],
        momento: Momento,
        registro: &mut RegistroDeHuellas,
        informe: &mut InformeDeInicializacion,
    ) {
        let ruta_propio = archivos::ruta(raiz, catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO);
        let falta_el_propio = !ruta_propio.is_file();
        let propio = if falta_el_propio {
            self.recoger_texto_propio(raiz, informe)
        } else {
            TextoPropio::default()
        };

        let archivados = con_valor(informe, ".gestor/archivo/", |informe| {
            sistema_anterior::archivar(raiz, momento, informe)
        })
        .unwrap_or_default();
        for archivado in &archivados {
            registro.olvidar(&archivado.origen);
        }

        paso(informe, ".gestor/", |informe| {
            estructura::asegurar(raiz, identidades, informe)
        });
        paso(informe, &ruta_del_estado(), |informe| {
            papeles::completar_en_disco(raiz, informe)
        });

        if falta_el_propio {
            paso(
                informe,
                catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO,
                |informe| crear_propio(raiz, &propio, &archivados, momento, informe),
            );
        } else if !archivados.is_empty() {
            informe.anotar_aviso(aviso_de_archivados_sin_propio(&archivados));
        }
        let archivo = if archivados.is_empty() {
            ArchivoDelSistemaAnterior::SinNovedades
        } else {
            ArchivoDelSistemaAnterior::RecienArchivado
        };
        paso(
            informe,
            sistema_anterior::CARPETA_ARCHIVO_SISTEMA_ANTERIOR,
            |informe| seguir_el_legado(raiz, registro, archivo, informe),
        );
    }

    /// Junta las reglas candidatas a aprendizaje común en la carpeta de datos, si la hay.
    fn recoger_aprendizaje(
        &self,
        raiz: &Path,
        momento: Momento,
        informe: &mut InformeDeInicializacion,
    ) {
        if let Some(datos) = self.configuracion.carpeta_de_datos() {
            paso(informe, aprendizaje::ARCHIVO_DE_CANDIDATAS, |informe| {
                recoger_candidatas(raiz, &datos, momento, informe)
            });
        }
    }

    /// Archivos que mantiene la aplicación: instrucciones, vigilante y permisos de Claude Code.
    fn actualizar_archivos_gestionados(
        &self,
        raiz: &Path,
        momento: Momento,
        registro: &mut RegistroDeHuellas,
        informe: &mut InformeDeInicializacion,
    ) {
        let entorno = self.fuente_de_entorno.buscar(raiz);
        for archivo in catalogo::archivos_gestionados(entorno.as_ref()) {
            paso(informe, archivo.ruta_relativa, |informe| {
                mantener_entero(raiz, &archivo, registro, momento, informe)
            });
        }

        if let Some(origen) = self.configuracion.ejecutable_del_vigilante() {
            paso(informe, vigilante::CARPETA_DEL_VIGILANTE, |informe| {
                vigilante::instalar(raiz, &origen, informe)
            });
        }
        let retirados = con_valor(informe, vigilante::CARPETA_DEL_VIGILANTE, |informe| {
            vigilante::retirar_lo_anterior(raiz, informe)
        })
        .unwrap_or_default();
        for retirado in &retirados {
            registro.olvidar(retirado);
        }

        paso(informe, permisos::ARCHIVO_DE_CLAUDE, |informe| {
            mantener_permisos(
                raiz,
                permisos::ARCHIVO_DE_CLAUDE,
                &permisos::REGLAS_DE_CLAUDE,
                informe,
            )
        });
    }

    /// Avisos de lo que conviene revisar: código que cita las reglas, bandejas, diarios y estado
    /// grandes, y lo que no cuadra en el canal.
    fn revisar_y_avisar(
        &self,
        raiz: &Path,
        momento: Momento,
        informe: &mut InformeDeInicializacion,
    ) {
        paso(informe, ".", |informe| {
            let citas = citas_de_reglas::buscar(raiz)?;
            if !citas.is_empty() {
                informe.anotar_aviso(textos::codigo_que_cita_las_reglas(&citas));
            }
            Ok(())
        });

        let limite_de_bandeja = self.configuracion.limite_de_bandeja();
        paso(informe, CARPETA_CANAL, |informe| {
            avisar_bandejas_grandes(raiz, limite_de_bandeja, informe)
        });
        paso(informe, CARPETA_GESTOR, |informe| {
            avisar_diarios_y_estado_grandes(raiz, limite_de_bandeja, informe)
        });
        let limite_de_latido = self.configuracion.limite_de_latido_horas();
        paso(informe, &ruta_del_estado(), |informe| {
            avisar_del_canal(raiz, momento, limite_de_latido, informe)
        });
    }

    /// Lee los archivos de instrucciones anteriores que haya y recoge su texto propio. Un archivo
    /// que no se puede leer se anota como error y no aporta nada.
    fn recoger_texto_propio(
        &self,
        raiz: &Path,
        informe: &mut InformeDeInicializacion,
    ) -> TextoPropio {
        let mut rutas: Vec<String> = catalogo::ARCHIVOS_DE_INSTRUCCIONES_ANTERIORES
            .iter()
            .map(|ruta| (*ruta).to_owned())
            .collect();
        match sistema_anterior::reglas_de_agy(raiz) {
            Ok(reglas) => rutas.extend(reglas),
            Err(error) => informe.anotar_error(format!(".agents/: {error}")),
        }
        let mut leidos: Vec<(String, String)> = Vec::new();
        for ruta_relativa in rutas {
            match archivos::leer(&archivos::ruta(raiz, &ruta_relativa)) {
                Ok(Some((texto, _))) => leidos.push((ruta_relativa, texto)),
                Ok(None) => {}
                Err(error) => informe.anotar_error(format!("{ruta_relativa}: {error}")),
            }
        }
        let anteriores: Vec<ArchivoAnterior<'_>> = leidos
            .iter()
            .map(|(ruta_relativa, texto)| ArchivoAnterior {
                ruta_relativa: ruta_relativa.as_str(),
                texto,
            })
            .collect();
        let antiguas = self.configuracion.leer_instrucciones_antiguas();
        let propio = migracion::recoger(
            &anteriores,
            &catalogo::textos_de_la_aplicacion(antiguas.as_deref()),
        );
        for danado in &propio.archivos_con_marcas_danadas {
            informe.anotar_aviso(textos::marcas_danadas(
                danado,
                catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO,
            ));
        }
        propio
    }
}

/// Crea el archivo propio del proyecto con lo recogido y lo archivado.
fn crear_propio(
    raiz: &Path,
    propio: &TextoPropio,
    archivados: &[Archivado],
    momento: Momento,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let encabezado =
        plantillas::encabezado_pendiente().replace(MARCADOR_FECHA, &momento.fecha_iso());
    let texto = migracion::componer_instrucciones_del_proyecto(
        &plantillas::instrucciones_del_proyecto(),
        &encabezado,
        &plantillas::aviso_archivados(),
        propio,
        archivados,
    );
    let ruta = archivos::ruta(raiz, catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO);
    if !archivos::crear_si_no_existe(&ruta, &texto)? {
        return Ok(());
    }
    let detalle = (!propio.esta_vacio() || !archivados.is_empty())
        .then(|| String::from(textos::DETALLE_PROPIO_CON_PENDIENTE));
    informe.anotar_cambio(
        catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO,
        TipoDeCambio::Creado,
        detalle,
    );
    Ok(())
}

/// Añade a candidatas-comunes.md, en la carpeta de datos, las reglas aprendidas del proyecto
/// marcadas como candidatas a común que todavía no estén.
fn recoger_candidatas(
    raiz: &Path,
    datos: &Path,
    momento: Momento,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let Some((propio, _)) = archivos::leer(&archivos::ruta(
        raiz,
        catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO,
    ))?
    else {
        return Ok(());
    };
    let candidatas = aprendizaje::candidatas(&propio);
    if candidatas.is_empty() {
        return Ok(());
    }
    let destino = datos.join(aprendizaje::ARCHIVO_DE_CANDIDATAS);
    let actual = archivos::leer(&destino)?;
    let proyecto = raiz
        .file_name()
        .map_or_else(|| raiz.to_string_lossy(), |nombre| nombre.to_string_lossy());
    if let Some(texto) = aprendizaje::anadir_candidatas(
        actual.as_ref().map(|(texto, _)| texto.as_str()),
        &candidatas,
        &proyecto,
        &momento.fecha_iso(),
    ) {
        let formato = actual.map(|(_, formato)| formato).unwrap_or_default();
        archivos::escribir(&destino, &texto, formato)?;
        informe.anotar_aviso(textos::candidatas_recogidas(&destino.to_string_lossy()));
    }
    Ok(())
}

/// Clave del registro de huellas que dice que la aplicación dejó un bloque de legado pendiente de
/// consolidar, con archivos llevados a .gestor/archivo/.
const CLAVE_LEGADO_PENDIENTE: &str = ".gestor/archivo/ (legado pendiente)";

/// Carpetas del archivo que deja la aplicación y que se borran al consolidar el legado (apartado 0
/// de consulta/metodo.md).
const CARPETAS_DEL_LEGADO: [&str; 2] = [
    sistema_anterior::CARPETA_ARCHIVO_SISTEMA_ANTERIOR,
    sistema_anterior::CARPETA_ARCHIVO_GEMINI,
];

/// Si la preparación en curso acaba de archivar algo del sistema de trabajo anterior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchivoDelSistemaAnterior {
    /// No ha archivado nada.
    SinNovedades,
    /// Acaba de archivar algo: no se borra nada en la misma preparación.
    RecienArchivado,
}

/// Lleva la cuenta del bloque de legado. Mientras el archivo propio tenga el bloque, lo anota en el
/// registro. Cuando un agente lo ha consolidado (el bloque ya no está) y la aplicación lo había
/// anotado, borra las carpetas del archivo que dejó ella, como pide el método, porque el permiso
/// de Claude Code no deja borrarlas al agente. Nunca borra nada que no haya dejado anotado, ni en
/// la misma preparación que archiva algo.
fn seguir_el_legado(
    raiz: &Path,
    registro: &mut RegistroDeHuellas,
    archivo: ArchivoDelSistemaAnterior,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let ruta = archivos::ruta(raiz, catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO);
    let Some((propio, _)) = archivos::leer(&ruta)? else {
        return Ok(());
    };
    let con_bloque = propio
        .lines()
        .any(|linea| marcas::es_inicio(linea, marcas::LEGADO));
    if con_bloque {
        registro.registrar_archivo(CLAVE_LEGADO_PENDIENTE, "pendiente");
        return Ok(());
    }
    if archivo == ArchivoDelSistemaAnterior::RecienArchivado
        || registro
            .huella_del_archivo(CLAVE_LEGADO_PENDIENTE)
            .is_none()
    {
        return Ok(());
    }
    for carpeta in CARPETAS_DEL_LEGADO {
        let ruta = archivos::ruta(raiz, carpeta);
        if ruta.is_dir() {
            fs::remove_dir_all(&ruta)?;
            informe.anotar_cambio(carpeta, TipoDeCambio::Borrado, None);
        }
    }
    registro.olvidar(CLAVE_LEGADO_PENDIENTE);
    Ok(())
}

/// Aviso cuando se archiva algo del sistema anterior en un proyecto que ya tenía su archivo
/// propio, que no se toca.
fn aviso_de_archivados_sin_propio(archivados: &[Archivado]) -> String {
    let pares: Vec<(&str, &str)> = archivados
        .iter()
        .map(|a| (a.origen.as_str(), a.destino.as_str()))
        .collect();
    textos::archivados_sin_propio(&pares, catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO)
}

/// Deja un archivo gestionado con su texto vigente, guardando antes una copia exacta si lo que
/// tenía no lo había escrito la aplicación.
fn mantener_entero(
    raiz: &Path,
    archivo: &ArchivoGestionado,
    registro: &mut RegistroDeHuellas,
    momento: Momento,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let ruta = archivos::ruta(raiz, archivo.ruta_relativa);
    let leido = archivos::leer(&ruta)?;
    let decision = sustitucion::decidir(
        leido.as_ref().map(|(texto, _)| texto.as_str()),
        &archivo.texto,
        registro
            .huella_del_archivo(archivo.ruta_relativa)
            .as_deref(),
    );

    match decision {
        Sustitucion::SinCambios => {}
        Sustitucion::Crear => {
            archivos::escribir(&ruta, &archivo.texto, FormatoDeTexto::default())?;
            informe.anotar_cambio(archivo.ruta_relativa, TipoDeCambio::Creado, None);
        }
        Sustitucion::Sustituir { respaldar } => {
            let detalle = if respaldar {
                let ruta_respaldo = archivos::ruta_de_respaldo(archivo.ruta_relativa, momento);
                archivos::respaldar(raiz, archivo.ruta_relativa, &ruta_respaldo)?;
                informe.anotar_cambio(
                    archivo.ruta_relativa,
                    TipoDeCambio::Respaldado,
                    Some(textos::copia_exacta_en(&ruta_respaldo)),
                );
                textos::sustituido_con_texto_ajeno(catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO)
            } else {
                String::from(textos::DETALLE_PUESTO_AL_DIA)
            };
            let formato = leido.map(|(_, formato)| formato).unwrap_or_default();
            archivos::escribir(&ruta, &archivo.texto, formato)?;
            informe.anotar_cambio(
                archivo.ruta_relativa,
                TipoDeCambio::Actualizado,
                Some(detalle),
            );
        }
    }
    registro.registrar_archivo(archivo.ruta_relativa, huella::calcular(&archivo.texto));
    Ok(())
}

/// Bandejas del canal que pasan de `limite` bytes, ordenadas por nombre, a partir del nombre y el
/// tamaño de cada archivo de .gestor/canal/. Solo cuentan las bandejas ([`es_bandeja`]): estado.md,
/// la guía de Programator y los históricos no tienen dueño que los pode. No toca el disco.
#[must_use]
pub fn bandejas_sin_podar(entradas: &[(String, u64)], limite: u64) -> Vec<(String, u64)> {
    let mut grandes: Vec<(String, u64)> = entradas
        .iter()
        .filter(|(nombre, tamano)| es_bandeja(nombre) && *tamano > limite)
        .cloned()
        .collect();
    grandes.sort();
    grandes
}

/// Avisa de cada bandeja del canal que pasa del límite: lee el canal y deja la decisión a
/// [`bandejas_sin_podar`]. Si el canal no existe no hay nada que avisar.
fn avisar_bandejas_grandes(
    raiz: &Path,
    limite: u64,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let carpeta = archivos::ruta(raiz, CARPETA_CANAL);
    let entradas = match fs::read_dir(&carpeta) {
        Ok(entradas) => entradas,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let mut leidas: Vec<(String, u64)> = Vec::new();
    for entrada in entradas {
        let entrada = entrada?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        leidas.push((nombre, entrada.metadata()?.len()));
    }
    for (nombre, tamano) in bandejas_sin_podar(&leidas, limite) {
        informe.anotar_aviso(textos::bandeja_sin_podar(
            &format!("{CARPETA_CANAL}/{nombre}"),
            tamano,
            limite,
        ));
    }
    Ok(())
}

/// Añade al archivo de permisos de una herramienta las reglas del arranque que le falten
/// ([`permisos::fusionar`]). Un archivo que no se entiende no se toca: se avisa.
fn mantener_permisos(
    raiz: &Path,
    ruta_relativa: &str,
    reglas: &[&str],
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let ruta = archivos::ruta(raiz, ruta_relativa);
    let leido = archivos::leer(&ruta)?;
    match permisos::fusionar(leido.as_ref().map(|(texto, _)| texto.as_str()), reglas) {
        Fusion::SinCambios => {}
        Fusion::Ilegible(motivo) => {
            informe.anotar_aviso(textos::permisos_ilegibles(ruta_relativa, &motivo));
        }
        Fusion::Escribir(texto) => {
            let formato = leido.as_ref().map(|(_, f)| *f).unwrap_or_default();
            archivos::escribir(&ruta, &texto, formato)?;
            let tipo = if leido.is_some() {
                TipoDeCambio::Actualizado
            } else {
                TipoDeCambio::Creado
            };
            informe.anotar_cambio(
                ruta_relativa,
                tipo,
                Some(String::from(textos::DETALLE_PERMISOS)),
            );
        }
    }
    Ok(())
}

/// Nombre del diario de cada agente dentro de su carpeta de .gestor/.
pub const NOMBRE_DEL_DIARIO: &str = "tarea-en-curso.md";

/// De los archivos que se leen enteros en cada arranque (los diarios y el estado del canal), con su
/// ruta relativa y su tamaño, los que pasan del límite, ordenados por ruta.
#[must_use]
pub fn archivos_de_arranque_grandes(entradas: &[(String, u64)], limite: u64) -> Vec<(String, u64)> {
    let mut grandes: Vec<(String, u64)> = entradas
        .iter()
        .filter(|(_, tamano)| *tamano > limite)
        .cloned()
        .collect();
    grandes.sort();
    grandes
}

/// Avisa de cada diario de agente (.gestor/<agente>/tarea-en-curso.md) y del estado del canal que
/// pasan del límite: se leen enteros en cada arranque, así que lo cerrado debe ir a su histórico.
fn avisar_diarios_y_estado_grandes(
    raiz: &Path,
    limite: u64,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let mut leidas: Vec<(String, u64)> = Vec::new();
    let estado = archivos::ruta(raiz, &ruta_del_estado());
    if let Ok(datos) = fs::metadata(&estado) {
        leidas.push((ruta_del_estado(), datos.len()));
    }
    let entradas = match fs::read_dir(archivos::ruta(raiz, CARPETA_GESTOR)) {
        Ok(entradas) => entradas,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for entrada in entradas {
        let entrada = entrada?;
        let diario = entrada.path().join(NOMBRE_DEL_DIARIO);
        if let Ok(datos) = fs::metadata(&diario) {
            let carpeta = entrada.file_name().to_string_lossy().into_owned();
            leidas.push((
                format!("{CARPETA_GESTOR}/{carpeta}/{NOMBRE_DEL_DIARIO}"),
                datos.len(),
            ));
        }
    }
    for (ruta, tamano) in archivos_de_arranque_grandes(&leidas, limite) {
        informe.anotar_aviso(textos::archivo_de_arranque_grande(&ruta, tamano, limite));
    }
    Ok(())
}

/// Revisa el estado del canal y las bandejas ([`revision_del_canal::revisar`]) y pone en el
/// informe un aviso por cada cosa que no cuadra. Sin estado.md no hay nada que revisar.
fn avisar_del_canal(
    raiz: &Path,
    momento: Momento,
    limite_horas: u64,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let Some((estado, _)) = archivos::leer(&archivos::ruta(raiz, &ruta_del_estado()))? else {
        return Ok(());
    };
    let Some(ahora) = momento.como_fecha_y_hora() else {
        return Ok(());
    };
    let bandejas = leer_bandejas(raiz)?;
    for aviso in revision_del_canal::revisar(&estado, &bandejas, ahora, limite_horas) {
        informe.anotar_aviso(texto_del_aviso(&aviso, limite_horas));
    }
    Ok(())
}

/// Las bandejas del canal ([`es_bandeja`]) con su texto, por orden de nombre.
///
/// # Errors
///
/// El error de disco al leer el canal o una bandeja.
pub fn leer_bandejas(raiz: &Path) -> io::Result<Vec<revision_del_canal::Bandeja>> {
    let carpeta = archivos::ruta(raiz, CARPETA_CANAL);
    let entradas = match fs::read_dir(&carpeta) {
        Ok(entradas) => entradas,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut bandejas = Vec::new();
    for entrada in entradas {
        let entrada = entrada?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        if !es_bandeja(&nombre) {
            continue;
        }
        if let Some((texto, _)) = archivos::leer(&entrada.path())? {
            bandejas.push(revision_del_canal::Bandeja { nombre, texto });
        }
    }
    bandejas.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    Ok(bandejas)
}

/// Texto del aviso que ve el usuario.
fn texto_del_aviso(aviso: &revision_del_canal::AvisoDelCanal, limite_horas: u64) -> String {
    use revision_del_canal::AvisoDelCanal;
    match aviso {
        AvisoDelCanal::FaltaApartado(apartado) => {
            textos::estado_sin_apartado(&ruta_del_estado(), apartado)
        }
        AvisoDelCanal::MutacionesVivas => textos::mutaciones_vivas(&ruta_del_estado()),
        AvisoDelCanal::ReservaIlegible(linea) => {
            textos::reserva_ilegible(&ruta_del_estado(), linea)
        }
        AvisoDelCanal::ReservaRepetida { ruta, agentes } => {
            textos::reserva_repetida(ruta, &agentes.0, &agentes.1)
        }
        AvisoDelCanal::ReservaSinLatido { agente, horas } => {
            textos::reserva_sin_latido(agente, *horas, limite_horas)
        }
    }
}

/// Ejecuta un paso y, si falla, anota el error con su descripción.
fn paso(
    informe: &mut InformeDeInicializacion,
    descripcion: &str,
    accion: impl FnOnce(&mut InformeDeInicializacion) -> io::Result<()>,
) {
    let _ = con_valor(informe, descripcion, accion);
}

/// Ejecuta un paso que devuelve un valor; si falla, anota el error y devuelve nada.
fn con_valor<T>(
    informe: &mut InformeDeInicializacion,
    descripcion: &str,
    accion: impl FnOnce(&mut InformeDeInicializacion) -> io::Result<T>,
) -> Option<T> {
    match accion(informe) {
        Ok(valor) => Some(valor),
        Err(error) => {
            informe.anotar_error(format!("{descripcion}: {error}"));
            None
        }
    }
}

/// Guarda el registro de huellas y los avisos de la preparación.
fn guardar_registros(
    raiz: &Path,
    registro: &RegistroDeHuellas,
    momento: Momento,
    informe: &mut InformeDeInicializacion,
) {
    paso(informe, registro_de_huellas::RUTA_RELATIVA, |_| {
        registro.guardar(raiz)
    });
    let avisos = informe.avisos().to_vec();
    paso(informe, avisos_de_preparacion::RUTA_RELATIVA, |_| {
        avisos_de_preparacion::guardar(raiz, &avisos, momento)
    });
}
