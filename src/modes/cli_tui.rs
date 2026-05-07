use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, List, ListItem, BorderType},
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

pub async fn run_tui(
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

    let runner = AgentRunner::new(agent, sessions.clone(), "cli_tui", model);
    let mut messages: Vec<(String, String)> = vec![("Nami".to_string(), "Welcome back, Noel! How can I architect today?".to_string())];
    let mut input = String::new();
    let mut is_thinking = false;
    let user_id = "default_user";
    let session_id = "tui_session";

    let text_style = Style::default().fg(Color::White);
    let nami_style = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // Chat Area
                    Constraint::Length(1), // HUD (Status Line)
                    Constraint::Length(3), // Input
                ])
                .split(f.area());

            // Chat Area - Minimalist (no borders)
            let chat_items: Vec<ListItem> = messages.iter().map(|(sender, msg)| {
                let sender_style = if sender == "Nami" { nami_style } else { Style::default().fg(Color::Cyan) };
                ListItem::new(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(format!("{}: ", sender), sender_style),
                    ratatui::text::Span::styled(msg, text_style),
                ]))
            }).collect();
            let chat = List::new(chat_items);
            f.render_widget(chat, chunks[0]);

            // HUD - Subtle line
            let hud_text = format!(" 🧠 {} / {} | Status: {} ", 
                provider, model_name, if is_thinking { "Busy" } else { "Active" });
            let hud = Paragraph::new(hud_text)
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(hud, chunks[1]);

            // Input
            let input_widget = Paragraph::new(format!(" > {}", input))
                .block(Block::default()
                    .borders(Borders::TOP)
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Input "));
            f.render_widget(input_widget, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => { input.pop(); }
                    KeyCode::Enter => {
                        if !input.is_empty() {
                            let trimmed = input.trim();
                            if trimmed == "/exit" || trimmed == "/quit" {
                                break;
                            }
                            
                            messages.push(("You".to_string(), input.clone()));
                            let prompt = input.clone();
                            input.clear();
                                                        
                            if let Ok(response) = runner.run(user_id, session_id, &prompt).await {
                                messages.push(("Nami".to_string(), response));
                            }
                            is_thinking = false;
                        }
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
