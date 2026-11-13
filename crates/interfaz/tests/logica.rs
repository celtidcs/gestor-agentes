//! Barreras de las piezas puras de la interfaz: fechas, orden, rutas de arranque y perfiles.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use chrono::{FixedOffset, TimeZone};
use gestor_agentes::perfiles::Disponibilidad;
use gestor_agentes::{fecha, icono, orden, perfiles, rutas, textos};
use lanzador::{Herramienta, Sistema};
use nucleo::agentes::TipoAgente;
use nucleo::configuracion::{HerramientaDelPerfil, PerfilDeAgente};

fn zona(horas: i32) -> FixedOffset {
    FixedOffset::east_opt(horas * 3600).unwrap_or_else(|| unreachable!())
}

// Fechas

#[test]
fn fecha_con_desfase_se_muestra_en_la_zona_local() {
    let texto = fecha::mostrar("2026-11-10T12:30:15.1234567+01:00", &zona(3));
    assert_eq!(texto, "10/11/2026 14:30");
}

#[test]
fn fecha_en_utc_con_z_tambien_se_lee() {
    assert_eq!(
        fecha::mostrar("2026-01-02T23:05:00Z", &zona(1)),
        "03/01/2026 00:05"
    );
}

#[test]
fn fecha_sin_desfase_se_muestra_tal_cual() {
    assert_eq!(
        fecha::mostrar("2025-03-04T05:06:07", &zona(5)),
        "04/03/2025 05:06"
    );
    assert_eq!(
        fecha::mostrar("2025-03-04T05:06:07.5", &zona(5)),
        "04/03/2025 05:06"
    );
}

#[test]
fn fecha_invalida_o_vacia_no_muestra_nada() {
    assert_eq!(fecha::mostrar("", &zona(0)), "");
    assert_eq!(fecha::mostrar("ayer", &zona(0)), "");
}

#[test]
fn fecha_se_escribe_con_siete_decimales_y_desfase() {
    let momento = zona(1)
        .with_ymd_and_hms(2026, 11, 10, 18, 20, 5)
        .single()
        .unwrap_or_else(|| unreachable!())
        + chrono::Duration::nanoseconds(123_456_789);
    assert_eq!(fecha::iso(&momento), "2026-11-10T18:20:05.1234567+01:00");
}

#[test]
fn fecha_sin_fraccion_lleva_siete_ceros_y_desfase_negativo() {
    let momento = zona(-5)
        .with_ymd_and_hms(2026, 1, 2, 3, 4, 5)
        .single()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(fecha::iso(&momento), "2026-01-02T03:04:05.0000000-05:00");
}

#[test]
fn fecha_escrita_se_vuelve_a_leer_igual() {
    let momento = zona(2)
        .with_ymd_and_hms(2026, 7, 8, 9, 10, 11)
        .single()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(
        fecha::mostrar(&fecha::iso(&momento), &zona(2)),
        "08/07/2026 09:10"
    );
}

// Orden

fn ordenados(nombres: &[&str]) -> Vec<String> {
    let mut lista: Vec<String> = nombres.iter().map(|n| (*n).to_owned()).collect();
    lista.sort_by(|a, b| orden::comparar_nombres(a, b));
    lista
}

#[test]
fn orden_ignora_mayusculas_y_tildes() {
    assert_eq!(
        ordenados(&["beta", "Álvaro", "alba", "Alzar"]),
        ["alba", "Álvaro", "Alzar", "beta"]
    );
}

#[test]
fn orden_pone_la_enie_entre_la_ene_y_la_o() {
    assert_eq!(
        ordenados(&["oso", "ñu", "nube", "Ñandú"]),
        ["nube", "Ñandú", "ñu", "oso"]
    );
}

#[test]
fn orden_desempata_por_tilde_y_luego_por_texto_exacto() {
    // «Pena» y «pena» solo difieren en la mayúscula: decide el texto exacto, y «P» va antes.
    assert_eq!(
        ordenados(&["peña", "pena", "Pena"]),
        ["Pena", "pena", "peña"]
    );
    assert_eq!(ordenados(&["cafe", "café"]), ["cafe", "café"]);
}

#[test]
fn ruta_igual_sin_distinguir_mayusculas() {
    assert!(orden::misma_ruta(r"C:\Proyectos\Uno", r"c:\proyectos\UNO"));
    assert!(orden::misma_ruta("/home/Ñu", "/home/ñU"));
    assert!(!orden::misma_ruta("/home/uno", "/home/uno/"));
    assert!(!orden::misma_ruta("/home/uno", "/home/dos"));
}

// Rutas de arranque

fn variables(pares: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<OsString> {
    move |nombre| {
        pares
            .iter()
            .find(|(clave, _)| *clave == nombre)
            .map(|(_, valor)| OsString::from(valor))
    }
}

#[test]
fn datos_junto_al_ejecutable() {
    assert_eq!(
        rutas::carpeta_de_datos(Path::new("/opt/gestor/gestor-agentes")),
        Some(PathBuf::from("/opt/gestor").join("datos"))
    );
    assert_eq!(rutas::carpeta_de_datos(Path::new("")), None);
}

#[test]
fn datos_de_aplicacion_en_windows_es_appdata() {
    let resultado = rutas::datos_de_aplicacion(
        Sistema::Windows,
        variables(&[("APPDATA", r"C:\Users\u\AppData\Roaming"), ("HOME", "/no")]),
    );
    assert_eq!(
        resultado,
        Some(PathBuf::from(r"C:\Users\u\AppData\Roaming"))
    );
    assert_eq!(
        rutas::datos_de_aplicacion(Sistema::Windows, variables(&[("APPDATA", "")])),
        None
    );
}

#[test]
fn datos_de_aplicacion_en_linux_prefiere_xdg_absoluta() {
    let con_xdg = variables(&[("XDG_CONFIG_HOME", "/xdg"), ("HOME", "/home/u")]);
    assert_eq!(
        rutas::datos_de_aplicacion(Sistema::Linux, con_xdg),
        Some(PathBuf::from("/xdg"))
    );

    let xdg_relativa = variables(&[("XDG_CONFIG_HOME", "relativa"), ("HOME", "/home/u")]);
    assert_eq!(
        rutas::datos_de_aplicacion(Sistema::Linux, xdg_relativa),
        Some(PathBuf::from("/home/u").join(".config"))
    );

    let solo_home = variables(&[("XDG_CONFIG_HOME", ""), ("HOME", "/home/u")]);
    assert_eq!(
        rutas::datos_de_aplicacion(Sistema::Linux, solo_home),
        Some(PathBuf::from("/home/u").join(".config"))
    );
    assert_eq!(
        rutas::datos_de_aplicacion(Sistema::Linux, variables(&[])),
        None
    );
}

// Perfiles

fn perfil(tipo: HerramientaDelPerfil, instancia: u32) -> PerfilDeAgente {
    PerfilDeAgente::new(tipo, instancia)
}

#[test]
fn etiquetas_sin_la_aplicacion_porque_todos_se_abren_en_terminal() {
    assert_eq!(
        perfiles::etiqueta(&perfil(HerramientaDelPerfil::Claude, 2)),
        "Abrir · Claude 2"
    );
    assert_eq!(
        perfiles::etiqueta(&perfil(HerramientaDelPerfil::Codex, 1)),
        "Abrir · Codex 1"
    );
    assert_eq!(
        perfiles::etiqueta(&perfil(HerramientaDelPerfil::Gemini, 1)),
        "Abrir · Gemini 1"
    );
    assert_eq!(
        perfiles::etiqueta(&perfil(HerramientaDelPerfil::Programator, 1)),
        "Abrir · Programator 1"
    );
}

#[test]
fn cada_herramienta_del_perfil_va_a_la_suya_del_lanzador() {
    let pares = [
        (HerramientaDelPerfil::Claude, Herramienta::Claude),
        (HerramientaDelPerfil::Codex, Herramienta::Codex),
        (HerramientaDelPerfil::Gemini, Herramienta::Gemini),
        (HerramientaDelPerfil::Programator, Herramienta::Programator),
    ];
    for (del_perfil, del_lanzador) in pares {
        assert_eq!(perfiles::herramienta(del_perfil), del_lanzador);
    }
}

#[test]
fn identidad_que_se_prepara() {
    let claude = perfiles::identidad_a_preparar(&perfil(HerramientaDelPerfil::Claude, 2))
        .unwrap_or_else(|| unreachable!());
    assert_eq!((claude.tipo(), claude.instancia()), (TipoAgente::Claude, 2));
    assert!(
        perfiles::identidad_a_preparar(&perfil(HerramientaDelPerfil::Programator, 1)).is_none()
    );
    assert!(perfiles::identidad_a_preparar(&perfil(HerramientaDelPerfil::Gemini, 0)).is_none());
}

#[test]
fn peticion_expande_la_carpeta_personal() {
    let claude = perfil(HerramientaDelPerfil::Claude, 2)
        .con_directorio("~/.claude2")
        .con_ejecutable("~/bin/claude");
    let casa = Path::new("/home/u");
    let peticion = perfiles::peticion(&claude, "Uno", Path::new("/p/uno"), Some(casa));
    assert_eq!(peticion.herramienta, Herramienta::Claude);
    assert_eq!(peticion.identidad, "Claude 2");
    assert_eq!(peticion.nombre_proyecto, "Uno");
    assert_eq!(peticion.carpeta_proyecto, PathBuf::from("/p/uno"));
    assert_eq!(
        peticion.directorio_configuracion,
        Some(casa.join(".claude2"))
    );
    assert_eq!(peticion.ejecutable, Some(casa.join("bin/claude")));
}

#[test]
fn peticion_lleva_el_mensaje_inicial_del_nucleo() {
    for herramienta in [
        HerramientaDelPerfil::Claude,
        HerramientaDelPerfil::Codex,
        HerramientaDelPerfil::Gemini,
    ] {
        let peticion = perfiles::peticion(
            &perfil(herramienta, 1),
            "Uno",
            Path::new("/p/uno"),
            Some(Path::new("/home/u")),
        );
        assert_eq!(
            peticion.mensaje_inicial,
            Some(nucleo::plantillas::mensaje_inicial())
        );
    }
}

#[test]
fn peticion_sin_carpeta_personal_deja_la_virgulilla_a_la_vista() {
    let claude = perfil(HerramientaDelPerfil::Claude, 1).con_directorio("~/.claude");
    let peticion = perfiles::peticion(&claude, "Uno", Path::new("/p/uno"), None);
    assert_eq!(
        peticion.directorio_configuracion,
        Some(Path::new("~").join(".claude"))
    );
    assert_eq!(peticion.ejecutable, None);
}

// Textos con datos

#[test]
fn textos_de_estado_como_la_2_3_2() {
    assert_eq!(
        textos::proyectos_registrados(3),
        "3 proyecto(s) registrado(s)"
    );
    assert_eq!(
        textos::abierto("Claude 2", "Uno", 0),
        "Claude 2 abierto en Uno"
    );
    assert_eq!(
        textos::abierto("Claude 2", "Uno", 4),
        "Claude 2 abierto en Uno; 4 cambio(s) en sus archivos de gestión"
    );
    assert_eq!(
        textos::preparando("Uno", "Codex 1"),
        "Preparando Uno para Codex 1…"
    );
}

// Argumentos, aviso de instrucciones cortadas y disponibilidad.

fn con_argumentos(tipo: HerramientaDelPerfil, argumentos: &[&str]) -> PerfilDeAgente {
    perfil(tipo, 1).con_argumentos(argumentos.iter().copied())
}

#[test]
fn la_peticion_lleva_los_argumentos_del_perfil() {
    let gemini = con_argumentos(
        HerramientaDelPerfil::Gemini,
        &["--model", "m", "--effort", "high"],
    );
    let peticion = perfiles::peticion(&gemini, "Uno", Path::new("/p/uno"), None);
    assert_eq!(peticion.herramienta, Herramienta::Gemini);
    assert_eq!(
        peticion.argumentos_extra,
        ["--model", "m", "--effort", "high"]
    );
    let claude = perfiles::peticion(
        &perfil(HerramientaDelPerfil::Claude, 1),
        "Uno",
        Path::new("/p/uno"),
        None,
    );
    assert!(claude.argumentos_extra.is_empty());
}

#[test]
fn sin_ejecutable_no_esta_disponible_aunque_tenga_cuenta() {
    assert_eq!(
        perfiles::disponibilidad(None, None),
        Disponibilidad::SinEjecutable
    );
    assert_eq!(
        perfiles::disponibilidad(None, Some((PathBuf::from("/h/.claude2"), true))),
        Disponibilidad::SinEjecutable
    );
}

#[test]
fn con_ejecutable_falta_la_cuenta_si_su_carpeta_no_existe() {
    let programa = PathBuf::from("/bin/claude");
    assert_eq!(
        perfiles::disponibilidad(Some(programa.clone()), None),
        Disponibilidad::Disponible(programa.clone())
    );
    assert_eq!(
        perfiles::disponibilidad(
            Some(programa.clone()),
            Some((PathBuf::from("/h/.claude2"), true))
        ),
        Disponibilidad::Disponible(programa.clone())
    );
    assert_eq!(
        perfiles::disponibilidad(Some(programa), Some((PathBuf::from("/h/.claude2"), false))),
        Disponibilidad::SinCuenta(PathBuf::from("/h/.claude2"))
    );
    assert!(Disponibilidad::Disponible(PathBuf::new()).disponible());
    assert!(!Disponibilidad::SinEjecutable.disponible());
    assert!(!Disponibilidad::SinCuenta(PathBuf::new()).disponible());
}

#[test]
fn solo_claude_y_codex_desde_la_segunda_exigen_su_carpeta_de_cuenta() {
    let casa = Path::new("/h");
    let con_carpeta =
        |tipo, instancia, carpeta: &str| perfil(tipo, instancia).con_directorio(carpeta);
    let exigida = |p: &PerfilDeAgente| perfiles::carpeta_de_cuenta_exigida(p, Some(casa));

    assert_eq!(
        exigida(&con_carpeta(HerramientaDelPerfil::Claude, 2, "~/.claude2")),
        Some(casa.join(".claude2"))
    );
    assert_eq!(
        exigida(&con_carpeta(HerramientaDelPerfil::Codex, 3, "~/.codex3")),
        Some(casa.join(".codex3"))
    );
    assert_eq!(
        exigida(&con_carpeta(HerramientaDelPerfil::Claude, 1, "~/.claude")),
        None
    );
    assert_eq!(exigida(&perfil(HerramientaDelPerfil::Claude, 2)), None);
    assert_eq!(
        exigida(&con_carpeta(HerramientaDelPerfil::Gemini, 2, "~/.gemini2")),
        None
    );
}

#[test]
fn el_ejecutable_fijado_se_expande_y_sin_ruta_no_hay() {
    let mut programator = perfil(HerramientaDelPerfil::Programator, 1);
    assert_eq!(
        perfiles::ejecutable_fijado(&programator, Some(Path::new("/h"))),
        None
    );
    programator.fijar_ejecutable("~/App/programator");
    assert_eq!(
        perfiles::ejecutable_fijado(&programator, Some(Path::new("/h"))),
        Some(Path::new("/h").join("App/programator"))
    );
}

#[test]
fn textos_de_disponibilidad() {
    assert_eq!(textos::disponibilidad(None), "Buscando agentes…");
    assert_eq!(
        textos::disponibilidad(Some(&Disponibilidad::SinEjecutable)),
        "No encontrado"
    );
    assert!(
        textos::disponibilidad(Some(&Disponibilidad::Disponible(PathBuf::from("/bin/x"))))
            .starts_with("Encontrado: ")
    );
    assert!(
        textos::disponibilidad(Some(&Disponibilidad::SinCuenta(PathBuf::from("/h/.c2"))))
            .starts_with("Falta la carpeta de su cuenta: ")
    );
}

#[test]
fn archivo_vigilado_segun_la_herramienta() {
    let vigilado = |p: &PerfilDeAgente| perfiles::archivo_vigilado(p);
    assert_eq!(
        vigilado(&perfil(HerramientaDelPerfil::Gemini, 1)),
        Some("AGENTS.md")
    );
    assert_eq!(
        vigilado(&perfil(HerramientaDelPerfil::Codex, 1)),
        Some("AGENTS.md")
    );
    assert_eq!(vigilado(&perfil(HerramientaDelPerfil::Claude, 1)), None);
    assert_eq!(
        vigilado(&perfil(HerramientaDelPerfil::Programator, 1)),
        None
    );
}

#[test]
fn aviso_solo_si_el_archivo_pasa_del_limite() {
    let gemini = perfil(HerramientaDelPerfil::Gemini, 1);
    assert_eq!(
        perfiles::aviso_de_tamano(&gemini, Some(40_123)).as_deref(),
        Some("AVISO: AGENTS.md mide 40.123 bytes y Gemini solo lee los primeros 24.000")
    );
    assert_eq!(perfiles::aviso_de_tamano(&gemini, Some(24_000)), None);
    assert_eq!(perfiles::aviso_de_tamano(&gemini, None), None);
    let codex = perfil(HerramientaDelPerfil::Codex, 1);
    assert_eq!(
        perfiles::aviso_de_tamano(&codex, Some(32_769)).as_deref(),
        Some("AVISO: AGENTS.md mide 32.769 bytes y Codex solo lee los primeros 32.768")
    );
    let claude = perfil(HerramientaDelPerfil::Claude, 1);
    assert_eq!(perfiles::aviso_de_tamano(&claude, Some(1_000_000)), None);
}

#[test]
fn el_limite_del_perfil_manda_sobre_el_de_su_tipo() {
    let codex = con_argumentos(
        HerramientaDelPerfil::Codex,
        &["-c", "project_doc_max_bytes=65536"],
    )
    .con_limite_de_instrucciones(65_536);
    assert_eq!(perfiles::aviso_de_tamano(&codex, Some(40_000)), None);
    assert_eq!(
        perfiles::aviso_de_tamano(&codex, Some(1_234_567)).as_deref(),
        Some("AVISO: AGENTS.md mide 1.234.567 bytes y Codex solo lee los primeros 65.536")
    );
}

#[test]
fn el_aviso_se_suma_al_estado() {
    assert_eq!(
        textos::con_aviso("Abierto", "AVISO: x"),
        "Abierto. AVISO: x"
    );
}

// Icono de la ventana.

/// Lado del icono de la 2.3.2, en píxeles.
const LADO_DEL_ICONO: u32 = 256;

#[test]
fn el_icono_incluido_se_decodifica_a_256_por_256_en_rgba() {
    let imagen =
        icono::decodificar(icono::ICONO_PNG).unwrap_or_else(|error| unreachable!("{error}"));
    assert_eq!(
        (imagen.width, imagen.height),
        (LADO_DEL_ICONO, LADO_DEL_ICONO)
    );
    assert_eq!(
        imagen.rgba.len(),
        (LADO_DEL_ICONO * LADO_DEL_ICONO * 4) as usize
    );
    assert!(imagen.rgba.iter().any(|&byte| byte != 0));
}

#[test]
fn unos_bytes_que_no_son_png_dan_error_y_no_panico() {
    assert!(icono::decodificar(b"no es un png").is_err());
    assert!(icono::decodificar(&icono::ICONO_PNG[..100]).is_err());
    assert!(icono::decodificar(&[]).is_err());
}
