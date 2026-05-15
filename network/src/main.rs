use common::error::Result;
use common::http::HttpRequest;
use common::http::check_auth;
use common::http::is_protected;
use common::http::parse_request;
use common::http::send_response;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

fn forward_request(
    stream: &mut TcpStream,
    req: &HttpRequest,
    target_addr: &str,
    path: &str,
) -> Result<()> {
    let mut backend = TcpStream::connect(target_addr)?;
    let header = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\n\r\n",
        req.method,
        path,
        target_addr,
        req.body.len()
    );
    backend.write_all(header.as_bytes())?;
    if !req.body.is_empty() {
        backend.write_all(&req.body)?;
    }
    let mut response = Vec::new();
    backend.read_to_end(&mut response)?;
    stream.write_all(&response)?;
    Ok(())
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let token = std::env::var("API_TOKEN").unwrap_or_else(|_| "secret".to_string());

    let network_addr = std::env::var("NETWORK_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let orchestrator_addr =
        std::env::var("ORCHESTRATOR_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".to_string());
    let storage_addr = std::env::var("STORAGE_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());
    let observability_addr =
        std::env::var("OBSERVABILITY_ADDR").unwrap_or_else(|_| "0.0.0.0:9090".to_string());

    let listener = TcpListener::bind(&network_addr)?;
    println!("[PROXY] Démarrage sur {}", network_addr);

    for stream in listener.incoming() {
        let mut stream = stream?;
        let req = parse_request(&stream)?;

        if is_protected(&req.method, &req.path) && !check_auth(&req, &token) {
            send_response(&mut stream, "401 Unauthorized", "Invalid token")?;
            continue;
        }

        let path = req.path.clone();
        let p = path.as_str();

        if p == "/health" {
            send_response(&mut stream, "200 OK", "OK")?;
        } else if p == "/metrics" {
            forward_request(&mut stream, &req, &observability_addr, "/metrics")?;
        } else if p.starts_with("/api") {
            let stripped = p.trim_start_matches("/api");
            let target = if stripped.is_empty() { "/" } else { stripped };
            forward_request(&mut stream, &req, &orchestrator_addr, target)?;
        } else if p.starts_with("/storage") {
            forward_request(&mut stream, &req, &storage_addr, p)?;
        } else {
            send_response(&mut stream, "404 Not Found", "Not found")?;
        }
    }
    Ok(())
}
