//! Tests de integración del modo comando (`--cmd`, `--exec`, `--yes`).
//!
//! Comprueban el contrato observable del proceso: sin `--exec` nunca se ejecuta
//! nada, con `--exec --yes` se ejecuta (sin TTY) y el código de salida del hijo
//! se propaga, y con `--exec` sin `--yes` en un `stdin` que no es TTY se aborta.
//! Los comandos usados son inocuos (`exit N`), nunca destructivos.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;

/// Directorio temporal que se elimina automáticamente al terminar el test.
struct TempDir(std::path::PathBuf);

impl TempDir {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Directorio temporal único por test dentro del `temp_dir`.
fn temp_dir(name: &str) -> TempDir {
    let dir = std::env::temp_dir().join(format!("dime-cmd-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("crear directorio temporal");
    TempDir(dir)
}

/// Escribe un `dime.yml` que apunta a un proveedor `ollama` local (sin token).
fn write_config(dir: &std::path::Path, base_url: &str) {
    let content = format!(
        "provider: ollama\nbase_url: {base_url}\nuse_https: false\nmodel: llama2\nlog_level: error\n"
    );
    std::fs::write(dir.join("dime.yml"), content).expect("escribir config");
}

/// Arranca un servidor HTTP de un solo uso que responde `body` con 200 OK.
///
/// Devuelve la `base_url` (`host:puerto`) y el `JoinHandle` del hilo servidor.
fn spawn_mock_server(body: &'static str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock");
    let port = listener.local_addr().expect("addr").port();

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0u8; 4096];
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

/// Ejecuta el binario `dime` en `dir`, con `stdin` cerrado (nunca un TTY).
fn run_dime(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_dime"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("ejecutar dime")
}

/// Respuesta del modelo envolviendo `command` en un bloque Markdown.
fn fenced_response(command: &str) -> &'static str {
    match command {
        "exit 3" => r#"{"choices":[{"message":{"content":"```bash\nexit 3\n```"}}]}"#,
        "exit 7" => r#"{"choices":[{"message":{"content":"```sh\nexit 7\n```"}}]}"#,
        "echo hola" => r#"{"choices":[{"message":{"content":"```bash\necho hola\n```"}}]}"#,
        other => panic!("respuesta sin preparar para: {other}"),
    }
}

/// Sin `--exec` el comando se imprime y NO se ejecuta.
#[test]
fn cmd_without_exec_only_prints_and_never_executes() {
    let (base_url, server) = spawn_mock_server(fenced_response("exit 3"));
    let dir = temp_dir("no-exec");
    write_config(dir.path(), &base_url);

    let output = run_dime(dir.path(), &["--cmd", "salir con 3"]);
    let _ = server.join();

    assert_eq!(
        output.status.code(),
        Some(0),
        "sin --exec no debe propagarse el código del comando: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "exit 3\n", "stdout: {stdout:?}");
}

/// El comando se imprime siempre limpio: sin fences ni texto adicional.
#[test]
fn cmd_prints_clean_command_without_fences() {
    let (base_url, server) = spawn_mock_server(fenced_response("echo hola"));
    let dir = temp_dir("clean");
    write_config(dir.path(), &base_url);

    let output = run_dime(dir.path(), &["--cmd", "saludar"]);
    let _ = server.join();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "echo hola\n", "stdout: {stdout:?}");
}

/// Con `--exec --yes` se ejecuta sin prompt y se propaga el código del hijo.
#[test]
fn exec_yes_executes_and_propagates_exit_code() {
    let (base_url, server) = spawn_mock_server(fenced_response("exit 3"));
    let dir = temp_dir("exec-yes");
    write_config(dir.path(), &base_url);

    let output = run_dime(dir.path(), &["--cmd", "salir con 3", "--exec", "--yes"]);
    let _ = server.join();

    assert_eq!(
        output.status.code(),
        Some(3),
        "debe propagar el código 3 del comando: {output:?}"
    );
}

/// Con `--exec` sin `--yes` y `stdin` que no es TTY se aborta con código ≠ 0.
#[test]
fn exec_without_yes_in_non_tty_aborts_without_executing() {
    let (base_url, server) = spawn_mock_server(fenced_response("exit 7"));
    let dir = temp_dir("exec-no-tty");
    write_config(dir.path(), &base_url);

    let output = run_dime(dir.path(), &["--cmd", "salir con 7", "--exec"]);
    let _ = server.join();

    let code = output.status.code();
    assert!(
        code.is_some_and(|code| code != 0),
        "debe terminar con código distinto de cero: {output:?}"
    );
    assert_ne!(
        code,
        Some(7),
        "el comando NO debe haberse ejecutado: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--yes"),
        "el mensaje debe sugerir --yes: {stderr}"
    );
}
