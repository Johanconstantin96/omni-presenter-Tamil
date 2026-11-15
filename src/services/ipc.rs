use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::mpsc::Sender,
    time::Duration,
};

fn lock_path() -> PathBuf {
    std::env::temp_dir().join("omni_presenter_ipc.lock")
}

pub fn send_paths_to_running_instance(paths: &[PathBuf]) -> bool {
    let lock_file = lock_path();
    let Ok(content) = fs::read_to_string(&lock_file) else {
        return false;
    };
    let Some((port_str, token)) = content.trim().split_once(':') else {
        let _ = fs::remove_file(&lock_file);
        return false;
    };
    let Ok(port) = port_str.parse::<u16>() else {
        let _ = fs::remove_file(&lock_file);
        return false;
    };

    let Ok(mut stream) = TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_millis(600),
    ) else {
        let _ = fs::remove_file(&lock_file);
        return false;
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(1500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(1500)));

    let mut message = format!("TOKEN {token}\n");
    for p in paths {
        let absolute = std::path::absolute(p).unwrap_or_else(|_| p.clone());
        message.push_str(&format!("IMPORT {}\n", absolute.to_string_lossy()));
    }
    message.push_str("END\n");

    if stream.write_all(message.as_bytes()).is_err() {
        return false;
    }

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).is_ok() && response.trim() == "OK"
}

pub struct IpcServer {
    _token: String,
}

impl IpcServer {
    pub fn start(
        tx: Sender<PathBuf>,
        ctx: Option<eframe::egui::Context>,
    ) -> Option<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").ok()?;
        let port = listener.local_addr().ok()?.port();
        let token = crate::core::preferences::new_id("ipc");

        let lock_file = lock_path();
        if fs::write(&lock_file, format!("{port}:{token}")).is_err() {
            return None;
        }

        let token_clone = token.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
                let mut reader = BufReader::new(stream);
                let mut first_line = String::new();
                if reader.read_line(&mut first_line).is_err() {
                    continue;
                }
                let expected = format!("TOKEN {token_clone}");
                if first_line.trim() != expected {
                    continue;
                }

                let mut stream = reader.into_inner();
                let mut reader = BufReader::new(&mut stream);
                let mut paths = Vec::new();
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() || line.trim() == "END" || line.is_empty() {
                        break;
                    }
                    if let Some(path_str) = line.trim().strip_prefix("IMPORT ") {
                        let path = PathBuf::from(path_str.trim());
                        paths.push(path);
                    }
                }

                let _ = stream.write_all(b"OK\n");

                for p in paths {
                    let _ = tx.send(p);
                }
                if let Some(ctx) = &ctx {
                    ctx.request_repaint();
                }
            }
        });

        Some(Self { _token: token })
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = fs::remove_file(lock_path());
    }
}
