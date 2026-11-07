## ADDED Requirements

### Requirement: Overrides por CLI no persistentes

Los flags `--model`, `--temperature` y `--provider` DEBEN sobreescribir la configuración sólo para la invocación en curso y NO DEBEN modificar ni reescribir el fichero `dime.yml`. El fichero se DEBE seguir localizando y cargando como hasta ahora.

#### Scenario: Modelo sin tocar el fichero

- Given un `dime.yml` con `model: gpt-3.5-turbo`
- When se ejecuta `dime --model gpt-4o -q hola`
- Then la petición usa `gpt-4o` y `dime.yml` sigue conteniendo `gpt-3.5-turbo`

#### Scenario: Proveedor sin tocar el fichero

- Given un `dime.yml` sin `provider` explícito
- When se ejecuta `dime --provider ollama -q hola`
- Then la petición usa el proveedor `ollama` y el fichero no cambia

#### Scenario: Temperatura no numérica

- Given que se pasa `--temperature` con un valor no numérico
- When se ejecuta `dime`
- Then se produce un error de uso y el proceso termina con código distinto de cero, sin realizar ninguna petición

#### Scenario: Proveedor desconocido

- Given que se pasa `--provider desconocido`
- When se ejecuta `dime`
- Then se produce un error legible y el proceso termina con código distinto de cero
