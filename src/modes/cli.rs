use crossterm::{ cursor, execute, style, terminal, style::Stylize };
use crossterm::event::{ Event, KeyCode, EventStream, KeyModifiers, KeyEventKind };
use futures::StreamExt;
use regex::Regex;
use rustyline::completion::{ Completer, Pair };
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{ Config, Context, Editor, Helper };
use std::borrow::Cow;
use std::io::{ self, Write };
use std::sync::Arc;
use std::sync::atomic::{ AtomicBool, Ordering };
use termimad::MadSkin;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::agent::get_compaction_config;
use crate::agent::agent::{ check_config_mtime, get_config_mtime, get_skills_mtime, create_agent };
use crate::modes::ui_utils;
use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::{ CreateRequest, GetRequest, SessionService };

struct NamiHelper;

impl Completer for NamiHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, word) = rustyline::completion::extract_word(
            line,
            pos,
            None,
            |c| c == ' ' || c == '\t'
        );

        if let Some(path_part) = word.strip_prefix('@') {
            let mut matches = Vec::new();

            // Search for files in the workspace directory
            let workspace_path = std::path::Path::new("workspace");
            if workspace_path.exists() {
                for entry in WalkDir::new(workspace_path)
                    .max_depth(5) // Don't go too deep to keep it fast
                    .into_iter()
                    .filter_map(|e| e.ok()) {
                    if
                        entry.file_type().is_file() &&
                        let Ok(relative_path) = entry.path().strip_prefix(workspace_path)
                    {
                        let path_str = relative_path.to_string_lossy().replace("\\", "/");

                        if path_str.to_lowercase().contains(&path_part.to_lowercase()) {
                            matches.push(Pair {
                                display: path_str.clone(),
                                replacement: path_str,
                            });
                        }
                    }
                }
            }

            // Limit matches to avoid overwhelming the UI
            matches.truncate(10);

            return Ok((start + 1, matches));
        }

        Ok((0, Vec::with_capacity(0)))
    }
}

impl Hinter for NamiHelper {
    type Hint = String;
}

impl Highlighter for NamiHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool
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
        _completion: rustyline::CompletionType
    ) -> Cow<'c, str> {
        Cow::Borrowed(candidate)
    }

    fn highlight_char(
        &self,
        _line: &str,
        _pos: usize,
        _kind: rustyline::highlight::CmdKind
    ) -> bool {
        false
    }
}

impl Validator for NamiHelper {}
impl Helper for NamiHelper {}

async fn process_file_references(input: &str) -> String {
    let mut final_prompt = input.to_string();
    // Match @ followed by valid path characters
    let re = Regex::new(r"@([\w\./\-]+)").unwrap();

    let mut appended_context = String::new();
    let mut seen_files = std::collections::HashSet::new();

    for cap in re.captures_iter(input) {
        let file_path_str = &cap[1];
        if seen_files.contains(file_path_str) {
            continue;
        }
        seen_files.insert(file_path_str.to_string());

        let workspace_path = std::path::Path::new("workspace");
        let path = workspace_path.join(file_path_str);

        if path.exists() && path.is_file() && let Ok(metadata) = std::fs::metadata(&path) {
            let size = metadata.len();
            // Threshold: 4KB
            if size < 4096 {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    appended_context.push_str(
                        &format!(
                            "\n\n--- Content from {} ---\n{}\n--- End of content ---\n",
                            file_path_str,
                            content
                        )
                    );
                }
            } else {
                appended_context.push_str(
                    &format!(
                        "\n\n[REFERENCE: {} (Size: {} bytes)]\nThis file is too large for direct injection. Use your filesystem tools (read_file) to inspect specific parts of this file if needed.\n",
                        file_path_str,
                        size
                    )
                );
            }
        }
    }

    if !appended_context.is_empty() {
        final_prompt.push_str("\n\n[FILE CONTEXT]");
        final_prompt.push_str(&appended_context);
    }

    final_prompt
}

fn render_banner(provider: &str, model_name: &str, session_id: &str) {
    println!(
        "{}",
        style
            ::style(
                r#"
   _  _____   __  _______
  / |/ / _ | /  |/  /  _/
 /    / __ |/ /|_/ // /  
/_/|_/_/ |_/_/  /_/___/  
                         
"#
            )
            .magenta()
    );
    println!(
        "{} {}",
        style
            ::style(format!("Nami CLI v{}", env!("CARGO_PKG_VERSION")))
            .bold()
            .magenta(),
        style::style(format!("({}) using {}", provider, model_name)).dim()
    );
    println!(
        "{} {}",
        style::style("Session ID:").bold().magenta(),
        style::style(session_id).dim()
    );
    println!("\nType /? for slash commands.");
    println!("Type @ followed by path to reference files (use Tab for completion).");
    println!("Press ESC during a request to cancel it.\n");
}

pub(crate) async fn ensure_session(
    sessions: &Arc<dyn SessionService>,
    app_name: &str,
    user_id: &str,
    session_id: &str
) -> anyhow::Result<()> {
    if
        sessions
            .get(GetRequest {
                app_name: app_name.to_string(),
                user_id: user_id.to_string(),
                session_id: session_id.to_string(),
                num_recent_events: Some(0),
                after: None,
            }).await
            .is_ok()
    {
        return Ok(());
    }

    sessions.create(CreateRequest {
        app_name: app_name.to_string(),
        user_id: user_id.to_string(),
        session_id: Some(session_id.to_string()),
        state: Default::default(),
    }).await?;

    Ok(())
}

pub(crate) async fn run_cli(
    mut agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    mut model: Arc<dyn Llm>,
    mut provider: String,
    mut model_name: String
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;

    let app_name = "cli";
    let user_id = "default_user";
    let mut session_id = Uuid::new_v4().to_string();

    render_banner(&provider, &model_name, &session_id);

    ensure_session(&sessions, app_name, user_id, &session_id).await?;

    let mut runner = Runner::builder()
        .app_name(app_name)
        .agent(agent.clone())
        .session_service(sessions.clone())
        .compaction_config(get_compaction_config(model.clone()))
        .build()?;

    let config = Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl: Editor<NamiHelper, rustyline::history::FileHistory> = Editor::with_config(config)?;
    rl.set_helper(Some(NamiHelper));
    let _ = rl.load_history(".cli_history");

    let mut nami_skin = MadSkin::default();
    nami_skin.paragraph.set_fg(termimad::crossterm::style::Color::White);
    nami_skin.bullet.set_fg(termimad::crossterm::style::Color::Magenta);

    handle_chat_loop(
        &mut rl,
        &sessions,
        &mut runner,
        app_name,
        user_id,
        &mut session_id,
        &mut agent,
        &mut model,
        &mut provider,
        &mut model_name,
        &nami_skin
    ).await
}

async fn handle_chat_loop(
    rl: &mut Editor<NamiHelper, rustyline::history::FileHistory>,
    sessions: &Arc<dyn SessionService>,
    runner: &mut Runner,
    app_name: &str,
    user_id: &str,
    session_id: &mut String,
    agent: &mut Arc<dyn Agent>,
    model: &mut Arc<dyn Llm>,
    provider: &mut String,
    model_name: &mut String,
    nami_skin: &MadSkin
) -> anyhow::Result<()> {
    let mut last_config_mtime = get_config_mtime();
    let mut last_skills_mtime = get_skills_mtime();

    loop {
        let mut config_changed = false;
        if let Some(new_config) = check_config_mtime(&mut last_config_mtime) {
            let (new_agent, new_model) = create_agent(&new_config).await?;
            *agent = new_agent;
            *model = new_model;
            *provider = new_config.model.provider.clone();
            *model_name = new_config.model.model_name.clone();
            config_changed = true;
        }

        let current_skills_mtime = get_skills_mtime();
        if last_skills_mtime != current_skills_mtime {
            last_skills_mtime = current_skills_mtime;
            config_changed = true;
        }

        if config_changed {
            *runner = Runner::builder()
                .app_name(app_name)
                .agent(agent.clone())
                .session_service(sessions.clone())
                .compaction_config(get_compaction_config(model.clone()))
                .build()?;

            println!(
                "\n{}\n",
                style::style("🧠 Agent re-initialized with new config or skills").cyan().bold()
            );
        }

        let line = rl.readline("You > ");
        match line {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // --- SLASH COMMANDS ---
                if trimmed == "/?" {
                    println!(
                        "\n/?       - Show commands
                        \n/exit     - Quit
                        \n/clear    - Clear screen
                        \n/new      - New session
                        \n/tasks    - List active tasks
                        \n/plan     - Initialize task
                        \n/wiki     - Wiki search
                        \n/memo     - Save memory
                        \n/status   - Agent status
                        \n/version  - CLI version\n"
                    );
                    continue;
                }
                if trimmed == "/exit" || trimmed == "/quit" {
                    break;
                }
                if trimmed == "/clear" {
                    execute!(
                        io::stdout(),
                        terminal::Clear(terminal::ClearType::All),
                        cursor::MoveTo(0, 0)
                    )?;
                    render_banner(provider, model_name, session_id);
                    continue;
                }
                if trimmed == "/new" {
                    *session_id = Uuid::new_v4().to_string();
                    ensure_session(sessions, app_name, user_id, session_id).await?;
                    execute!(
                        io::stdout(),
                        terminal::Clear(terminal::ClearType::All),
                        cursor::MoveTo(0, 0)
                    )?;
                    render_banner(provider, model_name, session_id);
                    println!("{}\n", style::style("\u{2728} New session started").green().bold());
                    continue;
                }
                if trimmed == "/version" {
                    println!("Nami CLI v{}\n", env!("CARGO_PKG_VERSION"));
                    continue;
                }

                if
                    trimmed.starts_with("/tasks") ||
                    trimmed.starts_with("/plan") ||
                    trimmed.starts_with("/wiki") ||
                    trimmed.starts_with("/memo") ||
                    trimmed.starts_with("/status") ||
                    trimmed.starts_with("/parallel")
                {
                    let cmd_prompt = if trimmed.starts_with("/tasks") {
                        "list_active_tasks".to_string()
                    } else if trimmed.starts_with("/wiki") {
                        format!("wiki_search: {}", trimmed.replace("/wiki", "").trim())
                    } else if trimmed.starts_with("/memo") {
                        format!("save_memory: {}", trimmed.replace("/memo", "").trim())
                    } else if trimmed.starts_with("/status") {
                        "get_system_status".to_string()
                    } else if trimmed.starts_with("/parallel") {
                        let replacement = trimmed.replace("/parallel", "");
                        let raw_tasks = replacement.trim();
                        let mut tasks_json = Vec::new();
                        for t in raw_tasks.split(',') {
                            let parts: Vec<&str> = t.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                tasks_json.push(
                                    format!(
                                        "{{\"specialist\": \"{}\", \"prompt\": \"{}\"}}",
                                        parts[0].trim(),
                                        parts[1].trim()
                                    )
                                );
                            }
                        }
                        format!(
                            "Use parallel_tasks with: {{\"tasks\": [{}]}}",
                            tasks_json.join(",")
                        )
                    } else {
                        format!("Initialize task: {}", trimmed.replace("/plan", "").trim())
                    };

                    let content = Content::new("user").with_text(cmd_prompt);
                    if let Ok(mut stream) = runner.run_str(user_id, session_id, content).await {
                        while let Some(Ok(event)) = stream.next().await {
                            if let Some(c) = event.llm_response.content {
                                for part in c.parts {
                                    if let Some(text) = part.text() {
                                        let rendered = nami_skin.inline(text).to_string();
                                        print!("{}", rendered);
                                        io::stdout().flush().ok();
                                    }
                                }
                            }
                        }
                        println!();
                    }
                    continue;
                }
                // --- END SLASH COMMANDS ---

                let _ = rl.add_history_entry(trimmed);
                rl.save_history(".cli_history")?;

                let enriched_prompt = process_file_references(trimmed).await;

                // --- THINKING INDICATOR ---
                // We use a simple indicator since the agent message is already in context
                print!("{} Agent is thinking...", style::style("⏳").magenta());
                io::stdout().flush().ok();

                let content = Content::new("user").with_text(enriched_prompt);
                let mut stream = runner.run_str(user_id, session_id, content).await?;

                let mut response_buffer = String::new();
                let mut cancelled = false;

                // Loop for LLM Stream + Interrupt
                let _ = terminal::enable_raw_mode();
                let mut event_reader = EventStream::new();

                loop {
                    tokio::select! {
                        result = stream.next() => {
                            match result {
                                Some(Ok(event)) => {
                                    if let Some(content) = &event.llm_response.content {
                                        for part in &content.parts {
                                            if let Some(text) = part.text() { 
                                                response_buffer.push_str(text); 
                                            }
                                            if let Part::FunctionCall { name, .. } = part {
                                                println!("\n{} {}", style::style("🛠️ Calling:").dim(), style::style(name).cyan().bold());
                                                io::stdout().flush().ok();
                                            }
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    response_buffer.push_str(&format!("\nError: {}", e));
                                    break;
                                }
                                None => break,
                            }
                        }
                        maybe_event = event_reader.next() => {
                            if let Some(Ok(Event::Key(key))) = maybe_event {
                                if key.kind == KeyEventKind::Press && (key.code == KeyCode::Esc || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))) {
                                    cancelled = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                print!("\r\x1B[K"); // Clear the thinking line
                // --- END INDICATORS ---

                if cancelled {
                    let _ = terminal::disable_raw_mode();
                    println!("\n{}", style::style("🚀 Request cancelled").dim());
                } else {
                    // Final Pretty Render for blocks (tables, code, etc.)
                    if !response_buffer.is_empty() {
                        // Pre-render
                        let rendered = nami_skin
                            .term_text(&response_buffer)
                            .to_string()
                            .trim()
                            .to_string();

                        println!("\n{}", rendered);
                    }
                    let _ = terminal::disable_raw_mode();
                }
                println!();
                println!();
            }
            Err(_) => {
                break;
            }
        }
    }
    Ok(())
}
