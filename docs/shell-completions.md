# Feature: shell completions

## Qué es

Flag `--completions <bash|zsh|fish>` que imprime en `stdout` el script de autocompletado del binario `dime` y termina con código 0. Corresponde a la **Fase 4** de `PLAN-003` (cambio OpenSpec `shell-completions`, PR #31 squash → `development`). Añade la spec `completions` y la dependencia `clap_complete`.

## Ficheros implicados

- `src/main.rs` — flag `--completions` (con `conflicts_with_all = ["question", "question_flag", "list_models"]`); `ValueEnum` propio `CompletionShell { Bash, Zsh, Fish }` y su `From` a `clap_complete::Shell`; función `generate_completions(shell, writer)`; rama temprana en `main`.
- `Cargo.toml` — nueva dependencia `clap_complete = "4"` (compatible con `clap` 4.6.7).
- `tests/completions.rs` — 8 tests de integración sobre el binario real.
- `README.md` — uso del flag e instalación del script.
- Spec: `openspec/specs/completions` (2 requisitos).

## Cómo funciona

- **Generación (`generate_completions`):** llama a `clap_complete::generate(shell, &mut Args::command(), "dime", writer)`. Toda la E/S va al `writer` recibido, de modo que se testea con un buffer en memoria sin depender del binario ni de `stdout`.
- **Ejecución temprana:** la rama de `--completions` corre al inicio de `main`, justo después de `Args::parse()` y **antes** de resolver/crear la configuración y antes de tocar `stdin`. Imprime el script, hace `flush` y retorna; **no** crea `dime.yml` ni accede a la red.
- **Shells soportados:** sólo `bash`, `zsh` y `fish` (los del plan). El resto de shells de `clap_complete` (elvish, powershell) no se exponen; un valor fuera del conjunto produce un error de uso de `clap` con código de salida **2**.
- **Conflictos:** `--completions` es incompatible con la pregunta (posicional o `-q/--question`) y con `--list-models`; combinarlos produce error de uso.

## Configuración

- No requiere variables ni claves nuevas: no lee ni necesita `dime.yml` (y no lo crea).
- La salida depende únicamente del shell indicado y del propio binario (sus flags y subcomandos vía `clap`).

## Cómo habilitar / usar

```bash
# Bash (sesión actual)
. <(dime --completions bash)

# Bash (instalación permanente, requiere permisos)
dime --completions bash | sudo tee /etc/bash_completion.d/dime > /dev/null

# Zsh
dime --completions zsh > ~/.zsh/completions/_dime

# Fish
dime --completions fish > ~/.config/fish/completions/dime.fish
```

Un shell no soportado (p. ej. `--completions elvish`), la pregunta o `--list-models` producen un error de uso.

## Decisiones

- Usar `clap_complete` (crate oficial) en lugar de scripts escritos a mano: evita que el completado quede desincronizado con los flags.
- `ValueEnum` propio `CompletionShell` en vez de reutilizar `clap_complete::Shell`, para ceñirse a los tres shells del plan y no exponer elvish/powershell.
- Factorizar la generación en `generate_completions(shell, writer)` con `&mut dyn Write` para poder testarla con un buffer en memoria.
- Atender el flag al inicio de `main`, antes de la configuración y de `stdin`, para que sea seguro en el arranque del shell (sin efectos secundarios).

## Follow-ups

- Valorar exponer el resto de shells de `clap_complete` (elvish, powershell).
- No se instalan los scripts en el sistema: es responsabilidad del usuario (Non-Goal de la fase).
