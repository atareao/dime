# config

## ADDED Requirements

### Requirement: Descubrimiento del fichero de configuración

El sistema DEBE localizar el fichero `dime.yml` en este orden: directorio actual, directorio del ejecutable, `~/.dime.yml`, directorio de configuración del sistema (`dime.yml`) y subdirectorio de la aplicación (`dime/dime.yml`).

#### Scenario: Existe en el directorio actual

- Given un `dime.yml` en el directorio actual
- When se arranca `dime`
- Then se usa ese fichero y no se busca en el resto de ubicaciones

#### Scenario: No existe en ninguna ubicación

- Given que no existe `dime.yml` en ninguna ubicación
- When se arranca `dime`
- Then se crea un `dime.yml` con valores por defecto en el directorio actual y se usa ese fichero

### Requirement: Valores por defecto

Al crear el fichero por defecto, los campos ausentes en un `dime.yml` DEBEN tomar sus valores por defecto (`provider="openai"`, `base_url="api.openai.com"`, `endpoint="v1/chat/completions"`, `model="gpt-3.5-turbo"`, `temperature="1"`, `num_ctx=32000`, `use_https=true`, `log_level="info"`, `token=""`).

#### Scenario: YAML parcial

- Given un `dime.yml` que sólo define `model`
- When se carga la configuración
- Then el resto de campos toman sus valores por defecto

### Requirement: Errores de configuración legibles y sin panic

Si el fichero no puede leerse o no es YAML válido, el sistema DEBE mostrar un mensaje que incluya el nombre correcto del fichero (`dime.yml`) y terminar con código de salida distinto de cero, sin entrar en *panic*.

#### Scenario: YAML inválido

- Given un `dime.yml` con contenido que no es YAML válido
- When se carga la configuración
- Then se informa de un error que menciona `dime.yml` y el proceso termina con código distinto de cero

#### Scenario: Token vacío con proveedor openai

- Given `provider="openai"` y `token=""`
- When se arranca `dime`
- Then se informa del error y el proceso termina con código distinto de cero, sin realizar ninguna petición

### Requirement: El token no se registra

El valor del token DEBE estar excluido de cualquier salida de log.

#### Scenario: Log de depuración

- Given `log_level="debug"` y un token configurado
- When se realiza una consulta
- Then ningún mensaje de log contiene el valor del token