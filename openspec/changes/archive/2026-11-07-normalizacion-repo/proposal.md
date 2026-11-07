# Proposal

## Why

El repositorio `dime` no cumple los criterios habituales del proyecto (gitflow, `plans/`, `docs/`, SDD+TDD) y arrastra deuda técnica: código muerto, manejo de errores con `unwrap()` que provoca *panics* ante respuestas inesperadas de la API, un campo documentado pero sin uso (`temperature`) y otro sin uso (`models_endpoint`), mensajes de error con un nombre de fichero incorrecto y ausencia total de tests. Además, `.gitignore` no protege `dime.yml`, que contiene el token de la API.

## What Changes

- **Repo/CI**: `.gitignore` protege los ficheros de configuración con secretos; la CI valida `fmt`, `clippy`, `build` y `test` en `main` y `development`; `release.yml` actualizado; Dependabot; estructura `plans/` y `docs/`; `HANDOFF.md`.
- **Errores**: se elimina el uso de `unwrap()` en el parseo de respuestas y se sustituye `Box<dyn Error>` por errores tipados. Ante una respuesta inesperada se devuelve un error, nunca un *panic*.
- **Configuración**: `IABot::read_content` devuelve `Result` en lugar de terminar el proceso por sí mismo; mensajes de error con la ruta correcta (`dime.yml`).
- **`temperature`**: pasa a enviarse en la petición (parseada a número).
- **BREAKING** (interno): se retiran del modelo de configuración los campos sin uso (`models_endpoint`) y los métodos muertos.
- **Tests**: se añaden tests unitarios sobre `src/ia.rs`.

## Capabilities

### New Capabilities

- `config`: descubrimiento, creación por defecto y validación del fichero de configuración.
- `chat`: construcción de la petición y tratamiento de la respuesta para los proveedores `openai` y `ollama`.

### Modified Capabilities

- Ninguna (no existe inventario previo en `openspec/specs/`).

## Impact

- Código: `src/main.rs`, `src/ia.rs`, `Cargo.toml` (nueva dependencia `thiserror`).
- CI: `.github/workflows/rust.yml`, `.github/workflows/release.yml`, `.github/dependabot.yml`.
- Repo: `.gitignore`, `plans/`, `docs/`, `HANDOFF.md`.
- El formato de `dime.yml` permanece compatible salvo la retirada de `models_endpoint` (no documentado).