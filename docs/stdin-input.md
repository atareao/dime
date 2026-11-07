# Feature: entrada por stdin / tuberías

## Qué es

Capacidad para usar `dime` dentro de pipelines Unix y redirecciones: la pregunta puede leerse de `stdin` cuando no se aporta `-q`/posicional, y el contenido canalizado puede aportarse como **contexto** del mensaje de usuario. Corresponde a la **Fase 2** de `PLAN-003` (cambio OpenSpec `stdin-input`, PR #26 → `development`). El uso interactivo (TTY) no cambia.

## Ficheros implicados

- `src/main.rs` — detección de TTY (`std::io::IsTerminal`), lógica pura de resolución (`StdinState`, `ResolvedInput`, `resolve_input`) y flujo principal.
- `src/ia.rs` — `compose_user_message(question, context)` y la variante tipada `IABotError::Input`.
- `README.md` — uso por tubería y redirección.
- Spec: `openspec/specs/chat` (sección de entrada desde stdin).
- Sin dependencias nuevas (`IsTerminal` está en la biblioteca estándar desde Rust 1.70).

## Cómo funciona

- **Detección de TTY:** si `io::stdin().is_terminal()` es cierto, no se lee nunca de `stdin` (evita bloqueos interactivos).
- **Resolución de la entrada (`resolve_input`)** — lógica pura y testeable, según el estado de `stdin`:
  - `Tty`: con pregunta distinta de `-` se usa tal cual; sin pregunta se emite un **error de uso** (código ≠ 0) sin leer `stdin`.
  - `Empty` (stdin no-TTY sin datos): con pregunta (≠ `-`) se usa; sin pregunta se devuelve `IABotError::Input` legible y el proceso termina sin petición.
  - `Data`: con ninguna pregunta o con `-` se lee la pregunta de `stdin`; con ambas (pregunta + datos) se devuelve `QuestionWithContext`.
- **Marcador `-`:** como `-q` o posicional fuerza explícitamente la lectura de la pregunta desde `stdin`.
- **Contexto:** con pregunta y `stdin` canalizado no vacío, `compose_user_message` concatena pregunta + separador + contenido; el mensaje `system` no cambia.
- **Errores:** siempre tipados (`IABotError::Input`), sin `unwrap`/`panic`.
- **`--list-models` ignora `stdin`**: nunca se bloquea esperando datos.

## Configuración

- No requiere variables ni claves nuevas; se apoya en la configuración `dime.yml` existente.
- El mecanismo es puramente de runtime: `stdin` canalizado → pregunta o contexto según el caso; `stdin` TTY → comportamiento interactivo original.

## Uso en CLI

```bash
# La pregunta viene de stdin
echo "¿A qué huelen las nubes?" | dime

# Redirección desde fichero
dime < fichero.md

# Marcador `-` para forzar la lectura de la pregunta desde stdin
echo "resume esto" | dime -

# Pregunta + contexto: git diff canalizado como contexto del mensaje
git diff | dime "resume estos cambios"

# También con redirección y pregunta
dime "explica" < fichero.md
```

## Decisiones

- Usar `std::io::IsTerminal` en lugar de las crates `atty` (deprecada) o `is-terminal` (innecesaria con std).
- **Relajar en `clap`** la obligatoriedad de la pregunta (`required_unless_present_any`) y validarla en runtime para poder leer de `stdin`.
- **Sólo se lee `stdin` cuando NO es un TTY**, para no bloquear la ejecución interactiva.
- Si hay pregunta y además se canaliza algo, `stdin` se interpreta como **contexto** (documentado en el README).
- Aislar la decisión en una función pura (`resolve_input`) con enums (`StdinState`/`ResolvedInput`) para cubrirla con tests unitarios sin E/S real.

## Follow-ups

- Cota configurable de tamaño al leer `stdin` (evitar consumir memoria sin límite con ficheros o diffs enormes).
- Valorar un `--context` explícito si se añaden más fuentes de contexto (Fase 5: ficheros/imágenes/clipboard).
