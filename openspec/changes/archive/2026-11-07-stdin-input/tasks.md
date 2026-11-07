# Tasks

## 1. Resolución de la entrada (TDD)

- [ ] 1.1 RED: tests de resolución de pregunta que cubran stdin sólo, marcador `-`, pregunta+contexto, stdin sin datos y uso interactivo sin pregunta. Verificación: `cargo test` falla en los casos nuevos.
- [ ] 1.2 GREEN: implementar la detección de TTY con `std::io::IsTerminal` y la resolución de la pregunta en `src/main.rs`. Verificación: `cargo test` en verde.
- [ ] 1.3 GREEN: marcador `-` para forzar la lectura desde stdin. Verificación: test unitario.
- [ ] 1.4 GREEN: composición del mensaje de usuario con el contexto de stdin en `src/ia.rs`. Verificación: test unitario.
- [ ] 1.5 GREEN: variante tipada de error de entrada, sin `panic!`. Verificación: test unitario.

## 2. Documentación

- [ ] 2.1 Documentar el uso por tubería y redirección en `README.md`. Verificación: revisión.

## 3. Verificación final

- [ ] 3.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` en verde.
- [ ] 3.2 `openspec validate stdin-input` sin errores.