use common::error::CloudError;
use common::error::Result;
use common::http::send_response;
use common::models::Metrics;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;

fn handle_request(mut stream: TcpStream) -> Result<()> {
    let mut buf_reader = BufReader::new(&stream);

    let mut first_line = String::new();
    buf_reader.read_line(&mut first_line)?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() < 2 {
        return Err(CloudError::Parse("Invalid HTTP request".to_string()));
    }

    let path = parts[1].to_string();

    match path.as_str() {
        "/metrics" => {
            let metrics = collect_metrics()?;
            let json = serde_json::to_string(&metrics)?;
            send_response(&mut stream, "200 OK", &json)?;
        }
        "/health" => {
            send_response(&mut stream, "200 OK", "OK")?;
        }
        _ => send_response(&mut stream, "404 Not Found", "Not found")?,
    }
    Ok(())
}

fn read_memory() -> Result<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/meminfo")?;
    let raw_mem_total = content
        .lines()
        .find(|l| l.starts_with("MemTotal"))
        .ok_or_else(|| CloudError::Parse("MemTotal not found".to_string()))?;
    let raw_mem_available = content
        .lines()
        .find(|l| l.starts_with("MemAvailable"))
        .ok_or_else(|| CloudError::Parse("MemAvailable not found".to_string()))?;
    let mem_total = raw_mem_total
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| CloudError::Parse("MemTotal value missing".to_string()))?
        .parse::<u64>()
        .map_err(|e| CloudError::Parse(e.to_string()))?;
    let mem_available = raw_mem_available
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| CloudError::Parse("MemAvailable value missing".to_string()))?
        .parse::<u64>()
        .map_err(|e| CloudError::Parse(e.to_string()))?;

    Ok((mem_total, mem_total - mem_available))
}

fn read_uptime() -> Result<u64> {
    let content = std::fs::read_to_string("/proc/uptime")?;
    let first_number = content
        .split_whitespace()
        .next()
        .ok_or_else(|| CloudError::Parse("Uptime value missing".to_string()))?
        .parse::<f64>()
        .map_err(|e| CloudError::Parse(e.to_string()))?;

    Ok(first_number as u64)
}

fn collect_metrics() -> Result<Metrics> {
    let (mem_total, mem_used) = read_memory()?;
    let uptime = read_uptime()?;
    Ok(Metrics {
        cpu_usage: 0.0,
        memory_used: mem_used,
        memory_total: mem_total,
        uptime_secs: uptime,
    })
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let observability_addr = std::env::var("OBSERVABILITY_ADDR").unwrap_or_else(|_| "0.0.0.0:9090".to_string());
    let listener = TcpListener::bind(&observability_addr)?;
    println!("[OBSERVABILITY] Démarrage sur {}", observability_addr);
    for stream in listener.incoming() {
        let stream = stream?;
        std::thread::spawn(move || {
            if let Err(e) = handle_request(stream) {
                eprintln!("Request error: {}", e);
            }
        });
    }
    Ok(())
}
