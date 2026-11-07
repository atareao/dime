# Tasks

## 1. Comprobación de estado en chat (TDD)

- [ ] 1.1 RED: tests unitarios de que `500` con cuerpo válido y `429` sin `error` fallan mencionando el código, y de que `401` con `error` conserva el mensaje del proveedor; `200` sigue siendo éxito. Verificación: `cargo test` falla.
- [ ] 1.2 GREEN: comprobar `response.status()` en `ask` y añadir la variante `IABotError::Status(u16)`. Verificación: `cargo test`.

## 2. Comprobación de estado en modelos (TDD)

- [ ] 2.1 RED: tests de que `500` con `data` válida y `401` sin `error` fallan mencionando el código en `list_models`. Verificación: `cargo test` falla.
- [ ] 2.2 GREEN: mismo criterio en `list_models`. Verificación: `cargo test`.

## 3. Integración

- [ ] 3.1 Añadir a `tests/http_mock.rs` el caso de `500` con cuerpo de chat válido (que antes pasaba como éxito) y verificar exit distinto de cero y ausencia de contenido en `stdout`.
- [ ] 3.2 Revisar que los tests existentes de `401`/`500`/cuerpos inválidos siguen en verde.

## 4. Documentación y verificación final

- [ ] 4.1 Actualizar `README.md` si menciona el tratamiento de errores del proveedor.
- [ ] 4.2 `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` en verde.
- [ ] 4.3 `openspec validate http-status-handling` sin errores.
