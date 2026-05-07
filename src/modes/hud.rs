use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use std::io;
use std::sync::Arc;
use adk_rust::{Agent, Llm};
use adk_session::SessionService;
use crate::runner::AgentRunner;

pub async fn run_hud(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
    provider: String,
    model_name: String,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let user_id = "default_user";
    let session_id = "hud_session";

    let mut logs: Vec<(String, String)> = vec![
        ("SYSTEM".to_string(), format!("Nami HUD initialized. Monitoring {} via {}.", model_name, provider)),
        ("INFO".to_string(), "Fetching active tasks...".to_string()),
    ];

    let runner = AgentRunner::new(agent.clone(), sessions.clone(), "hud", model.clone());
    if let Ok(tasks) = runner.run(user_id, session_id, "list_active_tasks").await {
        logs.push(("TASKS".to_string(), tasks));
    }

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Info Header
                    Constraint::Min(1),    // Log Area
                    Constraint::Length(1), // Footer
                ])
                .split(f.area());

            // Header - System Info
            let info_text = format!(" 🧠 Provider: {} | Model: {} | Session: {} ", 
                provider, model_name, session_id);
            let info = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title(" System Status "))
                .style(Style::default().fg(Color::Magenta));
            f.render_widget(info, chunks[0]);

            // Log Area
            let log_items: Vec<ListItem> = logs.iter().map(|(source, msg)| {
                let color = match source.as_str() {
                    "SYSTEM" => Color::Yellow,
                    "TASKS" => Color::Green,
                    "INFO" => Color::Blue,
                    _ => Color::White,
                };
                ListItem::new(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(format!("[{}] ", source), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    ratatui::text::Span::styled(msg, Style::default().fg(Color::White)),
                ]))
            }).collect();
            let log_list = List::new(log_items)
                .block(Block::default().borders(Borders::ALL).title(" Activity Log "));
            f.render_widget(log_list, chunks[1]);

            // Footer
            let footer = Paragraph::new(" Press ESC to exit HUD ")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
