# command Specification

## Purpose
TBD - created by archiving change command-mode. Update Purpose after archive.

## Requirements

### Requirement: Modo comando

Con `--cmd <objetivo>`, el sistema DEBE solicitar al modelo UNICAMENTE el comando que cumple el objetivo, usando un rol de sistema específico, e imprimir el comando limpio: sin fences de Markdown ni explicaciones. `--cmd` DEBE ser incompatible con la pregunta (`-q`/posicional), con `--list-models` y con `--completions`.

#### Scenario: Comando limpio

- Given `--cmd "listar ficheros .log"` y una respuesta del modelo envuelta en un bloque de código Markdown
- When se procesa la respuesta
- Then se imprime sólo el comando, sin fences ni texto adicional

#### Scenario: Incompatible con la pregunta

- Given `--cmd` y una pregunta (`-q` o posicional)
- When se ejecuta `dime`
- Then se produce un error de uso

### Requirement: No ejecutar sin --exec

Sin `--exec`, el sistema NO DEBE ejecutar nada: sólo imprime el comando.

#### Scenario: Por defecto sólo imprime

- Given `--cmd "..."` sin `--exec`
- When se ejecuta `dime`
- Then el comando se imprime y NO se ejecuta

### Requirement: Ejecución con confirmación

Con `--exec`, el sistema DEBE ejecutar el comando vía shell tras una confirmación interactiva que por defecto es negativa. Si `stdin` no es un TTY y no se ha pasado `--yes`, el sistema DEBE negarse a ejecutar y terminar con código distinto de cero. `--exec` y `--yes` REQUIEREN `--cmd`.

#### Scenario: Confirma y ejecuta

- Given `--cmd "..." --exec` en un TTY
- When el usuario confirma
- Then el comando se ejecuta

#### Scenario: Rechaza la confirmación

- Given `--cmd "..." --exec` en un TTY
- When el usuario rechaza (respuesta por defecto)
- Then el comando NO se ejecuta

#### Scenario: Sin TTY y sin --yes

- Given `--cmd "..." --exec` con `stdin` que no es un TTY y sin `--yes`
- When se ejecuta `dime`
- Then el comando NO se ejecuta y el proceso termina con código distinto de cero

#### Scenario: --exec sin --cmd

- Given `--exec` sin `--cmd`
- When se ejecuta `dime`
- Then se produce un error de uso

### Requirement: Ejecución sin confirmación

Con `--exec --yes`, el sistema DEBE ejecutar el comando sin pedir confirmación.

#### Scenario: --yes ejecuta sin prompt

- Given `--cmd "..." --exec --yes`
- When se ejecuta `dime`
- Then el comando se ejecuta sin pedir confirmación

### Requirement: Estado de salida del comando

Con `--exec`, el sistema DEBE propagar el estado de salida del comando ejecutado como su propio código de salida.

#### Scenario: Propagación del código

- Given un comando que termina con código `3`
- When se ejecuta con `--exec`
- Then `dime` termina con código de salida `3`
