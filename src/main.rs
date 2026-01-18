use std::{
    net::TcpListener,
    process::{Child, Command, ExitCode},
    time::Duration,
};

use crate::http::TinyHttp;

mod http;

const PORT: u16 = 6768;

fn main() -> ExitCode {
    let mut args = std::env::args();
    args.next();
    let command = args.next().unwrap();
    let mut cmd = Command::new(command)
        .args(args)
        .spawn()
        .expect("Failed to start process");

    let listener =
        TcpListener::bind(("localhost", PORT)).expect("Failed to start server on port 6767");
    listener
        .set_nonblocking(true)
        .expect("Failed to set non blocking mode");

    open::that(format!("http://localhost:{}", PORT)).expect("Failed to open link");

    let mut tiny_http = TinyHttp::new();
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => tiny_http.handle_req(stream).unwrap(),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if let Some(code) = exit_code(&mut cmd) {
                    return ExitCode::from(code);
                }
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }

    ExitCode::SUCCESS
}

fn exit_code(cmd: &mut Child) -> Option<u8> {
    cmd.try_wait()
        .unwrap()
        .map(|e| e.code().map(|e| e.try_into().ok()).flatten().unwrap_or(0))
}
