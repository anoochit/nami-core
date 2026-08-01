pub mod client;
pub mod error;
pub mod retry;
pub mod paths;
pub mod stats;
pub mod session;
pub mod shell;
pub mod text;
pub mod persist;
pub mod env;
pub mod providers;

pub mod db;
pub mod gemini_cache;
pub mod ignore;
pub mod markdown;

pub use client::http_client;
pub use db::{init_db_pool, db_pool};
pub use error::{ErrorCategory, categorize_error, clean_error_message};
pub use retry::with_retry;
pub use paths::{get_http_client, get_nami_dir, clean_unc_path,
    get_workspace_dir, sandbox, sandbox_with_ignore, get_km_dir};
pub use stats::{save_agent_statistic, fetch_models_for_provider};
pub use session::ensure_session;
pub use shell::{build_shell_command, spawn_shell_command};
pub use text::read_file_lines;
pub use persist::{load_json, save_json, load_json_async, save_json_async};
pub use env::EnvVarGuard;
pub use providers::{provider_env_var, default_models};