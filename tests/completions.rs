//! Tests de integración para `--completions`.
//!
//! Contrato observable del proceso: `--completions <bash|zsh|fish>` escribe en
//! `stdout` un script de completado no vacío y termina con código 0, sin exigir
//! pregunta, sin leer `stdin` y sin crear `dime.yml`. Un shell no soportado, la
//! pregunta o `--list-models` producen un error de uso (código distinto de cero).

use std::process::{Command, Output, Stdio};

/// Directorio temporal único por test dentro de `temp_dir`.
fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("dime-completions-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("crear directorio temporal");
    dir
}

/// Ejecuta el binario `dime` en `dir` con los argumentos dados y `stdin` cerrado
/// (para detectar cualquier intento de lectura de la entrada estándar).
fn run_dime(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dime"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("ejecutar dime")
}

/// Comprueba el contrato para un shell soportado.
fn assert_shell_script(shell: &str) {
    let dir = temp_dir(shell);
    let output = run_dime(&dir, &["--completions", shell]);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "--completions {shell} debe terminar con éxito: {:?}",
        output.status
    );
    assert!(
        !stdout.trim().is_empty(),
        "el script de {shell} no debe estar vacío"
    );
    assert!(
        stdout.contains("dime"),
        "el script de {shell} debe referenciar el binario `dime`"
    );
}

#[test]
fn completions_bash_outputs_script() {
    assert_shell_script("bash");
}

#[test]
fn completions_zsh_outputs_script() {
    assert_shell_script("zsh");
}

#[test]
fn completions_fish_outputs_script() {
    assert_shell_script("fish");
}

/// El script se escribe aunque no exista `dime.yml`, y NO se crea el fichero.
#[test]
fn completions_does_not_create_config() {
    let dir = temp_dir("no-config");
    let output = run_dime(&dir, &["--completions", "bash"]);
    let created = dir.join("dime.yml").exists();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(output.status.success(), "status: {:?}", output.status);
    assert!(!created, "`--completions` no debe crear dime.yml");
}

#[test]
fn completions_unknown_shell_is_usage_error() {
    let dir = temp_dir("unknown");
    let output = run_dime(&dir, &["--completions", "desconocido"]);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        !output.status.success(),
        "un shell no soportado debe fallar"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "debe ser un error de uso de clap (código 2)"
    );
}

#[test]
fn completions_conflicts_with_positional_question() {
    let dir = temp_dir("conflict-pos");
    let output = run_dime(&dir, &["--completions", "bash", "hola"]);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(!output.status.success(), "debe fallar por conflicto");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn completions_conflicts_with_question_flag() {
    let dir = temp_dir("conflict-flag");
    let output = run_dime(&dir, &["--completions", "bash", "-q", "hola"]);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(!output.status.success(), "debe fallar por conflicto");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn completions_conflicts_with_list_models() {
    let dir = temp_dir("conflict-list");
    let output = run_dime(&dir, &["--completions", "bash", "--list-models"]);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(!output.status.success(), "debe fallar por conflicto");
    assert_eq!(output.status.code(), Some(2));
}
