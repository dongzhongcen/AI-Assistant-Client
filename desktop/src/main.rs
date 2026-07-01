use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

const INDEX_HTML: &str = include_str!("../../index.html");
const APP_JS: &str = include_str!("../../app.js");
const STYLES_CSS: &str = include_str!("../../styles.css");

fn main() {
    let args: Vec<String> = env::args().collect();
    let data_dir = data_dir();

    if args.iter().any(|arg| arg == "--data-dir") {
        println!("{}", data_dir.display());
        return;
    }

    if args.iter().any(|arg| arg == "--clear-data") {
        match clear_data(&data_dir) {
            Ok(()) => {
                println!("Cleaned data directory: {}", data_dir.display());
                return;
            }
            Err(error) => {
                eprintln!("Failed to clean data directory: {error}");
                std::process::exit(1);
            }
        }
    }

    if let Err(error) = fs::create_dir_all(&data_dir) {
        eprintln!("Failed to create data directory {}: {error}", data_dir.display());
        std::process::exit(1);
    }

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind local server");
    let port = listener.local_addr().expect("failed to read local address").port();
    let url = format!("http://127.0.0.1:{port}/");

    println!("AI Assistant Client");
    println!("URL: {url}");
    println!("Data: {}", data_dir.display());
    println!("Clean: AI-Assistant-Client.exe --clear-data");

    open_url(&url);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0_u8; 8192];
    let bytes = match stream.read(&mut buffer) {
        Ok(bytes) => bytes,
        Err(_) => return,
    };
    let request = String::from_utf8_lossy(&buffer[..bytes]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    match path {
        "/" | "/index.html" => respond(&mut stream, "text/html; charset=utf-8", INDEX_HTML.as_bytes()),
        "/app.js" => respond(&mut stream, "application/javascript; charset=utf-8", APP_JS.as_bytes()),
        "/styles.css" => respond(&mut stream, "text/css; charset=utf-8", STYLES_CSS.as_bytes()),
        "/health" => respond(&mut stream, "text/plain; charset=utf-8", b"ok"),
        _ => respond_not_found(&mut stream),
    }
}

fn respond(stream: &mut TcpStream, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

fn respond_not_found(stream: &mut TcpStream) {
    let body = b"Not found";
    let header = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

fn data_dir() -> PathBuf {
    if let Ok(value) = env::var("AI_ASSISTANT_CLIENT_DATA") {
        return PathBuf::from(value);
    }
    let base = env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|_| env::var("USERPROFILE").map(|home| PathBuf::from(home).join("AppData").join("Local")))
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    base.join("AI-Assistant-Client")
}

fn clear_data(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn open_url(url: &str) {
    let edge_paths = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];

    for edge in edge_paths {
        if Path::new(edge).exists() {
            let _ = Command::new(edge)
                .args([
                    "--app",
                    url,
                    "--user-data-dir",
                    &data_dir().join("edge-profile").display().to_string(),
                    "--no-first-run",
                    "--disable-features=Translate",
                ])
                .spawn();
            return;
        }
    }

    let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
}
