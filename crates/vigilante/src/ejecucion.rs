//! Los tres modos del vigilante, con el disco, el reloj y los procesos. Lo que decide está en
//! [`crate::pasada`]; aquí solo se lee, se espera, se escribe y se entrega.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use canal::identidad::Identidad;

use crate::entrega;
use crate::hilo;
use crate::opciones::{Modo, Opciones, ProcesoMarcado, TipoDeEntrega, VIDA_HASTA_CAMBIO_SEGUNDOS};
use crate::pasada::{self, Resultado, Visto};
use crate::procesos::{self, Procesos};
use crate::recuerdo;
use crate::vivo;

/// Procesos ascendientes que se vigilan en --hasta-cambio: el que lanzó la orden (la envoltura),
/// la herramienta del agente y uno más, porque algunas herramientas ponen un proceso intermedio
/// (el mismo número que en la 3.5.1).
const ASCENDIENTES_VIGILADOS: usize = 3;

/// Hasta dónde se sube por el árbol buscando el proceso de Codex: la orden de Codex pasa por un
/// intérprete y alguna envoltura, nunca por más de seis niveles en lo medido con Codex CLI 0.167.
const NIVELES_HASTA_EL_AGENTE: usize = 6;

/// Prefijo del nombre del proceso de Codex CLI («codex» en Linux y «codex.exe» en Windows).
const PROGRAMA_DE_CODEX: &str = "codex";

/// Nombres exactos del proceso de Codex CLI que abre la terminal, para no tomar por él uno de sus
/// procesos auxiliares, cuyo nombre también empieza por «codex».
const NOMBRES_DE_CODEX: [&str; 2] = ["codex", "codex.exe"];

/// Código de salida cuando las opciones o el entorno no permiten vigilar.
pub const CODIGO_DE_USO: i32 = 2;

/// Lo que necesita cada modo: dónde está el canal y por quién se vigila.
pub struct Contexto {
    /// Carpeta .gestor/ del proyecto.
    pub gestor: PathBuf,
    /// Identidad del agente.
    pub identidad: Identidad,
}

impl Contexto {
    /// Nombre de los archivos del agente, como «claude2».
    fn nombre(&self) -> String {
        self.identidad.nombre_de_archivos()
    }

    /// Carpeta .gestor/canal/.
    fn canal(&self) -> PathBuf {
        self.gestor.join("canal")
    }

    /// Carpeta del recuerdo del agente.
    fn recuerdo(&self) -> PathBuf {
        recuerdo::carpeta(&self.gestor, &self.nombre())
    }

    /// Raíz del proyecto.
    fn raiz(&self) -> PathBuf {
        self.gestor
            .parent()
            .map_or_else(|| self.gestor.clone(), Path::to_path_buf)
    }

    /// Texto de la bandeja del agente, o vacío si no se puede leer en este momento.
    fn leer_propia(&self) -> String {
        fs::read(self.canal().join(format!("{}.md", self.nombre())))
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default()
    }

    /// Archivos vigilados del canal con su texto. Uno que no se puede leer en este momento (otro
    /// agente lo está escribiendo) se salta y se ve en la pasada siguiente.
    fn leer_canal(&self) -> io::Result<Vec<(String, String)>> {
        let propia = format!("{}.md", self.nombre());
        let mut archivos = Vec::new();
        for entrada in fs::read_dir(self.canal())? {
            let entrada = entrada?;
            let nombre = entrada.file_name().to_string_lossy().into_owned();
            if !pasada::se_vigila(&nombre, &propia) || !entrada.path().is_file() {
                continue;
            }
            if let Ok(bytes) = fs::read(entrada.path()) {
                archivos.push((nombre, String::from_utf8_lossy(&bytes).into_owned()));
            }
        }
        Ok(archivos)
    }
}

/// Ejecuta el modo pedido y devuelve el código de salida.
pub fn ejecutar(
    contexto: &Contexto,
    opciones: &Opciones,
    procesos: &mut dyn Procesos,
    salida: &mut dyn Write,
) -> i32 {
    let resultado = match opciones.modo {
        Modo::UnaVez => una_vez(contexto, salida),
        Modo::HastaCambio => hasta_cambio(contexto, opciones, procesos, salida),
        Modo::Persistente if opciones.hijo => persistente(contexto, opciones, procesos),
        Modo::Persistente if opciones.desde_terminal.is_some() => {
            desde_terminal(contexto, opciones, procesos)
        }
        Modo::Persistente => quedarse_en_segundo_plano(contexto, opciones, procesos, salida),
        Modo::AnotarHilo => anotar_hilo(contexto, salida),
    };
    match resultado {
        Ok(codigo) => codigo,
        Err(error) => {
            let _ = writeln!(salida, "VIGILANTE: error: {error}");
            1
        }
    }
}

/// Escribe una línea y la vacía enseguida, para que la herramienta del agente la vea al momento.
fn decir(salida: &mut dyn Write, linea: &str) {
    let _ = writeln!(salida, "{linea}");
    let _ = salida.flush();
}

/// Primera pasada sin referencia: lo guarda todo como visto y lo dice.
fn primera_pasada(
    contexto: &Contexto,
    archivos: &[(String, String)],
    salida: &mut dyn Write,
) -> io::Result<()> {
    let visto = pasada::primera(archivos, &contexto.identidad);
    recuerdo::guardar(&contexto.recuerdo(), &visto)?;
    decir(
        salida,
        "CANAL: primera pasada, sin referencia anterior; lee todas las bandejas de .gestor/canal/",
    );
    Ok(())
}

/// --una-vez.
fn una_vez(contexto: &Contexto, salida: &mut dyn Write) -> io::Result<i32> {
    let archivos = contexto.leer_canal()?;
    let Some(antes) = recuerdo::leer(&contexto.recuerdo()) else {
        primera_pasada(contexto, &archivos, salida)?;
        return Ok(0);
    };
    let resultado = pasada::comparar(
        &antes,
        &archivos,
        &contexto.identidad,
        &contexto.leer_propia(),
    );
    if resultado.cambios.is_empty() {
        decir(salida, "CANAL: sin cambios");
    } else {
        avisar(contexto, &resultado, salida);
    }
    if resultado.visto != antes {
        recuerdo::guardar(&contexto.recuerdo(), &resultado.visto)?;
    }
    Ok(0)
}

/// Escribe una línea por cambio.
fn avisar(contexto: &Contexto, resultado: &Resultado, salida: &mut dyn Write) {
    for cambio in &resultado.cambios {
        decir(salida, &cambio.linea(&contexto.identidad));
    }
}

/// Etiquetas y archivos de los cambios etiquetados, para el registro y la entrega.
fn resumen(resultado: &Resultado) -> (String, Vec<String>) {
    let mut etiquetas = Vec::new();
    if resultado
        .cambios
        .iter()
        .any(|c| c.encargo && !c.respuesta_de_programator())
    {
        etiquetas.push("encargo");
    }
    if resultado
        .cambios
        .iter()
        .any(pasada::Cambio::respuesta_de_programator)
    {
        etiquetas.push("respuesta");
    }
    if resultado.cambios.iter().any(|c| c.mutacion) {
        etiquetas.push("mutacion");
    }
    let archivos = resultado
        .cambios
        .iter()
        .filter(|c| c.etiquetado())
        .map(|c| c.archivo.clone())
        .collect();
    (etiquetas.join(","), archivos)
}

/// Por qué termina un vigilante que espera.
enum Fin {
    /// Otro vigilante del mismo agente lo ha relevado.
    Relevado,
    /// Ha muerto el proceso al que sirve.
    SinDueno,
    /// Ha cumplido su vida máxima.
    PorTiempo,
}

/// Lo que comparten los dos modos que esperan: relevo, dueños y vida.
struct Guardia {
    relevo: PathBuf,
    marca: String,
    duenos: Vec<ProcesoMarcado>,
    fin_de_vida: Option<u64>,
}

impl Guardia {
    /// Toma el relevo: anota la marca de este vigilante, que hace terminar al anterior.
    fn tomar(
        contexto: &Contexto,
        duenos: Vec<ProcesoMarcado>,
        vida: Option<u64>,
    ) -> io::Result<Self> {
        let relevo = vivo::ruta_de_relevo(&contexto.gestor, &contexto.nombre());
        if let Some(carpeta) = relevo.parent() {
            fs::create_dir_all(carpeta)?;
        }
        let marca = vivo::marca_propia();
        fs::write(&relevo, format!("{marca}\n"))?;
        Ok(Self {
            relevo,
            marca,
            duenos,
            fin_de_vida: vida.map(|segundos| vivo::ahora().saturating_add(segundos)),
        })
    }

    /// Si hay que terminar, y por qué.
    fn fin(&self, procesos: &mut dyn Procesos) -> Option<Fin> {
        if vivo::marca_escrita(&self.relevo) != self.marca {
            return Some(Fin::Relevado);
        }
        if !self.duenos.iter().all(|d| procesos::vivo(procesos, *d)) {
            return Some(Fin::SinDueno);
        }
        if self.fin_de_vida.is_some_and(|fin| vivo::ahora() >= fin) {
            return Some(Fin::PorTiempo);
        }
        None
    }

    /// Suelta el relevo si sigue siendo suyo.
    fn soltar(&self) {
        if vivo::marca_escrita(&self.relevo) == self.marca {
            let _ = fs::remove_file(&self.relevo);
        }
    }
}

/// Mensaje con que termina un vigilante que espera.
fn mensaje_de_fin(fin: &Fin, identidad: &Identidad) -> String {
    match fin {
        Fin::Relevado => {
            format!("VIGILANTE: relevado por otro vigilante de {identidad}; no lo rearmes")
        }
        Fin::SinDueno => String::from(
            "VIGILANTE: fin porque se cerró quien lo lanzó; relánzalo si sigues trabajando",
        ),
        Fin::PorTiempo => String::from("VIGILANTE: fin por tiempo; rearma"),
    }
}

/// --hasta-cambio.
fn hasta_cambio(
    contexto: &Contexto,
    opciones: &Opciones,
    procesos: &mut dyn Procesos,
    salida: &mut dyn Write,
) -> io::Result<i32> {
    let Some(antes) = recuerdo::leer(&contexto.recuerdo()) else {
        let archivos = contexto.leer_canal()?;
        primera_pasada(contexto, &archivos, salida)?;
        return Ok(0);
    };
    let duenos = procesos::ascendientes(procesos, std::process::id(), ASCENDIENTES_VIGILADOS)
        .iter()
        .map(procesos::DatosDeProceso::marcado)
        .collect();
    let vida = opciones.vida_segundos.or(Some(VIDA_HASTA_CAMBIO_SEGUNDOS));
    let guardia = Guardia::tomar(contexto, duenos, vida)?;
    let vivo_texto = escribir_vivo(contexto, procesos, Modo::HastaCambio);
    let espera = Duration::from_secs(opciones.intervalo_segundos);
    let codigo = loop {
        if let Some(fin) = guardia.fin(procesos) {
            decir(salida, &mensaje_de_fin(&fin, &contexto.identidad));
            break 0;
        }
        let archivos = contexto.leer_canal()?;
        let resultado = pasada::comparar(
            &antes,
            &archivos,
            &contexto.identidad,
            &contexto.leer_propia(),
        );
        if resultado.hay_etiquetas() {
            avisar(contexto, &resultado, salida);
            recuerdo::guardar(&contexto.recuerdo(), &resultado.visto)?;
            let (etiquetas, nombres) = resumen(&resultado);
            let _ = vivo::anotar_aviso(
                &contexto.recuerdo(),
                vivo::ahora(),
                &etiquetas,
                &nombres,
                "fin de la tarea",
            );
            break 0;
        }
        thread::sleep(espera);
    };
    guardia.soltar();
    if let Some(texto) = vivo_texto {
        vivo::borrar_vivo(&contexto.recuerdo(), &texto);
    }
    Ok(codigo)
}

/// Escribe vivo.json con el proceso propio; devuelve su texto para borrarlo al terminar.
fn escribir_vivo(contexto: &Contexto, procesos: &mut dyn Procesos, modo: Modo) -> Option<String> {
    let propio = procesos.datos(std::process::id())?.marcado();
    let texto = vivo::texto_vivo(&contexto.identidad, propio, modo, vivo::ahora());
    vivo::escribir_vivo(&contexto.recuerdo(), &texto).ok()?;
    Some(texto)
}

/// Datos que necesita la entrega a Codex.
struct DestinoDeCodex {
    codex: PathBuf,
    hilo: String,
    dueno: ProcesoMarcado,
}

/// Busca la sesión de Codex que ha lanzado al vigilante.
fn buscar_codex(
    opciones: &Opciones,
    procesos: &mut dyn Procesos,
) -> Result<DestinoDeCodex, String> {
    let hilo = opciones
        .hilo
        .clone()
        .or_else(|| std::env::var(entrega::VARIABLE_DE_LA_SESION).ok())
        .filter(|hilo| !hilo.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "no sé qué sesión de Codex avisar: falta {}",
                entrega::VARIABLE_DE_LA_SESION
            )
        })?;
    let agente = procesos::ascendiente_llamado(
        procesos,
        std::process::id(),
        PROGRAMA_DE_CODEX,
        NIVELES_HASTA_EL_AGENTE,
    )
    .ok_or_else(|| String::from("no encuentro el proceso de Codex que me ha lanzado"))?;
    let codex = opciones
        .codex
        .clone()
        .or_else(|| agente.ejecutable.clone())
        .ok_or_else(|| {
            String::from("no sé dónde está el ejecutable de Codex; pásalo con --codex")
        })?;
    Ok(DestinoDeCodex {
        codex,
        hilo,
        dueno: agente.marcado(),
    })
}

/// --persistente sin --hijo: comprueba que puede vigilar y se relanza a sí mismo desligado de la
/// orden que lo lanzó, para seguir vivo cuando esa orden termine.
fn quedarse_en_segundo_plano(
    contexto: &Contexto,
    opciones: &Opciones,
    procesos: &mut dyn Procesos,
    salida: &mut dyn Write,
) -> io::Result<i32> {
    let Some(TipoDeEntrega::Codex) = opciones.entrega else {
        return Ok(CODIGO_DE_USO);
    };
    let destino = match buscar_codex(opciones, procesos) {
        Ok(destino) => destino,
        Err(motivo) => {
            decir(salida, &format!("VIGILANTE: {motivo}"));
            return Ok(CODIGO_DE_USO);
        }
    };
    let mut argumentos = vec![
        String::from("--persistente"),
        String::from("--entrega"),
        String::from("codex"),
        String::from("--hijo"),
        String::from("--agente"),
        contexto.identidad.to_string(),
        String::from("--gestor"),
        contexto.gestor.to_string_lossy().into_owned(),
        String::from("--dueno"),
        format!("{}:{}", destino.dueno.pid, destino.dueno.arranque),
        String::from("--codex"),
        destino.codex.to_string_lossy().into_owned(),
        String::from("--hilo"),
        destino.hilo,
        String::from("--intervalo-segundos"),
        opciones.intervalo_segundos.to_string(),
    ];
    if let Some(vida) = opciones.vida_segundos {
        argumentos.extend([String::from("--vida-segundos"), vida.to_string()]);
    }
    let propio = std::env::current_exe()?;
    let pid = lanzar_desligado(&propio, &argumentos)?;
    decir(
        salida,
        &format!(
            "VIGILANTE: vigilando el canal para {} en segundo plano (proceso {pid}) hasta que se cierre esta sesión de Codex; no hace falta rearmarlo",
            contexto.identidad
        ),
    );
    Ok(0)
}

/// --anotar-hilo: deja la sesión de Codex (CODEX_THREAD_ID) en el proyecto para el vigilante que
/// lanzó Gestor Agentes desde la terminal.
fn anotar_hilo(contexto: &Contexto, salida: &mut dyn Write) -> io::Result<i32> {
    let Some(hilo) = std::env::var(entrega::VARIABLE_DE_LA_SESION)
        .ok()
        .filter(|hilo| !hilo.trim().is_empty())
    else {
        decir(
            salida,
            &format!(
                "VIGILANTE: falta {}; solo Codex puede anotar su sesión",
                entrega::VARIABLE_DE_LA_SESION
            ),
        );
        return Ok(CODIGO_DE_USO);
    };
    hilo::anotar(&contexto.recuerdo(), hilo.trim())?;
    decir(
        salida,
        &format!(
            "VIGILANTE: sesión de Codex anotada para {}; el vigilante que lanzó Gestor Agentes te avisará en ella",
            contexto.identidad
        ),
    );
    Ok(0)
}

/// --persistente --desde-terminal: lo lanza Gestor Agentes desde la terminal que abre Codex, fuera
/// del sandbox de Codex. Espera a que Codex arranque en esa terminal y anote su sesión, y entonces
/// vigila como el persistente, con ese Codex como dueño. Si la terminal se cierra antes, termina.
fn desde_terminal(
    contexto: &Contexto,
    opciones: &Opciones,
    procesos: &mut dyn Procesos,
) -> io::Result<i32> {
    let Some(pid) = opciones.desde_terminal else {
        return Ok(CODIGO_DE_USO);
    };
    let Some(terminal) = procesos.datos(pid).map(|datos| datos.marcado()) else {
        return Ok(CODIGO_DE_USO);
    };
    let arranque = vivo::ahora();
    let espera = Duration::from_secs(opciones.intervalo_segundos);
    while procesos::vivo(procesos, terminal) {
        let codex = procesos::descendiente_llamado(&procesos.todos(), pid, &NOMBRES_DE_CODEX);
        let sesion = hilo::leer(&contexto.recuerdo())
            .and_then(|(texto, modificado)| hilo::hilo_valido(&texto, modificado, arranque));
        if let (Some(codex), Some(sesion)) = (codex, sesion) {
            if let Some(ejecutable) = codex.ejecutable.clone() {
                let con_dueno = Opciones {
                    dueno: Some(codex.marcado()),
                    codex: Some(ejecutable),
                    hilo: Some(sesion),
                    hijo: true,
                    ..opciones.clone()
                };
                return persistente(contexto, &con_dueno, procesos);
            }
        }
        thread::sleep(espera);
    }
    Ok(0)
}

/// Lanza un proceso que sobrevive a quien lo lanza y no hereda su consola.
fn lanzar_desligado(programa: &Path, argumentos: &[String]) -> io::Result<u32> {
    let mut orden = Command::new(programa);
    orden
        .args(argumentos)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Banderas de CreateProcess (https://learn.microsoft.com/windows/win32/procthread/process-creation-flags):
        // DETACHED_PROCESS sin consola, CREATE_NEW_PROCESS_GROUP fuera del grupo de la orden y
        // CREATE_BREAKAWAY_FROM_JOB fuera del trabajo (job) con que la herramienta mata lo que
        // lanzó. Si el trabajo no permite salir, se lanza sin esa última bandera.
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
        orden.creation_flags(
            DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB,
        );
        if let Ok(hijo) = orden.spawn() {
            return Ok(hijo.id());
        }
        orden.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Grupo de procesos propio: no muere con el grupo de la orden que lo lanzó.
        orden.process_group(0);
    }
    orden.spawn().map(|hijo| hijo.id())
}

/// --persistente --hijo: vigila hasta que muere el proceso de Codex y entrega cada aviso.
fn persistente(
    contexto: &Contexto,
    opciones: &Opciones,
    procesos: &mut dyn Procesos,
) -> io::Result<i32> {
    let (Some(dueno), Some(codex), Some(hilo)) = (
        opciones.dueno,
        opciones.codex.clone(),
        opciones.hilo.clone(),
    ) else {
        return Ok(CODIGO_DE_USO);
    };
    let guardia = Guardia::tomar(contexto, vec![dueno], opciones.vida_segundos)?;
    let vivo_texto = escribir_vivo(contexto, procesos, Modo::Persistente);
    let mut visto: Visto = match recuerdo::leer(&contexto.recuerdo()) {
        Some(visto) => visto,
        None => {
            let primera = pasada::primera(&contexto.leer_canal()?, &contexto.identidad);
            recuerdo::guardar(&contexto.recuerdo(), &primera)?;
            primera
        }
    };
    let espera = Duration::from_secs(opciones.intervalo_segundos);
    let mut pendientes: Vec<String> = Vec::new();
    let mut etiquetas_pendientes = String::new();
    let mut ultima_entrega: Option<u64> = None;
    while guardia.fin(procesos).is_none() {
        if let Ok(archivos) = contexto.leer_canal() {
            let resultado = pasada::comparar(
                &visto,
                &archivos,
                &contexto.identidad,
                &contexto.leer_propia(),
            );
            if resultado.visto != visto {
                recuerdo::guardar(&contexto.recuerdo(), &resultado.visto)?;
                visto = resultado.visto.clone();
            }
            if resultado.hay_etiquetas() {
                let (etiquetas, nombres) = resumen(&resultado);
                for nombre in nombres {
                    if !pendientes.contains(&nombre) {
                        pendientes.push(nombre);
                    }
                }
                etiquetas_pendientes = etiquetas;
            }
        }
        let ahora = vivo::ahora();
        if !pendientes.is_empty() && entrega::toca_entregar(ultima_entrega, ahora) {
            let mensaje = entrega::texto_del_aviso(&contexto.identidad, &pendientes);
            let resultado = entrega::entregar(&codex, &contexto.raiz(), &hilo, &mensaje);
            let anotado = entrega::anotacion(&resultado);
            let _ = vivo::anotar_aviso(
                &contexto.recuerdo(),
                ahora,
                &etiquetas_pendientes,
                &pendientes,
                &anotado,
            );
            ultima_entrega = Some(ahora);
            pendientes.clear();
        }
        thread::sleep(espera);
    }
    guardia.soltar();
    if let Some(texto) = vivo_texto {
        vivo::borrar_vivo(&contexto.recuerdo(), &texto);
    }
    Ok(0)
}
