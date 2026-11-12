//! Servidor web estàtic mínim, similar a `python3 -m http.server`.
//! Només fa servir la llibreria estàndard de Rust (cap dependència).
//!
//! Ús: http-server [--bind ADREÇA] [--port PORT] [--dir CARPETA] [PORT]
//! Valors per defecte: --bind 0.0.0.0  --port 54321  --dir .

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_BIND: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 54321;

struct Config {
    bind: String,
    port: u16,
    root: PathBuf,
}

fn usage() -> ! {
    println!(
        "Ús: http-server [--bind ADREÇA] [--port PORT] [--dir CARPETA] [PORT]\n\
         \n\
         Serveix els fitxers d'una carpeta per HTTP (com python3 -m http.server).\n\
         \n\
         Opcions:\n\
         \x20 -b, --bind ADREÇA   Adreça on escoltar (per defecte {DEFAULT_BIND})\n\
         \x20 -p, --port PORT     Port (per defecte {DEFAULT_PORT})\n\
         \x20 -d, --dir CARPETA   Carpeta a servir (per defecte la carpeta actual)\n\
         \x20 -h, --help          Mostra aquesta ajuda"
    );
    std::process::exit(0);
}

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(2);
}

fn parse_args() -> Config {
    let mut bind = DEFAULT_BIND.to_string();
    let mut port = DEFAULT_PORT;
    let mut dir = PathBuf::from(".");
    let mut args = std::env::args().skip(1);
    let parse_port = |v: &str| v.parse::<u16>().unwrap_or_else(|_| fail(&format!("port no vàlid: {v}")));

    while let Some(arg) = args.next() {
        let (key, inline) = match arg.split_once('=') {
            Some((k, v)) if k.starts_with("--") => (k.to_string(), Some(v.to_string())),
            _ => (arg.clone(), None),
        };
        let mut value = |name: &str| inline.clone().or_else(|| args.next()).unwrap_or_else(|| fail(&format!("falta el valor de {name}")));
        match key.as_str() {
            "-h" | "--help" => usage(),
            "-b" | "--bind" => bind = value("--bind"),
            "-p" | "--port" => port = parse_port(&value("--port")),
            "-d" | "--dir" | "--directory" => dir = PathBuf::from(value("--dir")),
            other if !other.starts_with('-') => port = parse_port(other),
            other => fail(&format!("opció desconeguda: {other} (prova --help)")),
        }
    }
    let root = fs::canonicalize(&dir).unwrap_or_else(|e| fail(&format!("carpeta {}: {e}", dir.display())));
    Config { bind, port, root }
}

fn main() {
    let cfg = Arc::new(parse_args());
    let addr = format!("{}:{}", cfg.bind, cfg.port);
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| fail(&format!("no puc escoltar a {addr}: {e}")));
    println!("Servint {} a http://{addr}/ (Ctrl+C per aturar)", cfg.root.display());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cfg = Arc::clone(&cfg);
                thread::spawn(move || {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
                    if let Err(e) = handle(stream, &cfg) {
                        if e.kind() != io::ErrorKind::BrokenPipe && e.kind() != io::ErrorKind::ConnectionReset {
                            eprintln!("error de connexió: {e}");
                        }
                    }
                });
            }
            Err(e) => eprintln!("error acceptant connexió: {e}"),
        }
    }
}

fn handle(stream: TcpStream, cfg: &Config) -> io::Result<()> {
    let peer = stream.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|_| "-".into());
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut out = stream;

    let mut request_line = String::new();
    if reader.by_ref().take(8192).read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    // Llegir (i descartar) les capçaleres
    loop {
        let mut line = String::new();
        if reader.by_ref().take(8192).read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
    }

    let mut parts = request_line.split_whitespace();
    let (method, target) = match (parts.next(), parts.next()) {
        (Some(m), Some(t)) => (m.to_string(), t.to_string()),
        _ => return send_error(&mut out, &peer, "-", "-", 400, "Bad Request"),
    };
    let head_only = method == "HEAD";
    if method != "GET" && !head_only {
        return send_error(&mut out, &peer, &method, &target, 501, "Not Implemented");
    }

    let raw_path = target.split(['?', '#']).next().unwrap_or("/");
    let decoded = match percent_decode(raw_path) {
        Some(p) => p,
        None => return send_error(&mut out, &peer, &method, &target, 400, "Bad Request"),
    };

    // Construir la ruta de manera segura (sense sortir de la carpeta arrel)
    let mut path = cfg.root.clone();
    for comp in Path::new(&decoded).components() {
        match comp {
            Component::Normal(c) => path.push(c),
            Component::ParentDir => return send_error(&mut out, &peer, &method, &target, 403, "Forbidden"),
            _ => {}
        }
    }
    // Evitar enllaços simbòlics que surtin de l'arrel
    if let Ok(real) = fs::canonicalize(&path) {
        if !real.starts_with(&cfg.root) {
            return send_error(&mut out, &peer, &method, &target, 403, "Forbidden");
        }
    }

    if path.is_dir() {
        if !raw_path.ends_with('/') {
            let location = format!("{raw_path}/");
            log(&peer, &method, &target, 301);
            let header = format!(
                "HTTP/1.1 301 Moved Permanently\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            return out.write_all(header.as_bytes());
        }
        let index = path.join("index.html");
        if index.is_file() {
            return send_file(&mut out, &peer, &method, &target, &index, head_only);
        }
        return send_listing(&mut out, &peer, &method, &target, &path, &decoded, head_only);
    }
    if path.is_file() {
        return send_file(&mut out, &peer, &method, &target, &path, head_only);
    }
    send_error(&mut out, &peer, &method, &target, 404, "Not Found")
}

fn send_file(out: &mut TcpStream, peer: &str, method: &str, target: &str, path: &Path, head_only: bool) -> io::Result<()> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return send_error(out, peer, method, target, 403, "Forbidden"),
    };
    let len = file.metadata()?.len();
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {len}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        mime_type(path)
    );
    out.write_all(header.as_bytes())?;
    log(peer, method, target, 200);
    if !head_only {
        io::copy(&mut file, out)?;
    }
    out.flush()
}

fn send_listing(out: &mut TcpStream, peer: &str, method: &str, target: &str, dir: &Path, shown: &str, head_only: bool) -> io::Result<()> {
    let mut entries: Vec<(String, bool)> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| {
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            (e.file_name().to_string_lossy().into_owned(), is_dir)
        })
        .collect();
    entries.sort_by_key(|(name, _)| name.to_lowercase());

    let title = html_escape(&format!("Directory listing for {shown}"));
    let mut body = format!(
        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>{title}</title>\n</head>\n<body>\n<h1>{title}</h1>\n<hr>\n<ul>\n"
    );
    for (name, is_dir) in entries {
        let suffix = if is_dir { "/" } else { "" };
        body.push_str(&format!(
            "<li><a href=\"{}{suffix}\">{}{suffix}</a></li>\n",
            percent_encode(&name),
            html_escape(&name)
        ));
    }
    body.push_str("</ul>\n<hr>\n</body>\n</html>\n");

    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        body.len()
    );
    out.write_all(header.as_bytes())?;
    log(peer, method, target, 200);
    if !head_only {
        out.write_all(body.as_bytes())?;
    }
    out.flush()
}

fn send_error(out: &mut TcpStream, peer: &str, method: &str, target: &str, code: u16, reason: &str) -> io::Result<()> {
    let body = format!("<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\"><title>{code} {reason}</title></head><body><h1>{code} {reason}</h1></body></html>\n");
    let header = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    out.write_all(header.as_bytes())?;
    if method != "HEAD" {
        out.write_all(body.as_bytes())?;
    }
    log(peer, method, target, code);
    out.flush()
}

fn log(peer: &str, method: &str, target: &str, code: u16) {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (h, m, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    println!("{peer} - [{h:02}:{m:02}:{s:02} UTC] \"{method} {target}\" {code}");
}

fn mime_type(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "yaml" | "yml" => "application/yaml; charset=utf-8",
        "txt" | "md" => "text/plain; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "xml" => "application/xml",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    let decoded = String::from_utf8(out).ok()?;
    if decoded.contains('\0') { None } else { Some(decoded) }
}

fn percent_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
