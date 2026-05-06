use crossterm::event::{ Event, EventStream, KeyCode, KeyEventKind };
use crossterm::style::Stylize;
use crossterm::{ cursor, execute, style, terminal };
use futures::StreamExt;
use futures_util::FutureExt;
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
        let (start, word) =
            rustyline::completion::extract_word(line, pos, None, |c| c == ' ' || c == '\t');


        if let Some(path_part) = word.strip_prefix('@') {
            let mut matches = Vec::new();

            // Search for files in the workspace directory
            let workspace_path = std::path::Path::new("workspace");
            if workspace_path.exists() {
                for entry in WalkDir::new(workspace_path)
                    .max_depth(5) // Don't go too deep to keep it fast
                    .into_iter()
                    .filter_map(|e| e.ok()) {
                    if entry.file_type().is_file()
                        && let Ok(relative_path) = entry.path().strip_prefix(workspace_path) {
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

        if path.exists() && path.is_file()
            && let Ok(metadata) = std::fs::metadata(&path) {
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

fn render_banner(provider: &str, model_name: &str) {
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
    println!("\nType /exit to quit, /clear to wipe terminal, /new to start a new chat.");
    println!("Type @ followed by path to reference files (use Tab for completion).");
    println!("Press ESC during a request to cancel it.\n");
}

async fn ensure_session(
    sessions: &Arc<dyn SessionService>,
    app_name: &str,
    user_id: &str,
    session_id: &str
) -> anyhow::Result<()> {
    // If the session already exists, we are good to go.
    if sessions.get(GetRequest {
        app_name: app_name.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        num_recent_events: Some(0),
        after: None,
    }).await.is_ok() {
        return Ok(());
    }

    // Otherwise, create it.
    sessions.create(CreateRequest {
        app_name: app_name.to_string(),
        user_id: user_id.to_string(),
        session_id: Some(session_id.to_string()),
        state: Default::default(),
    }).await?;
    
    Ok(())
}

fn render_error(e: anyhow::Error) {
    println!(
        "\n{} {}",
        style::style(" ❌ Oh no! Nami encountered a hiccup:").bold().red(),
        style::style(e.to_string()).red()
    );

    let err_debug = format!("{:?}", e);

    // Provide helpful hints for common issues
    if err_debug.contains("exclusiveMaximum") || err_debug.contains("exclusiveMinimum") {
        println!(
            "{}",
            style
                ::style(
                    " 💡 Hint: It looks like Gemini rejected a tool's parameter schema (unsupported constraints like 'exclusiveMaximum')."
                )
                .yellow()
                .dim()
        );
    } else if err_debug.contains("401") || err_debug.contains("unauthorized") {
        println!(
            "{}",
            style::style(" 💡 Hint: This might be an API key issue. Check your .env file!").yellow().dim()
        );
    } else if err_debug.contains("quota") || err_debug.contains("429") {
        println!(
            "{}",
            style
                ::style(" 💡 Hint: You've hit a rate limit. Take a quick break and try again!")
                .yellow()
                .dim()
        );
    }
}

pub(crate) async fn run_cli(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
    provider: String,
    model_name: String
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;

    render_banner(&provider, &model_name);

    let app_name = "cli";
    let user_id = "default_user";
    let session_id = Uuid::new_v4().to_string();

    ensure_session(&sessions, app_name, user_id, &session_id).await?;

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions.clone())
        .compaction_config(get_compaction_config(model))
        .build()?;

    let config = Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl: Editor<NamiHelper, rustyline::history::FileHistory> = Editor::with_config(config)?;
    rl.set_helper(Some(NamiHelper));
    let _ = rl.load_history(".cli_history");

    let mut nami_skin = MadSkin::default();
    nami_skin.paragraph.set_fg(termimad::crossterm::style::Color::White);
    nami_skin.bullet.set_fg(termimad::crossterm::style::Color::Magenta);

    handle_chat_loop(&mut rl, &sessions, &runner, &nami_skin, app_name, user_id, &session_id).await
}

async fn handle_chat_loop(
    rl: &mut Editor<NamiHelper, rustyline::history::FileHistory>,
    sessions: &Arc<dyn SessionService>,
    runner: &Runner,
    nami_skin: &MadSkin,
    app_name: &str,
    user_id: &str,
    session_id: &str
) -> anyhow::Result<()> {
    loop {
        let readline = rl.readline("You > ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
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
                    continue;
                }
                if trimmed == "/new" {
                    let _ = sessions.delete(adk_session::DeleteRequest {
                        app_name: app_name.to_string(),
                        user_id: user_id.to_string(),
                        session_id: session_id.to_string(),
                    }).await;
                    sessions.create(CreateRequest {
                        app_name: app_name.to_string(),
                        user_id: user_id.to_string(),
                        session_id: Some(session_id.to_string()),
                        state: Default::default(),
                    }).await?;
                    println!("\n{}\n", style::style("--- Session reset ---").dim());
                    continue;
                }

                let _ = rl.add_history_entry(trimmed);
                rl.save_history(".cli_history")?;

                let enriched_prompt = process_file_references(trimmed).await;

                let mut response_buffer = String::new();
                let is_thinking = Arc::new(AtomicBool::new(true));
                let indicator = is_thinking.clone();
                let handle = tokio::spawn(async move {
                    let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                    let mut i = 0;
                    while indicator.load(Ordering::Relaxed) {
                        print!(
                            "\r{} Thinking...",
                            style::style(spinner[i % 10]).with(style::Color::Magenta)
                        );
                        io::stdout().flush().ok();
                        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
                        i += 1;
                    }
                });

                let content = Content::new("user").with_text(enriched_prompt);
                let mut stream = runner.run_str(user_id, session_id, content).await?;

                let mut reader = EventStream::new();
                let mut cancelled = false;
                let mut stream_error = None;

                terminal::enable_raw_mode()?;

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
                                                print!("\r\x1B[K{} {}\r\n", style::style(" 🛠️ Calling:").dim(), style::style(name).cyan().bold());
                                                io::stdout().flush().ok();
                                            }
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    stream_error = Some(e);
                                    break;
                                }
                                None => break,
                            }
                        }
                        maybe_event = reader.next().fuse() => {
                            if let Some(Ok(Event::Key(key))) = maybe_event
                                && key.code == KeyCode::Esc && key.kind == KeyEventKind::Press {
                                    cancelled = true;
                                    break;
                                }
                        }
                    }
                }

                terminal::disable_raw_mode()?;
                is_thinking.store(false, Ordering::Relaxed);
                handle.await?;
                print!("\r\x1B[K");
                io::stdout().flush().ok();

                if let Some(e) = stream_error {
                    render_error(e.into());
                } else if cancelled {
                    println!("\n{}", style::style("--- Request cancelled ---").dim());
                } else {
                    println!("\n{}", style::style("Nami").bold().magenta());
                    nami_skin.print_text(&response_buffer);
                }
                println!();
            }
            Err(_) => {
                break;
            }
        }
    }
    Ok(())
}
