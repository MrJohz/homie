use std::{path::Path, sync::Arc};

use tokio::{
    io::AsyncReadExt,
    sync::{RwLock, RwLockReadGuard},
};

use super::types::Task;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TaskToml {
    pub task: Vec<Task>,
}

#[derive(Clone)]
pub struct Store {
    tasks: Arc<RwLock<Vec<Task>>>,
}

impl Store {
    pub async fn from_file(path: impl AsRef<Path>) -> Self {
        let mut file = String::new();
        tokio::fs::File::open(path.as_ref())
            .await
            .unwrap()
            .read_to_string(&mut file)
            .await
            .unwrap();

        let TaskToml { task: tasks } = toml::from_str::<TaskToml>(&file).unwrap();

        Self {
            tasks: Arc::new(RwLock::new(tasks)),
        }
    }

    pub async fn tasks(&self) -> RwLockReadGuard<[Task]> {
        RwLockReadGuard::map(self.tasks.read().await, |item| item.as_slice())
    }
}
