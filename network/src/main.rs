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
    let content_type = req
        .headers
        .get("content-type")
        .map(|ct| format!("Content-Type: {}\r\n", ct))
        .unwrap_or_default();
    // "Connection: close" : le backend ferme la connexion après sa réponse,
    // sinon read_to_end bloquerait sur un serveur keep-alive (ex. Next.js).
    let header = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n",
        req.method,
        path,
        target_addr,
        content_type,
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

fn handle_connection(
    mut stream: TcpStream,
    token: &str,
    orchestrator_addr: &str,
    storage_addr: &str,
    observability_addr: &str,
    dashboard_addr: &str,
) -> Result<()> {
    let req = parse_request(&stream)?;

    if is_protected(&req.method, &req.path) && !check_auth(&req, token) {
        send_response(&mut stream, "401 Unauthorized", "Invalid token")?;
        return Ok(());
    }

    let path = req.path.clone();
    let p = path.as_str();

    if req.method == "OPTIONS" {
        send_response(&mut stream, "200 OK", "")?;
    } else if p == "/health" {
        send_response(&mut stream, "200 OK", "OK")?;
    } else if p == "/metrics" || p.starts_with("/metrics/") {
        if let Err(e) = forward_request(&mut stream, &req, observability_addr, p) {
            eprintln!("[PROXY] observability unavailable: {}", e);
            send_response(&mut stream, "502 Bad Gateway", "observability unavailable")?;
        }
    } else if p.starts_with("/api") {
        let stripped = p.trim_start_matches("/api");
        let target = if stripped.is_empty() { "/" } else { stripped };
        if let Err(e) = forward_request(&mut stream, &req, orchestrator_addr, target) {
            eprintln!("[PROXY] orchestrator unavailable: {}", e);
            send_response(&mut stream, "502 Bad Gateway", "orchestrator unavailable")?;
        }
    } else if p.starts_with("/storage") {
        if let Err(e) = forward_request(&mut stream, &req, storage_addr, p) {
            eprintln!("[PROXY] storage unavailable: {}", e);
            send_response(&mut stream, "502 Bad Gateway", "storage unavailable")?;
        }
    } else if p.starts_with("/dashboard") {
        let stripped = p.trim_start_matches("/dashboard");
        let target = if stripped.is_empty() { "/" } else { stripped };
        if let Err(e) = forward_request(&mut stream, &req, dashboard_addr, target) {
            eprintln!("[PROXY] dashboard unavailable: {}", e);
            send_response(&mut stream, "502 Bad Gateway", "dashboard unavailable")?;
        }
    } else {
        send_response(&mut stream, "404 Not Found", "Not found")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let token = std::env::var("API_TOKEN").unwrap_or_else(|_| "secret".to_string());

    let network_addr = std::env::var("NETWORK_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let orchestrator_addr =
        std::env::var("ORCHESTRATOR_ADDR").unwrap_or_else(|_| "127.0.0.1:8081".to_string());
    let storage_addr =
        std::env::var("STORAGE_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_string());
    let observability_addr =
        std::env::var("OBSERVABILITY_ADDR").unwrap_or_else(|_| "127.0.0.1:9090".to_string());
    let dashboard_addr =
        std::env::var("DASHBOARD_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    let listener = TcpListener::bind(&network_addr)?;
    println!("[PROXY] Démarrage sur {}", network_addr);

    for stream in listener.incoming() {
        let stream = stream?;
        let token = token.clone();
        let orchestrator_addr = orchestrator_addr.clone();
        let storage_addr = storage_addr.clone();
        let observability_addr = observability_addr.clone();
        let dashboard_addr = dashboard_addr.clone();
        std::thread::spawn(move || {
            if let Err(e) = handle_connection(
                stream,
                &token,
                &orchestrator_addr,
                &storage_addr,
                &observability_addr,
                &dashboard_addr,
            ) {
                eprintln!("[PROXY] Erreur: {}", e);
            }
        });
    }
    Ok(())
}
