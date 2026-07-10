use std::borrow::Cow;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use walkdir::WalkDir;

use crate::modes::command_registry::CommandRegistry;
use crate::utils::get_nami_dir;

/// Caching duration (TTL) of 5 seconds for file tab-completions
const CACHE_TTL: Duration = Duration::from_secs(5);

pub struct NamiHelper {
    cache: Mutex<Option<(Instant, Vec<String>)>>,
}

impl NamiHelper {
    pub fn new() -> Self {
        NamiHelper {
            cache: Mutex::new(None),
        }
    }

    /// Fetches all relative file paths under the active workspace, with caching.
    fn get_workspace_files(&self) -> Vec<String> {
        let mut cache_guard = self.cache.lock().unwrap();
        
        if let Some((cached_time, ref paths)) = *cache_guard {
            if cached_time.elapsed() < CACHE_TTL {
                return paths.clone();
            }
        }

        // Resolve dynamic workspace path
        let base_dir = if let Ok(env_ws) = std::env::var("NAMI_WORKSPACE") {
            if !env_ws.is_empty() {
                let path = std::path::PathBuf::from(env_ws);
                crate::utils::clean_unc_path(std::fs::canonicalize(&path).unwrap_or(path))
            } else if let Ok(current_dir) = std::env::current_dir() {
                crate::utils::clean_unc_path(std::fs::canonicalize(&current_dir).unwrap_or(current_dir))
            } else {
                std::path::PathBuf::from(".")
            }
        } else if let Ok(current_dir) = std::env::current_dir() {
            crate::utils::clean_unc_path(std::fs::canonicalize(&current_dir).unwrap_or(current_dir))
        } else {
            std::path::PathBuf::from(".")
        };

        let mut paths = Vec::new();
        if base_dir.exists() {
            for entry in WalkDir::new(&base_dir)
                .max_depth(5)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    name != ".git" && name != "target" && name != "node_modules" && name != "dist" && name != ".venv" && name != "build"
                })
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file()
                    && let Ok(relative_path) = entry.path().strip_prefix(&base_dir)
                {
                    let path_str = relative_path.to_string_lossy().replace("\\", "/");
                    paths.push(path_str);
                }
            }
        }

        *cache_guard = Some((Instant::now(), paths.clone()));
        paths
    }
}

impl Completer for NamiHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, word) =
            rustyline::completion::extract_word(line, pos, None, |c| c == ' ' || c == '\t');

        if word.starts_with('/') {
            let mut matches = Vec::new();
            let commands = vec![
                "/exit", "/quit", "/clear", "/new", "/status", "/switch", "/version"
            ];
            
            for cmd in commands {
                if cmd.to_lowercase().starts_with(&word.to_lowercase()) {
                    matches.push(Pair {
                        display: cmd.to_string(),
                        replacement: cmd.to_string(),
                    });
                }
            }
            
            // Try loading dynamic commands too
            let config_path = get_nami_dir().join("config.toml");
            if let Ok(registry) = CommandRegistry::load_from_config(&config_path.to_string_lossy()) {
                for name in registry.commands.keys() {
                    let cmd_with_slash = if name.starts_with('/') { name.clone() } else { format!("/{}", name) };
                    if cmd_with_slash.to_lowercase().starts_with(&word.to_lowercase()) {
                        matches.push(Pair {
                            display: cmd_with_slash.clone(),
                            replacement: cmd_with_slash,
                        });
                    }
                }
            }
            
            return Ok((start, matches));
        }

        if !word.is_empty() {
            let (replace_index, clean_search_pattern, prepend_at) = if let Some(path_part) = word.strip_prefix('@') {
                (start + 1, path_part.trim_start_matches(['/', '\\']), false)
            } else {
                (start, word.trim_start_matches(['/', '\\']), true)
            };

            let files = self.get_workspace_files();
            let mut matches = Vec::new();

            for path_str in files {
                if path_str.to_lowercase().contains(&clean_search_pattern.to_lowercase()) {
                    let replacement = if prepend_at {
                        format!("@{}", path_str)
                    } else {
                        path_str.clone()
                    };
                    matches.push(Pair {
                        display: path_str,
                        replacement,
                    });
                }
            }

            matches.truncate(10);
            return Ok((replace_index, matches));
        }

        Ok((0, Vec::new()))
    }
}

impl Hinter for NamiHelper {
    type Hint = String;
}

impl Highlighter for NamiHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        Cow::Borrowed(candidate)
    }

    fn highlight_char(
        &self,
        _line: &str,
        _pos: usize,
        _kind: rustyline::highlight::CmdKind,
    ) -> bool {
        false
    }
}

impl Validator for NamiHelper {}
impl Helper for NamiHelper {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationType {
    Esc,
    CtrlC,
}

/// Utility helper to check if a keypress event corresponds to a cancellation gesture (Esc or Ctrl+C)
pub fn check_cancellation_event(event: Event) -> Option<CancellationType> {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press {
            if key.code == KeyCode::Esc {
                return Some(CancellationType::Esc);
            } else if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Some(CancellationType::CtrlC);
            }
        }
    }
    None
}
