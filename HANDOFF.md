# HANDOFF

## Estado

- `main` y `development` en **`0.3.0`**; release **`v0.3.0`** publicada (binario `dime-linux-x86_64` + `.sha256`).
- Tarea pendiente: **ninguna**.

## Historial reciente

- **PLAN-001** (v0.2.0): normalización del repositorio y saneamiento del CLI — errores tipados con `thiserror`, sin `unwrap()`/`panic!` en producción, `temperature` operativa, token redactado, código muerto retirado, CI (`fmt`/`clippy`/`build`/`test`) en `main` y `development`, Dependabot, `.gitignore` protegiendo secretos. Specs OpenSpec `config` y `chat`.
- **PLAN-002** (v0.3.0): follow-ups — migración `serde_yaml` → `serde_norway`; `temperature` tipada como `f64` con deserialización tolerante (número o cadena numérica); nuevo comando `--list-models` (nueva spec `models`, soporta OpenAI `v1/models` y Ollama `api/tags`, reintroduce `models_endpoint`).

## Estado del código

- 49 tests unitarios en `src/ia.rs` y `src/main.rs`.
- `cargo fmt --check`, `cargo clippy -D warnings` y `cargo test` en verde.
- Specs vigentes en `openspec/specs/`: `config` (6), `chat` (5), `models` (2).

## Follow-ups

- Tipar `temperature` estrictamente como número en la documentación (el formato laxo de cadena sigue aceptado por compatibilidad).
- Valorar soporte multi-proveedor adicional y `--list-models` con salida formateada.
- Los warnings de `openspec validate` sobre `SHALL/MUST` son cosméticos (specs en español).

## Flujo

`feature/*|chore/*` → PR a `development` → PR `development` → `main` → tag `vX.Y.Z` (release automática). Bump de versión con `vampus upgrade --minor|--patch|--major`.
