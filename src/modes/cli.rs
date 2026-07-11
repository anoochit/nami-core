use crossterm::event::EventStream;
use crossterm::{cursor, execute, queue, style, style::Stylize, terminal};
use futures::StreamExt;
use regex::Regex;
use rustyline::{Config, Editor};
use std::io::{self, Write};
use std::sync::Arc;
use termimad::MadSkin;
use uuid::Uuid;

// use crate::agent::agent::{check_config_mtime, create_agent, get_config_mtime, get_skills_mtime};
use crate::agent::get_compaction_config;
use crate::modes::command_registry::CommandRegistry;
use crate::utils::get_nami_dir;

use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::{CreateRequest, GetRequest, SessionService};

use crate::modes::cli_helper::{NamiHelper, check_cancellation_event, CancellationType};

pub fn render_help(registry: &CommandRegistry) -> String {
    let mut help = String::new();
    help.push_str("Available Commands\n\n");
    
    // Render static commands
    help.push_str("- /clear: Clear screen\n");
    help.push_str("- /new: New session\n");
    help.push_str("- /copy: Copy last response to clipboard!\n");
    help.push_str("- /status: Agent status\n");
    help.push_str("- /version: CLI version\n");
    help.push_str("- /switch: Switch LLM model and provider dynamically\n");
    help.push_str("- /plan: Create a structured execution/implementation plan\n");
    help.push_str("- /exit: Quit\n");

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

    let formatted_args = if minified_args.chars().count() > 80 {
        let truncated: String = minified_args.chars().take(80).collect();
        format!("{}... (+{} chars)", truncated, minified_args.len() - truncated.len())
    } else {
        minified_args
    };

    println!("{} {} {}({})\r", 
        style::style("🔨").magenta(),
        style::style("Tool Call:").dim().bold(),
        style::style(name).cyan(),
        style::style(formatted_args).dim()
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

    let formatted_resp = if minified_resp.chars().count() > 80 {
        let truncated: String = minified_resp.chars().take(80).collect();
        format!("{}... (+{} chars)", truncated, minified_resp.len() - truncated.len())
    } else {
        minified_resp
    };

    println!("{} {} {}\r", 
        style::style("✅").green(),
        style::style("Tool Result:").dim().bold(),
        style::style(formatted_resp).dim()
    );

    io::stdout().flush()?;
    Ok(())
}

fn calculate_occupied_rows(text: &str, terminal_width: usize) -> usize {
    if text.is_empty() {
        return 0;
    }
    let mut rows = 0;
    for line in text.lines() {
        let len = line.len();
        if len == 0 {
            rows += 1;
        } else {
            rows += (len + terminal_width - 1) / terminal_width;
        }
    }
    // If the text ends with a newline, it adds an extra empty line
    if text.ends_with('\n') {
        rows += 1;
    }
    rows
}

pub async fn run_and_stream_prompt(
    runner: &mut Runner,
    user_id: &str,
    session_id: &str,
    prompt: &str,
    nami_skin: &MadSkin,
    provider: &str,
    model_name: &str,
) -> anyhow::Result<String> {
    // Trigger Gemini context cache invalidation handler if using a Gemini model
    if model_name.to_lowercase().contains("gemini") {
        let _ = crate::utils::gemini_cache::get_or_create_context_cache(model_name).await;
    }

    print_status_line(
        &mut io::stdout(),
        &format!(
            "{} {}",
            style::style("⠋").with(style::Color::Rgb { r: 255, g: 121, b: 198 }).bold(),
            style::style("Agent is thinking...").dim()
        ),
    )?;

    let content = Content::new("user").with_text(prompt.to_string());
    let start_thinking_time = std::time::Instant::now();
    let mut stream = runner.run_str(user_id, session_id, content).await?;
    let mut response_buffer = String::new();
    let mut cancelled = false;
    let mut cancelled_by_esc = false;
    let mut event_reader = EventStream::new();
    let mut spinner_tick = tokio::time::interval(std::time::Duration::from_millis(80));
    let spinner_chars = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut spinner_idx = 0;

    let mut started_thinking = false;
    let mut has_received_text = false;

    terminal::enable_raw_mode()?;

    loop {
        tokio::select! {
            _ = spinner_tick.tick() => {
                if !has_received_text {
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
            }
            result = stream.next() => {
                match result {
                    Some(Ok(event)) => {
                        if let Some(content) = &event.llm_response.content {
                            for part in &content.parts {
                                if started_thinking && (part.text().is_some() || matches!(part, Part::FunctionCall { .. } | Part::FunctionResponse { .. })) {
                                    clear_current_line(&mut io::stdout())?;
                                    println!("{}\r", style::style("──────────────────────────────────────────────────").dim());
                                    started_thinking = false;
                                }

                                if let Part::Thinking { thinking, .. } = part {
                                    if !thinking.is_empty() {
                                        clear_current_line(&mut io::stdout())?;
                                        if !started_thinking {
                                            println!("{}\r", style::style("🧠 Thinking Process:").dim().bold());
                                            started_thinking = true;
                                        }
                                        print!("{}", style::style(thinking).dim().italic().to_string().replace('\n', "\r\n"));
                                        io::stdout().flush()?;
                                    }
                                }

                                if let Some(text) = part.text() {
                                    if !text.is_empty() {
                                        if !has_received_text {
                                            has_received_text = true;
                                            clear_current_line(&mut io::stdout())?;
                                        }
                                        print!("{}", text.replace('\n', "\r\n"));
                                        io::stdout().flush()?;
                                        response_buffer.push_str(text);
                                    }
                                }
                                if let Part::FunctionCall { name, args, .. } = part {
                                    print_tool_call(name, &args.to_string())?;
                                    has_received_text = false; // Show spinner while tool processes / model thinks
                                }
                                if let Part::FunctionResponse { function_response, .. } = part {
                                    print_tool_response(&function_response.response.to_string())?;
                                    has_received_text = false; // Show spinner for next model turn
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
                if let Some(Ok(event)) = maybe_event {
                    if let Some(cancellation) = check_cancellation_event(event) {
                        runner.interrupt(session_id);
                        cancelled = true;
                        if cancellation == CancellationType::Esc {
                            cancelled_by_esc = true;
                        }
                        break;
                    }
                }
            }
        }
    }

    let should_render = response_buffer.len() <= 6000;

    terminal::disable_raw_mode()?;

    if should_render {
        if !response_buffer.is_empty() {
            let term_width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
            let rows_to_clear = calculate_occupied_rows(&response_buffer, term_width);
            if rows_to_clear > 0 {
                let mut stdout = io::stdout();
                let move_up = rows_to_clear.saturating_sub(1);
                if move_up > 0 {
                    let _ = queue!(stdout, cursor::MoveUp(move_up as u16));
                }
                let _ = queue!(
                    stdout,
                    terminal::Clear(terminal::ClearType::FromCursorDown),
                    cursor::MoveToColumn(0)
                );
                let _ = stdout.flush();
            }
        } else {
            clear_current_line(&mut io::stdout())?;
        }
    } else {
        println!();
    }

    let duration_secs = start_thinking_time.elapsed().as_secs_f64();
    let prompt_tokens = (prompt.len() as f64 / 4.0).round() as usize;
    let response_tokens = (response_buffer.len() as f64 / 4.0).round() as usize;
    let total_tokens = prompt_tokens + response_tokens;

    // Save statistics to .nami/stats.json
    crate::utils::save_agent_statistic(provider, model_name, duration_secs, total_tokens);

    // Print statistical summary
    let mut stdout = io::stdout();
    if should_render {
        let _ = clear_current_line(&mut stdout);
        if started_thinking {
            println!("{}\r", style::style(format!("🧠 Thought for {:.1}s, {} tokens", duration_secs, total_tokens)).italic());
            let _ = clear_current_line(&mut stdout);
            println!("{}\r", style::style("──────────────────────────────────────────────────").dim());
        } else {
            println!("{}\r", style::style(format!("🧠 Thought for {:.1}s, {} tokens", duration_secs, total_tokens)).italic());
        }
    } else {
        println!("{}\r", style::style(format!("🧠 Thought for {:.1}s, {} tokens (Fast Render)", duration_secs, total_tokens)).italic());
    }

    if cancelled {
        if !cancelled_by_esc {
            println!();
            println!("{}", style::style("🚀 Request cancelled").dim());
        }
        return Ok(String::new());
    }

    if should_render {
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

        let rendered = termimad::FmtText::from(
            nami_skin,
            &cleaned,
            Some(term_width),
        )
        .to_string();

        println!("{}", rendered);
        println!();
    } else {
        println!();
    }

    Ok(response_buffer)
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


async fn execute_silent_prompt(
    runner: &mut Runner,
    user_id: &str,
    session_id: &str,
    prompt: &str,
) -> anyhow::Result<String> {
    let content = Content::new("user").with_text(prompt);
    let mut stream = runner.run_str(user_id, session_id, content).await?;
    let mut response = String::new();
    while let Some(result) = stream.next().await {
        let event = result?;
        if let Some(content) = &event.llm_response.content {
            for part in &content.parts {
                if let Some(text) = part.text() {
                    response.push_str(text);
                }
            }
        }
    }
    Ok(response)
}

fn parse_plan_steps(plan: &str) -> Vec<String> {
    let mut steps = Vec::new();
    let mut in_steps_section = false;

    for line in plan.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with('#') && (trimmed.to_lowercase().contains("step") || trimmed.to_lowercase().contains("plan")) {
            in_steps_section = true;
            continue;
        }

        if in_steps_section {
            if trimmed.starts_with('-') || trimmed.starts_with('*') || (trimmed.chars().next().map_or(false, |c| c.is_digit(10)) && trimmed.contains('.')) {
                let cleaned = trimmed
                    .trim_start_matches(|c: char| c == '-' || c == '*' || c == ' ' || c == '[' || c == ']' || c == 'x' || c == 'X' || c.is_digit(10) || c == '.')
                    .trim()
                    .to_string();
                
                if !cleaned.is_empty() && cleaned.to_lowercase().contains("step") {
                    let mut step_desc = cleaned;
                    if let Some(colon_idx) = step_desc.find(':') {
                        let potential_step = &step_desc[..colon_idx].to_lowercase();
                        if potential_step.contains("step") {
                            step_desc = step_desc[colon_idx + 1..].trim().to_string();
                        }
                    } else if let Some(dash_idx) = step_desc.find('-') {
                        let potential_step = &step_desc[..dash_idx].to_lowercase();
                        if potential_step.contains("step") {
                            step_desc = step_desc[dash_idx + 1..].trim().to_string();
                        }
                    }
                    if !step_desc.is_empty() {
                        steps.push(step_desc);
                    }
                } else if !cleaned.is_empty() {
                    steps.push(cleaned);
                }
            }
        }
    }

    if steps.is_empty() {
        for line in plan.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                let cleaned = trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == ' ' || c == '[' || c == ']').trim().to_string();
                if cleaned.to_lowercase().contains("step") {
                    if let Some(colon_idx) = cleaned.find(':') {
                        steps.push(cleaned[colon_idx + 1..].trim().to_string());
                    } else {
                        steps.push(cleaned);
                    }
                }
            }
        }
    }

    steps
}

async fn run_grill_flow(
    args: &str,
    runner: &mut Runner,
    user_id: &str,
    session_id: &str,
    nami_skin: &MadSkin,
    provider: &str,
    model_name: &str,
    last_response: &mut Option<String>,
) -> anyhow::Result<()> {
    if args.trim().is_empty() {
        println!("{} Please provide a goal or topic. Usage: /grill <your goal>\n", style::style("⚠️").yellow());
        return Ok(());
    }

    let generate_questions_prompt = format!(
        "The user wants to plan the following goal: '{}'. \
         Please generate 3 to 5 highly precise, concise clarification questions to help design this plan. \
         Format your response ONLY as a plain list of questions, one per line, with each line starting with 'Q: ' and nothing else.",
         args
    );

    println!("\n{} Analyzing goal and generating clarification questions...", style::style("🧠").magenta().bold());
    let questions_resp = execute_silent_prompt(runner, user_id, session_id, &generate_questions_prompt).await?;
    
    let mut questions = Vec::new();
    for line in questions_resp.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Q:") {
            let q = trimmed["Q:".len()..].trim().to_string();
            if !q.is_empty() {
                questions.push(q);
            }
        } else if trimmed.starts_with("Q: ") {
            let q = trimmed["Q: ".len()..].trim().to_string();
            if !q.is_empty() {
                questions.push(q);
            }
        }
    }

    if questions.is_empty() {
        for line in questions_resp.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && (trimmed.ends_with('?') || trimmed.starts_with('-') || trimmed.starts_with('*')) {
                let q = trimmed.trim_start_matches(|c: char| c == '-' || c == '*' || c == ' ' || c.is_digit(10) || c == '.').trim().to_string();
                if !q.is_empty() {
                    questions.push(q);
                }
            }
        }
    }

    if questions.is_empty() {
        questions.push("Can you describe any specific requirements or preferences for this plan?".to_string());
    }

    let mut answers = Vec::new();
    for (i, q) in questions.iter().enumerate() {
        println!("\n{} {}/{} > {}", style::style("❓").magenta().bold(), i + 1, questions.len(), style::style(q).bold());
        print!("Your Answer > ");
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        let trimmed_answer = answer.trim().to_string();
        answers.push(format!("Q: {}\nA: {}", q, trimmed_answer));
    }

    println!("\n{} Synthesizing implementation plan...", style::style("⚙️").cyan().bold());
    let qa_context = answers.join("\n\n");
    let plan_prompt = format!(
        "Based on the user's goal: '{}' and their answers to the clarification questions:\n\n{}\n\n\
         Please synthesize a highly precise, step-by-step implementation plan. \
         The plan MUST be formatted as a Markdown document. \
         It MUST contain a section '## Implementation Steps' where each step is a checkbox list item of the exact format:\
         '- [ ] Step N: <detailed task explanation>'\n\
         For example:\
         '- [ ] Step 1: Create the main function'\n\
         '- [ ] Step 2: Implement error handling'\n\n\
         Keep the steps concrete and actionable so they can be parsed and executed programmatically.",
        args, qa_context
    );

    let plan_content = run_and_stream_prompt(runner, user_id, session_id, &plan_prompt, nami_skin, provider, model_name).await?;

    let nami_dir = crate::utils::get_nami_dir();
    let plans_dir = nami_dir.join("plans");
    let _ = std::fs::create_dir_all(&plans_dir);
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let plan_filename = format!("plan_{}.md", timestamp);
    let plan_path = plans_dir.join(&plan_filename);
    let _ = std::fs::write(&plan_path, &plan_content);

    println!("{} Plan saved to {}", style::style("💾").green(), style::style(plan_path.to_string_lossy()).underlined());

    let steps = parse_plan_steps(&plan_content);
    if steps.is_empty() {
        println!("{} No executable steps could be parsed from the plan.\n", style::style("⚠️").yellow());
        return Ok(());
    }

    println!("\n{} Parsed {} steps for execution.", style::style("📋").cyan().bold(), steps.len());
    for (i, step) in steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }

    print!("\n{} Do you want to execute this plan? (y/n) > ", style::style("🤔").yellow().bold());
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    if confirm.trim().to_lowercase() != "y" && confirm.trim().to_lowercase() != "yes" {
        println!("{} Plan execution cancelled.\n", style::style("❌").red());
        return Ok(());
    }

    println!("\n{} Starting execution of plan steps...", style::style("🚀").green().bold());
    for (i, step) in steps.iter().enumerate() {
        let header = format!("🚀 Executing Step {} of {}: {}", i + 1, steps.len(), step);
        println!("\n{}", style::style(&header).green().bold());
        println!("{}", style::style("─".repeat(header.chars().count())).green().dim());
        
        let step_prompt = format!(
            "Execute the following step of our plan. Retain full context of previous steps. \
             Step {} of {}: {}",
            i + 1, steps.len(), step
        );

        let resp = run_and_stream_prompt(runner, user_id, session_id, &step_prompt, nami_skin, provider, model_name).await?;
        *last_response = Some(resp);
    }

    println!("\n{} All {} steps of the plan have been successfully executed! 🎉\n", style::style("✅").green().bold(), steps.len());

    Ok(())
}


pub async fn handle_slash_command(
    trimmed: &str,
    runner: &mut Runner,
    sessions: &Arc<dyn SessionService>,
    artifacts: &Arc<dyn adk_artifact::ArtifactService>,
    model: &mut Arc<dyn Llm>,
    app_name: &str,
    user_id: &str,
    session_id: &mut String,
    nami_skin: &MadSkin,
    provider: &mut String,
    model_name: &mut String,
    registry: &CommandRegistry,
    mcp_count: &mut usize,
    skill_count: &mut usize,
    last_response: &mut Option<String>,
    agent: &mut Arc<dyn Agent>,
) -> anyhow::Result<bool> {
    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let command_name = parts[0];
    let args = parts.get(1).unwrap_or(&"");

    if command_name == "/switch" {
        if let Some((new_prov, new_model)) = run_switch_flow().await? {
            *provider = new_prov;
            *model_name = new_model;

            println!("🔄 Rebuilding Nami agent with the new LLM model client...");
            let (new_agent, fresh_model, _, _, fresh_mcp_count, fresh_skill_count) = crate::agent::build_agent().await?;
            
            *agent = new_agent;
            *model = fresh_model;
            *mcp_count = fresh_mcp_count;
            *skill_count = fresh_skill_count;

            *runner = Runner::builder()
                .app_name(app_name)
                .agent(agent.clone())
                .session_service(sessions.clone())
                .artifact_service(artifacts.clone())
                .compaction_config(get_compaction_config(model.clone()))
                .build()?;

            println!("{} Successfully switched to {} using model {}!\n", 
                style::style("✨").green(), 
                provider, 
                model_name
            );
        }
        return Ok(false);
    }

    if command_name == "/grill" {
        run_grill_flow(
            args,
            runner,
            user_id,
            session_id,
            nami_skin,
            provider,
            model_name,
            last_response,
        ).await?;
        return Ok(false);
    }

    if command_name == "/plan" {
        let prompt = format!(
            "Create a detailed step-by-step implementation plan for the following task. The plan must outline: \
             1. Goals & Requirements, 2. Design Decisions & Architecture, 3. Success Criteria & Verification Steps, \
             and 4. A sequential task list with concrete steps under a section '## Implementation Steps' where each step is a checkbox list item of the exact format:\n\
             '- [ ] Step N: <detailed task explanation>'\n\
             For example:\n\
             '- [ ] Step 1: Create the main function'\n\
             '- [ ] Step 2: Implement error handling'\n\n\
             Save the compiled plan to the workspace or `~/.nami/plans/` directory (e.g., as `plan_[date-time].md`) as a user-facing artifact, \
             and present it clearly to the user for feedback and approval before executing any code. Task: {}",
            args
        );
        let resp = run_and_stream_prompt(runner, user_id, session_id, &prompt, nami_skin, provider, model_name).await?;
        *last_response = Some(resp);
        return Ok(false);
    }

    if command_name == "/copy" {
        if let Some(ref text) = *last_response {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Err(e) = clipboard.set_text(text.clone()) {
                        println!("{} Failed to copy to clipboard: {}\n", style::style("❌").red(), e);
                    } else {
                        println!("{} Last response copied to clipboard!\n", style::style("📋").green());
                    }
                }
                Err(e) => {
                    println!("{} Failed to initialize clipboard: {}\n", style::style("❌").red(), e);
                }
            }
        } else {
            println!("{} No response available to copy yet.\n", style::style("⚠️").yellow());
        }
        return Ok(false);
    }

    // Dynamic registry lookup
    if let Some(prompt) = registry.format_prompt(command_name, args) {
        let resp = run_and_stream_prompt(runner, user_id, session_id, &prompt, nami_skin, provider, model_name).await?;
        *last_response = Some(resp);
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

            render_banner(provider, model_name, session_id, *mcp_count, *skill_count);
        }

        "/new" => {
            let session_id_new = Uuid::new_v4().to_string();
            ensure_session(sessions, app_name, user_id, &session_id_new).await?;

            execute!(
                io::stdout(),
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            )?;

            render_banner(provider, model_name, &session_id_new, *mcp_count, *skill_count);

            println!(
                "{}\n",
                style::style("✨ New session started").green()
            );
            *session_id = session_id_new;
        }

        "/version" => {
            println!(
                "{} {}\n",
                style::style("Nami CLI").bold(),
                env!("CARGO_PKG_VERSION")
            );
        }

        "/status" => {
            run_and_stream_prompt(runner, user_id, session_id, "Please retrieve and report the system status using your system_status skill.", nami_skin, provider, model_name).await?;
        }

        _ => {
            println!("{} {}\n", style::style("Unknown command:").red(), trimmed);
        }
    }
    Ok(false)
}


pub async fn run_cli(
    mut agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    artifacts: Arc<dyn adk_artifact::ArtifactService>,
    mut model: Arc<dyn Llm>,
    mut provider: String,
    mut model_name: String,
    mut mcp_count: usize,
    mut skill_count: usize,
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
        .artifact_service(artifacts.clone())
        .compaction_config(get_compaction_config(model.clone()))
        .build()?;


    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut rl: Editor<NamiHelper, rustyline::history::FileHistory> = Editor::with_config(config)?;

    rl.set_helper(Some(NamiHelper::new()));

    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Tab, rustyline::Modifiers::NONE),
        rustyline::Cmd::Complete,
    );

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
    let mut last_response: Option<String> = None;

    loop {
       
        let line = rl.readline("> ");

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
                        &artifacts,
                        &mut model,
                        app_name,
                        user_id,
                        &mut session_id,
                        &nami_skin,
                        &mut provider,
                        &mut model_name,
                        &registry,
                        &mut mcp_count,
                        &mut skill_count,
                        &mut last_response,
                        &mut agent,
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

                let resp = run_and_stream_prompt(
                    &mut runner,
                    user_id,
                    &session_id,
                    &enriched_prompt,
                    &nami_skin,
                    &provider,
                    &model_name,
                )
                .await?;

                last_response = Some(resp);
            }

            Err(_) => {
                break;
            }
        }
    }

Ok(())
}

async fn run_switch_flow() -> anyhow::Result<Option<(String, String)>> {
    use inquire::Select;
    use inquire::Text;

    println!("\n🔄 Let's switch your LLM provider and model dynamically!");

    let providers = vec!["gemini", "openai", "anthropic", "ollama", "openrouter"];
    let selected_provider = Select::new("Choose LLM Provider:", providers).prompt()?;

    let standard_models = match selected_provider {
        "gemini" => vec!["gemini-2.5-flash", "gemini-2.5-pro", "gemini-1.5-flash", "gemini-1.5-pro", "custom"],
        "openai" => vec!["gpt-4o", "gpt-4o-mini", "o1-mini", "o1-preview", "custom"],
        "anthropic" => vec!["claude-3-5-sonnet-latest", "claude-3-5-haiku-latest", "claude-3-opus-latest", "custom"],
        "ollama" => vec!["llama3", "mistral", "phi3", "custom"],
        "openrouter" => vec!["meta-llama/llama-3-70b-instruct", "mistralai/mistral-7b-instruct", "custom"],
        _ => vec!["custom"],
    };

    let model_choice = Select::new(&format!("Choose model for {}:", selected_provider), standard_models).prompt()?;

    let final_model = if model_choice == "custom" {
        Text::new("Enter custom model name:").prompt()?
    } else {
        model_choice.to_string()
    };

    let default_env = match selected_provider {
        "gemini" => "GOOGLE_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        "ollama" => "",
        "openrouter" => "OPENROUTER_API_KEY",
        _ => "",
    };

    let final_env = if !default_env.is_empty() {
        let env_prompt = Text::new("Enter Environment Variable Name for API Key:")
            .with_default(default_env)
            .prompt()?;
        Some(env_prompt)
    } else {
        None
    };

    // Prompt for API key value and update ~/.nami/.env if specified
    if let Some(ref env_name) = final_env {
        use inquire::Password;
        let prompt_text = format!("Enter API Key value for {}:", env_name);
        let key_input = Password::new(&prompt_text)
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?;
        if !key_input.is_empty() {
            let env_path = get_nami_dir().join(".env");
            let mut lines = Vec::new();
            let mut updated = false;
            if env_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&env_path) {
                    for line in content.lines() {
                        if let Some(idx) = line.find('=') {
                            let k = line[..idx].trim();
                            if k == env_name {
                                lines.push(format!("{}={}", env_name, key_input));
                                updated = true;
                            } else {
                                lines.push(line.to_string());
                            }
                        } else {
                            lines.push(line.to_string());
                        }
                    }
                }
            }
            if !updated {
                lines.push(format!("{}={}", env_name, key_input));
            }
            if let Err(e) = std::fs::write(&env_path, lines.join("\n") + "\n") {
                println!("⚠️ Failed to write API key to .env: {}", e);
            } else {
                println!("✅ Successfully updated API key in ~/.nami/.env");
            }
        }
    }

    // Update config.toml
    let config_path = get_nami_dir().join("config.toml");
    if config_path.exists() {
        if let Ok(config_str) = std::fs::read_to_string(&config_path) {
            if let Ok(mut toml_val) = toml::from_str::<toml::Value>(&config_str) {
                if let Some(model_table) = toml_val.get_mut("model") {
                    if let Some(table) = model_table.as_table_mut() {
                        table.insert("provider".to_string(), toml::Value::String(selected_provider.to_string()));
                        table.insert("model_name".to_string(), toml::Value::String(final_model.clone()));
                        if let Some(env) = final_env {
                            table.insert("api_key_env".to_string(), toml::Value::String(env));
                        } else {
                            table.remove("api_key_env");
                        }
                    }
                } else if let Some(root_table) = toml_val.as_table_mut() {
                    let mut model_table = toml::value::Table::new();
                    model_table.insert("provider".to_string(), toml::Value::String(selected_provider.to_string()));
                    model_table.insert("model_name".to_string(), toml::Value::String(final_model.clone()));
                    if let Some(env) = final_env {
                        model_table.insert("api_key_env".to_string(), toml::Value::String(env));
                    }
                    root_table.insert("model".to_string(), toml::Value::Table(model_table));
                }

                if let Ok(updated_str) = toml::to_string_pretty(&toml_val) {
                    if let Err(e) = std::fs::write(&config_path, updated_str) {
                        println!("⚠️ Failed to persist changes to config.toml: {}", e);
                    }
                }
            }
        }
    }

    Ok(Some((selected_provider.to_string(), final_model)))
}
