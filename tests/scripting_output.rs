//! Tests de integración para la salida orientada a scripting (`--json`).
//!
//! Comprueban el contrato observable del proceso: `stdout` queda limpio (sin
//! contaminación de logs ni ANSI) en modo `--json`, tanto en éxito como en
//! error, y el error termina con un código de salida distinto de cero.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

/// Directorio temporal único por test dentro del `temp_dir`.
fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("dime-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("crear directorio temporal");
    dir
}

/// Crea un `dime.yml` en `dir` con el contenido dado.
fn write_config(dir: &std::path::Path, content: &str) {
    std::fs::write(dir.join("dime.yml"), content).expect("escribir config");
}

/// Devuelve la ruta de un puerto local que no está a la escucha.
///
/// Se reserva un puerto efímero y se libera inmediatamente: la conexión
/// posterior será rechazada de forma determinista y sin acceso a red externa.
fn unreachable_base_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    format!("127.0.0.1:{port}")
}

/// Arranca un servidor HTTP de un solo uso que responde `body` con 200 OK.
///
/// Devuelve la `base_url` (host:puerto) y el `JoinHandle` del hilo servidor.
fn spawn_mock_server(body: &'static str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock");
    let port = listener.local_addr().expect("addr").port();

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0u8; 2048];
            let _ = stream.read(&mut buffer);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    (format!("127.0.0.1:{port}"), handle)
}

/// Ejecuta el binario `dime` en `dir` con los argumentos dados.
fn run_dime(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_dime"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("ejecutar dime")
}

/// Escenario `Respuesta con uso`: éxito con `--json`.
///
/// `stdout` debe ser exactamente un objeto JSON con `content`, `model` y
/// `usage`, sin ninguna línea de log. `stderr` no debe contener ANSI.
#[test]
fn json_success_writes_exactly_one_json_object_to_stdout() {
    let body = r#"{"model":"llama2","usage":{"total_tokens":7},"choices":[{"message":{"content":"Hola"}}]}"#;
    let (base_url, server) = spawn_mock_server(body);

    let dir = temp_dir("json-success");
    write_config(
        &dir,
        &format!(
            "provider: ollama\nmodel: llama2\nuse_https: false\nbase_url: {base_url}\nendpoint: v1/chat/completions\nlog_level: info\n"
        ),
    );

    let output = run_dime(&dir, &["--json", "-q", "hola"]);
    server.join().expect("unir servidor mock");
    let _ = std::fs::remove_dir_all(&dir);

    assert!(output.status.success(), "debe terminar con éxito");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().count(),
        1,
        "stdout debe ser exactamente una línea JSON, fue: {stdout:?}"
    );
    let value: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout debe ser JSON válido");
    assert_eq!(value["content"], "Hola");
    assert_eq!(value["model"], "llama2");
    assert_eq!(value["usage"]["total_tokens"], 7);
}

/// Contrato de `model`: si el proveedor no informa el modelo, la salida JSON usa
/// el modelo configurado, de modo que `model` es siempre una cadena.
#[test]
fn json_model_falls_back_to_configured_model() {
    let body = r#"{"usage":{"total_tokens":7},"choices":[{"message":{"content":"Hola"}}]}"#;
    let (base_url, server) = spawn_mock_server(body);

    let dir = temp_dir("json-model-fallback");
    write_config(
        &dir,
        &format!(
            "provider: ollama\nmodel: mi-modelo\nuse_https: false\nbase_url: {base_url}\nendpoint: v1/chat/completions\n"
        ),
    );

    let output = run_dime(&dir, &["--json", "-q", "hola"]);
    server.join().expect("unir servidor mock");
    let _ = std::fs::remove_dir_all(&dir);

    assert!(output.status.success(), "debe terminar con éxito");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout debe ser JSON válido");
    assert_eq!(value["model"], "mi-modelo");
}

/// Escenario `Error en modo JSON` con el error POST-init (red).
///
/// A diferencia del error de configuración (pre-init), aquí el fallo ocurre con
/// el logging ya inicializado: `stdout` debe quedar vacío (sin logs ni ANSI) y
/// el mensaje debe ir a `stderr`.
#[test]
fn json_mode_network_error_keeps_stdout_clean_and_exit_nonzero() {
    let base_url = unreachable_base_url();
    let dir = temp_dir("json-net-error");
    write_config(
        &dir,
        &format!(
            "provider: ollama\nuse_https: false\nbase_url: {base_url}\nendpoint: v1/chat/completions\nlog_level: info\n"
        ),
    );

    let output = run_dime(&dir, &["--json", "-q", "hola"]);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        !output.status.success(),
        "el proceso debe terminar con código distinto de cero"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_empty(),
        "stdout debe quedar vacío (sin logs ni JSON parcial), fue: {stdout:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error"),
        "stderr debe contener un mensaje legible, fue: {stderr:?}"
    );
    assert!(
        !stderr.contains('\u{1b}'),
        "stderr no debe contener ANSI cuando no es un TTY, fue: {stderr:?}"
    );
}

/// Escenario `Error en modo JSON` con error PRE-init (configuración).
#[test]
fn json_mode_config_error_writes_to_stderr_and_keeps_stdout_clean() {
    let dir = temp_dir("json-config-error");
    write_config(&dir, "provider: openai\ntoken: \"\"\n");

    let output = run_dime(&dir, &["--json", "-q", "hola"]);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        !output.status.success(),
        "el proceso debe terminar con código distinto de cero"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout debe quedar sin JSON parcial, fue: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error"),
        "stderr debe contener un mensaje legible, fue: {stderr:?}"
    );
}
