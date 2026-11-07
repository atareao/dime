# PLAN-003 — Features del CLI `dime`

## Objetivo

Añadir capacidades de uso avanzado al CLI `dime`: overrides por línea de comandos, entrada por stdin/pipes, salida para scripting, completions, contexto (ficheros/imágenes/clipboard), modo comando y tests de integración con servidor HTTP mock.

## Estado de partida

- Versión `0.3.0`; specs `config`, `chat` y `models`; 49 tests; CI en `main` y `development`.
- El CLI solo tiene `-i/--instructions`, `-q/--question`/posicional y `--list-models`.

## Estrategia

Un plan único con **7 fases**. Cada fase es un cambio OpenSpec propio y un **PR independiente** (gitflow `feature/*` → `development` → `main`), ejecutadas en orden. Cada fase añade sus tests y su documentación.

## Fases

### Fase 1 — Overrides por CLI (cambio `cli-overrides`)

Flags que pisan `dime.yml` **solo para esa invocación**, sin modificar el fichero: `--model`, `--temperature`, `--provider` y `--system` (alias/equivalente de `-i/--instructions`).
- Specs: `config` (los overrides no se persisten), `chat` (usan los valores efectivos).
- Criterio: con overrides, la petición usa esos valores y el fichero queda intacto.

### Fase 2 — Entrada por stdin/pipes (cambio `stdin-input`)

Si no se aporta pregunta y `stdin` no es un TTY, leer de `stdin`; admitir `-` como marcador explícito. Permitir componer `git diff | dime "resume estos cambios"` (instrucción + contexto) y `dime "explica" < file.md`.
- Specs: `chat`/`config` (resolución de la pregunta y del contexto).
- Criterio: la entrada por stdin se usa como pregunta o contexto según el caso.

### Fase 3 — Salida para scripting (cambio `scripting-output`)

`--json` (respuesta + metadatos: modelo y uso si el proveedor lo aporta) y `--no-color`; color automático solo si la salida es un TTY.
- Spec nueva: `output`.
- Criterio: `--json` emite JSON válido; `--no-color` no emite códigos ANSI.

### Fase 4 — Shell completions (cambio `shell-completions`)

Con `clap_complete`: `--completions <bash|zsh|fish>` imprime el script de completado.
- Spec nueva: `completions`.
- Criterio: `dime --completions bash` imprime un script válido.

### Fase 5 — Contexto: ficheros, imágenes y clipboard (cambio `context-input`)

`--file <ruta>` (repetible), `--image <ruta>` (repetible; multimodal) y `--copy` (copiar la respuesta al portapapeles). Imágenes: OpenAI (content parts con data URL base64) y Ollama (`images`).
- Specs: `chat` (mensajes con contexto), `config`.
- Criterio: `dime --file README.md "resume"` incluye el contenido; `--image` envía la imagen; `--copy` copia la respuesta.
- Dependencias: `arboard` (clipboard) y `base64`.

### Fase 6 — Modo comando (cambio `command-mode`)

`--cmd "<objetivo>"` pide únicamente el comando (sin fences Markdown); `--exec` lo ejecuta **con confirmación**; `--yes` evita el prompt. Seguridad: por defecto solo se imprime; **nunca** se ejecuta sin `--exec`.
- Spec nueva: `command`.
- Criterio: `--cmd` devuelve el comando limpio; `--exec` pide confirmación; sin `--exec` no ejecuta nada.
- Riesgos: ejecución de shell; requiere una spec explícita de seguridad.

### Fase 7 — Tests de integración HTTP (cambio `http-mock-tests`)

`wiremock` como dependencia de desarrollo para cubrir `ask` y `list_models` contra un servidor local: cabecera de autorización, cuerpos, respuestas 4xx/5xx y formas inesperadas.
- Criterio: `cargo test` cubre la ruta real de red (hoy sin test).

## Criterios de aceptación globales

- [ ] Cada fase: cambio OpenSpec aprobado antes del código, tests, `fmt` y `clippy` verdes, PR a `development`.
- [ ] Sin regresiones en las specs existentes (`config`, `chat`, `models`).
- [ ] Documentación en `docs/` y `HANDOFF.md` actualizado al cerrar cada fase.

## Riesgos

- Fases 5 y 6 (imágenes y ejecución de comandos) tienen la mayor superficie: exigen specs de formato y de seguridad.
- Aumento del número de flags: mantener la ayuda de `clap` clara y los conflictos bien definidos.

## Versionado

`0.4.0` al cerrar las fases 1–3; las fases 4–7 en incrementos posteriores.
