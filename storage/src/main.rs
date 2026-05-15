use common::error::Result;
use common::http::parse_request;
use common::http::send_response;
use common::models::ObjectStore;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;

fn handle_request(mut stream: TcpStream, store: Arc<ObjectStore>) -> Result<()> {
    let req = parse_request(&mut stream)?;
    let method = req.method;
    let path = req.path;

    match (method.as_str(), path.as_str()) {
        ("PUT", p) if p.starts_with("/storage/") => {
            let key = p.trim_start_matches("/storage/");
            store.put(key, &req.body)?;
            println!("[STORAGE] Fichier stocké : {}", key);
            send_response(&mut stream, "201 Created", key)?;
        }
        ("GET", "/storage") => {
            let keys = store.list()?;
            let json = serde_json::to_string(&keys)?;
            send_response(&mut stream, "200 OK", &json)?;
        }
        ("GET", p) if p.starts_with("/storage/") => {
            let key = p.trim_start_matches("/storage/");
            match store.get(key) {
                Some(data) => {
                    let body = String::from_utf8_lossy(&data);
                    send_response(&mut stream, "200 OK", &body)?;
                }
                None => send_response(&mut stream, "404 Not Found", "Not found")?,
            }
        }
        ("DELETE", p) if p.starts_with("/storage/") => {
            let key = p.trim_start_matches("/storage/");
            store.delete(key)?;
            send_response(&mut stream, "200 OK", "Deleted")?;
        }
        _ => {
            send_response(&mut stream, "404 Not Found", "Not found")?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let storage_addr = std::env::var("STORAGE_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());
    let store = Arc::new(ObjectStore::new("/tmp/my-storage")?);

    let listener = TcpListener::bind(&storage_addr)?;
    println!("[STORAGE] Démarrage sur {}", storage_addr);

    for stream in listener.incoming() {
        let stream = stream?;
        let store_clone = Arc::clone(&store);
        std::thread::spawn(move || {
            if let Err(e) = handle_request(stream, store_clone) {
                eprintln!("Request error: {}", e);
            }
        });
    }
    Ok(())
}
