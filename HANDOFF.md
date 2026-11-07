# HANDOFF

## Estado

- `development` en **`0.4.0`** con las **Fases 1–6 de PLAN-003** mergeadas (`cli-overrides` PR #29 squash; `stdin-input` PR #26; `scripting-output` PR #27 squash; `shell-completions` PR #31 squash; `context-input` PR #33 squash; `command-mode` PR #35 squash). `main` sigue en `0.3.0`; release **`v0.3.0`** publicada (binario `dime-linux-x86_64` + `.sha256`).
- Tarea pendiente: **resto de PLAN-003** — Fase 7 (`http-mock-tests`).

## Historial reciente

- **PLAN-001** (v0.2.0): normalización del repositorio y saneamiento del CLI — errores tipados con `thiserror`, sin `unwrap()`/`panic!` en producción, `temperature` operativa, token redactado, código muerto retirado, CI (`fmt`/`clippy`/`build`/`test`) en `main` y `development`, Dependabot, `.gitignore` protegiendo secretos. Specs OpenSpec `config` y `chat`.
- **PLAN-002** (v0.3.0): follow-ups — migración `serde_yaml` → `serde_norway`; `temperature` tipada como `f64` con deserialización tolerante (número o cadena numérica); nuevo comando `--list-models` (nueva spec `models`, soporta OpenAI `v1/models` y Ollama `api/tags`, reintroduce `models_endpoint`).
- **PLAN-003 Fase 2** (`stdin-input`, spec `chat`): entrada por stdin/pipes — pregunta desde stdin o marcador `-`, stdin canalizado como contexto del mensaje de usuario, error tipado `IABotError::Input`, `--list-models` ignora stdin. Lógica pura `resolve_input`/`StdinState`/`ResolvedInput` en `src/main.rs` y `compose_user_message` en `src/ia.rs`.
- **PLAN-003 Fase 3** (`scripting-output`, nueva spec `output`): `--json` (objeto `{content, model, usage}`, `model` con fallback al configurado, `usage` `null` si falta), `--no-color`, color automático sólo en TTY, logs `tracing` a `stderr`. Tipo `ChatResponse`, `render_output`/`should_colorize`, tests de integración `tests/scripting_output.rs` con mock HTTP.
- **PLAN-003 Fase 4** (`shell-completions`, nueva spec `completions`): flag `--completions <bash|zsh|fish>` que imprime en `stdout` el script de autocompletado del binario y termina con código 0, sin pregunta, sin leer `stdin` y sin crear `dime.yml`. `ValueEnum` propio `CompletionShell { Bash, Zsh, Fish }` mapeado a `clap_complete::Shell` (elvish/powershell → error de uso, código 2), `conflicts_with_all = ["question", "question_flag", "list_models"]`, ejecución temprana al inicio de `main` y generación factorizada en `generate_completions(shell, writer)`. Nueva dependencia `clap_complete = "4"`. Sin re-bump: se mantiene `0.4.0`.
- **PLAN-003 Fase 5** (`context-input`, nuevas specs `attachments` y `clipboard`; modifica `chat`): `--file <ruta>` (repetible) añade el contenido de ficheros como contexto del mensaje de usuario, tras el de `stdin`, con cabecera `## <ruta>` (fichero inexistente/no UTF-8 → `IABotError::Attachment`, sin petición). `--image <ruta>` (repetible, multimodal) codifica base64 + MIME por extensión (`png`/`jpg`/`jpeg`/`gif`/`webp`); en `openai` el `content` pasa a array de content parts sólo con imágenes (sin imágenes sigue cadena), en `ollama` van en `images`. `--copy` copia el campo `content` al portapapeles con `arboard` (`--json` incluido); *best-effort*: sin portapapeles avisa por `stderr` y sale 0, con el guardián del portapapeles vivo hasta el final de `main`. Funciones `compose_context`/`read_files`/`read_images`/`mime_from_path`/`openai_content` (`src/ia.rs`) y `compose_user_text`/`copy_to_clipboard`/`clipboard_text` (`src/main.rs`). Nuevas deps `arboard` y `base64`. Sin re-bump: se mantiene `0.4.0`.
- **PLAN-003 Fase 6** (`command-mode`, nueva spec `command`): `--cmd <objetivo>` usa el rol `COMMAND_SYSTEM_ROLE` (pide sólo el comando) e imprime la respuesta limpia con `clean_command` (fences multilínea/de una línea y backticks envolventes; fence sólo-lenguaje → cadena vacía). Incompatible con pregunta, `--list-models` y `--completions`; no usa `stdin` como contexto. Sin `--exec` **nunca** se ejecuta; con `--exec` se muestra el comando y se pide confirmación con default **negativo**; en `stdin` no-TTY sin `--yes` aborta con código ≠ 0; `--yes` ejecuta sin prompt. `--exec`/`--yes` requieren `--cmd`. Ejecución con `sh -c` y propagación del código del hijo (`128 + señal` en Unix; *fallback* 1 fuera de Unix). Decisión pura `should_execute` (tabla de 16 casos), nueva variante `IABotError::Execution` y 4 tests de integración en `tests/command_mode.rs` (mock HTTP sin TTY). README advierte de *prompt injection* al combinar `--file`/`--image` con `--exec --yes`. Sin re-bump: se mantiene `0.4.0`.
- **PLAN-003 Fase 1** (`cli-overrides`, specs `config` y `chat`): flags `--model`/`--temperature`/`--provider` que pisan `dime.yml` sólo para la invocación (fichero byte a byte intacto), alias `--system` de `-i/--instructions`, `IABot::apply_overrides` aplicado antes de `validate()` (por eso `--provider ollama` exime de token), proveedor inválido → `IABotError::Provider`, `--temperature` no numérica → error de uso de `clap`, `Args::parse()` al inicio de `main` (`--help`/`--version` ya no crean `dime.yml`). Bump a `0.4.0`.

## Estado del código

- 149 tests unitarios (`src/ia.rs`, `src/main.rs`; +1 marcado `#[ignore]` para el roundtrip real del portapapeles) + 2 de integración (`tests/attachments.rs`) + 4 de integración (`tests/command_mode.rs`) + 8 de integración (`tests/completions.rs`) + 4 de integración (`tests/scripting_output.rs`).
- `cargo fmt --check`, `cargo clippy -D warnings` y `cargo test` en verde.
- 8 specs vigentes en `openspec/specs/`: `chat` (7), `config` (7), `models` (2), `output` (2), `completions` (2), `attachments` (2), `clipboard` (1), `command` (5).

## Follow-ups

- Tipar `temperature` estrictamente como número en la documentación (el formato laxo de cadena sigue aceptado por compatibilidad).
- Valorar soporte multi-proveedor adicional y `--list-models` con salida formateada.
- Los warnings de `openspec validate` sobre `SHALL/MUST` son cosméticos (specs en español).
- Cota de tamaño al leer `stdin` (evitar consumo de memoria sin límite con ficheros/diffs grandes).
- Cota de tamaño al leer adjuntos (`--file`/`--image`; hoy sin límite).
- Portapapeles: en X11 **sin** *clipboard manager* el contenido no persiste tras salir el proceso (limitación de X11; funciona en Wayland/macOS/Windows y X11 con manager). El test de roundtrip real está marcado `#[ignore]`.
- Usar `Cow<str>` en `render_output` para evitar copias innecesarias al construir la salida.
- Overrides: valorar que `--provider` infiera `use_https`/endpoint y añadir flags para el resto de campos de `dime.yml`.
- Completions: valorar exponer el resto de shells de `clap_complete` (elvish, powershell).
- `--exec`: valorar **sandboxing** (límites de tiempo, entorno restringido, directorio controlado); hoy ejecuta con `sh -c` y los permisos del usuario.
- `clean_command`: el heurístico de fences de una línea es conservador (no descarta el lenguaje si el resto empieza por `-`, `./` o `/`); revisarlo si aparecen nuevos formatos de respuesta.
- `--cmd`: valorar cota de tamaño para adjuntos usados con `--exec` y un modo *dry-run* explícito (hoy `--cmd` sin `--exec` ya lo cubre).

## Flujo

`feature/*|chore/*` → PR a `development` → PR `development` → `main` → tag `vX.Y.Z` (release automática). Bump de versión con `vampus upgrade --minor|--patch|--major`.
