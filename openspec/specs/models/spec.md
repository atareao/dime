# models Specification

## Purpose
TBD - created by archiving change mejoras-cli-v2. Update Purpose after archive.

## Requirements

### Requirement: Listado de modelos

Con la opción `--list-models`, el sistema DEBE consultar `{protocolo}://{base_url}/{models_endpoint}` (respetando `use_https`) incluyendo la cabecera `Authorization: Bearer <token>` cuando el proveedor la use, y DEBE mostrar los identificadores de los modelos disponibles, uno por línea. Ante un error de red o de API, DEBE mostrar un error legible y terminar con código distinto de cero, sin *panic*.

#### Scenario: Endpoint por defecto

- Given `base_url="api.openai.com"`, `models_endpoint="v1/models"` y `use_https=true`
- When se ejecuta `dime --list-models`
- Then se consulta `https://api.openai.com/v1/models`

#### Scenario: Listado correcto

- Given una respuesta con `data[*].id` de varios modelos
- When se procesa la respuesta
- Then se muestran los identificadores de los modelos

#### Scenario: Error de API

- Given una respuesta con campo `error`
- When se procesa la respuesta
- Then se muestra un mensaje de error legible y el proceso termina con código distinto de cero

### Requirement: Modo listado sin pregunta

En el modo `--list-models` NO se DEBE exigir la pregunta. En el modo de consulta normal la pregunta DEBE seguir siendo obligatoria.

#### Scenario: Listado sin pregunta

- Given el argumento `--list-models` y ninguna pregunta
- When se arranca `dime`
- Then no se produce error de uso y se listan los modelos

#### Scenario: Consulta sin pregunta

- Given que no se pasa `--list-models` ni ninguna pregunta
- When se arranca `dime`
- Then `clap` produce un error de uso
