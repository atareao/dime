# completions Specification

## Purpose
TBD - created by archiving change shell-completions. Update Purpose after archive.

## Requirements

### Requirement: Generación de scripts de completado

Con `--completions <shell>`, el sistema DEBE imprimir en `stdout` el script de autocompletado del binario `dime` para el shell indicado y terminar con código de salida cero, SIN exigir pregunta, sin leer `stdin` y sin necesidad de un fichero de configuración válido (no DEBE crear `dime.yml`).

#### Scenario: Bash

- Given el flag `--completions bash`
- When se ejecuta `dime`
- Then `stdout` contiene un script de completado de bash no vacío y el proceso termina con código 0

#### Scenario: Zsh

- Given el flag `--completions zsh`
- When se ejecuta `dime`
- Then `stdout` contiene un script de completado de zsh no vacío y el proceso termina con código 0

#### Scenario: Fish

- Given el flag `--completions fish`
- When se ejecuta `dime`
- Then `stdout` contiene un script de completado de fish no vacío y el proceso termina con código 0

#### Scenario: Shell no soportado

- Given el flag `--completions desconocido`
- When se ejecuta `dime`
- Then se produce un error de uso y el proceso termina con código distinto de cero

#### Scenario: No crea configuración

- Given un directorio sin `dime.yml`
- When se ejecuta `dime --completions bash`
- Then no se crea ningún `dime.yml`

### Requirement: Sin efectos secundarios

`--completions` DEBE ser incompatible con la pregunta (posiciónal o `-q`) y con `--list-models`, y DEBE ejecutarse antes de cualquier acceso a la red.

#### Scenario: Incompatible con pregunta

- Given el flag `--completions bash` y una pregunta
- When se ejecuta `dime`
- Then se produce un error de uso

#### Scenario: Incompatible con --list-models

- Given el flag `--completions bash` y `--list-models`
- When se ejecuta `dime`
- Then se produce un error de uso
