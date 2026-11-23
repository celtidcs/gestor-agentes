# cargo doc con los avisos como errores

Estado: propuesta
Problema: `cargo doc` termina bien aunque la documentación tenga enlaces rotos o avisos, y la puerta del linter no los ve.
Cómo se reconoce: avisos de rustdoc en la salida de `cargo doc` con código 0.
Solución: ejecutar `cargo doc` con `RUSTDOCFLAGS=-Dwarnings` en las puertas.
Evidencia: propuesta el 2026-11-18; falta el registro de una ejecución.
Herramientas y versiones: Rust estable.
Aplica a: proyectos en Rust.
