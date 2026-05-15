use common::container::launch_container;
use common::error::CloudError;
use common::error::Result;
use common::http::parse_request;
use common::http::send_response;
use common::models::ContainerBuilder;
use common::models::ContainerRepository;
use common::models::ContainerStatus;
use common::models::FileRepository;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

fn reconcilie(state: Arc<dyn ContainerRepository + Send + Sync>) {
    loop {
        std::thread::sleep(Duration::from_secs(5));

        let containers = match state.list() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur liste containers: {}", e);
                continue;
            }
        };

        for container in containers {
            match container.status {
                ContainerStatus::Running => {
                    if let Some(pid) = container.pid {
                        let alive = std::path::Path::new(&format!("/proc/{}", pid)).exists();
                        if !alive {
                            println!("[ORCHESTRATOR] Container {} a crashé", container.name);
                            if let Err(e) =
                                state.update_status(&container.id, ContainerStatus::Crashed)
                            {
                                eprintln!("Erreur update status: {}", e);
                            }
                        }
                    } else {
                        match std::process::Command::new(&container.image).spawn() {
                            Ok(child) => {
                                let new_pid = child.id();
                                if let Err(e) = state.update_pid(&container.id, Some(new_pid)) {
                                    eprintln!("Erreur update pid: {}", e);
                                }
                                if let Err(e) =
                                    state.update_status(&container.id, ContainerStatus::Running)
                                {
                                    eprintln!("Erreur update status: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("Erreur spawn container {}: {}", container.name, e)
                            }
                        }
                    }
                }
                ContainerStatus::Stopped => {}
                ContainerStatus::Crashed => {
                    match std::process::Command::new(&container.image).spawn() {
                        Ok(child) => {
                            let new_pid = child.id();
                            if let Err(e) =
                                state.update_status(&container.id, ContainerStatus::Running)
                            {
                                eprintln!("Erreur update status: {}", e);
                            }
                            if let Err(e) = state.update_pid(&container.id, Some(new_pid)) {
                                eprintln!("Erreur update pid: {}", e);
                            }
                        }
                        Err(e) => eprintln!("Erreur spawn container {}: {}", container.name, e),
                    }
                }
            }
        }
    }
}

fn handle_request(
    mut stream: TcpStream,
    state: Arc<dyn ContainerRepository + Send + Sync>,
) -> Result<()> {
    let req = parse_request(&mut stream)?;
    let method = req.method;
    let path = req.path;
    let body = String::from_utf8_lossy(&req.body);

    match (method.as_str(), path.as_str()) {
        ("POST", "/containers") => {
            let json: serde_json::Value = serde_json::from_str(&body)?;
            let name = json["name"]
                .as_str()
                .ok_or_else(|| CloudError::Parse("Champ 'name' manquant".to_string()))?;
            let image = json["image"]
                .as_str()
                .ok_or_else(|| CloudError::Parse("Champ 'image' manquant".to_string()))?;
            let start = json["start"].as_bool().unwrap_or(false);
            let args: Vec<String> = json["args"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            let container = ContainerBuilder::new()
                .name(name)
                .image(image)
                .args(args)
                .build()?;

            state.add(container.clone())?;

            if start {
                let pid = launch_container(&container)?;
                state.update_pid(&container.id, Some(pid))?;
                state.update_status(&container.id, ContainerStatus::Running)?;
            }

            let response_body = serde_json::to_string(&container)?;
            send_response(&mut stream, "201 Created", &response_body)?;
        }
        ("GET", "/containers") => {
            let raw = state.list()?;
            let json = serde_json::to_string(&raw)?;
            send_response(&mut stream, "200 OK", &json)?;
        }
        ("DELETE", p) if p.starts_with("/containers/") => {
            let id = p.trim_start_matches("/containers/");
            state.delete(id)?;
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
    let orchestrator_addr =
        std::env::var("ORCHESTRATOR_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".to_string());
    let containers_file = std::env::var("CONTAINERS_FILE")
        .unwrap_or_else(|_| "/var/my-cloud/containers.json".to_string());
    let state: Arc<dyn ContainerRepository + Send + Sync> =
        Arc::new(FileRepository::new(&containers_file)?);
    let listener = TcpListener::bind(&orchestrator_addr)?;
    println!("[ORCHESTRATOR] Démarrage sur {}", orchestrator_addr);

    let state_reconcile = Arc::clone(&state);
    std::thread::spawn(move || {
        reconcilie(state_reconcile);
    });

    for stream in listener.incoming() {
        let stream = stream?;
        let state_clone = Arc::clone(&state);
        std::thread::spawn(move || {
            if let Err(e) = handle_request(stream, state_clone) {
                eprintln!("Erreur requête: {}", e);
            }
        });
    }
    Ok(())
}
