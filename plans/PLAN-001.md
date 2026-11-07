# PLAN-001 — Normalización del repositorio `dime`

## Objetivo

Dejar el repositorio `dime` conforme a los criterios habituales del proyecto (AGENTS.md): gitflow + gitmoji, estructura `plans/` y `docs/`, flujo SDD (OpenSpec) + TDD, y saneamiento del código (calidad, manejo de errores y tests) sin romper funcionalidad.

## Estado de partida

- `/data/rust/dime` no era un repositorio git. El remoto `atareao/dime` (público) ya existe con historia en `main`.
- El árbol local coincidía con `origin/main` salvo `Cargo.toml`/`Cargo.lock` (bump de dependencias), ya comprometido en la rama de trabajo.
- `cargo check` compila con 1 warning (`dead_code`); `cargo test` ejecuta 0 tests.
- CI (`rust.yml`) sólo dispara en `main`; `release.yml` usa `softprops/action-gh-release@v1` (desactualizado).
- `.gitignore` sólo ignora `/target` → `dime.yml` (contiene el token de la API) quedaría versionado.

## Estrategia de ramas (gitflow)

- `main` — producción (= `origin/main`).
- `development` — integración (creada y publicada en el remoto).
- `feature/normalizacion-repo` — rama de trabajo, desde `development`.
- Todo entra por PR `feature/*` → `development`. Nunca commits directos a `main` ni a `development`.

## Tareas

### Bloque A — Repositorio, git y convenciones (sin código de aplicación)

- **A1.** `.gitignore`: añadir `dime.yml`, `.dime.yml` y `config.yml` para no versionar el token de la API. Mantener los `*.example.yml`.
- **A2.** Crear la estructura `plans/` y `docs/` (este `PLAN-001` y `docs/normalizacion-repo.md`).
- **A3.** `HANDOFF.md` en la raíz del proyecto.
- **A4.** CI `rust.yml`: disparar en `main` y `development` (push y pull_request); pasos: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`, `cargo test`.
- **A5.** CI `release.yml`: actualizar `softprops/action-gh-release` de `v1` a `v2` y revisar el toolchain/target musl.
- **A6.** `.github/dependabot.yml` para `cargo` y `github-actions` (semanal).
- **A7.** `justfile` con recetas `fmt`, `lint`, `test`, `check`, `ci` (opcional pero recomendado).

### Bloque B — Código (SDD + TDD) — requiere spec aprobada

- **B1.** Eliminar código muerto: `get_base_url`, `set_token`, `save` (o justificar su uso).
- **B2.** Manejo de errores tipado: sustituir `Box<dyn Error>` y los múltiples `.unwrap()` por errores tipados (`thiserror`) y deserialización con structs `serde` (`ChatResponse`, `ApiError`). Sin *panics* ante respuestas inesperadas.
- **B3.** `IABot::read_content` / `write_default` / `default()`: no invocar `process::exit` dentro de la lógica; devolver `Result` y decidir la salida en `main`.
- **B4.** Corregir mensajes de error erróneos: `config.yml` → `dime.yml`.
- **B5.** `temperature` (hoy `String` y sin uso): incluir en el cuerpo de la petición (parseando a número) o eliminar del modelo.
- **B6.** `models_endpoint` sin uso: exponerlo (p. ej. argumento `--list-models`) o eliminar.
- **B7.** Revisar los argumentos de `clap` (`conflicts_with`, `required_unless_present`) y documentarlos.
- **B8.** Tests: unitarios de configuración (defaults de serde, parseo YAML), construcción del cuerpo por proveedor (`openai`/`ollama`) y parseo de respuestas (caso OK y caso error). Objetivo: cubrir `src/ia.rs`.
- **B9.** Dejar `cargo clippy -- -D warnings` sin warnings.

### Bloque C — Verificación

- **C1.** `cargo fmt --check`, `cargo clippy -- -D warnings` y `cargo test` en verde.
- **C2.** Auditoría con `@rust-reviewer`.
- **C3.** Abrir PR de `feature/normalizacion-repo` → `development` y fusionar.

## Entregables

- `plans/PLAN-001.md` (este documento) y `docs/normalizacion-repo.md`.
- Propuesta de cambio en `openspec/changes/normalizacion-repo/`.
- Código saneado y con tests.
- CI en verde en `main` y `development`.

## Riesgos y follow-ups

- Cambiar el modelo de errores puede alterar los mensajes de salida; preservar una salida legible en terminal.
- `serde_yaml` está sin mantenimiento (deprecado): evaluar migración a `serde_yml`/`serde_norway` en un plan futuro.
- Posible añadir `--list-models`, `--temperature` por CLI y compatibilidad multi-proveedor en planes posteriores.

## Criterios de aceptación

- [ ] Repo con `main` y `development`, remoto configurado y flujo por PR.
- [ ] `.gitignore` protege los ficheros con secretos.
- [ ] CI valida `fmt`, `clippy`, `build` y `test` en `main` y `development`.
- [ ] `cargo clippy -- -D warnings` sin warnings.
- [ ] Cobertura de tests sobre `src/ia.rs`.
- [ ] `docs/` y `plans/` creados y `HANDOFF.md` presente.