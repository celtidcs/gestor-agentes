//! Barreras del arranque de los agentes: el mensaje inicial que manda el
//! lanzador, la cabecera de AGENTS.md con el arranque obligatorio en pasos, la pregunta del papel
//! al Director, el vigilante del canal por herramienta y el tamaño de AGENTS.md frente al límite
//! de agy.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::path::PathBuf;

use nucleo::plantillas::{self, DocumentoDeConsulta, EsqueletoGestor, GuionDelVigilante};
use nucleo::proyecto::catalogo;
use nucleo::proyecto::localizador::InstruccionesDelEspacioDeTrabajo;

/// Caracteres que cmd.exe interpreta en una línea de órdenes, más la comilla doble, que corta el
/// argumento. A codex.cmd, el codex que instala npm en Windows, el mensaje le llega a través de
/// cmd.exe (véase documentation/defectos-conocidos.md, «Codex instalado con npm en Windows»).
const PELIGROSOS_EN_CMD: [char; 8] = ['&', '%', '^', '|', '<', '>', '!', '"'];

/// Comillas tipográficas, que PowerShell 5.1 trata como comillas al interpretar una orden.
const COMILLAS_TIPOGRAFICAS: [char; 4] = ['\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}'];

/// Límite de bytes por archivo de reglas que lee Antigravity: «Antigravity truncates any single
/// rule file that exceeds 24,000 bytes» (https://antigravity.google/docs/rules/).
const LIMITE_DE_AGY: usize = 24_000;

/// Tamaño de unas normas de la carpeta proyecto holgadas, el doble de unas normas habituales de
/// unos 3.000 bytes.
const NORMAS_DE_CARPETA_DE_PRUEBA: usize = 6_000;

/// Título del apartado del arranque en la cabecera, que citan el mensaje inicial y el puntero.
const APARTADO_ARRANQUE: &str = "Arranque obligatorio";

#[test]
fn el_mensaje_inicial_es_una_sola_linea_sin_caracteres_peligrosos() {
    let mensaje = plantillas::mensaje_inicial();

    assert!(!mensaje.trim().is_empty());
    assert!(!mensaje.contains('\n'), "{mensaje:?}");
    for caracter in PELIGROSOS_EN_CMD.iter().chain(&COMILLAS_TIPOGRAFICAS) {
        assert!(
            !mensaje.contains(*caracter),
            "el mensaje inicial lleva {caracter:?}: {mensaje}"
        );
    }
}

#[test]
fn el_mensaje_inicial_remite_al_arranque_de_agents_md() {
    let mensaje = plantillas::mensaje_inicial();

    assert!(mensaje.contains("AGENTS.md"));
    assert!(mensaje.contains(APARTADO_ARRANQUE));
    assert!(plantillas::cabecera().contains(&format!("## {APARTADO_ARRANQUE}")));
    assert!(plantillas::puntero_claude().contains(APARTADO_ARRANQUE));
}

#[test]
fn el_arranque_esta_en_pasos_numerados_y_en_orden() {
    let cabecera = plantillas::cabecera();
    let pasos = [
        "1. **Identidad.**",
        "2. **Preséntate**",
        "3. **Tus archivos.**",
        "4. **Lee enteros:**",
        "5. **Bloque pendiente.**",
        "6. **Tu papel: Jefe de proyecto o ayudante.**",
        "7. **Encargos.**",
        "8. **Vigilante del canal.**",
        "9. **Resumen al usuario,**",
    ];
    let mut desde = 0;
    for paso in pasos {
        let posicion = cabecera[desde..]
            .find(paso)
            .unwrap_or_else(|| panic!("falta o está fuera de orden: {paso}"));
        desde += posicion + paso.len();
    }
}

#[test]
fn el_papel_se_pregunta_al_director_y_se_espera_la_respuesta() {
    let cabecera = plantillas::cabecera();

    assert!(
        cabecera.contains("\"¿Cuál es mi papel en este proyecto: Jefe de proyecto o ayudante?\"")
    );
    assert!(cabecera.contains("**espera su respuesta sin hacer nada más**"));
    assert!(cabecera.contains("`## Papeles`"));
    assert!(plantillas::esqueleto_gestor(EsqueletoGestor::Estado, None).contains("\n## Papeles\n"));
}

#[test]
fn la_cabecera_da_la_orden_del_vigilante_de_cada_guion_y_modo() {
    let cabecera = plantillas::cabecera();

    for guion in GuionDelVigilante::TODOS {
        assert!(
            cabecera.contains(guion.ruta_relativa()),
            "{}",
            guion.ruta_relativa()
        );
    }
    assert!(cabecera.contains("vigilar-canal.sh --una-vez"));
    assert!(cabecera.contains("vigilar-canal.ps1 -UnaVez"));
    assert!(cabecera.contains("Monitor"));
}

#[test]
fn los_guiones_del_vigilante_se_escriben_y_el_de_powershell_es_ascii() {
    let gestionados: Vec<&str> = catalogo::archivos_gestionados(None)
        .iter()
        .map(|a| a.ruta_relativa)
        .collect();
    for guion in GuionDelVigilante::TODOS {
        assert!(gestionados.contains(&guion.ruta_relativa()));
    }
    assert!(GuionDelVigilante::PowerShell.texto().is_ascii());
    assert!(GuionDelVigilante::Bash
        .texto()
        .starts_with("#!/usr/bin/env bash\n"));
}

#[test]
fn el_nucleo_dice_cuando_se_lee_cada_documento_de_consulta() {
    let nucleo = plantillas::nucleo_comun();
    for documento in DocumentoDeConsulta::TODOS {
        let linea = nucleo
            .lines()
            .find(|l| l.starts_with("| ") && l.contains(documento.ruta_relativa()));
        assert!(linea.is_some(), "{}", documento.ruta_relativa());
    }
}

#[test]
fn agents_md_cabe_holgado_en_lo_que_lee_agy() {
    let espacio = InstruccionesDelEspacioDeTrabajo {
        ruta: PathBuf::from("/proyectos/CLAUDE.md"),
        texto: "Norma de la carpeta proyecto.\n".repeat(NORMAS_DE_CARPETA_DE_PRUEBA / 30),
    };
    let reglas = catalogo::reglas(Some(&espacio));

    assert!(
        reglas.len() < LIMITE_DE_AGY / 2 + NORMAS_DE_CARPETA_DE_PRUEBA,
        "AGENTS.md mide {} bytes",
        reglas.len()
    );
}
