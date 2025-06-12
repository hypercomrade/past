use std::io;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::layout::{Layout, Constraint, Direction};

pub fn interactive_search(commands: &[String]) -> Option<String> {
    // Set up terminal
    let stdout = io::stdout().into_raw_mode().ok()?;
    let stdout = stdout.into_alternate_screen().ok()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).ok()?;

    // Initial state
    let mut input = String::new();
    let mut selected = 0;
    
    // Create a vector of unique commands in reverse order (most recent first)
    let mut unique_commands = Vec::new();
    for cmd in commands.iter().rev() {
        if !unique_commands.iter().any(|x| *x == cmd) {
            unique_commands.push(cmd);
        }
    }
    let mut filtered_commands: Vec<&String> = unique_commands.clone();

    loop {
        // Filter commands based on input
        filtered_commands = unique_commands
            .iter()
            .filter(|cmd| cmd.to_lowercase().contains(&input.to_lowercase()))
            .copied()
            .collect();

        // Limit to 20 most recent matches
        let display_commands = filtered_commands.iter().take(20).collect::<Vec<_>>();

        // Draw the UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let block = Block::default()
                .title("Interactive Search (ESC to quit)")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[0]);

            let list = List::new(
                display_commands
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| {
                        let content = if i == selected {
                            format!("> {}", cmd)
                        } else {
                            format!("  {}", cmd)
                        };
                        ListItem::new(content)
                    })
                    .collect::<Vec<_>>(),
            )
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(list, chunks[1]);
        }).ok()?;

        // Handle input
        if let Some(key) = io::stdin().lock().keys().next() {
            match key.unwrap() {
                Key::Char('\n') => {
                    // Return selected command
                    if !filtered_commands.is_empty() {
                        return Some(filtered_commands[selected].to_string());
                    }
                }
                Key::Char(c) => {
                    // Add character to search
                    input.push(c);
                    selected = 0;
                }
                Key::Backspace => {
                    // Remove last character
                    input.pop();
                    selected = 0;
                }
                Key::Up => {
                    // Move selection up
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                Key::Down => {
                    // Move selection down
                    if selected < display_commands.len().saturating_sub(1) {
                        selected += 1;
                    }
                }
                Key::Esc => {
                    // Exit without selection
                    return None;
                }
                _ => {}
            }
        }
    }
}
