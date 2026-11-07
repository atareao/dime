# Feature: overrides por CLI

## Qué es

Flags que sobreescriben la configuración de `dime.yml` **sólo para la invocación en curso**, sin modificar el fichero: `--model <str>`, `--temperature <f64>` y `--provider <str>`. Además, `--system` actúa como alias de `-i/--instructions`. Corresponde a la **Fase 1** de `PLAN-003` (cambio OpenSpec `cli-overrides`, PR #29 squash → `development`). Modifica las specs `config` y `chat`.

## Ficheros implicados

- `src/main.rs` — flags `--model`/`--temperature`/`--provider` y alias `--system` en `clap`; `Args::parse()` movido al inicio de `main`.
- `src/ia.rs` — `IABot::apply_overrides(model, temperature, provider)` y la variante tipada `IABotError::Provider`.
- `README.md` — uso de los overrides.
- Specs: `openspec/specs/config` (overrides no persistentes) y `openspec/specs/chat` (valores efectivos).
- Sin dependencias nuevas.

## Cómo funciona

- **Aplicación (`apply_overrides`):** se ejecuta tras `read_content` y **antes** de `validate()`. Sólo muta los campos cuyo flag está presente; el resto conserva el valor de `dime.yml`. Los overrides viven únicamente en memoria: el fichero **nunca** se reescribe (hay un test que lo compara byte a byte).
- **Orden respecto a la validación:** al aplicarse antes de `validate()`, `--provider ollama` **exime de token** (Ollama no requiere `Authorization`).
- **`--temperature`:** tipado como `f64`; un valor no numérico lo rechaza `clap` con error de uso y código de salida 2, sin realizar petición.
- **`--provider`:** válido sólo `openai` u `ollama` (comparación insensible a mayúsculas; se normaliza a minúsculas). Un valor inválido produce `IABotError::Provider` con el mensaje «proveedor no soportado: `X`; valores válidos: `openai`, `ollama`», sin *panic*.
- **`--system`:** alias de `-i/--instructions`; si no se pasa ninguno, se usa el `system_role` del fichero. Los overrides de modelo/temperatura/proveedor **no** cambian el rol de sistema.
- **Parseo temprano:** `Args::parse()` se resuelve antes de tocar el fichero, de modo que `--help`/`--version` (y los errores de uso) ya no crean ni leen `dime.yml`.

## Configuración

- No requiere variables ni claves nuevas; parte de `dime.yml` y sólo lo pisa en memoria.
- El modelo de reserva sigue siendo el configurado; el override de proveedor no infiere `use_https`/endpoint (fuera de alcance en esta fase).

## Uso en CLI

```bash
# Otro modelo sólo para esta invocación
dime --model gpt-4o -q "resume este error"

# Proveedor y temperatura puntuales
dime --provider ollama --temperature 0.2 -q "explícame esto"

# Alias de instrucciones de sistema
dime --system "eres un experto en bash" -q "¿cómo listo los ficheros por tamaño?"
```

## Decisiones

- Mutar los campos del `IABot` ya cargado (portador de la configuración efectiva) en lugar de un struct de overrides aparte.
- Aplicar los overrides **antes** de `validate()` para que el proveedor elegido determine los requisitos de token.
- `--provider` case-insensitive y normalizado; error legible y tipado en vez de `panic`.
- Validar `--temperature` en `clap` (numérica) y dejar `dime.yml` intacto por contrato.

## Follow-ups

- ¿Debe `--provider` inferir también `use_https`/endpoint? Decisión: no en esta fase.
- Valorar flags de override para el resto de campos de `dime.yml` (p. ej. `--system-role` explícito).
