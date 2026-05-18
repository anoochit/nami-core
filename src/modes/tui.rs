use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::SessionService;
use crossterm::{
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste,
        Event, KeyCode, KeyEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use futures::stream::BoxStream;
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
};
use std::io;
use std::sync::Arc;
use tokio::sync::mpsc;
use tui_textarea::TextArea;
use uuid::Uuid;

use crate::agent::get_compaction_config;

#[derive(Debug, Clone)]
enum MessageRole {
    User,
    Assistant,
    System,
    ToolCall,
    ToolResponse,
}

#[derive(Debug, Clone)]
struct Message {
    role: MessageRole,
    content: String,
}

struct App<'a> {
    input: TextArea<'a>,
    messages: Vec<Message>,
    list_state: ListState,
    is_thinking: bool,
    session_id: String,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl<'a> App<'a> {
    fn new(session_id: String) -> App<'a> {
        App {
            input: TextArea::default(),
            messages: Vec::new(),
            list_state: ListState::default(),
            is_thinking: false,
            session_id,
            history: Vec::new(),
            history_index: None,
        }
    }

    fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(Message {
            role,
            content,
        });
        let len = self.messages.len();
        if len > 0 {
            // Auto-scroll to bottom on new message
            self.list_state.select(Some(len - 1));
        }
    }

    fn update_last_message(&mut self, chunk: &str) {
        if let Some(last) = self.messages.last_mut() {
            if matches!(last.role, MessageRole::Assistant) {
                last.content.push_str(chunk);
                // Auto-scroll to bottom while streaming
                self.list_state.select(Some(self.messages.len() - 1));
                return;
            }
        }
        self.add_message(MessageRole::Assistant, chunk.to_string());
    }

    fn scroll_down(&mut self) {
        if self.messages.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.messages.len() - 1 { i } else { i + 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn scroll_up(&mut self) {
        if self.messages.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { 0 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

enum AppEvent {
    TerminalEvent(Event),
}

pub(crate) async fn run_tui(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    model: Arc<dyn Llm>,
    provider: String,
    model_name: String,
) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_name = "tui";
    let user_id = "default_user";
    let session_id = Uuid::new_v4().to_string();

    let workspace = std::env::current_dir()?.to_string_lossy().to_string();
    let branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "main".to_string());

    // Ensure session exists in DB
    crate::modes::cli::ensure_session(&sessions, app_name, user_id, &session_id).await?;

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent.clone())
        .session_service(sessions.clone())
        .memory_service(memory.clone())
        .compaction_config(get_compaction_config(model.clone()))
        .build()?;

    let mut app = App::new(session_id.clone());
    app.add_message(
        MessageRole::System,
        format!(
            "Nami TUI v{} ({} using {})",
            env!("CARGO_PKG_VERSION"),
            provider,
            model_name
        ),
    );

    let (tx, mut rx) = mpsc::channel(100);

    // Main loop
    let res = run_app(
        &mut terminal,
        app,
        tx,
        &mut rx,
        runner,
        user_id,
        sessions,
        memory,
        agent,
        model,
        provider,
        model_name,
        &workspace,
        &branch,
    )
    .await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'_>,
    tx: mpsc::Sender<AppEvent>,
    rx: &mut mpsc::Receiver<AppEvent>,
    runner: Runner,
    user_id: &str,
    sessions: Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    provider: String,
    model_name: String,
    workspace: &str,
    branch: &str,
) -> anyhow::Result<()> {
    // Background thread for terminal events
    let tx_event = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(ev) = event::read() {
                if let Err(_) = tx_event.send(AppEvent::TerminalEvent(ev)).await {
                    break;
                }
            }
        }
    });

    let mut stream: Option<
        BoxStream<'static, std::result::Result<adk_session::Event, adk_rust::AdkError>>,
    > = None;

    loop {
        terminal
            .draw(|f| ui(f, &mut app, workspace, branch, &model_name))
            .map_err(|e| anyhow::anyhow!("Draw error: {}", e))?;

        tokio::select! {
            // Handle agent stream if active
            maybe_result = async {
                if let Some(ref mut s) = stream {
                    s.next().await
                } else {
                    std::future::pending().await
                }
            } => {
                match maybe_result {
                    Some(Ok(event)) => {
                        if let Some(content) = &event.llm_response.content {
                            for part in &content.parts {
                                if let Some(text) = part.text() {
                                    app.update_last_message(text);
                                }

                                if let Part::FunctionCall { name, args, .. } = part {
                                    let args_str = args.to_string().replace('\n', " ").replace("  ", " ");
                                    app.add_message(MessageRole::ToolCall, format!("{}({})", name, args_str));
                                }

                                if let Part::FunctionResponse { function_response, .. } = part {
                                    app.add_message(MessageRole::ToolResponse, format!("{}: {}", function_response.name, function_response.response));
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        app.is_thinking = false;
                        app.add_message(MessageRole::System, format!("Error: {}", e));
                        stream = None;
                    }
                    None => {
                        // Stream finished
                        app.is_thinking = false;
                        stream = None;
                    }
                }
            }

            Some(event) = rx.recv() => {
                match event {
                    AppEvent::TerminalEvent(ev) => {
                        match ev {
                            Event::Key(key) => {
                                if key.kind == KeyEventKind::Press {
                                    match key.code {
                                        // ... existing key handling ...
                                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                            return Ok(());
                                        }
                                        KeyCode::Esc => {
                                             if app.is_thinking {
                                                 runner.interrupt(&app.session_id);
                                                 app.is_thinking = false;
                                                 stream = None;
                                             }
                                        }
                                        KeyCode::Enter if !app.is_thinking => {
                                            let input_text = app.input.lines().join("\n");
                                            let trimmed = input_text.trim();
                                            if !trimmed.is_empty() {
                                                app.history.push(input_text.clone());
                                                app.history_index = None;
                                                app.add_message(MessageRole::User, input_text.clone());
                                                app.input = TextArea::default();

                                                // Support Slash Commands in TUI
                                                if trimmed.starts_with('/') {
                                                    match trimmed {
                                                        "/exit" => return Ok(()),
                                                        "/new" => {
                                                            app.session_id = Uuid::new_v4().to_string();
                                                            app.messages.clear();
                                                            app.add_message(MessageRole::System, "Started a new session.".to_string());
                                                            crate::modes::cli::ensure_session(&sessions, "tui", user_id, &app.session_id).await?;
                                                            continue;
                                                        }
                                                        "/clear" => {
                                                            app.messages.clear();
                                                            app.list_state.select(None);
                                                            continue;
                                                        }
                                                        "/?" => {
                                                            let registry = crate::modes::command_registry::CommandRegistry::load_from_config("config.toml")
                                                                .unwrap_or(crate::modes::command_registry::CommandRegistry { commands: Default::default() });
                                                            let help = crate::modes::cli::render_help(&registry);
                                                            app.add_message(MessageRole::System, help);
                                                            continue;
                                                        }
                                                        _ => {}
                                                    }

                                                    let agent_clone = agent.clone();
                                                    let sessions_clone = sessions.clone();
                                                    let memory_clone = memory.clone();
                                                    let model_clone = model.clone();
                                                    let app_name = "tui";
                                                    let user_id_clone = user_id.to_string();
                                                    let mut session_id_clone = app.session_id.clone();
                                                    let mut provider_clone = provider.clone();
                                                    let mut model_name_clone = model_name.clone();
                                                    let cmd = trimmed.to_string();

                                                    let tx_cmd = tx.clone();

                                                    tokio::spawn(async move {
                                                        let mut runner_clone = Runner::builder()
                                                            .app_name(app_name)
                                                            .agent(agent_clone)
                                                            .session_service(sessions_clone.clone())
                                                            .memory_service(memory_clone)
                                                            .compaction_config(get_compaction_config(model_clone))
                                                            .build().unwrap();

                                                        let registry = crate::modes::command_registry::CommandRegistry::load_from_config("config.toml")
                                                            .unwrap_or(crate::modes::command_registry::CommandRegistry { commands: Default::default() });

                                                        let _ = crate::modes::cli::handle_slash_command(
                                                            &cmd,
                                                            &mut runner_clone,
                                                            &sessions_clone,
                                                            app_name,
                                                            &user_id_clone,
                                                            &mut session_id_clone,
                                                            &termimad::MadSkin::default(),
                                                            &mut provider_clone,
                                                            &mut model_name_clone,
                                                            &registry,
                                                        ).await;

                                                        let _ = tx_cmd.send(AppEvent::TerminalEvent(Event::Key(KeyCode::Null.into()))).await;
                                                    });
                                                    continue;
                                                }

                                                app.add_message(MessageRole::Assistant, String::new());
                                                app.is_thinking = true;

                                                let content = Content::new("user").with_text(input_text);
                                                match runner.run_str(user_id, &app.session_id, content).await {
                                                    Ok(s) => {
                                                        stream = Some(s);
                                                    }
                                                    Err(e) => {
                                                        app.is_thinking = false;
                                                        app.add_message(MessageRole::System, format!("Error: {}", e));
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Up => {
                                            if !app.is_thinking {
                                                if !app.history.is_empty() {
                                                    let idx = app.history_index.map(|i| if i > 0 { i - 1 } else { i }).unwrap_or(app.history.len() - 1);
                                                    app.history_index = Some(idx);
                                                    app.input = TextArea::from(app.history[idx].lines().map(|s| s.to_string()));
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            if !app.is_thinking {
                                                if let Some(idx) = app.history_index {
                                                    if idx < app.history.len() - 1 {
                                                        let next = idx + 1;
                                                        app.history_index = Some(next);
                                                        app.input = TextArea::from(app.history[next].lines().map(|s| s.to_string()));
                                                    } else {
                                                        app.history_index = None;
                                                        app.input = TextArea::default();
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::PageUp => {
                                            app.scroll_up();
                                        }
                                        KeyCode::PageDown => {
                                            app.scroll_down();
                                        }
                                        _ => {
                                            if !app.is_thinking {
                                                app.input.input(ev);
                                            }
                                        }
                                    }
                                }
                            }
                            Event::Paste(content) => {
                                if !app.is_thinking {
                                    app.input.insert_str(content);
                                }
                            }
                            _ => {}
                        }                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App, workspace: &str, branch: &str, model: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(4), // Header + gap
                Constraint::Min(1),    // Messages
                Constraint::Length(3), // Input
                Constraint::Length(2), // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    // --- Header ---
    let status = if app.is_thinking {
        Span::styled("● thinking", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("● ready", Style::default().fg(Color::Green))
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("⚡ ", Style::default().fg(Color::Magenta)),
            Span::styled(
                format!("Nami TUI v{}", env!("CARGO_PKG_VERSION")),
                Style::default().bold(),
            ),
            Span::raw(" ".repeat(4)),
            Span::styled("Session:", Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(&app.session_id, Style::default().fg(Color::Cyan)),
            Span::raw(" ".repeat(4)),
            status,
        ]),
        Line::from(""), // Gap
    ]);
    f.render_widget(header, chunks[0]);

    // --- Messages ---
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| {
            let (role_text, color) = match m.role {
                MessageRole::User => ("\n🧔 You > ", Color::Cyan),
                MessageRole::Assistant => ("\n🤖 Nami > ", Color::Magenta),
                MessageRole::System => ("\n⚙️ System > ", Color::Yellow),
                MessageRole::ToolCall => ("\n🔨 Calling > ", Color::Blue),
                MessageRole::ToolResponse => ("\n✅ Response > ", Color::DarkGray),
            };

            let mut lines = Vec::new();

            // Role header
            lines.push(Line::from(vec![Span::styled(
                role_text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )]));

            // Gap
            lines.push(Line::from(""));

            // Markdown content
            let md = tui_markdown::from_str(&m.content);
            for line in md.lines {
                // Add simple indentation
                let mut spans = vec![Span::raw("  ")];
                spans.extend(line.spans);
                lines.push(Line::from(spans));
            }

            // End gap
            lines.push(Line::from(""));

            ListItem::new(lines)
        })
        .collect();

    let messages_list =
        List::new(messages).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(messages_list, chunks[1], &mut app.list_state);

    // --- Input Area ---
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::new(1, 0, 0, 0));

    app.input.set_block(input_block);
    app.input
        .set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));

    let input_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(chunks[2]);

    f.render_widget(
        Paragraph::new(">").style(Style::default().fg(Color::Yellow)),
        input_layout[0],
    );
    f.render_widget(&app.input, input_layout[1]);

    // --- Footer ---
    let footer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[3]);

    let labels = Line::from(vec![
        Span::styled(
            "workspace (/directory)",
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" ".repeat(2)),
        Span::styled("branch", Style::default().fg(Color::DarkGray)),
        Span::raw(" ".repeat(2)),
        Span::styled("model", Style::default().fg(Color::DarkGray)),
    ]);

    let values = Line::from(vec![
        Span::styled(workspace, Style::default()),
        Span::raw(" | "),
        Span::styled(branch, Style::default()),
        Span::raw(" | "),
        Span::styled(model, Style::default()),
    ]);

    f.render_widget(Paragraph::new(labels), footer_layout[0]);
    f.render_widget(Paragraph::new(values), footer_layout[1]);
}
