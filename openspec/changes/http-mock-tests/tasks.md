# Tasks

## 1. Dependencia y andamiaje

- [ ] 1.1 Añadir `wiremock` a `[dev-dependencies]` (`cargo add --dev wiremock`). Verificación: `cargo build` sigue compilando el binario sin código de test.

## 2. Petición de chat (TDD)

- [ ] 2.1 RED: test que afirma `POST` en la ruta del `endpoint`, `Authorization: Bearer <token>` y cuerpo con `model`, mensajes `system`/`user` y `temperature`; y respuesta en `stdout` con código 0. Verificación: `cargo test` falla si el contrato no se cumple.
- [ ] 2.2 RED: test de que sin token no se envía la cabecera `Authorization`.
- [ ] 2.3 GREEN: ajustar lo necesario para que ambos pasen.

## 3. Listado de modelos (TDD)

- [ ] 3.1 RED: test de que `--list-models` llama a la ruta del `models_endpoint` y muestra los identificadores devueltos. Verificación: `cargo test`.
- [ ] 3.2 GREEN: ajustar lo necesario.

## 4. Errores HTTP (TDD)

- [ ] 4.1 RED: tests de `401`, `500`, cuerpo sin el campo esperado y cuerpo no-JSON → código distinto de cero, mensaje legible, sin *panic*. Verificación: `cargo test`.
- [ ] 4.2 GREEN: ajustar lo necesario.

## 5. Documentación y verificación final

- [ ] 5.1 Nota en `README.md` sobre `wiremock` como dependencia de desarrollo y cómo ejecutar los tests.
- [ ] 5.2 `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` en verde.
- [ ] 5.3 `openspec validate http-mock-tests` sin errores.
