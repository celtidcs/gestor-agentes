//! Barreras del formato físico, las marcas de región, las huellas y el analizador de regiones.
//! Proceden de las pruebas de la 2.3.2, más los vectores de SHA-256 del NIST. Las del
//! fusionador por regiones se retiraron con él en la 3.0.0: desde entonces la
//! aplicación escribe enteros sus archivos y lo propio del proyecto vive en otro archivo; ese
//! contrato lo fijan tests/migracion.rs y tests/proyecto.rs.

use nucleo::instrucciones::analizador;
use nucleo::instrucciones::formato::FormatoDeTexto;
use nucleo::instrucciones::huella::{self, HuellasPorRegion};
use nucleo::instrucciones::marcas;

const NUCLEO_V1: &str = "Núcleo versión 1\nsegunda línea";
const NUCLEO_V2: &str = "Núcleo versión 2\nsegunda línea";

/// Regiones que reconocen las pruebas del analizador; las marcas de cualquier otra, como ENTORNO
/// aquí, son texto libre.
const IDS: [&str; 2] = [marcas::CABECERA, marcas::NUCLEO_COMUN];

fn region(id: &str, contenido: &str) -> String {
    format!("{}\n{contenido}\n{}", marcas::inicio(id), marcas::fin(id))
}

// Formato y marcas.

#[test]
fn archivo_con_bom_y_crlf_se_lee_y_se_vuelve_a_escribir_byte_a_byte() {
    let mut original = vec![0xEF, 0xBB, 0xBF];
    original.extend_from_slice("línea uno\r\nlínea dos\r\n".as_bytes());

    let (texto, formato) = FormatoDeTexto::leer(&original);

    assert_eq!(texto, "línea uno\nlínea dos\n");
    assert!(formato.tiene_bom);
    assert!(formato.usa_crlf);
    assert_eq!(formato.codificar(&texto), original);
}

#[test]
fn archivo_sin_bom_con_lf_conserva_su_formato() {
    let original = b"a\nb\n".to_vec();

    let (texto, formato) = FormatoDeTexto::leer(&original);

    assert_eq!(formato, FormatoDeTexto::default());
    assert_eq!(formato.codificar(&texto), original);
}

#[test]
fn saltos_mixtos_eligen_crlf_solo_si_son_al_menos_la_mitad() {
    let (_, mitad) = FormatoDeTexto::leer(b"a\r\nb\n");
    let (_, minoria) = FormatoDeTexto::leer(b"a\r\nb\nc\n");

    assert!(mitad.usa_crlf);
    assert!(!minoria.usa_crlf);
}

#[test]
fn neutralizar_desactiva_solo_las_marcas_de_linea_completa() {
    let texto = "  <!-- GESTOR:CABECERA:INICIO -->\nmenciona <!-- NUCLEO-COMUN:FIN --> en una frase\n<!-- NUCLEO-COMUN:FIN -->";

    let neutralizado = marcas::neutralizar(texto);

    assert_eq!(
        neutralizado,
        "<!-- legado GESTOR:CABECERA:INICIO -->\nmenciona <!-- NUCLEO-COMUN:FIN --> en una frase\n<!-- legado NUCLEO-COMUN:FIN -->"
    );
}

#[test]
fn neutralizar_admite_espacios_dentro_del_comentario_y_rechaza_identificadores_invalidos() {
    let texto = "<!--UNIDAD-DE-TRABAJO:INICIO-->\n<!--   A:B-C:FIN   -->\n<!-- minusculas:INICIO -->\n<!-- A::B:FIN -->\n<!-- A:OTRO -->";

    let neutralizado = marcas::neutralizar(texto);

    assert_eq!(
        neutralizado,
        "<!-- legado UNIDAD-DE-TRABAJO:INICIO -->\n<!-- legado A:B-C:FIN -->\n<!-- minusculas:INICIO -->\n<!-- A::B:FIN -->\n<!-- A:OTRO -->"
    );
}

#[test]
fn contiene_marcas_propias_distingue_las_marcas_ajenas() {
    let ajena = "texto\n<!-- UNIDAD-DE-TRABAJO:INICIO -->\nnorma\n<!-- UNIDAD-DE-TRABAJO:FIN -->\n";

    assert!(marcas::contiene_marcas(ajena));
    assert!(!marcas::contiene_marcas_propias(ajena));
    for id in [
        marcas::CABECERA,
        marcas::NUCLEO_COMUN,
        marcas::ENTORNO,
        marcas::PUNTERO,
        marcas::LEGADO,
    ] {
        assert!(marcas::contiene_marcas_propias(&marcas::inicio(id)), "{id}");
        assert!(marcas::contiene_marcas_propias(&marcas::fin(id)), "{id}");
    }
}

// Huella: vectores de FIPS 180-4 y del NIST, y bordes del relleno calculados con hashlib.

#[test]
fn huella_coincide_con_los_vectores_del_nist() {
    assert_eq!(
        huella::calcular(""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        huella::calcular("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        huella::calcular("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    assert_eq!(
        huella::calcular(&"a".repeat(1_000_000)),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn huella_es_correcta_en_los_bordes_del_relleno() {
    let casos = [
        (
            55,
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
        ),
        (
            56,
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
        ),
        (
            63,
            "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
        ),
        (
            64,
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ),
        (
            65,
            "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
        ),
    ];
    for (longitud, esperada) in casos {
        assert_eq!(
            huella::calcular(&"a".repeat(longitud)),
            esperada,
            "{longitud}"
        );
    }
}

#[test]
fn huella_de_texto_con_acentos_usa_utf8() {
    assert_eq!(
        huella::calcular(NUCLEO_V1),
        "d2eb2fe8a575a5abaab905c0b2d73c3486ac0df9b4edf664bb2a18d4db086af5"
    );
}

#[test]
fn huellas_por_region_conservan_el_orden_y_sustituyen_sin_mover() {
    let mut huellas = HuellasPorRegion::new();
    huellas.insertar("B", "1");
    huellas.insertar("A", "2");
    huellas.insertar("B", "3");

    let entradas: Vec<_> = huellas.iter().collect();
    assert_eq!(entradas, vec![("B", "3"), ("A", "2")]);
    assert_eq!(huellas.obtener("A"), Some("2"));
    assert_eq!(huellas.obtener("C"), None);
}

// Analizador de regiones.

#[test]
fn marcas_danadas_se_explican_igual_que_en_la_2_3_2() {
    let casos = [
        (
            "<!-- NUCLEO-COMUN:INICIO -->\nsin cierre\n",
            "la región NUCLEO-COMUN no tiene marca de cierre",
        ),
        (
            "<!-- NUCLEO-COMUN:FIN -->\n",
            "línea 1: cierre de la región NUCLEO-COMUN sin su apertura",
        ),
        (
            "<!-- NUCLEO-COMUN:INICIO -->\na\n<!-- NUCLEO-COMUN:FIN -->\n<!-- NUCLEO-COMUN:INICIO -->\nb\n<!-- NUCLEO-COMUN:FIN -->\n",
            "línea 4: la región NUCLEO-COMUN aparece dos veces",
        ),
        (
            "<!-- NUCLEO-COMUN:INICIO -->\n<!-- GESTOR:CABECERA:INICIO -->\n<!-- NUCLEO-COMUN:FIN -->\n",
            "línea 2: marca de GESTOR:CABECERA dentro de la región NUCLEO-COMUN",
        ),
    ];
    for (existente, detalle) in casos {
        let error = analizador::analizar(existente, &IDS).map(|_| ());

        assert_eq!(
            error.map_err(|e| e.to_string()),
            Err(String::from(detalle)),
            "{existente}"
        );
    }
}

#[test]
fn marca_citada_dentro_de_una_frase_no_cuenta_como_marca() {
    let existente = "La región empieza con <!-- NUCLEO-COMUN:INICIO --> en su propia línea.\n";

    let segmentos = analizador::analizar(existente, &IDS);

    assert!(segmentos
        .as_ref()
        .is_ok_and(|s| !analizador::tiene_regiones(s)));
}

#[test]
fn analizar_y_componer_reconstruyen_el_archivo_exactamente() {
    let original = format!(
        "previo\n\n{}\nlibre\n  {}  \nfinal\n",
        region(marcas::CABECERA, "a\n\nb"),
        marcas::inicio(marcas::ENTORNO)
    );
    let segmentos = analizador::analizar(&original, &IDS).unwrap_or_default();

    assert_eq!(analizador::componer(&segmentos), original);
}

#[test]
fn marca_con_espacios_alrededor_cuenta_como_marca_de_linea_completa() {
    let existente = format!(
        "  {}\t\n{NUCLEO_V2}\n {} \n",
        marcas::inicio(marcas::NUCLEO_COMUN),
        marcas::fin(marcas::NUCLEO_COMUN)
    );
    let segmentos = analizador::analizar(&existente, &IDS);

    assert!(segmentos
        .as_ref()
        .is_ok_and(|s| analizador::tiene_regiones(s)));
    assert!(marcas::es_fin(
        " <!-- NUCLEO-COMUN:FIN -->\u{3000}",
        marcas::NUCLEO_COMUN
    ));
}
