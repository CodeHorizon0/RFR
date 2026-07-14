use crate::worker::JsWorker;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::{path::Path, sync::Arc};
use tokio::fs;

pub struct FunctionsRuntime {
    workers: Arc<DashMap<String, Arc<JsWorker>>>,
}

impl FunctionsRuntime {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(DashMap::new()),
        }
    }

    pub async fn load_directory(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut entries = fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|x| x.to_str()) == Some("js") {
                let name = path.file_stem()
                    .ok_or_else(|| anyhow!("invalid file"))?
                    .to_string_lossy()
                    .to_string();

                let code = fs::read_to_string(path).await?;
                self.deploy(&name, &code).await?;
            }
        }

        Ok(())
    }

    pub async fn deploy(&self, name: &str, source: &str) -> Result<()> {
        let worker = Arc::new(JsWorker::new(source.to_string()).await?);
        self.workers.insert(name.to_string(), worker);
        Ok(())
    }

    pub async fn execute(&self, name: &str, request: String) -> Result<String> {
        let worker = self.workers
            .get(name)
            .ok_or_else(|| anyhow!("function not found"))?;

        worker.execute(request).await
    }
}
