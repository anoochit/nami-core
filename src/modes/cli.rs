use chrono::Utc;
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

use crate::agent::agent::{check_config_mtime, create_agent, get_config_mtime, get_skills_mtime};
use crate::agent::get_compaction_config;
use crate::modes::command_registry::CommandRegistry;
use crate::tools::scheduler::{load_schedule, save_schedule};
use crate::tools::state_manager::{TaskStatus, load_states};

use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::{CreateRequest, GetRequest, SessionService};

struct NamiHelper;

fn render_help(registry: &CommandRegistry) {
    println!("{}", style::style("Available Commands:").yellow().bold());
    
    // Render static commands
    println!("{}  Show commands", style::style("/?").cyan().bold());
    println!("{}  Quit", style::style("/exit").cyan().bold());
    println!("{}  Clear screen", style::style("/clear").cyan().bold());
    println!("{}  New session", style::style("/new").cyan().bold());
    println!("{}  List active tasks", style::style("/tasks").cyan().bold());
    println!("{}  Agent status", style::style("/status").cyan().bold());
    println!("{}  CLI version", style::style("/version").cyan().bold());

    // Render dynamic commands from registry
    println!("\n{}", style::style("Custom Commands:").magenta().bold());
    let mut commands: Vec<_> = registry.commands.iter().collect();
    commands.sort_by(|a, b| a.0.cmp(b.0));
    for (name, cmd) in commands {
        println!("{}  {}", style::style(name).cyan().bold(), cmd.help);
    }

    println!(
        r#"
Examples:
  /plan Build AI research system
  /wiki Rust async traits
  /memo User prefers concise output
"#
    );
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

        let workspace_path = std::path::Path::new("workspace");
        let path = workspace_path.join(file_path_str);

        if path.exists()
            && path.is_file()
            && let Ok(metadata) = std::fs::metadata(&path)
        {
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
            style::style("⏳").magenta(),
            style::style("Agent is thinking...").dim()
        ),
    )?;

    let content = Content::new("user").with_text(prompt);
    let mut stream = runner.run_str(user_id, session_id, content).await?;
    let mut response = String::new();
    let mut cancelled = false;
    let mut cancelled_by_esc = false;
    let mut event_reader = EventStream::new();

    terminal::enable_raw_mode()?;

    loop {
        tokio::select! {
            result = stream.next() => {
                match result {
                    Some(Ok(event)) => {
                        if let Some(content) = &event.llm_response.content {
                            for part in &content.parts {
                                if let Some(text) = part.text() {
                                    response.push_str(text);
                                }
                                if let Part::FunctionCall { name, args, .. } = part {
                                    let args_str = args.to_string().replace('\n', " ").replace("  ", " ");
                                    let compact_args = if args_str.chars().count() > 60 {
                                        format!("{}...", args_str.chars().take(57).collect::<String>())
                                    } else {
                                        args_str
                                    };
                                    print_status_line(
                                        &mut io::stdout(),
                                        &format!(
                                            "{} {} {}({})",
                                            style::style("🔨"),
                                            style::style("Calling").dim().bold(),
                                            style::style(name).cyan(),
                                            style::style(compact_args).dim()
                                        ),
                                    )?;
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

fn render_banner(provider: &str, model_name: &str, session_id: &str) {
    println!(
        "{}",
        style::style(
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
        style::style(format!("Nami CLI v{}", env!("CARGO_PKG_VERSION")))
            .bold()
            .magenta(),
        style::style(format!("({}) using {}", provider, model_name)).dim()
    );

    println!(
        "{} {}",
        style::style("Session ID:").bold().magenta(),
        style::style(session_id).dim()
    );

    println!("\nType /? for commands.");
    println!("Use @file for references.\n");
}

pub(crate) async fn ensure_session(
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

async fn handle_slash_command(
    trimmed: &str,
    runner: &mut Runner,
    sessions: &Arc<dyn SessionService>,
    user_id: &str,
    session_id: &mut String,
    nami_skin: &MadSkin,
    provider: &mut String,
    model_name: &mut String,
    registry: &CommandRegistry,
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
            render_help(registry);
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

            render_banner(provider, model_name, session_id);
        }

        "/new" => {
            let session_id_new = Uuid::new_v4().to_string();
            ensure_session(sessions, "cli", user_id, &session_id_new).await?;

            execute!(
                io::stdout(),
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            )?;

            render_banner(provider, model_name, &session_id_new);

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


pub(crate) async fn run_cli(
    mut agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    mut model: Arc<dyn Llm>,
    mut provider: String,
    mut model_name: String,
) -> anyhow::Result<()> {
    execute!(
        io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

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

    // Spawn scheduler background loop
    let bg_agent = agent.clone();
    let bg_sessions = sessions.clone();
    let bg_model = model.clone();
    tokio::spawn(async move {
        if let Err(e) = run_scheduler_loop(bg_agent, bg_sessions, bg_model).await {
            log::error!("Scheduler error: {:?}", e);
        }
    });

    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut rl: Editor<NamiHelper, rustyline::history::FileHistory> = Editor::with_config(config)?;

    rl.set_helper(Some(NamiHelper));

    let _ = rl.load_history(".cli_history");

    let mut nami_skin = MadSkin::default();

    nami_skin
        .paragraph
        .set_fg(termimad::crossterm::style::Color::White);

    nami_skin
        .bullet
        .set_fg(termimad::crossterm::style::Color::Magenta);

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
                    let registry = CommandRegistry::load_from_config("config.toml")
                        .unwrap_or(CommandRegistry { commands: Default::default() });

                    if handle_slash_command(
                        trimmed,
                        &mut runner,
                        &sessions,
                        user_id,
                        &mut session_id,
                        &nami_skin,
                        &mut provider,
                        &mut model_name,
                        &registry,
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

                rl.save_history(".cli_history")?;

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
                                                let compact_args = if args_str.chars().count() > 60 {
                                                    format!("{}...", args_str.chars().take(57).collect::<String>())
                                                } else {
                                                    args_str
                                                };
                                                print_status_line(
                                                    &mut io::stdout(),
                                                    &format!(
                                                        "{} {} {}({})",
                                                        style::style("🔨"),
                                                        style::style("Calling").dim().bold(),
                                                        style::style(name).cyan(),
                                                        style::style(compact_args).dim()
                                                    ),
                                                )?;
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

async fn run_scheduler_loop(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    let app_name = "scheduler";
    let user_id = "system";
    let session_id = "background_tasks";

    ensure_session(&sessions, app_name, user_id, session_id).await?;

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions)
        .compaction_config(get_compaction_config(model))
        .build()?;

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        let mut tasks = match load_schedule().await {
            Ok(t) => t,
            Err(_) => {
                continue;
            }
        };

        let now = Utc::now();
        let mut changed = false;

        for task in tasks.iter_mut() {
            if !task.is_active {
                continue;
            }

            let schedule = match <cron::Schedule as std::str::FromStr>::from_str(&task.cron_expr) {
                Ok(s) => s,
                Err(_) => {
                    continue;
                }
            };

            let should_run = match task.last_run {
                Some(last) => {
                    if let Some(due) = schedule.after(&last).next() {
                        now >= due
                    } else {
                        false
                    }
                }
                None => true,
            };

            if should_run {
                let states = load_states().await.unwrap_or_default();
                let current_status = states
                    .iter()
                    .find(|s| s.goal == task.goal)
                    .map(|s| s.status.clone())
                    .unwrap_or(TaskStatus::InProgress);

                if current_status != TaskStatus::Completed {
                    log::info!("Scheduler triggering task: {}", task.goal);

                    let content = Content::new("user").with_text(&format!(
                        "SCHEDULED RUN: {}. Please continue working on this goal.",
                        task.goal
                    ));

                    let mut stream = runner.run_str(user_id, session_id, content).await?;
                    while let Some(_) = stream.next().await {}

                    task.last_run = Some(now);
                    changed = true;
                }
            }
        }

        if changed {
            let _ = save_schedule(&tasks).await;
        }
    }
}
