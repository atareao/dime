# Feature: el código de estado HTTP decide el resultado

## Qué es

Bugfix del tratamiento de respuestas del proveedor. Antes, `IABot::ask` y
`IABot::list_models` leían el cuerpo con `text()` y lo pasaban al parser **sin
mirar `response.status()`**: la forma del cuerpo era el único criterio. Ahora un
`check_status` puro y compartido exige un estado **`2xx`** para aceptar la
respuesta; cualquier otro estado (incluidos `3xx`, `4xx` y `5xx`) es un error
tipado, aunque el cuerpo tenga forma válida. Corresponde al cambio OpenSpec
`http-status-handling` (archivado el 2026-11-07, PR #40 squash →
`development`) y a la versión **`0.5.1`**. Amplía las specs `chat` (8 requisitos)
y `models` (3 requisitos).

## Ficheros implicados

- `src/ia.rs` — helper puro `check_status(status, body)`, `find_provider_error`,
  constructor `api_error(message, Option<StatusCode>)`, variante
  `IABotError::Status(StatusCode)` y `IABotError::Api(String)` con su texto ya
  compuesto. `ask` y `list_models` capturan `response.status()` **antes** de
  `response.text()` y llaman a `check_status` antes del parser.
- `tests/http_mock.rs` — test e2e `server_error_with_valid_chat_body_is_not_accepted`
  (la suite pasa de 11 a 12 tests): `500` + cuerpo de chat válido debe salir con
  código distinto de cero, `stderr` con `500` y `stdout` sin el contenido.
- Tests unitarios en `src/ia.rs` — `check_status_*` (sincrónicos, sin red) y los
  de `ask`/`list_models` contra `wiremock`.
- `README.md` — nota del listado de modelos sobre el criterio `2xx`.
- Specs: `openspec/specs/chat` (8 requisitos) y `openspec/specs/models`
  (3 requisitos).

## Cómo funciona

1. Se envía la petición y se guarda `response.status()` **antes** de consumir la
   respuesta con `text()`.
2. `check_status(status, &body)` decide:
   - `2xx` → `Ok(())` y el cuerpo pasa al parser (`parse_response` o
     `parse_models_response`), que mantiene su comportamiento anterior.
   - Cualquier otro estado → error, sin llegar al parser.
3. Para estados no exitosos se aplica esta precedencia:

| Prioridad | Condición | Variante | Ejemplo de mensaje |
|---|---|---|---|
| 1 | `2xx` | éxito (sin error) | — (el cuerpo se interpreta) |
| 2 | No `2xx` + `error.message` o `error` como cadena | `IABotError::Api` | `Error devuelto por el proveedor (HTTP 401): invalid_api_key` |
| 3 | No `2xx` + `error` sin mensaje utilizable (`{}`, un número) o sin campo `error` | `IABotError::Status` | `El proveedor respondió con el estado HTTP 500 Internal Server Error` |

`find_provider_error` distingue `Message(String)` (objeto con `message` o
cadena), `Unnamed` (`error` presente pero sin mensaje) y `None` (sin `error` o
`error: null`). `api_error` antepone el código cuando lo conoce. `Status` porta
un `reqwest::StatusCode`, así que su `Display` incluye la razón canónica sin
tabla de códigos propia.

### El bug original, con un ejemplo concreto

El proveedor responde `500 Internal Server Error` pero con un cuerpo que parece
una respuesta de chat correcta:

```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{"choices":[{"message":{"content":"texto incorrecto"}}],"model":"gpt-4o-mini"}
```

- **Antes:** el estado se ignoraba; `parse_response` encontraba `choices` y
  devolvía `texto incorrecto`. El proceso salía con **0** y el contenido
  incorrecto llegaba a `stdout`: un fallo silencioso.
- **Después:** `check_status` ve el `500` y falla con
  `El proveedor respondió con el estado HTTP 500 Internal Server Error`. El
  proceso sale con código distinto de cero, el mensaje va a `stderr` y `stdout`
  queda vacío.

El mismo criterio se aplica a `--list-models`: un `500` con un `data` válido ya
no imprime identificadores; falla con el código.

## Variables / configuración

- No añade variables de entorno, flags ni campos de `dime.yml`.
- Se aplica a todos los proveedores (`openai` y `ollama`) y a todas las
  peticiones de chat y de listado de modelos.
- No hay nada que activar o desactivar: es el comportamiento por defecto desde
  `0.5.1`.

## Cómo habilitar / desplegar

Nada especial: el cambio va dentro del binario. Basta con compilar y desplegar
la versión `0.5.1` como cualquier otra.

```bash
# Verificación completa
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test

# Solo el caso e2e del bug
cargo test --test http_mock server_error_with_valid_chat_body_is_not_accepted

# Solo los tests unitarios del criterio de estado
cargo test check_status
```

## Decisiones

- **Éxito solo en `200..=299`** (`StatusCode::is_success`), no `status < 400`:
  un `3xx` sin `Location` (o con redirecciones desactivadas en el futuro) no
  debe aceptarse por la forma del cuerpo.
- **Un único criterio compartido**: `check_status` es puro (sin red ni `IABot`)
  y lo usan tanto `ask` como `list_models`, evitando divergencias.
- **Precedencia mensaje del proveedor > código**: si el cuerpo trae `error` con
  mensaje, se conserva la variante `Api` y se le añade el código HTTP; así no se
  pierde el diagnóstico. Si `error` existe pero no aporta mensaje, `Status` es
  más informativo que `UnexpectedResponse`.
- **`Status(StatusCode)` en lugar de `u16`**: la razón canónica sale del propio
  tipo y se evitan códigos imposibles.
- **Estado capturado antes de `text()`**: `text()` consume la respuesta.
- **`Api(String)` compone el texto en `api_error`**: el mensaje mostrado sigue
  empezando por «Error devuelto por el proveedor» y los tests por subcadena no
  cambian.

## Follow-ups

- **Resuelto aquí:** el follow-up de `ask()`/`response.status()` que figuraba en
  `docs/http-mock-tests.md` y en `HANDOFF.md`; ya no está pendiente.
- `--exec`: valorar *sandboxing* (límites de tiempo, entorno restringido,
  directorio controlado); hoy ejecuta con `sh -c` y los permisos del usuario.
- Cota de tamaño al leer adjuntos (`--file`/`--image`; hoy sin límite).
- `clean_command`: el heurístico de fences de una línea es conservador (no
  descarta el lenguaje si el resto empieza por `-`, `./` o `/`).
- Portapapeles: en X11 **sin** *clipboard manager* el contenido no persiste tras
  salir el proceso; el test de roundtrip real está marcado `#[ignore]`.
- Suite HTTP: valorar un escenario de timeout/red caída y otro de `--list-models`
  con respuesta malformada (más allá del `401`).
