use crossterm::{cursor, execute, style, terminal};
use std::io::{self, Write};
use termimad::MadSkin;

pub fn clear_raw_output(stdout: &mut io::Stdout, start_pos: (u16, u16), buffer: &str) -> anyhow::Result<()> {
    let (term_width, _) = terminal::size().unwrap_or((80, 24));
    let mut raw_lines = 0;
    for line in buffer.split('\n') {
        raw_lines += (line.len() as u16) / term_width.max(1) + 1;
    }

    execute!(stdout, cursor::MoveTo(start_pos.0, start_pos.1))?;
    for _ in 0..raw_lines {
        execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine), cursor::MoveDown(1))?;
    }
    execute!(stdout, cursor::MoveTo(start_pos.0, start_pos.1))?;
    Ok(())
}

pub fn render_pretty(
    stdout: &mut io::Stdout,
    nami_skin: &MadSkin,
    rendered_text: &str,
    start_pos: Option<(u16, u16)>,
    buffer: &str
) -> anyhow::Result<()> {
    execute!(stdout, cursor::Hide)?;
    
    if let Some(pos) = start_pos {
        clear_raw_output(stdout, pos, buffer)?;
    }
    
    print!("{}", rendered_text);
    execute!(stdout, cursor::Show)?;
    stdout.flush()?;
    Ok(())
}
