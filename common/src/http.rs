use crate::error::CloudError;
use crate::error::Result;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub fn send_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
         Access-Control-Allow-Headers: Authorization, Content-Type\r\n\
         \r\n",
        status,
        body.len()
    );
    let response = format!("{}{}", header, body);
    stream.write_all(response.as_bytes())?;
    Ok(())
}

/// Renvoie une erreur HTTP adaptée au type d'erreur, pour que le client
/// reçoive toujours une réponse au lieu d'une connexion fermée sans rien.
pub fn send_error(stream: &mut TcpStream, err: &CloudError) -> Result<()> {
    let status = match err {
        CloudError::Parse(_) | CloudError::Json(_) => "400 Bad Request",
        CloudError::NotFound(_) => "404 Not Found",
        _ => "500 Internal Server Error",
    };
    send_response(stream, status, &err.to_string())
}

pub fn parse_request(stream: &TcpStream) -> Result<HttpRequest> {
    let mut buf_reader = BufReader::new(stream);

    let mut first_line = String::new();
    buf_reader.read_line(&mut first_line)?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(CloudError::Parse("Invalid HTTP request line".to_string()));
    }
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    let version = parts[2].to_string();

    let mut headers = HashMap::new();
    let mut content_length = 0;
    loop {
        let mut line = String::new();
        buf_reader.read_line(&mut line)?;
        let line = line.trim_end().to_string();
        if line.is_empty() {
            break;
        }
        // Les noms de headers HTTP sont insensibles à la casse : on les stocke en minuscules
        // (Node/Next.js les envoie en minuscules, curl en "Title-Case").
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim();
            if key == "content-length" {
                content_length = value
                    .parse::<usize>()
                    .map_err(|e| CloudError::Parse(e.to_string()))?;
            }
            headers.insert(key, value.to_string());
        }
    }

    let mut body = vec![0u8; content_length];
    buf_reader.read_exact(&mut body)?;

    Ok(HttpRequest {
        method,
        path,
        version,
        headers,
        body,
    })
}

pub fn check_auth(req: &HttpRequest, token: &str) -> bool {
    let auth = match req.headers.get("authorization") {
        Some(a) => a,
        None => return false,
    };

    let t = auth.trim_start_matches("Bearer ");
    t == token
}

pub fn is_protected(method: &str, path: &str) -> bool {
    match (method, path) {
        ("OPTIONS", _) => false,
        (_, "/health") => false,
        (_, "/metrics") => false,
        ("GET", _) => false,
        _ => true,
    }
}
