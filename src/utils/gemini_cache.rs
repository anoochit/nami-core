use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Sha256, Digest};
use crate::utils::{get_nami_dir, get_http_client};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheState {
    pub cache_name: String,
    pub model_name: String,
    pub file_hashes: HashMap<String, String>,
}

static CACHE_STATE_FILE: OnceLock<PathBuf> = OnceLock::new();

fn get_cache_state_file() -> &'static PathBuf {
    CACHE_STATE_FILE.get_or_init(|| get_nami_dir().join("gemini_cache_state.json"))
}

/// Computes SHA256 of file contents
async fn compute_file_hash(path: &Path) -> Option<String> {
    let content = tokio::fs::read(path).await.ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Some(result.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Recursively scans workspace directories to collect eligible files and compute their hashes.
async fn scan_workspace_files(root: &Path) -> HashMap<String, PathBuf> {
    let mut files = HashMap::new();
    let mut queue = vec![root.to_path_buf()];

    while let Some(dir) = queue.pop() {
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue,
        };

        while let Some(entry) = entries.next_entry().await.ok().flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            // Ignore common output, temporary, and dependency directories
            if path.is_dir() {
                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
                    || file_name == "workspace"
                    || file_name == "dist"
                    || file_name == "build"
                {
                    continue;
                }
                queue.push(path);
            } else {
                // Keep file list restricted to text-based or source code files for caching
                let extension = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
                if extension == "rs"
                    || extension == "toml"
                    || extension == "md"
                    || extension == "json"
                    || extension == "js"
                    || extension == "ts"
                    || extension == "tsx"
                    || extension == "css"
                    || extension == "html"
                    || extension == "yaml"
                    || extension == "yml"
                {
                    if let Ok(relative) = path.strip_prefix(root) {
                        files.insert(relative.to_string_lossy().to_string(), path);
                    }
                }
            }
        }
    }
    files
}

/// Smart cache invalidation handler for Gemini model context.
/// Checks if files in the workspace have been modified, invalidates and deletes the old cache, and creates a fresh context cache.
pub async fn get_or_create_context_cache(model_name: &str) -> Option<String> {
    let api_key = std::env::var("GOOGLE_API_KEY").ok()?;
    if api_key.is_empty() {
        return None;
    }

    let root = std::env::current_dir().ok()?;
    let workspace_files = scan_workspace_files(&root).await;

    // 1. Compute current SHA256 hashes of all workspace files
    let mut current_hashes = HashMap::new();
    for (rel_path, abs_path) in &workspace_files {
        if let Some(hash) = compute_file_hash(abs_path).await {
            current_hashes.insert(rel_path.clone(), hash);
        }
    }

    // 2. Load stored cache state
    let state_file = get_cache_state_file();
    let mut existing_state: Option<CacheState> = None;
    if state_file.exists() {
        if let Ok(content) = tokio::fs::read_to_string(state_file).await {
            existing_state = serde_json::from_str(&content).ok();
        }
    }

    // 3. Determine if cache invalidation is needed
    let mut cache_is_valid = false;
    if let Some(ref state) = existing_state {
        if state.model_name == model_name && state.file_hashes.len() == current_hashes.len() {
            cache_is_valid = state.file_hashes.iter().all(|(k, v)| {
                current_hashes.get(k) == Some(v)
            });
        }
    }

    if cache_is_valid {
        if let Some(state) = existing_state {
            log::info!("[Gemini Cache] Reusing existing context cache: {}", state.cache_name);
            return Some(state.cache_name);
        }
    }

    // 4. Invalidation Triggered: Delete old cache resource on Google API if one exists
    if let Some(state) = existing_state {
        log::info!("[Gemini Cache] Invalidation triggered. Deleting stale context cache: {}", state.cache_name);
        let client = get_http_client();
        let delete_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/{}?key={}",
            state.cache_name, api_key
        );
        let _ = client.delete(&delete_url).send().await;
    }

    // 5. Concatenate workspace files to form the warm static context
    let mut context_builder = String::new();
    context_builder.push_str("━━━ REPOSITORY CONTEXT MATRIX ━━━\n\n");
    for (rel_path, abs_path) in &workspace_files {
        if let Ok(content) = tokio::fs::read_to_string(abs_path).await {
            context_builder.push_str(&format!("--- FILE: {} ---\n", rel_path));
            context_builder.push_str(&content);
            context_builder.push_str("\n\n");
        }
    }

    if context_builder.trim().is_empty() {
        return None;
    }

    // 6. Create new Gemini Context Cache via Google REST API
    log::info!("[Gemini Cache] Creating new context cache on Gemini API...");
    let client = get_http_client();
    let create_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/cachedContents?key={}",
        api_key
    );

    let request_body = json!({
        "model": format!("models/{}", model_name),
        "contents": [
            {
                "role": "user",
                "parts": [
                    {
                        "text": context_builder
                    }
                ]
            }
        ],
        "ttl": "3600s" // Warm cache TTL set to 1 hour
    });

    let response = client.post(&create_url)
        .json(&request_body)
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        if let Ok(err_text) = response.text().await {
            log::warn!("[Gemini Cache] Failed to create context cache: {}", err_text);
        }
        return None;
    }

    #[derive(Deserialize)]
    struct CreateResponse {
        pub name: String,
    }

    let create_res: CreateResponse = response.json().await.ok()?;
    let new_cache_name = create_res.name;

    log::info!("[Gemini Cache] Created cache successfully: {}", new_cache_name);

    // 7. Save new cache state locally
    let new_state = CacheState {
        cache_name: new_cache_name.clone(),
        model_name: model_name.to_string(),
        file_hashes: current_hashes,
    };

    if let Ok(serialized) = serde_json::to_string_pretty(&new_state) {
        let _ = tokio::fs::write(state_file, serialized).await;
    }

    Some(new_cache_name)
}
