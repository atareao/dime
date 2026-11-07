# Design

## Context

Ver `proposal.md`. El código actual concentra la lógica en `src/ia.rs` (`IABot` e `IABotError`) y `src/main.rs` (CLI, descubrimiento de configuración y arranque). No hay tests. El inventario de deuda está en `plans/PLAN-001.md`.

## Goals / Non-Goals

- Goal: eliminar los *panics* de la ruta petición/respuesta y sustituirlos por errores tipados.
- Goal: cubrir `src/ia.rs` con tests unitarios.
- Non-Goal: rediseñar la CLI ni añadir proveedores nuevos.
- Non-Goal: migrar `serde_yaml` (deprecado).

## Decisions

### D1. Manejo de errores con `thiserror`

Se introduce `thiserror` para definir un enum de error (`IABotError`) con variantes `Config`, `Http`, `Api`, `UnexpectedResponse` y `NumberParse`.

- Alternativa: `anyhow`. Descartada porque la librería conviene que exponga errores tipados.
- Alternativa: mantener `Box<dyn Error>` manual. Descartada por verbosidad y pérdida de tipos.

### D2. Deserialización con structs `serde` en lugar de `serde_json::Value`

Se definen `ChatResponse { choices: Vec<Choice> }`, `Choice { message: Message }`, `Message { content: String }` y `ApiErrorBody { error: ApiErrorDetail }`. Se intenta primero el error y luego la respuesta.

- Alternativa: seguir con `Value` y `get().unwrap()`. Descartada por los *panics* ante formas inesperadas.

### D3. Uso de `temperature`

Se parsea `temperature: String` a `f64` al construir el cuerpo. OpenAI: campo `temperature` en el nivel superior. Ollama: `options.temperature`.

- Alternativa: cambiar el tipo a `f64` en serde. Descartada para no romper ficheros existentes que la tienen como cadena; se anota como follow-up.

### D4. Retirada de `models_endpoint` y métodos muertos

Se eliminan `models_endpoint`, `get_base_url`, `set_token` y `save` por no tener uso.

### D5. Separación de responsabilidades

`IABot` construye el cuerpo de la petición en una función pura (`build_body`) testeable sin red. El envío HTTP y el parseo se separan para poder testear el parseo con datos fijos.

## Risks / Trade-offs

- [El parseo estricto puede rechazar respuestas antes aceptadas por accidente] → Cubrir con tests los formatos de OpenAI y Ollama.
- [`temperature` como cadena es propensa a error] → Validar en la construcción y anotar el follow-up de tiparla.
- [Nueva dependencia `thiserror`] → Es ligera, ampliamente usada y sólo de compilación.

## Migration Plan

1. Añadir `thiserror` a `Cargo.toml`.
2. Refactor de `ia.rs` (errores, DTOs, `build_body`, parseo).
3. Ajustar `main.rs` a las nuevas firmas `Result`.
4. Tests + `clippy`/`fmt`/`test` en verde.
5. PR a `development`.

Rollback: revert del PR; el formato de `dime.yml` permanece compatible.

## Open Questions

- Ninguna.