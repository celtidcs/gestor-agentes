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
//! 5. Archivo propio del proyecto: se crea si falta, con lo recogido en el paso 2 y la lista de
//!    lo archivado en el paso 3. Si ya existe no se toca nunca.
//! 6. Archivos que la aplicación escribe enteros: AGENTS.md, CLAUDE.md y los documentos de
//!    consulta, con copia previa de lo que no hubiera escrito ella.
//! 7. Aviso de las bandejas del canal que han crecido demasiado y de lo que no cuadra en el estado
//!    del canal (apartados, mutaciones vivas y reservas), con [`revision_del_canal`].
//! 8. Registro de huellas de lo escrito.
//!
//! Un paso que falla se anota en el informe y no impide los siguientes. Nada se sustituye en
//! silencio: todo cambio queda en el informe y todo texto desplazado, en un respaldo o en el
//! archivo de .gestor/.

use std::fs;
use std::io;
use std::path::Path;

use super::archivos;
use super::catalogo::{self, ArchivoGestionado};
use super::estructura;
use super::informe::{InformeDeInicializacion, TipoDeCambio};
use super::localizador::FuenteDeEntorno;
use super::momento::{Momento, Reloj};
use super::registro_de_huellas::{self, RegistroDeHuellas};
use super::revision_del_canal;
use super::sistema_anterior;
use crate::agentes::{IdentidadAgente, TipoAgente, CARPETA_CANAL};
use crate::git::{self, ServicioGit};
use crate::instrucciones::formato::FormatoDeTexto;
use crate::instrucciones::huella;
use crate::instrucciones::migracion::{self, Archivado, ArchivoAnterior, TextoPropio};
use crate::instrucciones::sustitucion::{self, Sustitucion};
use crate::plantillas;
use crate::textos;

/// Marcador de la fecha en el encabezado del bloque pendiente de consolidar.
const MARCADOR_FECHA: &str = "{{FECHA}}";

/// Archivos del canal que no son la bandeja de nadie, en minúsculas: el estado común y los que
/// Programator reserva en su propio canal (src/protocolo/canal.rs del proyecto Programator), entre
/// ellos la guía COMO-ENCARGAR-A-PROGRAMATOR.md que publica al arrancar.
pub const ARCHIVOS_DEL_CANAL_QUE_NO_SON_BANDEJAS: [&str; 4] = [
    "estado.md",
    "readme.md",
    "indice.md",
    "como-encargar-a-programator.md",
];

/// Prefijos de los históricos que se guardan en el canal, con y sin tilde, como los reconoce
/// Programator. Un histórico no es una bandeja viva.
pub const PREFIJOS_DE_HISTORICO: [&str; 2] = ["historico-", "histórico-"];

/// Indica si un archivo de .gestor/canal/ es la bandeja de un agente: un .md que no es uno de
/// [`ARCHIVOS_DEL_CANAL_QUE_NO_SON_BANDEJAS`] ni un histórico. No distingue mayúsculas.
#[must_use]
pub fn es_bandeja(nombre: &str) -> bool {
    let nombre = nombre.to_lowercase();
    nombre.ends_with(".md")
        && !ARCHIVOS_DEL_CANAL_QUE_NO_SON_BANDEJAS.contains(&nombre.as_str())
        && !PREFIJOS_DE_HISTORICO
            .iter()
            .any(|prefijo| nombre.starts_with(prefijo))
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

        self.git.asegurar_repositorio(raiz, &mut informe);
        paso(&mut informe, git::ARCHIVO_IGNORADOS, |informe| {
            git::asegurar_ignorado(raiz, informe)
        });

        let mut registro = RegistroDeHuellas::cargar(raiz, &mut informe);
        let ruta_propio = archivos::ruta(raiz, catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO);
        let falta_el_propio = !ruta_propio.is_file();
        let propio = if falta_el_propio {
            self.recoger_texto_propio(raiz, &mut informe)
        } else {
            TextoPropio::default()
        };

        let archivados = con_valor(&mut informe, ".gestor/archivo/", |informe| {
            sistema_anterior::archivar(raiz, momento, informe)
        })
        .unwrap_or_default();
        for archivado in &archivados {
            registro.olvidar(&archivado.origen);
        }

        paso(&mut informe, ".gestor/", |informe| {
            estructura::asegurar(raiz, &identidades, informe)
        });

        if falta_el_propio {
            paso(
                &mut informe,
                catalogo::RUTA_INSTRUCCIONES_DEL_PROYECTO,
                |informe| crear_propio(raiz, &propio, &archivados, momento, informe),
            );
        } else if !archivados.is_empty() {
            informe.anotar_aviso(aviso_de_archivados_sin_propio(&archivados));
        }

        let entorno = self.fuente_de_entorno.buscar(raiz);
        for archivo in catalogo::archivos_gestionados(entorno.as_ref()) {
            paso(&mut informe, archivo.ruta_relativa, |informe| {
                mantener_entero(raiz, &archivo, &mut registro, momento, informe)
            });
        }

        paso(&mut informe, CARPETA_CANAL, |informe| {
            avisar_bandejas_grandes(raiz, self.configuracion.limite_de_bandeja(), informe)
        });
        let limite_de_latido = self.configuracion.limite_de_latido_horas();
        paso(&mut informe, ARCHIVO_DE_ESTADO_DEL_CANAL, |informe| {
            avisar_del_canal(raiz, momento, limite_de_latido, informe)
        });
        paso(&mut informe, registro_de_huellas::RUTA_RELATIVA, |_| {
            registro.guardar(raiz)
        });

        informe
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

/// Ruta del estado del canal dentro del proyecto.
const ARCHIVO_DE_ESTADO_DEL_CANAL: &str = ".gestor/canal/estado.md";

/// Revisa el estado del canal y las bandejas ([`revision_del_canal::revisar`]) y pone en el
/// informe un aviso por cada cosa que no cuadra. Sin estado.md no hay nada que revisar.
fn avisar_del_canal(
    raiz: &Path,
    momento: Momento,
    limite_horas: u64,
    informe: &mut InformeDeInicializacion,
) -> io::Result<()> {
    let Some((estado, _)) = archivos::leer(&archivos::ruta(raiz, ARCHIVO_DE_ESTADO_DEL_CANAL))?
    else {
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
            textos::estado_sin_apartado(ARCHIVO_DE_ESTADO_DEL_CANAL, apartado)
        }
        AvisoDelCanal::MutacionesVivas => textos::mutaciones_vivas(ARCHIVO_DE_ESTADO_DEL_CANAL),
        AvisoDelCanal::ReservaIlegible(linea) => {
            textos::reserva_ilegible(ARCHIVO_DE_ESTADO_DEL_CANAL, linea)
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
