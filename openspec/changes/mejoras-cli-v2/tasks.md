# Tasks

## 1. Migración YAML

- [ ] 1.1 Sustituir `serde_yaml` por `serde_norway` en `Cargo.toml` y actualizar los usos en `src/ia.rs`. Verificación: `cargo build`.
- [ ] 1.2 Test de round-trip de `IABot`. Verificación: `cargo test`.

## 2. temperature f64

- [ ] 2.1 Tipo `f64` con deserializador tolerante (número o cadena numérica), default `1.0`. Verificación: tests con `0.2`, "0.2" y "alta".
- [ ] 2.2 Valor no numérico → error de configuración legible. Verificación: test unitario.
- [ ] 2.3 Adaptar `build_body`. Verificación: tests de `openai` y `ollama`.

## 3. --list-models

- [ ] 3.1 Reintroducir `models_endpoint` (default `v1/models`) y actualizar los ejemplos. Verificación: tests de default y personalización.
- [ ] 3.2 Flag `--list-models` en `clap`; pregunta no obligatoria en ese modo. Verificación: `dime --list-models` sin pregunta no da error de uso.
- [ ] 3.3 `list_models()` con DTO `ModelsResponse` y errores tipados. Verificación: tests de respuesta OK y de error de API.
- [ ] 3.4 Documentar `--list-models` en `README.md`. Verificación: revisión.

## 4. Verificación final

- [ ] 4.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` en verde.
- [ ] 4.2 `openspec validate mejoras-cli-v2` sin errores.