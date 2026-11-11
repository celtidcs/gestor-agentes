//! Preparación de un proyecto: archivo único de reglas, puntero de Claude, documentos de
//! consulta, archivo propio del proyecto, estructura de .gestor/, archivo del sistema anterior,
//! instrucciones del espacio de trabajo, registro de huellas e informe. La lógica que decide está
//! en [`crate::instrucciones`]; aquí está la parte que toca el disco.

pub mod archivos;
pub mod catalogo;
pub mod estado;
pub mod estructura;
pub mod informe;
pub mod inicializador;
pub mod localizador;
pub mod momento;
pub mod registro_de_huellas;
pub mod sistema_anterior;
