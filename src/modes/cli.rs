use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::{cursor, execute, queue, style, style::Stylize, terminal};
use futures::StreamExt;
use regex::Regex;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Config, Context, Editor, Helper};
use std::borrow::Cow;
use std::io::{self, Write};
use std::sync::Arc;
use termimad::MadSkin;
use uuid::Uuid;
use walkdir::WalkDir;

// use crate::agent::agent::{check_config_mtime, create_agent, get_config_mtime, get_skills_mtime};
use crate::agent::get_compaction_config;
use crate::modes::command_registry::CommandRegistry;
use crate::utils::get_nami_dir;

use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::{CreateRequest, GetRequest, SessionService};

struct NamiHelper;

pub fn render_help(registry: &CommandRegistry) -> String {
    let mut help = String::new();
    help.push_str("Available Commands\n\n");
    
    // Render static commands
    help.push_str("- /exit: Quit\n");
    help.push_str("- /clear: Clear screen\n");
    help.push_str("- /new: New session\n");
    help.push_str("- /tasks: List active tasks\n");
    help.push_str("- /status: Agent status\n");
    help.push_str("- /version: CLI version\n");

    // Render dynamic commands from registry
    help.push_str("\nCustom Commands\n\n");
    let mut commands: Vec<_> = registry.commands.iter().collect();
    commands.sort_by(|a, b| a.0.cmp(b.0));
    for (name, cmd) in commands {
        help.push_str(&format!("- {}: {}\n", name, cmd.help));
    }

    help.push_str("\nExamples:\n  /plan Build AI research system\n  /wiki Rust async traits\n  /memo User prefers concise output\n");
    help
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

        if let Some(path_part) = word.strip_prefix('@') {
            let mut matches = Vec::new();

            let workspace_path = std::path::Path::new("workspace");

            if workspace_path.exists() {
                for entry in WalkDir::new(workspace_path)
                    .max_depth(5)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file()
                        && let Ok(relative_path) = entry.path().strip_prefix(workspace_path)
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

            matches.truncate(10);

            return Ok((start + 1, matches));
        }

        if word.starts_with('/') {
            let mut matches = Vec::new();
            let commands = vec![
                "/exit", "/quit", "/clear", "/new", "/tasks", "/status", "/version", "/help"
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

async fn process_file_references(input: &str) -> String {
    let mut final_prompt = input.to_string();

    let re = Regex::new(r"@([\w\./\-]+)").unwrap();

    let mut appended_context = String::new();
    let mut seen_files = std::collections::HashSet::new();

    for cap in re.captures_iter(input) {
        let file_path_str = &cap[1];

        if seen_files.contains(file_path_str) {
            continue;
        }

        seen_files.insert(file_path_str.to_string());

        // Use the sandbox utility for security
        match crate::utils::sandbox(file_path_str).await {
            Ok(path) => {
                if path.exists() && path.is_file() {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        let size = metadata.len();

                        if size < 4096 {
                            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                                appended_context.push_str(&format!(
                                    "\n\n--- Content from {} ---\n{}\n--- End ---\n",
                                    file_path_str, content
                                ));
                            }
                        } else {
                            appended_context.push_str(&format!(
                                "\n\n[REFERENCE: {} ({size} bytes)]\nUse filesystem tools.\n",
                                file_path_str
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Security check failed for file reference '@{}': {}", file_path_str, e);
                appended_context.push_str(&format!(
                    "\n\n[ERROR: Reference '@{}' access denied: {}]\n",
                    file_path_str, e
                ));
            }
        }
    }

    if !appended_context.is_empty() {
        final_prompt.push_str("\n\n[FILE CONTEXT]");
        final_prompt.push_str(&appended_context);
    }

    final_prompt
}

fn format_error(e: impl std::fmt::Display) -> String {
    let clean_msg = crate::utils::clean_error_message(e);
    format!("\n\n> ❌ Error\n> \n> {}\n\n", clean_msg)
}

fn highlight_json(json: &str) -> String {
    let mut result = Vec::new();
    let re_key_val = Regex::new(r#"^(\s*)"([^"]+)"(\s*:\s*)(.*)$"#).unwrap();
    let re_str_val = Regex::new(r#"^"([^"]*)"(.*)$"#).unwrap();
    let re_num_bool_null = Regex::new(r#"^(true|false|null|-?\d+(?:\.\d+)?)(.*)$"#).unwrap();

    for line in json.lines() {
        let formatted_line = if let Some(caps) = re_key_val.captures(line) {
            let indent = &caps[1];
            let key = &caps[2];
            let colon = &caps[3];
            let val = &caps[4];
            
            let styled_key = style::style(format!("\"{}\"", key)).with(style::Color::Rgb { r: 0, g: 240, b: 255 }).bold();
            let styled_colon = style::style(colon).dim();
            
            let mut styled_val = val.to_string();
            if val.starts_with('"') {
                if let Some(val_caps) = re_str_val.captures(val) {
                    let str_content = &val_caps[1];
                    let suffix = &val_caps[2];
                    styled_val = format!("{}{}", style::style(format!("\"{}\"", str_content)).with(style::Color::Rgb { r: 255, g: 0, b: 128 }), style::style(suffix).dim());
                }
            } else if val == "{" || val == "[" || val == "}," || val == "]," || val == "}" || val == "]" {
                styled_val = style::style(val).white().bold().to_string();
            } else {
                if let Some(num_caps) = re_num_bool_null.captures(val) {
                    let num_val = &num_caps[1];
                    let suffix = &num_caps[2];
                    let color_val = match num_val {
                        "true" | "false" => style::style(num_val).with(style::Color::Rgb { r: 180, g: 100, b: 255 }),
                        "null" => style::style(num_val).dark_grey(),
                        _ => style::style(num_val).with(style::Color::Rgb { r: 100, g: 255, b: 100 }), // green numbers
                    };
                    styled_val = format!("{}{}", color_val, style::style(suffix).dim());
                }
            }
            
            format!("{}{}{}{}", indent, styled_key, styled_colon, styled_val)
        } else {
            style::style(line).white().bold().to_string()
        };
        result.push(formatted_line);
    }
    result.join("\n")
}

fn print_tool_call(name: &str, args: &str) -> io::Result<()> {
    clear_current_line(&mut io::stdout())?;
    
    let border_color = style::Color::Rgb { r: 180, g: 100, b: 255 }; // Violet
    
    println!("{}", style::style(format!("┌── 🔨 Tool Call: {} ──────────────────────────────────────────────────", name)).with(border_color).bold());
    
    let formatted_args = if let Ok(val) = serde_json::from_str::<serde_json::Value>(args) {
        serde_json::to_string_pretty(&val).unwrap_or_else(|_| args.to_string())
    } else {
        args.to_string()
    };
    
    let highlighted = highlight_json(&formatted_args);
    for line in highlighted.lines() {
        println!("{} {}", style::style("│").with(border_color), line);
    }
    
    println!("{}", style::style("└───────────────────────────────────────────────────────────────────────").with(border_color));
    io::stdout().flush()?;
    Ok(())
}

fn print_tool_response(response: &str) -> io::Result<()> {
    clear_current_line(&mut io::stdout())?;
    
    let border_color = style::Color::Rgb { r: 0, g: 240, b: 255 }; // Cyan
    
    println!("{}", style::style("┌── ✅ Tool Result ─────────────────────────────────────────────────────").with(border_color).bold());
    
    let formatted_resp = if let Ok(val) = serde_json::from_str::<serde_json::Value>(response) {
        serde_json::to_string_pretty(&val).unwrap_or_else(|_| response.to_string())
    } else {
        response.to_string()
    };
    
    let highlighted = highlight_json(&formatted_resp);
    for line in highlighted.lines() {
        println!("{} {}", style::style("│").with(border_color), line);
    }
    
    println!("{}", style::style("└───────────────────────────────────────────────────────────────────────").with(border_color));
    io::stdout().flush()?;
    Ok(())
}

async fn run_system_prompt(
    runner: &mut Runner,
    user_id: &str,
    session_id: &str,
    prompt: &str,
    nami_skin: &MadSkin,
) -> anyhow::Result<()> {
    print_status_line(
        &mut io::stdout(),
        &format!(
            "{} {}",
            style::style("⠋").with(style::Color::Rgb { r: 255, g: 0, b: 128 }).bold(),
            style::style("Agent is thinking...").dim()
        ),
    )?;

    let content = Content::new("user").with_text(prompt);
    let mut stream = runner.run_str(user_id, session_id, content).await?;
    let mut response = String::new();
    let mut cancelled = false;
    let mut cancelled_by_esc = false;
    let mut event_reader = EventStream::new();
    let mut spinner_tick = tokio::time::interval(std::time::Duration::from_millis(80));
    let spinner_chars = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut spinner_idx = 0;

    terminal::enable_raw_mode()?;

    loop {
        tokio::select! {
            _ = spinner_tick.tick() => {
                spinner_idx = (spinner_idx + 1) % spinner_chars.len();
                let spinner_char = spinner_chars[spinner_idx];
                print_status_line(
                    &mut io::stdout(),
                    &format!(
                        "{} {}",
                        style::style(spinner_char).with(style::Color::Rgb { r: 255, g: 0, b: 128 }).bold(),
                        style::style("Agent is thinking...").dim()
                    ),
                )?;
            }
            result = stream.next() => {
                match result {
                    Some(Ok(event)) => {
                        if let Some(content) = &event.llm_response.content {
                            for part in &content.parts {
                                if let Some(text) = part.text() {
                                    response.push_str(text);
                                }
                                if let Part::FunctionCall { name, args, .. } = part {
                                    print_tool_call(name, &args.to_string())?;
                                }
                                if let Part::FunctionResponse { function_response, .. } = part {
                                    print_tool_response(&function_response.response.to_string())?;
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        response.push_str(&format_error(e));
                        break;
                    }
                    None => break,
                }
            }
            maybe_event = event_reader.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    if key.kind == KeyEventKind::Press {
                        if key.code == KeyCode::Esc {
                            runner.interrupt(session_id);
                            cancelled = true;
                            cancelled_by_esc = true;
                            break;
                        } else if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            runner.interrupt(session_id);
                            cancelled = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    clear_current_line(&mut io::stdout())?;

    if cancelled {
        if !cancelled_by_esc {
            println!();
            println!("{}", style::style("🚀 Request cancelled").dim());
        }
        return Ok(());
    }

    println!();

    let rendered = termimad::FmtText::from(
        nami_skin,
        &response,
        Some(
            terminal::size()
                .map(|(w, _)| w as usize)
                .unwrap_or(80)
                .saturating_sub(4),
        ),
    )
    .to_string();

    println!("{}", rendered);
    println!();

    Ok(())
}

fn render_banner(provider: &str, model_name: &str, session_id: &str, mcp_count: usize, skill_count: usize) {
    let violet = style::Color::Rgb { r: 180, g: 100, b: 255 };
    let magenta = style::Color::Rgb { r: 255, g: 0, b: 128 };
    let cyan = style::Color::Rgb { r: 0, g: 240, b: 255 };

    let header_text = format!("⚡ Nami CLI v{} ", env!("CARGO_PKG_VERSION"));
    
    // Print top rule with header text
    print!("{}", style::style(header_text).with(magenta).bold());
    println!("{}", style::style("─".repeat(50usize.saturating_sub(13 + env!("CARGO_PKG_VERSION").len()))).with(violet));
    
    // Print details
    println!(
        "  {} {} ({})  {} {} MCP, {} skills",
        style::style("●").with(magenta),
        style::style("Model:").dim(),
        style::style(format!("{} using {}", provider, model_name)).with(cyan),
        style::style("●").with(magenta),
        style::style(format!("{} servers", mcp_count)).with(cyan),
        style::style(format!("{} skills", skill_count)).with(cyan),
    );
    let workspace_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    println!(
        "  {} {} {}",
        style::style("●").with(magenta),
        style::style("Session:").dim(),
        style::style(session_id).with(cyan).dim(),
    );
    println!(
        "  {} {} {}",
        style::style("●").with(magenta),
        style::style("Workspace:").dim(),
        style::style(workspace_dir).with(cyan),
    );
    
    // Print bottom rule
    println!("{}", style::style("─".repeat(50)).with(violet));

    println!("\nType {} for commands.", style::style("/?").with(magenta).bold());
    println!("Use {} for file references.\n", style::style("@file").with(cyan).bold());
}

pub async fn ensure_session(
    sessions: &Arc<dyn SessionService>,
    app_name: &str,
    user_id: &str,
    session_id: &str,
) -> anyhow::Result<()> {
    if sessions
        .get(GetRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            num_recent_events: Some(0),
            after: None,
        })
        .await
        .is_ok()
    {
        return Ok(());
    }

    sessions
        .create(CreateRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: Some(session_id.to_string()),
            state: Default::default(),
        })
        .await?;

    Ok(())
}

fn clear_current_line(stdout: &mut io::Stdout) -> io::Result<()> {
    queue!(
        stdout,
        terminal::Clear(terminal::ClearType::CurrentLine),
        cursor::MoveToColumn(0)
    )?;

    stdout.flush()?;

    Ok(())
}

fn print_status_line(stdout: &mut io::Stdout, text: &str) -> io::Result<()> {
    queue!(
        stdout,
        cursor::SavePosition,
        terminal::Clear(terminal::ClearType::CurrentLine),
        cursor::MoveToColumn(0),
        style::Print(text),
        cursor::RestorePosition
    )?;

    stdout.flush()?;

    Ok(())
}

pub async fn handle_slash_command(
    trimmed: &str,
    runner: &mut Runner,
    sessions: &Arc<dyn SessionService>,
    app_name: &str,
    user_id: &str,
    session_id: &mut String,
    nami_skin: &MadSkin,
    provider: &mut String,
    model_name: &mut String,
    registry: &CommandRegistry,
    mcp_count: usize,
    skill_count: usize,
) -> anyhow::Result<bool> {
    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let command_name = parts[0];
    let args = parts.get(1).unwrap_or(&"");

    // Dynamic registry lookup
    if let Some(prompt) = registry.format_prompt(command_name, args) {
        run_system_prompt(runner, user_id, session_id, &prompt, nami_skin).await?;
        return Ok(false);
    }

    // Fallback to static commands
    match trimmed {
        "/?" => {
            println!("{}", render_help(registry));
        }

        "/exit" | "/quit" => {
            return Ok(true);
        }

        "/clear" => {
            execute!(
                io::stdout(),
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            )?;

            render_banner(provider, model_name, session_id, mcp_count, skill_count);
        }

        "/new" => {
            let session_id_new = Uuid::new_v4().to_string();
            ensure_session(sessions, app_name, user_id, &session_id_new).await?;

            execute!(
                io::stdout(),
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            )?;

            render_banner(provider, model_name, &session_id_new, mcp_count, skill_count);

            println!(
                "{}\n",
                style::style("✨ New session started").green()
            );
            *session_id = session_id_new;
        }

        "/version" => {
            println!(
                "{} {}\n",
                style::style("Nami CLI").magenta().bold(),
                env!("CARGO_PKG_VERSION")
            );
        }
        
        "/tasks" => {
            run_system_prompt(runner, user_id, session_id, "list_active_tasks", nami_skin).await?;
        }

        "/status" => {
            run_system_prompt(runner, user_id, session_id, "get_system_status", nami_skin).await?;
        }

        _ => {
            println!("{} {}\n", style::style("Unknown command:").red(), trimmed);
        }
    }
    Ok(false)
}


pub async fn run_cli(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
    mut provider: String,
    mut model_name: String,
    mcp_count: usize,
    skill_count: usize,
) -> anyhow::Result<()> {
    execute!(
        io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    let app_name = "cli";
    let user_id = "default_user";

    let mut session_id = Uuid::new_v4().to_string();

    render_banner(&provider, &model_name, &session_id, mcp_count, skill_count);

    ensure_session(&sessions, app_name, user_id, &session_id).await?;

    let mut runner = Runner::builder()
        .app_name(app_name)
        .agent(agent.clone())
        .session_service(sessions.clone())
        .compaction_config(get_compaction_config(model.clone()))
        .build()?;


    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut rl: Editor<NamiHelper, rustyline::history::FileHistory> = Editor::with_config(config)?;

    rl.set_helper(Some(NamiHelper));

    let history_path = get_nami_dir().join(".cli_history");
    let _ = rl.load_history(&history_path);

    let mut nami_skin = MadSkin::default();

    nami_skin.paragraph.set_fg(termimad::crossterm::style::Color::Rgb { r: 240, g: 240, b: 245 });
    nami_skin.bold.set_fg(termimad::crossterm::style::Color::Rgb { r: 255, g: 0, b: 128 }); // Synthwave Pink
    nami_skin.italic.set_fg(termimad::crossterm::style::Color::Rgb { r: 0, g: 240, b: 255 }); // Cyan
    nami_skin.inline_code.set_fg(termimad::crossterm::style::Color::Rgb { r: 180, g: 100, b: 255 }); // Violet
    nami_skin.inline_code.set_bg(termimad::crossterm::style::Color::Rgb { r: 25, g: 20, b: 35 });
    nami_skin.code_block.set_fg(termimad::crossterm::style::Color::Rgb { r: 0, g: 240, b: 255 });
    nami_skin.code_block.set_bg(termimad::crossterm::style::Color::Rgb { r: 20, g: 15, b: 30 });
    nami_skin.bullet.set_fg(termimad::crossterm::style::Color::Rgb { r: 255, g: 0, b: 128 });
    
    // Headers
    nami_skin.headers[0].set_fg(termimad::crossterm::style::Color::Rgb { r: 255, g: 0, b: 128 }); // H1 - Hot Pink
    nami_skin.headers[1].set_fg(termimad::crossterm::style::Color::Rgb { r: 0, g: 240, b: 255 }); // H2 - Cyan
    nami_skin.headers[2].set_fg(termimad::crossterm::style::Color::Rgb { r: 180, g: 100, b: 255 }); // H3 - Violet

    // let mut last_config_mtime = get_config_mtime();
    // let mut last_skills_mtime = get_skills_mtime();

    loop {
        // let mut config_changed = false;

        // if let Some(new_config) = check_config_mtime(&mut last_config_mtime) {
        //     let (new_agent, new_model) = create_agent(&new_config).await?;

        //     agent = new_agent;
        //     model = new_model;

        //     provider = new_config
        //         .model
        //         .provider
        //         .clone()
        //         .unwrap_or_else(|| "gemini".to_string());
        //     model_name = new_config.model.model_name.clone();

        //     config_changed = true;
        // }

        // let current_skills_mtime = get_skills_mtime();

        // if last_skills_mtime != current_skills_mtime {
        //     last_skills_mtime = current_skills_mtime;
        //     config_changed = true;
        // }

        // if config_changed {
        //     runner = Runner::builder()
        //         .app_name(app_name)
        //         .agent(agent.clone())
        //         .session_service(sessions.clone())
        //         .compaction_config(get_compaction_config(model.clone()))
        //         .build()?;

        //     println!("\n{}\n", style::style("🧠 Agent reloaded").cyan());
        // }

        let line = rl.readline("You > ");

        match line {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.starts_with('/') {
                    let config_path = get_nami_dir().join("config.toml");
                    let registry = CommandRegistry::load_from_config(&config_path.to_string_lossy())
                        .unwrap_or(CommandRegistry { commands: Default::default() });

                    if handle_slash_command(
                        trimmed,
                        &mut runner,
                        &sessions,
                        app_name,
                        user_id,
                        &mut session_id,
                        &nami_skin,
                        &mut provider,
                        &mut model_name,
                        &registry,
                        mcp_count,
                        skill_count,
                    )
                    .await?
                    {
                        break;
                    }
                    continue;
                }

                if trimmed.is_empty() {
                    continue;
                }

                if trimmed == "/exit" {
                    break;
                }

                let _ = rl.add_history_entry(trimmed);

                let _ = rl.save_history(&history_path);

                let enriched_prompt = process_file_references(trimmed).await;

                print_status_line(
                    &mut io::stdout(),
                    &format!(
                        "{} {}",
                        style::style("⏳").magenta(),
                        style::style("Agent is thinking...").dim()
                    ),
                )?;

                let content = Content::new("user").with_text(enriched_prompt);
                let mut stream = runner.run_str(user_id, &session_id, content).await?;
                let mut response_buffer = String::new();
                let mut function_response_buffer: Vec<Part> = Vec::new();
                let mut cancelled = false;
                let mut cancelled_by_esc = false;
                let mut event_reader = EventStream::new();

                terminal::enable_raw_mode()?;

                loop {
                    tokio::select! {
                        result = stream.next() => {
                            match result {
                                Some(Ok(event)) => {
                                    if let Some(content) =
                                        &event.llm_response.content
                                    {
                                        for part in &content.parts {
                                            if let Some(text) =
                                                part.text()
                                            {
                                                response_buffer
                                                    .push_str(text);
                                            }

                                            if let Part::FunctionCall { name, args, .. } = part {
                                                let args_str = args.to_string().replace('\n', " ").replace("  ", " ");
                                                
                                                clear_current_line(&mut io::stdout())?;
                                                println!("{} {} {}({})\r", 
                                                    style::style("🔨").magenta(),
                                                    style::style("Tool Call:").dim().bold(),
                                                    style::style(name).cyan(),
                                                    style::style(args_str).dim()
                                                );
                                                io::stdout().flush()?;
                                            }
                                            if let Part::FunctionResponse { .. } = part {
                                                function_response_buffer.push(part.clone());
                                            }
                                        }
                                    }

                                    // Re-print the thinking status if we are still waiting for more
                                    print_status_line(
                                        &mut io::stdout(),
                                        &format!(
                                            "{} {}",
                                            style::style("⏳").magenta(),
                                            style::style("Agent is thinking...").dim()
                                        ),
                                    )?;
                                }

                                Some(Err(e)) => {
                                    response_buffer.push_str(&format_error(e));

                                    break;
                                }

                                None => break,
                            }
                        }

                        maybe_event = event_reader.next() => {
                            if let Some(Ok(Event::Key(key))) =
                                maybe_event
                            {
                                if key.kind == KeyEventKind::Press {
                                    if key.code == KeyCode::Esc {
                                        runner.interrupt(&session_id);
                                        cancelled = true;
                                        cancelled_by_esc = true;
                                        break;
                                    } else if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                        runner.interrupt(&session_id);
                                        cancelled = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                // Flush collected function responses if any were gathered
                if !function_response_buffer.is_empty() {
                    let response_content = Content {
                        role: "function".to_string(),
                        parts: function_response_buffer,
                    };
                    let mut response_stream = runner.run_str(user_id, &session_id, response_content).await?;
                    // Consume the stream to complete the turn
                    while let Some(_) = response_stream.next().await {}
                }

                terminal::disable_raw_mode()?;
                clear_current_line(&mut io::stdout())?;

                if cancelled {
                    if !cancelled_by_esc {
                        println!();
                        println!("{}", style::style("🚀 Request cancelled").dim());
                    }
                    continue;
                }
                println!();

                let cleaned = response_buffer
                    .lines()
                    .map(|line| line.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n");

                let term_width = terminal::size()
                    .map(|(w, _)| w as usize)
                    .unwrap_or(80)
                    .saturating_sub(4);

                let rendered =
                    termimad::FmtText::from(&nami_skin, &cleaned, Some(term_width)).to_string();

                println!("{}", rendered);

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