use std::path::Path;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn load_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_json<T: Serialize>(path: &Path, data: &T) -> std::result::Result<(), String> {
    let serialized = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(path, serialized)
        .map_err(|e| format!("Failed to write file: {}", e))
}

pub fn load_json_async<T: DeserializeOwned>(path: &Path) -> impl std::future::Future<Output = Option<T>> {
    let path = path.to_path_buf();
    async move {
        if !path.exists() {
            return None;
        }
        let content = tokio::fs::read_to_string(&path).await.ok()?;
        serde_json::from_str(&content).ok()
    }
}

pub async fn save_json_async<T: Serialize>(path: &Path, data: &T) -> std::result::Result<(), String> {
    let serialized = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    tokio::fs::write(path, serialized)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))
}