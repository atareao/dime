# Tasks

## 1. Respuesta con metadatos (TDD)

- [ ] 1.1 RED: tests de que el parseo de la respuesta expone contenido, modelo y uso. Verificación: cargo test falla en los casos nuevos.
- [ ] 1.2 GREEN: introducir el tipo de respuesta con content, model y usage y adaptar ask y sus llamadores. Verificación: cargo test en verde.
- [ ] 1.3 GREEN: usage ausente se representa como null. Verificación: test unitario.

## 2. Salida JSON

- [ ] 2.1 RED: test de que --json produce un objeto JSON válido con content, model y usage. Verificación: cargo test falla.
- [ ] 2.2 GREEN: flag --json y serialización en src/main.rs. Verificación: cargo test en verde.
- [ ] 2.3 GREEN: en modo --json, los errores van a stderr y stdout queda limpio. Verificación: test unitario.

## 3. Control de color

- [ ] 3.1 RED: tests de que --no-color y la salida canalizada no emiten ANSI. Verificación: cargo test falla o es trivialmente verde y se documenta.
- [ ] 3.2 GREEN: flag --no-color y detección de TTY con std::io::IsTerminal. Verificación: cargo test en verde.

## 4. Documentación

- [ ] 4.1 Documentar --json y --no-color en README.md. Verificación: revisión.

## 5. Verificación final

- [ ] 5.1 cargo fmt --check && cargo clippy -- -D warnings && cargo test en verde.
- [ ] 5.2 openspec validate scripting-output sin errores.