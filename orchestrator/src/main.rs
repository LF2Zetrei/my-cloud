use common::container::is_alive;
use common::container::launch_container;
use common::container::stop_container;
use common::error::CloudError;
use common::error::Result;
use common::http::parse_request;
use common::http::send_error;
use common::http::send_response;
use common::models::Container;
use common::models::ContainerBuilder;
use common::models::ContainerRepository;
use common::models::ContainerStatus;
use common::models::FileRepository;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

/// Relance un container via le même chemin que la création (args, overlay, cgroups).
fn restart(state: &Arc<dyn ContainerRepository + Send + Sync>, container: &Container) {
    match launch_container(container) {
        Ok(new_pid) => {
            if let Err(e) = state.update_pid(&container.id, Some(new_pid)) {
                eprintln!("Erreur update pid: {}", e);
            }
            if let Err(e) = state.update_status(&container.id, ContainerStatus::Running) {
                eprintln!("Erreur update status: {}", e);
            }
            println!("[ORCHESTRATOR] Container {} relancé (PID {})", container.name, new_pid);
        }
        Err(e) => eprintln!("Erreur spawn container {}: {}", container.name, e),
    }
}

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
                ContainerStatus::Running => match container.pid {
                    Some(pid) if !is_alive(pid) => {
                        println!("[ORCHESTRATOR] Container {} a crashé", container.name);
                        if let Err(e) = state.update_status(&container.id, ContainerStatus::Crashed)
                        {
                            eprintln!("Erreur update status: {}", e);
                        }
                    }
                    Some(_) => {}
                    None => restart(&state, &container),
                },
                ContainerStatus::Stopped => {}
                ContainerStatus::Crashed => restart(&state, &container),
            }
        }
    }
}

fn handle_request(
    stream: &mut TcpStream,
    state: &Arc<dyn ContainerRepository + Send + Sync>,
) -> Result<()> {
    let req = parse_request(stream)?;
    let method = req.method;
    let path = req.path;
    let body = String::from_utf8_lossy(&req.body);

    match (method.as_str(), path.as_str()) {
        ("OPTIONS", _) => {
            send_response(stream, "200 OK", "")?;
        }
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

            // Renvoie l'état à jour (status/pid modifiés si start=true)
            let container = state.get(&container.id)?.unwrap_or(container);
            let response_body = serde_json::to_string(&container)?;
            send_response(stream, "201 Created", &response_body)?;
        }
        ("GET", "/containers") => {
            let raw = state.list()?;
            let json = serde_json::to_string(&raw)?;
            send_response(stream, "200 OK", &json)?;
        }
        ("DELETE", p) if p.starts_with("/containers/") => {
            let id = p.trim_start_matches("/containers/");
            let container = state
                .get(id)?
                .ok_or_else(|| CloudError::NotFound(format!("container {}", id)))?;
            // On passe d'abord en Stopped pour que la boucle de réconciliation
            // ne relance pas le processus entre le kill et la suppression.
            state.update_status(id, ContainerStatus::Stopped)?;
            if let Some(pid) = container.pid {
                stop_container(pid)?;
            }
            state.delete(id)?;
            send_response(stream, "200 OK", "Deleted")?;
        }
        _ => {
            send_response(stream, "404 Not Found", "Not found")?;
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
        let mut stream = stream?;
        let state_clone = Arc::clone(&state);
        std::thread::spawn(move || {
            if let Err(e) = handle_request(&mut stream, &state_clone) {
                eprintln!("Erreur requête: {}", e);
                let _ = send_error(&mut stream, &e);
            }
        });
    }
    Ok(())
}
