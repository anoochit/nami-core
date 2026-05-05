use crate::utils::get_workspace_dir;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;
use tokio::fs;

pub struct NamiIgnore {
    set: GlobSet,
}

impl NamiIgnore {
    pub async fn load() -> Self {
        let mut builder = GlobSetBuilder::new();
        
        // Default ignores
        let defaults = vec![".git/**", "target/**", ".env", "sessions.db"];
        for pattern in defaults {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }

        if let Ok(root) = get_workspace_dir().await {
            let ignore_path = root.join(".namiignore");
            if let Ok(content) = fs::read_to_string(ignore_path).await {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Ok(glob) = Glob::new(trimmed) {
                        builder.add(glob);
                    }
                }
            }
        }

        Self {
            set: builder.build().unwrap_or_else(|_| GlobSetBuilder::new().build().unwrap()),
        }
    }

    pub fn is_ignored<P: AsRef<Path>>(&self, path: P) -> bool {
        self.set.is_match(path)
    }
}
