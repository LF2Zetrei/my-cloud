use common::error::CloudError;
use common::error::Result;
use common::http::parse_request;
use common::http::send_error;
use common::http::send_response;
use common::models::ContainerRepository;
use common::models::FileRepository;
use common::models::Metrics;
use serde::Serialize;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Serialize)]
struct ProcessMetrics {
    container_id: String,
    pid: u32,
    cpu_percent: f64,
    memory_rss_kb: u64,
}

fn handle_request(stream: &mut TcpStream) -> Result<()> {
    let req = parse_request(stream)?;

    if req.method == "OPTIONS" {
        send_response(stream, "200 OK", "")?;
    } else if req.path == "/health" {
        send_response(stream, "200 OK", "OK")?;
    } else if req.path == "/metrics" {
        let metrics = collect_metrics()?;
        let json = serde_json::to_string(&metrics)?;
        send_response(stream, "200 OK", &json)?;
    } else if let Some(container_id) = req.path.strip_prefix("/metrics/") {
        match collect_process_metrics(container_id) {
            Ok(m) => {
                let json = serde_json::to_string(&m)?;
                send_response(stream, "200 OK", &json)?;
            }
            Err(e) => send_response(stream, "404 Not Found", &e.to_string())?,
        }
    } else {
        send_response(stream, "404 Not Found", "Not found")?;
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

/// Lit la ligne agrégée "cpu" de /proc/stat : (jiffies totaux, jiffies idle).
fn read_cpu_times() -> Result<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/stat")?;
    let line = content
        .lines()
        .find(|l| l.starts_with("cpu "))
        .ok_or_else(|| CloudError::Parse("cpu line not found".to_string()))?;

    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse::<u64>().unwrap_or(0))
        .collect();
    // idle + iowait
    let idle = values.get(3).copied().unwrap_or(0) + values.get(4).copied().unwrap_or(0);
    Ok((values.iter().sum::<u64>(), idle))
}

/// Les compteurs de /proc/stat sont cumulés depuis le boot : deux échantillons
/// à 100 ms d'intervalle donnent l'utilisation actuelle et non une moyenne depuis le boot.
fn read_cpu() -> Result<f64> {
    let (total1, idle1) = read_cpu_times()?;
    std::thread::sleep(Duration::from_millis(100));
    let (total2, idle2) = read_cpu_times()?;

    let delta_total = total2.saturating_sub(total1) as f64;
    let delta_idle = idle2.saturating_sub(idle1) as f64;
    if delta_total == 0.0 {
        return Ok(0.0);
    }
    Ok(((delta_total - delta_idle) / delta_total) * 100.0)
}

fn read_proc_stat_jiffies(pid: u32) -> Result<u64> {
    let content = std::fs::read_to_string(format!("/proc/{}/stat", pid))?;
    let fields: Vec<&str> = content.split_whitespace().collect();
    let utime = fields
        .get(13)
        .ok_or_else(|| CloudError::Parse("utime missing in /proc/pid/stat".to_string()))?
        .parse::<u64>()
        .map_err(|e| CloudError::Parse(e.to_string()))?;
    let stime = fields
        .get(14)
        .ok_or_else(|| CloudError::Parse("stime missing in /proc/pid/stat".to_string()))?
        .parse::<u64>()
        .map_err(|e| CloudError::Parse(e.to_string()))?;
    Ok(utime + stime)
}

fn read_total_cpu_jiffies() -> Result<u64> {
    let content = std::fs::read_to_string("/proc/stat")?;
    let line = content
        .lines()
        .find(|l| l.starts_with("cpu "))
        .ok_or_else(|| CloudError::Parse("cpu line not found in /proc/stat".to_string()))?;
    Ok(line
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse::<u64>().unwrap_or(0))
        .sum())
}

fn read_process_memory_kb(pid: u32) -> Result<u64> {
    let content = std::fs::read_to_string(format!("/proc/{}/status", pid))?;
    let line = content
        .lines()
        .find(|l| l.starts_with("VmRSS:"))
        .ok_or_else(|| CloudError::Parse("VmRSS not found in /proc/pid/status".to_string()))?;
    line.split_whitespace()
        .nth(1)
        .ok_or_else(|| CloudError::Parse("VmRSS value missing".to_string()))?
        .parse::<u64>()
        .map_err(|e| CloudError::Parse(e.to_string()))
}

fn collect_process_metrics(container_id: &str) -> Result<ProcessMetrics> {
    let containers_file = std::env::var("CONTAINERS_FILE")
        .unwrap_or_else(|_| "/var/my-cloud/containers.json".to_string());
    let repo = FileRepository::new(&containers_file)?;
    let container = repo
        .get(container_id)?
        .ok_or_else(|| CloudError::Parse(format!("container {} not found", container_id)))?;
    let pid = container
        .pid
        .ok_or_else(|| CloudError::Parse(format!("container {} has no PID", container_id)))?;

    // Deux échantillons à 100ms d'intervalle pour calculer le % CPU
    let proc1 = read_proc_stat_jiffies(pid)?;
    let total1 = read_total_cpu_jiffies()?;
    std::thread::sleep(Duration::from_millis(100));
    let proc2 = read_proc_stat_jiffies(pid)?;
    let total2 = read_total_cpu_jiffies()?;

    let delta_proc = proc2.saturating_sub(proc1) as f64;
    let delta_total = total2.saturating_sub(total1) as f64;
    let cpu_percent = if delta_total > 0.0 {
        (delta_proc / delta_total) * 100.0
    } else {
        0.0
    };

    Ok(ProcessMetrics {
        container_id: container_id.to_string(),
        pid,
        cpu_percent,
        memory_rss_kb: read_process_memory_kb(pid)?,
    })
}

fn collect_metrics() -> Result<Metrics> {
    let (mem_total, mem_used) = read_memory()?;
    let uptime = read_uptime()?;
    Ok(Metrics {
        cpu_usage: read_cpu()?,
        memory_used: mem_used,
        memory_total: mem_total,
        uptime_secs: uptime,
    })
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let observability_addr =
        std::env::var("OBSERVABILITY_ADDR").unwrap_or_else(|_| "0.0.0.0:9090".to_string());
    let listener = TcpListener::bind(&observability_addr)?;
    println!("[OBSERVABILITY] Démarrage sur {}", observability_addr);
    for stream in listener.incoming() {
        let mut stream = stream?;
        std::thread::spawn(move || {
            if let Err(e) = handle_request(&mut stream) {
                eprintln!("Request error: {}", e);
                let _ = send_error(&mut stream, &e);
            }
        });
    }
    Ok(())
}
