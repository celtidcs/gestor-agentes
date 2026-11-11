//! Barreras de la configuración portable: perfiles (las de ConfigServiceTests de la 2.3.2 más
//! Programator), proyectos, migración desde la 2.x y estado de un proyecto.

mod comun;

use std::path::Path;
use std::time::Duration;

use comun::CarpetaTemporal;
use nucleo::ajustes::Ajustes;
use nucleo::configuracion::{
    self, Configuracion, ErrorDeConfiguracion, HerramientaDelPerfil, PerfilDeAgente,
    ProyectoRegistrado,
};
use nucleo::proyecto::estado::{ArchivosDeInstrucciones, EstadoDelProyecto};
use nucleo::proyecto::inicializador::ConfiguracionDeInstrucciones;

const PERFILES_ANTERIORES: &str = r#"[{"Tipo":"Claude","Instancia":1},{"Tipo":"Claude","Instancia":2},{"Tipo":"Codex","Instancia":1}]"#;

fn cargar(carpeta: &CarpetaTemporal) -> Vec<PerfilDeAgente> {
    Configuracion::new(carpeta.ruta())
        .cargar_perfiles()
        .unwrap_or_else(|error| unreachable!("{error}"))
}

fn nombres(perfiles: &[PerfilDeAgente]) -> Vec<String> {
    perfiles.iter().map(PerfilDeAgente::nombre).collect()
}

fn cuantos(perfiles: &[PerfilDeAgente], herramienta: HerramientaDelPerfil) -> usize {
    perfiles.iter().filter(|p| p.tipo() == herramienta).count()
}

#[test]
fn instalacion_nueva_crea_claude_codex_gemini_y_programator() {
    let carpeta = CarpetaTemporal::new();

    let perfiles = cargar(&carpeta);

    assert_eq!(
        nombres(&perfiles),
        [
            "Claude 1",
            "Claude 2",
            "Codex 1",
            "Gemini 1",
            "Programator 1"
        ]
    );
    assert!(carpeta.existe(configuracion::ARCHIVO_PERFILES));
    assert_eq!(carpeta.leer(configuracion::ARCHIVO_VERSION_PERFILES), "4");
}

#[test]
fn archivo_de_version_anterior_recibe_gemini_y_programator_una_sola_vez() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(configuracion::ARCHIVO_PERFILES, PERFILES_ANTERIORES);

    let primera = cargar(&carpeta);
    let segunda = cargar(&carpeta);

    assert_eq!(cuantos(&primera, HerramientaDelPerfil::Gemini), 1);
    assert_eq!(cuantos(&primera, HerramientaDelPerfil::Programator), 1);
    assert_eq!(primera.len(), 5);
    assert_eq!(segunda, primera);
}

#[test]
fn gemini_quitado_por_el_usuario_no_vuelve_a_aparecer() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(configuracion::ARCHIVO_PERFILES, PERFILES_ANTERIORES);
    cargar(&carpeta);

    carpeta.escribir(configuracion::ARCHIVO_PERFILES, PERFILES_ANTERIORES);
    let perfiles = cargar(&carpeta);

    assert_eq!(cuantos(&perfiles, HerramientaDelPerfil::Gemini), 0);
    assert_eq!(cuantos(&perfiles, HerramientaDelPerfil::Programator), 0);
}

#[test]
fn archivo_anterior_con_gemini_no_lo_duplica() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        configuracion::ARCHIVO_PERFILES,
        r#"[{"Tipo":"Claude","Instancia":1},{"Tipo":"Gemini","Instancia":2}]"#,
    );

    let perfiles = cargar(&carpeta);

    assert_eq!(
        nombres(&perfiles),
        ["Claude 1", "Gemini 2", "Programator 1"]
    );
}

#[test]
fn archivo_de_la_2_3_2_recibe_solo_programator() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(configuracion::ARCHIVO_PERFILES, PERFILES_ANTERIORES);
    carpeta.escribir(configuracion::ARCHIVO_VERSION_PERFILES, "2");

    let perfiles = cargar(&carpeta);

    assert_eq!(
        nombres(&perfiles),
        ["Claude 1", "Claude 2", "Codex 1", "Programator 1"]
    );
}

#[test]
fn perfiles_repetidos_se_rechazan() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        configuracion::ARCHIVO_PERFILES,
        r#"[{"Tipo":"Claude","Instancia":2},{"Tipo":"Claude","Instancia":2}]"#,
    );
    carpeta.escribir(configuracion::ARCHIVO_VERSION_PERFILES, "3");

    let resultado = Configuracion::new(carpeta.ruta()).cargar_perfiles();

    assert!(
        matches!(&resultado, Err(ErrorDeConfiguracion::PerfilRepetido { nombre, .. }) if nombre == "Claude 2"),
        "{resultado:?}"
    );
}

#[test]
fn instancia_cero_se_rechaza() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        configuracion::ARCHIVO_PERFILES,
        r#"[{"Tipo":"Codex","Instancia":0}]"#,
    );
    carpeta.escribir(configuracion::ARCHIVO_VERSION_PERFILES, "3");

    let resultado = Configuracion::new(carpeta.ruta()).cargar_perfiles();

    assert!(
        matches!(resultado, Err(ErrorDeConfiguracion::PerfilInvalido { .. })),
        "{resultado:?}"
    );
}

#[test]
fn perfiles_danados_dan_error_de_formato_sin_sobrescribir() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(configuracion::ARCHIVO_PERFILES, "[{no es json");

    let resultado = Configuracion::new(carpeta.ruta()).cargar_perfiles();

    assert!(
        matches!(resultado, Err(ErrorDeConfiguracion::FormatoInvalido { .. })),
        "{resultado:?}"
    );
    assert_eq!(
        carpeta.leer(configuracion::ARCHIVO_PERFILES),
        "[{no es json"
    );
}

#[test]
fn perfiles_con_bom_y_ejecutable_se_leen() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        configuracion::ARCHIVO_PERFILES,
        "\u{FEFF}[{\"Tipo\":\"Programator\",\"Instancia\":1,\"Ejecutable\":\"~/bin/programator\"}]",
    );
    carpeta.escribir(configuracion::ARCHIVO_VERSION_PERFILES, "3");

    let perfiles = cargar(&carpeta);

    assert_eq!(perfiles.len(), 1);
    assert_eq!(
        perfiles[0].ejecutable_expandido(Path::new("/home/x")),
        Some(Path::new("/home/x").join("bin/programator"))
    );
}

#[test]
fn la_tilde_se_expande_a_la_carpeta_del_usuario() {
    let usuario = Path::new("/home/x");
    let con =
        |valor: &str| PerfilDeAgente::new(HerramientaDelPerfil::Claude, 1).con_directorio(valor);

    assert_eq!(
        con("~").directorio_configuracion_expandido(usuario),
        Some(usuario.to_path_buf())
    );
    assert_eq!(
        con("~/.claude2").directorio_configuracion_expandido(usuario),
        Some(usuario.join(".claude2"))
    );
    assert_eq!(
        con("~\\.claude2").directorio_configuracion_expandido(usuario),
        Some(usuario.join(".claude2"))
    );
    assert_eq!(
        con("/otra/~x").directorio_configuracion_expandido(usuario),
        Some(Path::new("/otra/~x").to_path_buf())
    );
    assert_eq!(con("  ").directorio_configuracion_expandido(usuario), None);
}

#[test]
fn proyectos_se_guardan_y_se_leen_con_los_nombres_de_la_2x() {
    let carpeta = CarpetaTemporal::new();
    let config = Configuracion::new(carpeta.ruta());
    let proyectos = vec![ProyectoRegistrado::new(
        "Gestor",
        "/proyectos/gestor",
        "2026-11-10T12:00:00+01:00",
    )];

    assert!(config.guardar_proyectos(&proyectos).is_ok());
    let leidos = config.cargar_proyectos();

    assert_eq!(leidos.ok(), Some(proyectos));
    let texto = carpeta.leer(configuracion::ARCHIVO_PROYECTOS);
    for campo in ["\"Name\"", "\"Path\"", "\"LastOpened\""] {
        assert!(texto.contains(campo), "falta {campo} en {texto}");
    }
}

#[test]
fn sin_archivo_de_proyectos_la_lista_esta_vacia() {
    let carpeta = CarpetaTemporal::new();

    let leidos = Configuracion::new(carpeta.ruta()).cargar_proyectos();

    assert_eq!(leidos.ok(), Some(Vec::new()));
}

#[test]
fn config_json_de_la_2x_se_lee() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        configuracion::ARCHIVO_PROYECTOS,
        "\u{FEFF}[\r\n  {\r\n    \"Name\": \"Ejemplo\",\r\n    \"Path\": \"X:\\\\proyecto\\\\Ejemplo\",\r\n    \"LastOpened\": \"2026-10-26T15:30:00.1234567+01:00\"\r\n  }\r\n]",
    );

    let leidos = Configuracion::new(carpeta.ruta()).cargar_proyectos();

    assert_eq!(
        leidos.ok(),
        Some(vec![ProyectoRegistrado::new(
            "Ejemplo",
            "X:\\proyecto\\Ejemplo",
            "2026-10-26T15:30:00.1234567+01:00",
        )])
    );
}

#[test]
fn migracion_copia_la_2x_sin_borrar_el_original() {
    let origen = CarpetaTemporal::new();
    let datos = CarpetaTemporal::new();
    origen.escribir(configuracion::ARCHIVO_PROYECTOS, "[]");
    origen.escribir(configuracion::ARCHIVO_PERFILES, PERFILES_ANTERIORES);
    origen.escribir(configuracion::ARCHIVO_VERSION_PERFILES, "2");
    let destino = datos.ruta().join("datos");

    let copiados = Configuracion::new(&destino).migrar_desde(origen.ruta());

    assert_eq!(
        copiados.ok(),
        Some(vec![
            configuracion::ARCHIVO_PROYECTOS,
            configuracion::ARCHIVO_PERFILES,
            configuracion::ARCHIVO_VERSION_PERFILES,
        ])
    );
    assert!(origen.existe(configuracion::ARCHIVO_PERFILES));
    assert_eq!(
        std::fs::read_to_string(destino.join(configuracion::ARCHIVO_PERFILES)).ok(),
        Some(PERFILES_ANTERIORES.to_owned())
    );
}

#[test]
fn migracion_no_toca_una_configuracion_existente() {
    let origen = CarpetaTemporal::new();
    let datos = CarpetaTemporal::new();
    origen.escribir(configuracion::ARCHIVO_PERFILES, PERFILES_ANTERIORES);
    origen.escribir(configuracion::ARCHIVO_INSTRUCCIONES_ANTIGUAS, "antigua");
    datos.escribir(configuracion::ARCHIVO_PROYECTOS, "[]");

    let copiados = Configuracion::new(datos.ruta()).migrar_desde(origen.ruta());

    assert_eq!(copiados.ok(), Some(Vec::new()));
    assert!(!datos.existe(configuracion::ARCHIVO_PERFILES));
    assert!(!datos.existe(configuracion::ARCHIVO_INSTRUCCIONES_ANTIGUAS));
}

#[test]
fn migracion_sin_origen_no_hace_nada() {
    let datos = CarpetaTemporal::new();

    let copiados = Configuracion::new(datos.ruta()).migrar_desde(&datos.ruta().join("no-existe"));

    assert_eq!(copiados.ok(), Some(Vec::new()));
}

#[test]
fn carpeta_de_la_2x_cuelga_de_los_datos_de_aplicacion() {
    assert_eq!(
        configuracion::carpeta_de_la_2x(Path::new("/home/x/.config")),
        Path::new("/home/x/.config/GestorClaude")
    );
}

#[test]
fn instrucciones_antiguas_se_leen_de_la_carpeta_de_datos() {
    let carpeta = CarpetaTemporal::new();
    let config = Configuracion::new(carpeta.ruta());
    assert_eq!(config.leer_instrucciones_antiguas(), None);

    carpeta.escribir(configuracion::ARCHIVO_INSTRUCCIONES_ANTIGUAS, "plantilla");

    assert_eq!(
        config.leer_instrucciones_antiguas().as_deref(),
        Some("plantilla")
    );
}

#[test]
fn estado_del_proyecto_segun_agents_md_y_el_archivo_propio() {
    let carpeta = CarpetaTemporal::new();
    let estado = || EstadoDelProyecto::de(carpeta.ruta());
    assert_eq!(estado(), EstadoDelProyecto::SinConfigurar);

    carpeta.escribir("CLAUDE.md", "Instrucciones de antes\n");
    assert_eq!(estado(), EstadoDelProyecto::FormatoAntiguo);

    carpeta.escribir(
        "AGENTS.md",
        "\u{FEFF}Cabecera\r\n  <!-- NUCLEO-COMUN:INICIO -->  \r\nnúcleo\r\n",
    );
    assert_eq!(
        estado(),
        EstadoDelProyecto::FormatoAntiguo,
        "sin el archivo propio del proyecto falta configurarlo"
    );

    carpeta.escribir(".gestor/instrucciones-del-proyecto.md", "Propio\n");
    assert_eq!(estado(), EstadoDelProyecto::Configurado);

    carpeta.escribir("AGENTS.md", "texto <!-- NUCLEO-COMUN:INICIO --> en medio\n");
    assert_eq!(estado(), EstadoDelProyecto::FormatoAntiguo);

    carpeta.escribir("AGENTS.md", "\u{FEFF}<!-- NUCLEO-COMUN:INICIO -->\rresto");
    assert_eq!(estado(), EstadoDelProyecto::Configurado);
}

#[test]
fn solo_un_gemini_md_anterior_cuenta_como_formato_antiguo() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("GEMINI.md", "Reglas de antes\n");

    assert_eq!(
        EstadoDelProyecto::de(carpeta.ruta()),
        EstadoDelProyecto::FormatoAntiguo
    );
}

#[test]
fn el_estado_se_decide_sin_tocar_el_disco() {
    let configurado = ArchivosDeInstrucciones {
        reglas: Some(String::from("<!-- NUCLEO-COMUN:INICIO -->\nnúcleo\n")),
        hay_otros: true,
        hay_archivo_propio: true,
    };
    let vacio = ArchivosDeInstrucciones {
        reglas: None,
        hay_otros: false,
        hay_archivo_propio: true,
    };

    assert_eq!(
        EstadoDelProyecto::segun(&configurado),
        EstadoDelProyecto::Configurado
    );
    assert_eq!(
        EstadoDelProyecto::segun(&vacio),
        EstadoDelProyecto::SinConfigurar
    );
}

#[test]
fn textos_del_estado_son_los_de_la_2_3_2() {
    assert_eq!(EstadoDelProyecto::SinConfigurar.texto(), "Sin configurar");
    assert_eq!(EstadoDelProyecto::Configurado.texto(), "Configurado");
    assert_eq!(EstadoDelProyecto::FormatoAntiguo.texto(), "Formato antiguo");
}

// Argumentos, límite de instrucciones y fin del campo Modo (esquema 4).

fn cargar_con_version(
    perfiles_json: &str,
    version: &str,
) -> (
    CarpetaTemporal,
    Result<Vec<PerfilDeAgente>, ErrorDeConfiguracion>,
) {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(configuracion::ARCHIVO_PERFILES, perfiles_json);
    carpeta.escribir(configuracion::ARCHIVO_VERSION_PERFILES, version);
    let resultado = Configuracion::new(carpeta.ruta()).cargar_perfiles();
    (carpeta, resultado)
}

fn cargar_version_actual(perfiles_json: &str) -> Result<Vec<PerfilDeAgente>, ErrorDeConfiguracion> {
    cargar_con_version(perfiles_json, "4").1
}

fn rechazado_por(perfiles_json: &str, motivo_esperado: &str) {
    let resultado = cargar_version_actual(perfiles_json);
    assert!(
        matches!(&resultado, Err(ErrorDeConfiguracion::PerfilInvalido { motivo, .. }) if motivo.contains(motivo_esperado)),
        "{resultado:?}"
    );
}

#[test]
fn un_limite_de_instrucciones_de_cero_se_rechaza() {
    rechazado_por(
        r#"[{"Tipo":"Codex","Instancia":1,"LimiteDeInstrucciones":0}]"#,
        "no puede ser 0",
    );
}

#[test]
fn los_argumentos_valen_en_cualquier_perfil() {
    let perfiles = cargar_version_actual(
        r#"[{"Tipo":"Gemini","Instancia":1,"Argumentos":["--model","x"]},{"Tipo":"Claude","Instancia":1,"Argumentos":["--verbose"]}]"#,
    )
    .unwrap_or_else(|error| unreachable!("{error}"));

    assert_eq!(perfiles[0].argumentos(), ["--model", "x"]);
    assert_eq!(perfiles[1].argumentos(), ["--verbose"]);
}

#[test]
fn el_modo_de_un_archivo_anterior_se_ignora_y_no_se_vuelve_a_escribir() {
    let (carpeta, resultado) = cargar_con_version(
        r#"[{"Tipo":"Claude","Instancia":1,"Modo":"Terminal"},{"Tipo":"Programator","Instancia":1,"Modo":"Aplicacion"}]"#,
        "3",
    );

    let perfiles = resultado.unwrap_or_else(|error| unreachable!("{error}"));
    assert_eq!(nombres(&perfiles), ["Claude 1", "Programator 1"]);
    let reescrito = carpeta.leer(configuracion::ARCHIVO_PERFILES);
    assert!(!reescrito.contains("Modo"), "{reescrito}");
    assert_eq!(carpeta.leer(configuracion::ARCHIVO_VERSION_PERFILES), "4");
}

#[test]
fn al_migrar_se_funden_la_aplicacion_y_la_terminal_del_mismo_agente() {
    let (carpeta, resultado) = cargar_con_version(
        r#"[{"Tipo":"Claude","Instancia":1},{"Tipo":"Codex","Instancia":1,"DirectorioConfiguracion":"~/.codex"},{"Tipo":"Gemini","Instancia":1},{"Tipo":"Codex","Instancia":1,"Modo":"Terminal","Argumentos":["-c","x=1"]},{"Tipo":"Gemini","Instancia":1,"Modo":"Terminal","Argumentos":["--model","m"],"LimiteDeInstrucciones":30000}]"#,
        "3",
    );

    let perfiles = resultado.unwrap_or_else(|error| unreachable!("{error}"));
    assert_eq!(nombres(&perfiles), ["Claude 1", "Codex 1", "Gemini 1"]);
    let codex = &perfiles[1];
    assert_eq!(codex.argumentos(), ["-c", "x=1"]);
    assert_eq!(codex.directorio_configuracion(), Some("~/.codex"));
    let gemini = &perfiles[2];
    assert_eq!(gemini.argumentos(), ["--model", "m"]);
    assert_eq!(gemini.limite_de_instrucciones(), Some(30_000));
    assert_eq!(cargar(&carpeta), perfiles);
}

#[test]
fn al_migrar_gemini_de_escritorio_pierde_el_ejecutable_de_antigravity_y_codex_lo_conserva() {
    let perfiles = cargar_con_version(
        r#"[{"Tipo":"Gemini","Instancia":1,"Ejecutable":"C:/Antigravity/Antigravity.exe"},{"Tipo":"Codex","Instancia":1,"Ejecutable":"C:/codex/codex.exe"},{"Tipo":"Gemini","Instancia":2,"Modo":"Terminal","Ejecutable":"C:/agy/agy.exe"}]"#,
        "3",
    )
    .1
    .unwrap_or_else(|error| unreachable!("{error}"));

    assert_eq!(perfiles[0].ejecutable(), None);
    assert_eq!(perfiles[1].ejecutable(), Some("C:/codex/codex.exe"));
    assert_eq!(perfiles[2].ejecutable(), Some("C:/agy/agy.exe"));
}

#[test]
fn el_mismo_agente_dos_veces_en_terminal_sigue_repetido_al_migrar() {
    let resultado = cargar_con_version(
        r#"[{"Tipo":"Codex","Instancia":1,"Modo":"Terminal"},{"Tipo":"Codex","Instancia":1,"Modo":"Terminal"}]"#,
        "3",
    )
    .1;
    assert!(
        matches!(&resultado, Err(ErrorDeConfiguracion::PerfilRepetido { nombre, .. }) if nombre == "Codex 1"),
        "{resultado:?}"
    );
}

#[test]
fn con_el_esquema_4_ya_no_se_funde_nada() {
    let resultado = cargar_version_actual(
        r#"[{"Tipo":"Gemini","Instancia":1},{"Tipo":"Gemini","Instancia":1,"Modo":"Terminal"}]"#,
    );
    assert!(
        matches!(&resultado, Err(ErrorDeConfiguracion::PerfilRepetido { nombre, .. }) if nombre == "Gemini 1"),
        "{resultado:?}"
    );
}

#[test]
fn un_perfil_de_la_2x_se_reescribe_sin_los_campos_nuevos() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(configuracion::ARCHIVO_PERFILES, PERFILES_ANTERIORES);
    carpeta.escribir(configuracion::ARCHIVO_VERSION_PERFILES, "2");

    let _ = cargar(&carpeta);

    let reescrito = carpeta.leer(configuracion::ARCHIVO_PERFILES);
    assert!(reescrito.contains("Programator"), "{reescrito}");
    for campo in ["Modo", "Argumentos", "LimiteDeInstrucciones"] {
        assert!(!reescrito.contains(campo), "{campo} en {reescrito}");
    }
}

#[test]
fn un_perfil_con_argumentos_sobrevive_a_la_reescritura() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir(
        configuracion::ARCHIVO_PERFILES,
        r#"[{"Tipo":"Codex","Instancia":1,"Modo":"Terminal","Argumentos":["-c","project_doc_max_bytes=65536"],"LimiteDeInstrucciones":65536}]"#,
    );
    carpeta.escribir(configuracion::ARCHIVO_VERSION_PERFILES, "2");

    let primera = cargar(&carpeta);
    let segunda = cargar(&carpeta);

    assert_eq!(primera, segunda);
    let codex = &segunda[0];
    assert_eq!(codex.argumentos(), ["-c", "project_doc_max_bytes=65536"]);
    assert_eq!(codex.limite_de_instrucciones(), Some(65_536));
}

#[test]
fn guardar_perfiles_escribe_lo_que_se_vuelve_a_cargar() {
    let carpeta = CarpetaTemporal::new();
    let config = Configuracion::new(carpeta.ruta());
    let mut perfiles = cargar(&carpeta);
    perfiles[0].fijar_ejecutable("~/bin/claude");
    perfiles.push(PerfilDeAgente::new(HerramientaDelPerfil::Claude, 3));

    let guardado = config.guardar_perfiles(&perfiles);

    assert!(guardado.is_ok(), "{guardado:?}");
    assert_eq!(cargar(&carpeta), perfiles);
    assert_eq!(carpeta.leer(configuracion::ARCHIVO_VERSION_PERFILES), "4");
}

#[test]
fn guardar_perfiles_invalidos_no_toca_el_archivo() {
    let carpeta = CarpetaTemporal::new();
    let config = Configuracion::new(carpeta.ruta());
    let _ = cargar(&carpeta);
    let antes = carpeta.leer(configuracion::ARCHIVO_PERFILES);
    let repetidos = vec![
        PerfilDeAgente::new(HerramientaDelPerfil::Codex, 2),
        PerfilDeAgente::new(HerramientaDelPerfil::Codex, 2),
    ];

    let guardado = config.guardar_perfiles(&repetidos);

    assert!(
        matches!(&guardado, Err(ErrorDeConfiguracion::PerfilRepetido { nombre, .. }) if nombre == "Codex 2"),
        "{guardado:?}"
    );
    assert_eq!(carpeta.leer(configuracion::ARCHIVO_PERFILES), antes);
}

#[test]
fn el_limite_por_defecto_sale_del_tipo() {
    let limite = |tipo| PerfilDeAgente::new(tipo, 1).limite_de_instrucciones();
    assert_eq!(
        limite(HerramientaDelPerfil::Gemini),
        Some(configuracion::LIMITE_DE_INSTRUCCIONES_GEMINI)
    );
    assert_eq!(
        limite(HerramientaDelPerfil::Codex),
        Some(configuracion::LIMITE_DE_INSTRUCCIONES_CODEX)
    );
    assert_eq!(limite(HerramientaDelPerfil::Claude), None);
    assert_eq!(limite(HerramientaDelPerfil::Programator), None);
    assert_eq!(configuracion::LIMITE_DE_INSTRUCCIONES_GEMINI, 24_000);
    assert_eq!(configuracion::LIMITE_DE_INSTRUCCIONES_CODEX, 32_768);
}

// El estado de perfiles y proyectos cambia con operaciones del dominio.

#[test]
fn fijar_ejecutable_recorta_la_ruta_y_en_blanco_lo_quita() {
    let mut perfil = PerfilDeAgente::new(HerramientaDelPerfil::Codex, 1);

    perfil.fijar_ejecutable("  /opt/codex/bin/codex  ");
    assert_eq!(perfil.ejecutable(), Some("/opt/codex/bin/codex"));

    for vacia in ["", "   ", "\t"] {
        perfil.fijar_ejecutable(vacia);
        assert_eq!(perfil.ejecutable(), None, "{vacia:?}");
    }
}

#[test]
fn anotar_apertura_cambia_solo_la_ultima_apertura() {
    let mut proyecto = ProyectoRegistrado::new("uno", "/p/uno", "2026-01-01T00:00:00");

    proyecto.anotar_apertura(String::from("2026-11-11T18:40:00"));

    assert_eq!(proyecto.ultima_apertura(), "2026-11-11T18:40:00");
    assert_eq!((proyecto.nombre(), proyecto.ruta()), ("uno", "/p/uno"));
}

// Ajustes de funcionamiento en datos/ajustes.json.

#[test]
fn sin_archivo_de_ajustes_se_escriben_los_valores_por_defecto() {
    let carpeta = CarpetaTemporal::new();
    let config = Configuracion::new(carpeta.ruta());

    let ajustes = config.cargar_ajustes();

    assert_eq!(ajustes.ok(), Some(Ajustes::default()));
    assert_eq!(
        Ajustes::default().tiempo_maximo_git(),
        Duration::from_secs(30)
    );
    assert_eq!(Ajustes::default().limite_de_bandeja(), 16 * 1024);
    let escrito = carpeta.leer("ajustes.json");
    assert!(
        escrito.contains("\"TiempoMaximoGitSegundos\": 30"),
        "{escrito}"
    );
    assert!(
        escrito.contains("\"LimiteDeBandejaBytes\": 16384"),
        "{escrito}"
    );
}

#[test]
fn los_ajustes_del_archivo_se_usan_y_lo_que_falta_toma_su_valor_por_defecto() {
    let carpeta = CarpetaTemporal::new();
    carpeta.escribir("ajustes.json", "\u{FEFF}{\"TiempoMaximoGitSegundos\": 90}");

    let ajustes = Configuracion::new(carpeta.ruta()).cargar_ajustes().ok();

    assert_eq!(
        ajustes.map(|a| (a.tiempo_maximo_git(), a.limite_de_bandeja())),
        Some((Duration::from_secs(90), 16 * 1024))
    );
}

#[test]
fn un_ajuste_de_cero_se_rechaza_y_dice_cual() {
    for (json, clave) in [
        (
            "{\"TiempoMaximoGitSegundos\": 0}",
            "TiempoMaximoGitSegundos",
        ),
        ("{\"LimiteDeBandejaBytes\": 0}", "LimiteDeBandejaBytes"),
    ] {
        let carpeta = CarpetaTemporal::new();
        carpeta.escribir("ajustes.json", json);

        let error = Configuracion::new(carpeta.ruta()).cargar_ajustes();

        assert!(
            matches!(&error, Err(ErrorDeConfiguracion::AjusteInvalido { .. })),
            "{error:?}"
        );
        assert!(
            error.err().is_some_and(|e| e.to_string().contains(clave)),
            "{clave}"
        );
        assert_eq!(carpeta.leer("ajustes.json"), json, "no se sobrescribe");
    }
}
