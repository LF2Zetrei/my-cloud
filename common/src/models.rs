use crate::error::CloudError;
use crate::error::Result;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metrics {
    pub cpu_usage: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub uptime_secs: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Crashed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub args: Vec<String>,
    pub status: ContainerStatus,
    pub pid: Option<u32>,
}

pub struct ObjectStore {
    pub base_path: String,
}

impl ObjectStore {
    pub fn new(base_path: &str) -> Result<ObjectStore> {
        std::fs::create_dir_all(format!("{}/data", base_path))?;
        std::fs::create_dir_all(format!("{}/meta", base_path))?;

        Ok(ObjectStore {
            base_path: base_path.to_string(),
        })
    }

    /// Chemin du fichier d'une clé. Les clés sont des noms plats : on refuse
    /// les séparateurs et ".." pour qu'une clé ne puisse pas sortir de data/.
    fn data_path(&self, key: &str) -> Result<String> {
        if key.is_empty() || key == "." || key == ".." || key.contains(['/', '\\', '\0']) {
            return Err(CloudError::Parse(format!("clé invalide : {:?}", key)));
        }
        Ok(format!("{}/data/{}", self.base_path, key))
    }

    pub fn put(&self, key: &str, data: &[u8]) -> Result<()> {
        std::fs::write(self.data_path(key)?, data)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        std::fs::read(self.data_path(key).ok()?).ok()
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let file_path = self.data_path(key)?;
        std::fs::remove_file(&file_path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => CloudError::NotFound(key.to_string()),
            _ => CloudError::Io(e),
        })
    }

    pub fn list(&self) -> Result<Vec<String>> {
        std::fs::read_dir(format!("{}/data", self.base_path))?
            .map(|entry| {
                entry
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .map_err(CloudError::from)
            })
            .collect()
    }
}

pub trait ContainerRepository: Send + Sync {
    fn add(&self, container: Container) -> Result<()>;
    fn list(&self) -> Result<Vec<Container>>;
    fn get(&self, id: &str) -> Result<Option<Container>>;
    fn delete(&self, id: &str) -> Result<()>;
    fn update_status(&self, id: &str, status: ContainerStatus) -> Result<()>;
    fn update_pid(&self, id: &str, pid: Option<u32>) -> Result<()>;
}

pub struct InMemoryRepository {
    containers: Arc<Mutex<Vec<Container>>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        InMemoryRepository {
            containers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ContainerRepository for InMemoryRepository {
    fn add(&self, container: Container) -> Result<()> {
        self.containers
            .lock()
            .map_err(|e| CloudError::Parse(e.to_string()))?
            .push(container);
        Ok(())
    }

    fn list(&self) -> Result<Vec<Container>> {
        Ok(self
            .containers
            .lock()
            .map_err(|e| CloudError::Parse(e.to_string()))?
            .clone())
    }

    fn get(&self, id: &str) -> Result<Option<Container>> {
        Ok(self
            .containers
            .lock()
            .map_err(|e| CloudError::Parse(e.to_string()))?
            .iter()
            .find(|c| c.id == id)
            .cloned())
    }

    fn update_status(&self, id: &str, status: ContainerStatus) -> Result<()> {
        let mut containers = self
            .containers
            .lock()
            .map_err(|e| CloudError::Parse(e.to_string()))?;
        if let Some(c) = containers.iter_mut().find(|c| c.id == id) {
            c.status = status;
        }
        Ok(())
    }

    fn update_pid(&self, id: &str, pid: Option<u32>) -> Result<()> {
        let mut containers = self
            .containers
            .lock()
            .map_err(|e| CloudError::Parse(e.to_string()))?;
        if let Some(c) = containers.iter_mut().find(|c| c.id == id) {
            c.pid = pid;
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.containers
            .lock()
            .map_err(|e| CloudError::Parse(e.to_string()))?
            .retain(|c| c.id != id);
        Ok(())
    }
}

pub struct ContainerBuilder {
    name: Option<String>,
    image: Option<String>,
    args: Vec<String>,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        ContainerBuilder {
            name: None,
            image: None,
            args: Vec::new(),
        }
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn image(mut self, image: &str) -> Self {
        self.image = Some(image.to_string());
        self
    }

    pub fn build(self) -> Result<Container> {
        let name = self
            .name
            .ok_or_else(|| CloudError::Parse("name manquant".to_string()))?;
        let image = self
            .image
            .ok_or_else(|| CloudError::Parse("image manquant".to_string()))?;

        Ok(Container {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            image,
            args: self.args,
            status: ContainerStatus::Stopped,
            pid: None,
        })
    }
}

pub struct FileRepository {
    path: String,
    // Sérialise les cycles load → modification → save entre threads
    lock: Mutex<()>,
}

impl FileRepository {
    pub fn new(path: &str) -> Result<Self> {
        std::fs::create_dir_all(
            std::path::Path::new(path)
                .parent() // récupère le dossier parent
                .unwrap_or(std::path::Path::new(".")),
        )?;

        if !std::path::Path::new(path).exists() {
            std::fs::write(path, "[]")?;
        }

        Ok(FileRepository {
            path: path.to_string(),
            lock: Mutex::new(()),
        })
    }

    pub fn load(&self) -> Result<Vec<Container>> {
        let content = std::fs::read_to_string(&self.path)?;
        let containers = serde_json::from_str(&content)?;
        Ok(containers)
    }

    fn guard(&self) -> Result<std::sync::MutexGuard<'_, ()>> {
        self.lock.lock().map_err(|e| CloudError::Parse(e.to_string()))
    }

    pub fn save(&self, containers: &Vec<Container>) -> Result<()> {
        let json = serde_json::to_string_pretty(containers)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }
}

impl ContainerRepository for FileRepository {
    fn add(&self, container: Container) -> Result<()> {
        let _g = self.guard()?;
        let mut containers = self.load()?;
        containers.push(container);
        self.save(&containers)
    }

    fn list(&self) -> Result<Vec<Container>> {
        let _g = self.guard()?;
        self.load()
    }

    fn get(&self, id: &str) -> Result<Option<Container>> {
        let _g = self.guard()?;
        Ok(self.load()?.into_iter().find(|c| c.id == id))
    }

    fn delete(&self, id: &str) -> Result<()> {
        let _g = self.guard()?;
        let mut containers = self.load()?;
        containers.retain(|c| c.id != id);
        self.save(&containers)
    }

    fn update_status(&self, id: &str, status: ContainerStatus) -> Result<()> {
        let _g = self.guard()?;
        let mut containers = self.load()?;
        if let Some(c) = containers.iter_mut().find(|c| c.id == id) {
            c.status = status;
        }
        self.save(&containers)
    }

    fn update_pid(&self, id: &str, pid: Option<u32>) -> Result<()> {
        let _g = self.guard()?;
        let mut containers = self.load()?;
        if let Some(c) = containers.iter_mut().find(|c| c.id == id) {
            c.pid = pid;
        }
        self.save(&containers)
    }
}
