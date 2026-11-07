# Feature: normalización del repositorio `dime`

## Qué es

Trabajo de saneamiento integral del repositorio `dime`: normalización de la organización gitflow, la integración continua y la estructura documental (`plans/`, `docs/`, `HANDOFF.md`), junto con un refactor del código según las capacidades `config` y `chat`. El objetivo es dejar el repo conforme a los criterios del proyecto (AGENTS.md) sin romper funcionalidad.

## Ficheros implicados

**Repositorio / CI / convenciones (Bloque A):**

- `.gitignore` — protege `dime.yml`, `.dime.yml`, `config.yml` (secretos) y mantiene los `*.example.yml`.
- `.github/workflows/rust.yml` — CI que valida `fmt`, `clippy`, `build` y `test` en `main` y `development`.
- `.github/workflows/release.yml` — release con `softprops/action-gh-release@v2`.
- `.github/dependabot.yml` — actualizaciones semanales para `cargo` y `github-actions`.
- `plans/PLAN-001.md`, `docs/normalizacion-repo.md`, `HANDOFF.md`.

**Código (Bloque B, según specs `config` y `chat`):**

- `src/main.rs` — argumentos de `clap`, flujo de configuración y decisión de salida.
- `src/ia.rs` — construcción de la petición y parseo de la respuesta por proveedor.
- `Cargo.toml` — dependencia `thiserror`.

**Especificaciones:**

- `openspec/changes/normalizacion-repo/specs/config/spec.md`
- `openspec/changes/normalizacion-repo/specs/chat/spec.md`

## Cómo funciona

### Normalización gitflow / CI

- **Ramas:** `main` (producción) y `development` (integración); el trabajo entra por PR `feature/*` → `development`, y los hotfix por `hotfix/*` → `main`. Nunca hay commits directos a `main` ni a `development`.
- **CI (`rust.yml`):** se dispara en `push` y `pull_request` para `main` y `development`. Usa `actions-rust-lang/setup-rust-toolchain@v1` con `toolchain: stable` y `components: rustfmt, clippy`, y ejecuta `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --verbose` y `cargo test --verbose`.
- **Release (`release.yml`):** se dispara por tags; construye el binario estático musl y publica con `softprops/action-gh-release@v2`.
- **Dependabot:** PRs semanales para `cargo` y `github-actions` (límite 5 PRs abiertos por ecosistema).

### Refactor del código (specs `config` y `chat`)

- **`config`:** descubrimiento, creación por defecto y validación del fichero de configuración. `IABot::read_content` devuelve `Result` en lugar de terminar el proceso por sí mismo; la decisión de salida se toma en `main`. Los mensajes de error apuntan a la ruta correcta (`dime.yml`).
- **`chat`:** construcción de la petición y tratamiento de la respuesta para los proveedores `openai` y `ollama`. Se elimina el uso de `unwrap()` en el parseo y se sustituye `Box<dyn Error>` por errores tipados (`thiserror`), de modo que ante una respuesta inesperada se devuelve un error y nunca se produce un *panic*. Se retiran del modelo los campos sin uso (`models_endpoint`) y el código muerto (`get_base_url`, `set_token`, `save`). `temperature` pasa a enviarse en la petición.
- **Tests:** unitarios sobre `src/ia.rs` (defaults de serde y parseo YAML de configuración, construcción del cuerpo por proveedor y parseo de respuestas en casos OK y error).

## Configuración

- **Ficheros de configuración ignorados por git:** `dime.yml`, `.dime.yml`, `config.yml`.
- **Plantillas versionadas:** `config.ollama.example.yml`, `config.openai.example.yml`.
- **CI:** `env: CARGO_TERM_COLOR: always`; sin secretos adicionales salvo `secrets.GITHUB_TOKEN` en el release.
- **Dependabot:** `open-pull-requests-limit: 5`, intervalo `weekly`.

## Decisiones

- Ignorar en `.gitignore` solo las configuraciones reales y no las de ejemplo, para no filtrar la API key sin perder las plantillas.
- Toda la validación de calidad (`fmt`, `clippy` con `-D warnings`, `build`, `test`) ocurre en CI, no solo en local.
- Migrar los flujos de error a `Result` + `thiserror` para evitar *panics* ante respuestas inesperadas de la API y conservar mensajes legibles en terminal.
- Retirar del modelo los campos y métodos sin uso en lugar de mantenerlos, salvo `temperature`, que pasa a tener efecto real.

## Follow-ups

- **Migración de `serde_yaml`**: está deprecado; evaluar `serde_yml`/`serde_norway` en un plan futuro.
- **`--list-models`**: exponer `models_endpoint` como argumento CLI (hoy retirado del modelo) si se recupera la funcionalidad.
- **Tipar `temperature`**: pasar de `String` a un tipo numérico validado (p. ej. `f32`/`f64`) y, opcionalmente, exponerlo por CLI.
- Considerar un `justfile` con recetas `fmt`, `lint`, `test`, `check`, `ci` (opcional, recomendado en `PLAN-001`).
