# Design

## Context

Ver `proposal.md` y los deltas de `chat` y `models`. `IABot::ask` y
`IABot::list_models` (`src/ia.rs`) hacían la petición, leían el cuerpo con
`text()` y lo pasaban al parser sin mirar `response.status()`. Consecuencia: un
`500` con un cuerpo de chat válido se trataba como éxito (código de salida cero
y contenido incorrecto en `stdout`), y un error del proveedor sin el campo
`error` se degradaba al genérico `UnexpectedResponse`, perdiendo el código.

## Goals / Non-Goals

- Goal: el código de estado decide el resultado, en ambas funciones y con un
  único criterio compartido.
- Goal: no perder información en el diagnóstico (mensaje del proveedor **y**
  código HTTP; en su defecto, código con su razón canónica).
- Non-Goal: cambiar las peticiones, las URLs, los cuerpos o el formato de las
  respuestas correctas.
- Non-Goal: implementar reintentos, *backoff* o manejo especial de `429`.

## Decisions

### D1. Éxito sólo en `200..=299`

Se usa `StatusCode::is_success()`, que acepta exactamente `2xx`. Antes el
criterio era `status < 400`, que daba por bueno un `3xx`. Aunque `reqwest`
sigue las redirecciones por defecto, una respuesta `3xx` sin `Location` (o con
la política de redirecciones desactivada en el futuro) llegaría al parser y su
cuerpo podría tener forma de respuesta válida: se trata como error. Un `2xx`
deja pasar el cuerpo al parser, de modo que la aceptación de respuestas
correctas no cambia.

### D2. Precedencia: `Api` con código > `Status`

`check_status(status, body)` es un helper puro (sin red, sin `IABot`) compartido
por `ask` y `list_models`:

1. `2xx` → `Ok(())` y el cuerpo va al parser.
2. Cuerpo con `error` del proveedor y mensaje utilizable → `IABotError::Api`
   con el código HTTP incluido: `Error devuelto por el proveedor (HTTP 401):
   invalid_api_key`. Se conserva la variante `Api(String)`, por lo que los
   tests y el render existentes que buscan el mensaje del proveedor por
   subcadena siguen valiendo.
3. Cualquier otro caso (sin `error`, o `error` presente pero sin mensaje
   utilizable: `{}`, un número…) → `IABotError::Status`.

Se mantiene el orden del contrato: primero el mensaje del proveedor, después el
código. Si `error` existe pero no aporta mensaje, `Status` es más informativo
que `UnexpectedResponse` (que afirmaría que el cuerpo no contiene `error`).

### D3. `Status(StatusCode)` y razón canónica

`IABotError::Api` pasa a mostrar su contenido tal cual (`#[error("{0}")]`)
porque el texto definitivo (con o sin código) lo compone un constructor
`api_error(message, status)`.

`Status` porta un `reqwest::StatusCode` en lugar de un `u16`: el `Display` de
`http::StatusCode` incluye la razón canónica, así que el mensaje final es
`El proveedor respondió con el estado HTTP 500 Internal Server Error` sin tabla
de códigos propia ni dependencias nuevas. Tipar el campo también evita códigos
imposibles.

El estado se captura **antes** de `response.text()` (que consume la respuesta).

### D4. Tests

- Unitarios directos de `check_status` (sincrónicos, sin servidor): `200`
  aceptado; `302` y `500` con cuerpo válido como `Status` con su razón; `429`
  sin `error`; `401` con mensaje como `Api` con `HTTP 401`; y `500` con
  `error` sin mensaje como `Status`.
- Unitarios de `ask`/`list_models` contra `mock_bot()` + `wiremock`: `500` con
  cuerpo válido, `429` sin `error`, `401` con mensaje y `200` correcto.
- Integración en `tests/http_mock.rs`: `500` con cuerpo de chat válido (el caso
  que antes pasaba como éxito) debe salir con código distinto de cero, `stderr`
  con `500` y `stdout` sin el contenido del modelo.

## Risks / Trade-offs

- [Un `3xx` que `reqwest` no sigue ahora falla] → Es el comportamiento
  buscado: sin `Location` no hay respuesta válida que interpretar.
- [`Status(StatusCode)` amplía el tipo público de la variante] → Sólo afecta a
  quien haga *pattern matching* sobre ella; en el binario se limita al render.
- [El mensaje de `Api` pierde el prefijo del `#[error(...)]`] → Lo compone
  `api_error`, así que el texto mostrado sigue empezando por «Error devuelto por
  el proveedor» y los tests por subcadena no cambian.
