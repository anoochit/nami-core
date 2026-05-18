use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::SessionService;
use crossterm::{
    event::{ self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEventKind },
    execute,
    terminal::{ EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode },
};
use futures::StreamExt;
use futures::stream::BoxStream;
use ratatui::{
    Frame,
    Terminal,
    backend::{ Backend, CrosstermBackend },
    layout::{ Constraint, Direction, Layout },
    style::{ Color, Modifier, Style },
    text::{ Line, Span },
    widgets::{ Block, Borders, List, ListItem, ListState, Padding, Paragraph },
};
use rustyline::{ Config, Editor, history::FileHistory };
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
    rendered_lines: Vec<Line<'static>>,
}

struct App<'a> {
    input: TextArea<'a>,
    messages: Vec<Message>,
    list_state: ListState,
    is_thinking: bool,
    session_id: String,
    last_width: usize,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl<'a> App<'a> {
    fn new(session_id: String) -> App<'a> {
        let history = Self::load_history_file().unwrap_or_default();
        App {
            input: TextArea::default(),
            messages: Vec::new(),
            list_state: ListState::default(),
            is_thinking: false,
            session_id,
            last_width: 0,
            history,
            history_index: None,
        }
    }

    fn load_history_file() -> anyhow::Result<Vec<String>> {
        let config = Config::builder().build();
        let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;
        let _ = rl.load_history(".cli_history");
        let mut history = Vec::new();
        for entry in rl.history().iter() {
            history.push(entry.to_string());
        }
        Ok(history)
    }

    fn save_history_entry(entry: &str) -> anyhow::Result<()> {
        let config = Config::builder().build();
        let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;
        let _ = rl.load_history(".cli_history");
        rl.add_history_entry(entry)?;
        rl.save_history(".cli_history")?;
        Ok(())
    }

    fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(Message {
            role,
            content,
            rendered_lines: Vec::new(),
        });
        let len = self.messages.len();
        if len > 0 {
            if self.last_width > 0 {
                self.render_message(len - 1, self.last_width);
            }
            // Auto-scroll to bottom on new message
            self.list_state.select(Some(len - 1));
        }
    }

    fn update_last_message(&mut self, chunk: &str) {
        if let Some(last) = self.messages.last_mut() {
            if matches!(last.role, MessageRole::Assistant) {
                last.content.push_str(chunk);
                if self.last_width > 0 {
                    let idx = self.messages.len() - 1;
                    self.render_message(idx, self.last_width);
                }
                // Auto-scroll to bottom while streaming
                self.list_state.select(Some(self.messages.len() - 1));
                return;
            }
        }
        self.add_message(MessageRole::Assistant, chunk.to_string());
    }

    fn render_message(&mut self, index: usize, width: usize) {
        if let Some(m) = self.messages.get_mut(index) {
            let (role_text, color) = match m.role {
                MessageRole::User => ("\n🧔 You > ", Color::Cyan),
                MessageRole::Assistant => ("\n🤖 Nami > ", Color::Magenta),
                MessageRole::System => ("\n📢 System > ", Color::Yellow),
                MessageRole::ToolCall => ("\n🔨 Calling > ", Color::Blue),
                MessageRole::ToolResponse => ("\n✅ Response > ", Color::DarkGray),
            };

            let mut lines = Vec::new();

            // Role header
            lines.push(
                Line::from(
                    vec![
                        Span::styled(
                            role_text,
                            Style::default().fg(color).add_modifier(Modifier::BOLD)
                        )
                    ]
                )
            );

            // Gap
            lines.push(Line::from(""));

            // Markdown content using Termimad for table support
            let skin = termimad::MadSkin::default();
            // Subtract a bit for indentation and borders
            let content_width = width.saturating_sub(4);
            let text = skin.text(&m.content, Some(content_width));

            for line in text.lines {
                let mut spans = vec![Span::raw("  ")]; // Indentation
                match line {
                    termimad::FmtLine::Normal(composite) => {
                        render_composite(&mut spans, &composite, &skin);
                    }
                    termimad::FmtLine::TableRow(row) => {
                        spans.push(Span::raw("|"));
                        for cell in row.cells {
                            render_composite(&mut spans, &cell, &skin);
                            spans.push(Span::raw("|"));
                        }
                    }
                    termimad::FmtLine::TableRule(rule) => {
                        let mut s = String::from("+");
                        for &w in &rule.widths {
                            s.push_str(&"-".repeat(w));
                            s.push('+');
                        }
                        spans.push(Span::raw(s));
                    }
                    termimad::FmtLine::HorizontalRule => {
                        spans.push(Span::raw("-".repeat(content_width)));
                    }
                }
                lines.push(Line::from(spans));
            }

            // End gap
            lines.push(Line::from(""));
            m.rendered_lines = lines;
        }
    }

    fn re_render_all(&mut self, width: usize) {
        self.last_width = width;
        for i in 0..self.messages.len() {
            self.render_message(i, width);
        }
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

fn render_composite(
    spans: &mut Vec<Span<'static>>,
    composite: &termimad::FmtComposite<'_>,
    skin: &termimad::MadSkin
) {
    let mut current_width = 0;

    let mut base_style = Style::default();
    match composite.kind {
        termimad::CompositeKind::Header(level) => {
            if let Some(h) = skin.headers.get((level as usize) - 1) {
                apply_compound_style(&mut base_style, &h.compound_style);
            }
        }
        termimad::CompositeKind::Code => {
            apply_compound_style(&mut base_style, &skin.code_block.compound_style);
        }
        termimad::CompositeKind::Quote => {
            apply_compound_style(&mut base_style, &skin.paragraph.compound_style);
        }
        _ => {
            apply_compound_style(&mut base_style, &skin.paragraph.compound_style);
        }
    }

    for compound in &composite.compounds {
        let mut style = base_style;
        if compound.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if compound.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if compound.strikeout {
            style = style.add_modifier(Modifier::CROSSED_OUT);
        }
        if compound.code {
            apply_compound_style(&mut style, &skin.inline_code);
        }

        let text = compound.src.to_string();
        current_width += text.chars().count();
        spans.push(Span::styled(text, style));
    }

    if let Some(spacing) = &composite.spacing {
        if spacing.width > current_width {
            let padding = " ".repeat(spacing.width - current_width);
            spans.push(Span::styled(padding, base_style));
        }
    }
}

fn apply_compound_style(style: &mut Style, c_style: &termimad::CompoundStyle) {
    if let Some(fg) = c_style.get_fg() {
        *style = style.fg(map_termimad_color(fg));
    }
    if let Some(bg) = c_style.get_bg() {
        *style = style.bg(map_termimad_color(bg));
    }
}

fn map_termimad_color(c: crossterm::style::Color) -> Color {
    match c {
        crossterm::style::Color::Reset => Color::Reset,
        crossterm::style::Color::Black => Color::Black,
        crossterm::style::Color::DarkGrey => Color::DarkGray,
        crossterm::style::Color::Red => Color::LightRed,
        crossterm::style::Color::DarkRed => Color::Red,
        crossterm::style::Color::Green => Color::LightGreen,
        crossterm::style::Color::DarkGreen => Color::Green,
        crossterm::style::Color::Yellow => Color::LightYellow,
        crossterm::style::Color::DarkYellow => Color::Yellow,
        crossterm::style::Color::Blue => Color::LightBlue,
        crossterm::style::Color::DarkBlue => Color::Blue,
        crossterm::style::Color::Magenta => Color::LightMagenta,
        crossterm::style::Color::DarkMagenta => Color::Magenta,
        crossterm::style::Color::Cyan => Color::LightCyan,
        crossterm::style::Color::DarkCyan => Color::Cyan,
        crossterm::style::Color::White => Color::White,
        crossterm::style::Color::Grey => Color::Gray,
        crossterm::style::Color::Rgb { r, g, b } => Color::Rgb(r, g, b),
        crossterm::style::Color::AnsiValue(v) => Color::Indexed(v),
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
    model_name: String
) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_name = "tui";
    let user_id = "default_user";
    let session_id = Uuid::new_v4().to_string();

    let workspace = std::env::current_dir()?.to_string_lossy().to_string();
    let branch = std::process::Command
        ::new("git")
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
        format!("Nami TUI v{} ({} using {})", env!("CARGO_PKG_VERSION"), provider, model_name)
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
        &branch
    ).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableBracketedPaste)?;
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
    branch: &str
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
        BoxStream<'static, std::result::Result<adk_session::Event, adk_rust::AdkError>>
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
                                                let _ = App::save_history_entry(trimmed);
                                                app.history_index = None;
                                                app.add_message(MessageRole::User, input_text.clone());
                                                app.input = TextArea::default();

                                                // Support Slash Commands in TUI
                                                if trimmed.starts_with('/') {
                                                    let registry = crate::modes::command_registry::CommandRegistry::load_from_config("config.toml")
                                                        .unwrap_or(crate::modes::command_registry::CommandRegistry { commands: Default::default() });

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
                                                            let help = crate::modes::cli::render_help(&registry);
                                                            app.add_message(MessageRole::System, help);
                                                            continue;
                                                        }
                                                        cmd if cmd.starts_with('/') => {
                                                            let cmd_name = &cmd[1..];
                                                            if let Some(command) = registry.get_command(cmd_name) {
                                                                let prompt = command.template.clone();
                                                                app.add_message(MessageRole::User, format!("Running command: {}", cmd_name));
                                                                app.add_message(MessageRole::Assistant, String::new());
                                                                app.is_thinking = true;
                                                                
                                                                let content = Content::new("user").with_text(prompt);
                                                                match runner.run_str(user_id, &app.session_id, content).await {
                                                                    Ok(s) => { stream = Some(s); }
                                                                    Err(e) => {
                                                                        app.is_thinking = false;
                                                                        app.add_message(MessageRole::System, format!("Error: {}", e));
                                                                    }
                                                                }
                                                                continue;
                                                            }
                                                        }
                                                        _ => {}
                                                    }
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
                Constraint::Length(3), // Header + gap
                Constraint::Min(1), // Messages
                Constraint::Length(3), // Input
                Constraint::Length(2), // Footer
            ].as_ref()
        )
        .split(f.area());

    // --- Header ---
    let status = if app.is_thinking {
        Span::styled("● thinking", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("● ready", Style::default().fg(Color::Green))
    };

    let header = Paragraph::new(
        vec![
            Line::from(
                vec![
                    Span::styled("⚡ ", Style::default().fg(Color::Magenta)),
                    Span::styled(
                        format!("Nami TUI v{}", env!("CARGO_PKG_VERSION")),
                        Style::default().bold()
                    ),
                    Span::raw(" ".repeat(4)),
                    Span::styled("Session:", Style::default().fg(Color::DarkGray)),
                    Span::raw(" "),
                    Span::styled(&app.session_id, Style::default().fg(Color::Cyan)),
                    Span::raw(" ".repeat(4)),
                    status
                ]
            ),
            Line::from("") // Gap
        ]
    );
    f.render_widget(header, chunks[0]);

    // --- Messages ---
    let list_width = chunks[1].width as usize;
    if app.last_width != list_width {
        app.re_render_all(list_width);
    }

    let messages: Vec<ListItem> = app.messages
        .iter()
        .map(|m| ListItem::new(m.rendered_lines.clone()))
        .collect();

    let messages_list = List::new(messages).highlight_style(
        Style::default().add_modifier(Modifier::BOLD)
    );

    f.render_stateful_widget(messages_list, chunks[1], &mut app.list_state);

    // --- Input Area ---
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::new(1, 0, 0, 0));

    app.input.set_block(input_block);
    app.input.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));

    let input_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(chunks[2]);

    f.render_widget(Paragraph::new(">").style(Style::default().fg(Color::Yellow)), input_layout[0]);
    f.render_widget(&app.input, input_layout[1]);

    // --- Footer ---
    let footer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[3]);

    let labels = Line::from(
        vec![
            Span::styled("workspace (/directory)", Style::default().fg(Color::DarkGray)),
            Span::raw(" ".repeat(2)),
            Span::styled("branch", Style::default().fg(Color::DarkGray)),
            Span::raw(" ".repeat(2)),
            Span::styled("model", Style::default().fg(Color::DarkGray))
        ]
    );

    let values = Line::from(
        vec![
            Span::styled(workspace, Style::default()),
            Span::raw(" | "),
            Span::styled(branch, Style::default()),
            Span::raw(" | "),
            Span::styled(model, Style::default())
        ]
    );

    f.render_widget(Paragraph::new(labels), footer_layout[0]);
    f.render_widget(Paragraph::new(values), footer_layout[1]);
}
