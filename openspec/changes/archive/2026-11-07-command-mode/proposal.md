# Proposal

## Why

Frecuentemente se quiere que el modelo devuelva sólo un comando de shell (para pegarlo o ejecutarlo), pero hoy `dime` devuelve texto libre con explicaciones y bloques Markdown. Además no hay forma de ejecutar el comando obtenido. Esta fase añade un modo comando con **ejecución opt-in y segura**.

## What Changes

- `--cmd <objetivo>`: pide al modelo **únicamente** el comando que cumple el objetivo e imprime el comando limpio (sin fences de Markdown ni explicaciones).
- `--exec`: ejecuta el comando **previa confirmación interactiva**. Por defecto NUNCA se ejecuta: sin `--exec` sólo se imprime.
- `--yes`: ejecuta sin pedir confirmación (`--exec` + `--yes`); sin `TTY` y sin `--yes` el sistema se niega a ejecutar.
- `--exec` y `--yes` REQUIEREN `--cmd`.
- **Seguridad**: la ejecución es opt-in, con confirmación, y el comando se muestra antes de ejecutarse.

## Capabilities

### New Capabilities
- `command`: modo comando (`--cmd`) y ejecución segura (`--exec`/`--yes`).

### Modified Capabilities
<!-- Ninguna. -->

## Impact
- `src/main.rs` (flags y flujo de confirmación/ejecución), `src/ia.rs` (rol de sistema del modo comando y limpieza de la respuesta), `README.md`.
- Sin dependencias nuevas (`std::process::Command`).
