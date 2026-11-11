//! Lógica pura de las instrucciones de un proyecto: marcas de región, formato físico, huellas,
//! análisis de regiones, decisión de sustitución de los archivos que la aplicación escribe enteros
//! y recogida del texto propio del proyecto al pasar al archivo único. Nada de aquí toca el disco
//! ni el reloj.

pub mod analizador;
pub mod formato;
pub mod huella;
pub mod marcas;
pub mod migracion;
pub mod sustitucion;
