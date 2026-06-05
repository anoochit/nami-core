use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::SessionService;
use crossterm::{
    event::{ self, DisableBracketedPaste, EnableBracketedPaste, EnableMouseCapture, DisableMouseCapture, Event, KeyCode, KeyEventKind },
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
    widgets::{ Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarState, ScrollbarOrientation },
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
    scroll_offset: usize,
    auto_scroll: bool,
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
            scroll_offset: 0,
            auto_scroll: true,
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
        let history_path = crate::utils::get_nami_dir().join(".cli_history");
        let _ = rl.load_history(&history_path);
        let mut history = Vec::new();
        for entry in rl.history().iter() {
            history.push(entry.to_string());
        }
        Ok(history)
    }

    fn save_history_entry(entry: &str) -> anyhow::Result<()> {
        let config = Config::builder().build();
        let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;
        let history_path = crate::utils::get_nami_dir().join(".cli_history");
        let _ = rl.load_history(&history_path);
        rl.add_history_entry(entry)?;
        rl.save_history(&history_path)?;
        Ok(())
    }

    fn total_lines(&self) -> usize {
        self.messages.iter().map(|m| m.rendered_lines.len()).sum()
    }

    fn get_all_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for m in &self.messages {
            lines.extend(m.rendered_lines.clone());
        }
        lines
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
            self.auto_scroll = true;
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
                self.auto_scroll = true;
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

    fn scroll_down_by(&mut self, amount: usize, viewport_height: usize) {
        let total = self.total_lines();
        let max_scroll = total.saturating_sub(viewport_height);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
        if self.scroll_offset >= max_scroll {
            self.auto_scroll = true;
        }
    }

    fn scroll_up_by(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.auto_scroll = false;
    }

    fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = false;
    }

    fn scroll_to_bottom(&mut self, viewport_height: usize) {
        let total = self.total_lines();
        self.scroll_offset = total.saturating_sub(viewport_height);
        self.auto_scroll = true;
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

pub async fn run_tui(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    model: Arc<dyn Llm>,
    provider: String,
    model_name: String,
    mcp_count: usize,
    skill_count: usize,
) -> anyhow::Result<()> {
    // Set panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableBracketedPaste, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste, EnableMouseCapture)?;
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
        &branch,
        mcp_count,
        skill_count
    ).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableBracketedPaste, DisableMouseCapture)?;
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
    _memory: Arc<dyn adk_rust::Memory>,
    _agent: Arc<dyn Agent>,
    _model: Arc<dyn Llm>,
    _provider: String,
    model_name: String,
    workspace: &str,
    branch: &str,
    mcp_count: usize,
    skill_count: usize,
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
            .draw(|f| ui(f, &mut app, workspace, branch, &model_name, mcp_count, skill_count))
            .map_err(|e| anyhow::anyhow!("Draw error: {}", e))?;

        let mut function_response_buffer: Vec<Part> = Vec::new();

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

                                if let Part::FunctionResponse { .. } = part {
                                    function_response_buffer.push(part.clone());
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
                        // Stream finished. Flush batched responses if any.
                        if !function_response_buffer.is_empty() {
                            // Display responses in TUI
                            for part in &function_response_buffer {
                                if let Part::FunctionResponse { function_response, .. } = part {
                                    app.add_message(MessageRole::ToolResponse, format!("{}: {}", function_response.name, function_response.response));
                                }
                            }
                            
                            let response_content = Content {
                                role: "function".to_string(),
                                parts: function_response_buffer.drain(..).collect(),
                            };
                            if let Ok(mut response_stream) = runner.run_str(user_id, &app.session_id, response_content).await {
                                while let Some(_) = response_stream.next().await {}
                            }
                        }
                        
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
                                            if key.modifiers.contains(event::KeyModifiers::ALT) || key.modifiers.contains(event::KeyModifiers::CONTROL) {
                                                app.input.insert_newline();
                                            } else {
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
                                                        let config_path = crate::utils::get_nami_dir().join("config.toml");
                                                        let registry = crate::modes::command_registry::CommandRegistry::load_from_config(&config_path.to_string_lossy())
                                                            .unwrap_or(crate::modes::command_registry::CommandRegistry { commands: Default::default() });

                                                        match trimmed {
                                                            "/exit" => return Ok(()),
                                                            "/new" => {
                                                                app.session_id = Uuid::new_v4().to_string();
                                                                app.messages.clear();
                                                                app.scroll_offset = 0;
                                                                app.auto_scroll = true;
                                                                app.add_message(MessageRole::System, "Started a new session.".to_string());
                                                                crate::modes::cli::ensure_session(&sessions, "tui", user_id, &app.session_id).await?;
                                                                continue;
                                                            }
                                                            "/clear" => {
                                                                app.messages.clear();
                                                                app.scroll_offset = 0;
                                                                app.auto_scroll = true;
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
                                        }
                                        KeyCode::Up if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                            app.scroll_up_by(1);
                                        }
                                        KeyCode::Down if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                            let viewport_height = terminal.size().map(|s| s.height.saturating_sub(9) as usize).unwrap_or(15);
                                            app.scroll_down_by(1, viewport_height);
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
                                            let viewport_height = terminal.size().map(|s| s.height.saturating_sub(9) as usize).unwrap_or(15);
                                            app.scroll_up_by(viewport_height.saturating_sub(1));
                                        }
                                        KeyCode::PageDown => {
                                            let viewport_height = terminal.size().map(|s| s.height.saturating_sub(9) as usize).unwrap_or(15);
                                            app.scroll_down_by(viewport_height.saturating_sub(1), viewport_height);
                                        }
                                        KeyCode::Home => {
                                            app.scroll_to_top();
                                        }
                                        KeyCode::End => {
                                            let viewport_height = terminal.size().map(|s| s.height.saturating_sub(9) as usize).unwrap_or(15);
                                            app.scroll_to_bottom(viewport_height);
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
                            Event::Mouse(mouse_event) => {
                                let viewport_height = terminal.size().map(|s| s.height.saturating_sub(9) as usize).unwrap_or(15);
                                match mouse_event.kind {
                                    event::MouseEventKind::ScrollUp => {
                                        app.scroll_up_by(3);
                                    }
                                    event::MouseEventKind::ScrollDown => {
                                        app.scroll_down_by(3, viewport_height);
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App, workspace: &str, branch: &str, model: &str, mcp_count: usize, skill_count: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2), // Header with Bottom Border
                Constraint::Min(1), // Messages
                Constraint::Length(3), // Bordered Input Area
                Constraint::Length(2), // Status Footer + Shortcuts
            ].as_ref()
        )
        .split(f.area());

    // --- Header ---
    let status = if app.is_thinking {
        Span::styled("● thinking", Style::default().fg(Color::Yellow).bold())
    } else {
        Span::styled("● ready", Style::default().fg(Color::Green).bold())
    };

    let header = Paragraph::new(
        Line::from(
            vec![
                Span::styled("⚡ ", Style::default().fg(Color::Magenta)),
                Span::styled(
                    format!("Nami TUI v{}", env!("CARGO_PKG_VERSION")),
                    Style::default().fg(Color::LightMagenta).bold()
                ),
                Span::raw("   "),
                Span::styled("Session: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&app.session_id, Style::default().fg(Color::Cyan)),
                Span::raw("   "),
                status
            ]
        )
    ).block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));

    f.render_widget(header, chunks[0]);

    // --- Messages ---
    let list_width = chunks[1].width as usize;
    let viewport_height = chunks[1].height as usize;
    if app.last_width != list_width {
        app.re_render_all(list_width);
    }

    let all_lines = app.get_all_lines();
    let total_lines = all_lines.len();

    // Clamp scroll_offset and handle auto-scroll
    let max_scroll = total_lines.saturating_sub(viewport_height);
    if app.auto_scroll {
        app.scroll_offset = max_scroll;
    } else {
        app.scroll_offset = app.scroll_offset.min(max_scroll);
    }

    let paragraph = Paragraph::new(all_lines)
        .block(Block::default().borders(Borders::NONE))
        .scroll((app.scroll_offset as u16, 0));

    f.render_widget(paragraph, chunks[1]);

    // Render modern stateful scrollbar if history overflows viewport
    if total_lines > viewport_height {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(max_scroll)
            .position(app.scroll_offset);

        f.render_stateful_widget(
            scrollbar,
            chunks[1],
            &mut scrollbar_state,
        );
    }

    // --- Input Area (Bordered and colored dynamically by thinking state) ---
    let input_border_color = if app.is_thinking {
        Color::Yellow
    } else {
        Color::Magenta
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(input_border_color))
        .title(Span::styled(" [Input Prompt] ", Style::default().fg(input_border_color).bold()))
        .padding(Padding::new(1, 1, 0, 0));

    app.input.set_block(input_block);
    app.input.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));

    f.render_widget(&app.input, chunks[2]);

    // --- Footer ---
    let footer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // High-density System Status Line
            Constraint::Length(1), // Shortcuts Legend
        ])
        .split(chunks[3]);

    let footer_info = Line::from(vec![
        Span::styled(" Workspace: ", Style::default().fg(Color::DarkGray)),
        Span::styled(workspace, Style::default().fg(Color::Blue).bold()),
        Span::styled(" | Branch: ", Style::default().fg(Color::DarkGray)),
        Span::styled(branch, Style::default().fg(Color::Green).bold()),
        Span::styled(" | Model: ", Style::default().fg(Color::DarkGray)),
        Span::styled(model, Style::default().fg(Color::Cyan).bold()),
        Span::styled(" | MCPs: ", Style::default().fg(Color::DarkGray)),
        Span::styled(mcp_count.to_string(), Style::default().fg(Color::Yellow).bold()),
        Span::styled(" | Skills: ", Style::default().fg(Color::DarkGray)),
        Span::styled(skill_count.to_string(), Style::default().fg(Color::Magenta).bold()),
    ]);

    let legend = Line::from(vec![
        Span::styled(" Enter ", Style::default().bg(Color::DarkGray).fg(Color::White).bold()),
        Span::raw(" Send  "),
        Span::styled(" Alt+Enter ", Style::default().bg(Color::DarkGray).fg(Color::White).bold()),
        Span::raw(" New Line  "),
        Span::styled(" PgUp / PgDn ", Style::default().bg(Color::DarkGray).fg(Color::White).bold()),
        Span::raw(" Scroll  "),
        Span::styled(" Esc ", Style::default().bg(Color::DarkGray).fg(Color::White).bold()),
        Span::raw(" Interrupt  "),
        Span::styled(" Ctrl+C ", Style::default().bg(Color::DarkGray).fg(Color::White).bold()),
        Span::raw(" Quit"),
    ]);

    f.render_widget(Paragraph::new(footer_info), footer_layout[0]);
    f.render_widget(Paragraph::new(legend), footer_layout[1]);
}
