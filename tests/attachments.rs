//! Tests de integración para los adjuntos (`--file`/`--image`).
//!
//! Comprueban el contrato observable del proceso: un adjunto ilegible termina
//! con código de salida distinto de cero y **sin** realizar la petición al
//! proveedor, mostrando un error legible por `stderr`.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Directorio temporal único por test dentro del `temp_dir`.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dime-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("crear directorio temporal");
    dir
}

/// Crea un `dime.yml` en `dir` apuntando a `base_url`.
fn write_config(dir: &Path, base_url: &str) {
    let content = format!(
        "provider: ollama\nuse_https: false\nbase_url: {base_url}\nendpoint: v1/chat/completions\nlog_level: info\n"
    );
    std::fs::write(dir.join("dime.yml"), content).expect("escribir config");
}

/// Ejecuta el binario `dime` en `dir` con los argumentos dados.
fn run_dime(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_dime"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("ejecutar dime")
}

/// Escucha en un puerto local que **no debe** recibir ninguna conexión.
///
/// Tras ejecutar `dime`, una llamada no bloqueante a `accept()` revela si el
/// proveedor recibió alguna petición (la conexión quedaría en la cola del
/// sistema operativo aunque el proceso ya hubiera terminado).
fn assert_attachment_error_without_request(flag: &str, missing: &str) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).expect("modo no bloqueante");
    let port = listener.local_addr().expect("addr").port();
    let base_url = format!("127.0.0.1:{port}");

    let dir = temp_dir(flag.trim_start_matches('-'));
    write_config(&dir, &base_url);

    let output = run_dime(&dir, &[flag, missing, "-q", "hola"]);

    // `dime` ya terminó: si hubiera contactado al proveedor, la conexión estaría
    // pendiente y `accept()` la devolvería.
    let contacted = listener.accept().is_ok();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        !output.status.success(),
        "un adjunto ilegible debe terminar con código distinto de cero"
    );
    assert!(
        !contacted,
        "el proveedor no debe recibir ninguna petición cuando el adjunto falla"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error"),
        "stderr debe contener un mensaje legible, fue: {stderr:?}"
    );
}

/// Escenario `Fichero inexistente`: error legible, código ≠ 0 y sin petición.
#[test]
fn missing_file_errors_without_request() {
    assert_attachment_error_without_request("--file", "no-existe.txt");
}

/// Escenario `Imagen inexistente`: error legible, código ≠ 0 y sin petición.
#[test]
fn missing_image_errors_without_request() {
    assert_attachment_error_without_request("--image", "no-existe.png");
}
