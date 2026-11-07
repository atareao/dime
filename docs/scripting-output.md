# Feature: salida para scripting

## Qué es

Salida apta para scripts y consumidores como `jq`: el flag `--json` emite en `stdout` un único objeto JSON con la respuesta y metadatos, y `--no-color` garantiza la ausencia de secuencias ANSI. Los logs se mueven a `stderr` y el color de la respuesta se decide por el destino real. Corresponde a la **Fase 3** de `PLAN-003` (cambio OpenSpec `scripting-output`, PR #27 squash → `development`). Introduce la spec `output`.

## Ficheros implicados

- `src/ia.rs` — tipo `ChatResponse { content, model, usage }` (devuelto por `ask`) y `IABot::get_model()`.
- `src/main.rs` — flags `--json` y `--no-color`, `render_output` y `should_colorize`, y configuración del log a `stderr`.
- `tests/scripting_output.rs` — tests de integración con servidor HTTP mock.
- Specs: `openspec/specs/output` (nueva) y `openspec/specs/chat` (respuesta con metadatos).
- Sin dependencias nuevas (`serde_json` ya presente).

## Cómo funciona

- **`--json`** (`render_output`): serializa exactamente un objeto `{content, model, usage}` en `stdout`.
  - `content`: la respuesta como cadena.
  - `model`: **siempre** cadena; si el proveedor no lo informa se usa el modelo configurado (`get_model()`).
  - `usage`: objeto crudo del proveedor, o `null` si no se aporta.
- **`--no-color` y salida no-TTY:** la respuesta en `stdout` nunca contiene ANSI; el color sólo se aplica si `stdout` es un TTY. La garantía es *forward-compatible* (hoy la respuesta no añade color).
- **Logs (`tracing`) a `stderr`:** su ANSI se activa sólo si `stderr` es un TTY y no se ha pasado `--no-color` (`should_colorize(no_color, stderr_is_terminal)`), de modo que `stdout` queda limpio para el consumidor.
- **Errores:** se escriben en `stderr` como texto legible con código de salida ≠ 0; `stdout` nunca contiene JSON parcial.
- **`--json` es incompatible con `--list-models`** (`conflicts_with` en `clap`).
- **Parseo:** `ChatResponse` expone `model`/`usage` opcionales; una ausencia se traduce a `null`/modelo configurado, sin `panic`.

## Configuración

- El color de los logs depende de `log_level` (`info`/`debug`) y del destino real (`stderr`).
- No requiere claves nuevas; el modelo de reserva para `model` es el configurado en `dime.yml`.

## Uso en CLI

```bash
# JSON para consumidores
dime --json "¿A qué huelen las nubes?"

# Encadenar con jq
dime --json -q "resume este error" | jq -r '.content'
echo "explica este log" | dime --json | jq '.usage.total_tokens'

# Sin códigos ANSI (para ficheros o logs)
dime --no-color "resume" > salida.txt
```

## Decisiones

- `ask` devuelve un tipo con metadatos (`ChatResponse`) en lugar de un `String`, para no duplicar la petición al obtener modelo/uso.
- Serializar con `serde_json` y tratar `usage` como `Value` opaco (su forma varía entre proveedores); el contrato sólo garantiza el campo y su ausencia como `null`.
- Resolver `model` a cadena siempre (fallback al modelo configurado) para que el JSON sea estable para scripts.
- En `--json`, errores a `stderr` y salida ≠ 0, dejando `stdout` sin JSON parcial.
- Mover los logs a `stderr` y decidir su ANSI por destino real, no por `stdout`.

## Follow-ups

- Usar `Cow<str>` en `render_output` para evitar copias innecesarias al construir la salida.
- Fuera de alcance en esta fase: color real en la respuesta y streaming.
- Revisar la interacción `--json` + `--list-models` si en el futuro se desea salida JSON de modelos (hoy son mutuamente excluyentes).
