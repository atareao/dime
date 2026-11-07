# config

## MODIFIED Requirements

### Requirement: Valores por defecto

Al crear el fichero por defecto, los campos ausentes en un `dime.yml` DEBEN tomar sus valores por defecto (`provider="openai"`, `base_url="api.openai.com"`, `endpoint="v1/chat/completions"`, `models_endpoint="v1/models"`, `model="gpt-3.5-turbo"`, `temperature=1.0`, `num_ctx=32000`, `use_https=true`, `log_level="info"`, `token=""`).

#### Scenario: YAML parcial

- Given un `dime.yml` que sólo define `model`
- When se carga la configuración
- Then el resto de campos toman sus valores por defecto, incluido `models_endpoint="v1/models"` y `temperature=1.0`

## ADDED Requirements

### Requirement: temperature numérica tolerante

El campo `temperature` DEBE almacenarse como número y DEBE aceptar tanto un número YAML (`0.2`) como una cadena numérica (`"0.2"`). Un valor no numérico DEBE provocar un error de configuración legible al cargar el fichero.

#### Scenario: Número

- Given `temperature: 0.2`
- When se carga la configuración
- Then la temperatura es el número `0.2`

#### Scenario: Cadena numérica

- Given `temperature: "0.2"`
- When se carga la configuración
- Then la temperatura es el número `0.2`

#### Scenario: Cadena no numérica

- Given `temperature: "alta"`
- When se carga la configuración
- Then se devuelve un error de configuración legible y no se realiza ninguna petición

### Requirement: Endpoint de modelos configurable

La configuración DEBE incluir el campo `models_endpoint`, usado para listar los modelos disponibles.

#### Scenario: Valor personalizado

- Given `models_endpoint="api/tags"`
- When se carga la configuración
- Then el endpoint de modelos es `api/tags`