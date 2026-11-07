# Tasks

## 1. Flags y overrides (TDD)

- [ ] 1.1 RED: tests de `apply_overrides` (modelo, temperatura y proveedor) y de que el fichero no se reescribe. Verificación: `cargo test` falla en los casos nuevos.
- [ ] 1.2 GREEN: flags `--model`/`--temperature`/`--provider` y alias `--system` en `clap`. Verificación: `cargo test`.
- [ ] 1.3 GREEN: `IABot::apply_overrides` aplicado antes de `validate()`; proveedor inválido → error legible. Verificación: test unitario.
- [ ] 1.4 GREEN: `--temperature` no numérica → error de uso de `clap`. Verificación: test unitario.

## 2. Documentación

- [ ] 2.1 Documentar los overrides por CLI en `README.md`. Verificación: revisión.

## 3. Verificación final

- [ ] 3.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` en verde.
- [ ] 3.2 `openspec validate cli-overrides` sin errores.
