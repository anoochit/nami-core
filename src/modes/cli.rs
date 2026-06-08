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

            // Resolve dynamic workspace path
            let base_dir = if let Ok(current_dir) = std::env::current_dir() {
                let canonical_current = crate::utils::clean_unc_path(std::fs::canonicalize(&current_dir).unwrap_or(current_dir.clone()));
                let (active_opt, list) = crate::utils::get_workspaces_info();
                let mut matched_workspace = None;
                for ws_path in &list {
                    let canonical_ws = crate::utils::clean_unc_path(std::fs::canonicalize(ws_path).unwrap_or_else(|_| ws_path.clone()));
                    if canonical_current == canonical_ws || canonical_current.starts_with(&canonical_ws) {
                        matched_workspace = Some(canonical_ws);
                        break;
                    }
                }
                matched_workspace.or(active_opt).unwrap_or(canonical_current)
            } else {
                std::path::PathBuf::from(".")
            };

            let clean_search_pattern = path_part.trim_start_matches(['/', '\\']);

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

                        if path_str.to_lowercase().contains(&clean_search_pattern.to_lowercase()) {
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

#[allow(dead_code)]
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
    
    let minified_args = if let Ok(val) = serde_json::from_str::<serde_json::Value>(args) {
        serde_json::to_string(&val).unwrap_or_else(|_| args.to_string())
    } else {
        args.to_string()
    };

    println!("{} {} {}({})\r", 
        style::style("🔨").magenta(),
        style::style("Tool Call:").dim().bold(),
        style::style(name).cyan(),
        style::style(minified_args).dim()
    );
    io::stdout().flush()?;
    Ok(())
}

fn print_tool_response(response: &str) -> io::Result<()> {
    clear_current_line(&mut io::stdout())?;
    
    let minified_resp = if let Ok(val) = serde_json::from_str::<serde_json::Value>(response) {
        serde_json::to_string(&val).unwrap_or_else(|_| response.to_string())
    } else {
        response.to_string()
    };

    println!("{} {} {}\r", 
        style::style("✅").green(),
        style::style("Tool Result:").dim().bold(),
        style::style(minified_resp).dim()
    );

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
            style::style("⠋").with(style::Color::Rgb { r: 255, g: 121, b: 198 }).bold(),
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
                        style::style(spinner_char).with(style::Color::Rgb { r: 255, g: 121, b: 198 }).bold(),
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
    let violet = style::Color::Rgb { r: 189, g: 147, b: 249 }; // Dracula Purple
    let magenta = style::Color::Rgb { r: 255, g: 121, b: 198 }; // Dracula Pink
    let cyan = style::Color::Rgb { r: 139, g: 233, b: 253 }; // Dracula Cyan

    let header_text = format!("Nami CLI v{}", env!("CARGO_PKG_VERSION"));
    
    // Print header line
    println!("{}", style::style("─".repeat(50)).with(violet));
    
    // Print header text with no trailing lines after version number
    println!("{}", style::style(header_text).with(magenta).bold());
    
    // Print details on separate lines with indentation and no bullet points
    println!(
        "{} {}",
        style::style("Model:").dim(),
        style::style(format!("{} using {}", provider, model_name)).with(cyan),
    );
    println!(
        "{} {} MCP servers, {} skills",
        style::style("Capabilities:").dim(),
        style::style(mcp_count).with(cyan),
        style::style(skill_count).with(cyan),
    );

    let workspace_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    println!(
        "{} {}",
        style::style("Session:").dim(),
        style::style(session_id).with(cyan).dim(),
    );
    println!(
        "{} {}",
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

async fn run_cli_grill(
    model: &Arc<dyn Llm>,
    _runner: &mut Runner,
    _user_id: &str,
    session_id: &str,
    goal: &str,
    nami_skin: &MadSkin,
) -> anyhow::Result<()> {
    use rustyline::DefaultEditor;
    use serde_json::json;

    println!("\n{}\n", style::style("🤖 Initiating Interactive Alignment Loop (Grill-Me Mode)...").magenta().bold());
    println!("Analyzing goal: {}\n", style::style(goal).cyan());
    
    // 1. Generate questions
    print_status_line(&mut io::stdout(), "Generating alignment questions...")?;
    let questions = match crate::tools::plan::PlanGrill::generate_questions(model, goal).await {
        Ok(q) => q,
        Err(e) => {
            clear_current_line(&mut io::stdout())?;
            println!("{} {}\n", style::style("Error generating questions:").red().bold(), e);
            return Ok(());
        }
    };
    clear_current_line(&mut io::stdout())?;

    println!("Great! Let's clarify a few details to make the plan robust:\n");

    let mut qa_pairs = Vec::new();
    let mut rl = DefaultEditor::new()?;

    for (i, question) in questions.iter().enumerate() {
        println!("{}. {}", style::style(format!("{}", i + 1)).magenta().bold(), style::style(question).bold());
        let answer = loop {
            let res = rl.readline("Answer > ");
            match res {
                Ok(line) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        break trimmed.to_string();
                    }
                    println!("{}", style::style("Please provide a brief answer to help build a precise plan.").yellow());
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("\n{}", style::style("Grill-Me session cancelled.").red());
                    return Ok(());
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Readline error: {}", e));
                }
            }
        };
        println!();
        qa_pairs.push((question.clone(), answer));
    }

    // 2. Synthesize Plan
    print_status_line(&mut io::stdout(), "Synthesizing and registering aligned plan...")?;
    
    let steps_val = match crate::tools::plan::PlanGrill::synthesize_plan(model, goal, &qa_pairs).await {
        Ok(steps) => steps,
        Err(e) => {
            clear_current_line(&mut io::stdout())?;
            println!("{} {}\n", style::style("Error synthesizing plan:").red().bold(), e);
            return Ok(());
        }
    };
    clear_current_line(&mut io::stdout())?;

    let plan_name = format!("grill-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    
    // Create the plan using PlanCreate
    let plan_tool = crate::tools::plan::PlanCreate::new(model.clone());
    let create_args = json!({
        "name": plan_name,
        "objective": goal,
        "steps": steps_val
    });

    let context: Arc<dyn adk_rust::tool::ToolContext> = Arc::new(adk_tool::SimpleToolContext::new(session_id));
    match plan_tool.execute(context.clone(), create_args).await {
        Ok(_) => {
            println!("{}\n", style::style(format!("✨ Plan '{}' successfully synthesized and registered!", plan_name)).green().bold());
            // Show the newly created plan
            let show_tool = crate::tools::plan::PlanShow;
            if let Ok(show_res) = show_tool.execute(context.clone(), json!({"name": plan_name})).await {
                if let Some(content) = show_res["content"].as_str() {
                    let markdown = nami_skin.term_text(content);
                    println!("{}", markdown);
                }
            }
        }
        Err(e) => {
            println!("{} {}\n", style::style("Error registering plan:").red().bold(), e);
        }
    }

    Ok(())
}

pub async fn handle_slash_command(
    trimmed: &str,
    runner: &mut Runner,
    sessions: &Arc<dyn SessionService>,
    model: &Arc<dyn Llm>,
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

    if command_name == "/grill" {
        if args.is_empty() {
            println!("{} Please specify a goal, e.g. `/grill Build a weather dashboard`\n", style::style("Error:").red().bold());
            return Ok(false);
        }
        run_cli_grill(model, runner, user_id, session_id, args, nami_skin).await?;
        return Ok(false);
    }

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

    nami_skin.paragraph.set_fg(termimad::crossterm::style::Color::Rgb { r: 248, g: 248, b: 242 }); // Dracula FG
    nami_skin.bold.set_fg(termimad::crossterm::style::Color::Rgb { r: 255, g: 121, b: 198 }); // Dracula Pink
    nami_skin.italic.set_fg(termimad::crossterm::style::Color::Rgb { r: 139, g: 233, b: 253 }); // Dracula Cyan
    nami_skin.inline_code.set_fg(termimad::crossterm::style::Color::Rgb { r: 189, g: 147, b: 249 }); // Dracula Purple
    nami_skin.inline_code.set_bg(termimad::crossterm::style::Color::Rgb { r: 40, g: 42, b: 54 }); // Dracula BG
    nami_skin.code_block.set_fg(termimad::crossterm::style::Color::Rgb { r: 139, g: 233, b: 253 }); // Dracula Cyan
    nami_skin.code_block.set_bg(termimad::crossterm::style::Color::Rgb { r: 40, g: 42, b: 54 }); // Dracula BG
    nami_skin.bullet.set_fg(termimad::crossterm::style::Color::Rgb { r: 255, g: 121, b: 198 }); // Dracula Pink
    
    // Headers
    nami_skin.headers[0].set_fg(termimad::crossterm::style::Color::Rgb { r: 255, g: 121, b: 198 }); // Dracula Pink
    nami_skin.headers[1].set_fg(termimad::crossterm::style::Color::Rgb { r: 139, g: 233, b: 253 }); // Dracula Cyan
    nami_skin.headers[2].set_fg(termimad::crossterm::style::Color::Rgb { r: 189, g: 147, b: 249 }); // Dracula Purple

    loop {
       
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
                        &model,
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
                        style::style("⠋").with(style::Color::Rgb { r: 255, g: 121, b: 198 }).bold(),
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
                                    style::style(spinner_char).with(style::Color::Rgb { r: 255, g: 121, b: 198 }).bold(),
                                    style::style("Agent is thinking...").dim()
                                ),
                            )?;
                        }
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
                                            if let Part::FunctionResponse { function_response, .. } = part {
                                                let resp_str = function_response.response.to_string();
                                                let minified_resp = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&resp_str) {
                                                    serde_json::to_string(&val).unwrap_or_else(|_| resp_str.clone())
                                                } else {
                                                    resp_str.clone()
                                                };
                                                
                                                clear_current_line(&mut io::stdout())?;
                                                println!("{} {} {}\r", 
                                                    style::style("✅").green(),
                                                    style::style("Tool Result:").dim().bold(),
                                                    style::style(minified_resp).dim()
                                                );
                                                io::stdout().flush()?;
                                                
                                                function_response_buffer.push(part.clone());
                                            }
                                        }
                                    }
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
            }

            Err(_) => {
                break;
            }
        }
    }

Ok(())
}