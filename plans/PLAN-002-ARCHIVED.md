# PLAN-002 — Follow-ups del CLI `dime`

## Objetivo

Cerrar los follow-ups de `PLAN-001`: migrar `serde_yaml` (sin mantenimiento), tipar `temperature` como número y añadir el comando `--list-models`.

## Estado de partida

- `main` y `development` en `0.2.0`; release `v0.2.0` publicada.
- Dependencia `serde_yaml 0.9.34+deprecated` en uso.
- `temperature` es `String` y se parsea al construir la petición.
- `models_endpoint` se retiró y no hay forma de listar los modelos.

## Ramas

- `feature/mejoras-cli-v2` desde `development` → PR a `development` → PR `development` → `main` → release `0.3.0`.

## Tareas

### Bloque A — Migración `serde_yaml` → `serde_norway`

- **A1.** Sustituir la dependencia `serde_yaml` por `serde_norway` en `Cargo.toml` y actualizar los usos (`to_string`, `from_str`). Verificación: `cargo build`.
- **A2.** Tests de round-trip (serializar y deserializar `IABot` conserva los valores). Verificación: `cargo test`.
- **A3.** `cargo fmt` y `cargo clippy -- -D warnings`.

### Bloque B — `temperature` como `f64`

- **B1.** Cambiar el tipo a `f64` con deserializador tolerante que acepte número o cadena numérica; por defecto `1.0`. Verificación: tests con `0.2`, `"0.2"` y valor no numérico.
- **B2.** Valor no numérico → error de configuración legible en la carga (no al enviar). Verificación: test unitario.
- **B3.** Adaptar `build_body` (la temperatura ya es numérica). Verificación: tests de `openai`/`ollama`.

### Bloque C — `--list-models`

- **C1.** Reintroducir `models_endpoint` (por defecto `v1/models`) en el modelo y en los ejemplos. Verificación: test de default y de personalización.
- **C2.** Bandera `--list-models` en `clap`; en ese modo no se exige la pregunta. Verificación: `--list-models` sin pregunta funciona; consulta sin pregunta → error de uso.
- **C3.** `list_models()` que consulta `{protocolo}://{base_url}/{models_endpoint}` con la cabecera de autorización según proveedor y extrae los `id`. Verificación: tests de parseo de la lista y de error de API.
- **C4.** Documentar en `README.md` y ejemplos. Verificación: revisión.

### Bloque D — Verificación

- **D1.** `cargo fmt --check && cargo clippy -- -D warnings && cargo test` en verde.
- **D2.** Auditoría con `@rust-reviewer`.
- **D3.** PR a `development`, luego `development` → `main` y release `0.3.0`.

## Criterios de aceptación

- [ ] `serde_norway` sustituye a `serde_yaml` con round-trip verificado.
- [ ] `temperature` es `f64` tolerante (número o cadena numérica).
- [ ] `dime --list-models` lista los identificadores y maneja errores sin panic.
- [ ] `clippy -D warnings` sin warnings y tests ampliados.
- [ ] Release `0.3.0` publicada.
