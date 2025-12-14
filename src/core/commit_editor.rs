use crate::core::error::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

pub struct CommitEditorState {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    dirty: bool,
}

impl CommitEditorState {
    pub fn new(initial: Option<String>) -> Self {
        let lines = if let Some(msg) = initial {
            msg.lines().map(|l| l.to_string()).collect()
        } else {
            vec![String::new()]
        };

        CommitEditorState {
            lines,
            cursor_line: 0,
            cursor_col: 0,
            dirty: false,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.cursor_line < self.lines.len() {
            let line = &mut self.lines[self.cursor_line];
            if self.cursor_col <= line.len() {
                line.insert(self.cursor_col, ch);
                self.cursor_col += 1;
                self.dirty = true;
            }
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_line < self.lines.len() {
            let line = &mut self.lines[self.cursor_line];
            if self.cursor_col < line.len() {
                line.remove(self.cursor_col);
                self.dirty = true;
            }
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_line < self.lines.len() {
            let line = &mut self.lines[self.cursor_line];
            if self.cursor_col > 0 {
                line.remove(self.cursor_col - 1);
                self.cursor_col -= 1;
                self.dirty = true;
            } else if self.cursor_line > 0 {
                let current = self.lines.remove(self.cursor_line);
                self.cursor_line -= 1;
                let prev = &mut self.lines[self.cursor_line];
                self.cursor_col = prev.len();
                prev.push_str(&current);
                self.dirty = true;
            }
        }
    }

    pub fn new_line(&mut self) {
        if self.cursor_line < self.lines.len() {
            let line = &mut self.lines[self.cursor_line];
            let rest = line[self.cursor_col..].to_string();
            line.truncate(self.cursor_col);
            self.lines.insert(self.cursor_line + 1, rest);
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.dirty = true;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            if let Some(line) = self.lines.get(self.cursor_line) {
                self.cursor_col = line.len();
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_line < self.lines.len() {
            if let Some(line) = self.lines.get(self.cursor_line) {
                if self.cursor_col < line.len() {
                    self.cursor_col += 1;
                } else if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            if let Some(line) = self.lines.get(self.cursor_line) {
                self.cursor_col = self.cursor_col.min(line.len());
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            if let Some(line) = self.lines.get(self.cursor_line) {
                self.cursor_col = self.cursor_col.min(line.len());
            }
        }
    }

    pub fn home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn end(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_line) {
            self.cursor_col = line.len();
        }
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|l| l.is_empty())
    }
}

pub fn run_commit_editor(initial_message: Option<String>) -> Result<Option<String>> {
    enable_raw_mode().map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;
    let mut stdout = io::stdout();
    
    execute!(stdout, crossterm::cursor::Show)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    let mut state = CommitEditorState::new(initial_message);

    loop {
        terminal
            .draw(|f| ui(f, &state))
            .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

        if let Event::Key(key) = event::read()
            .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?
        {
            match key.code {
                KeyCode::Esc => {
                    disable_raw_mode().ok();
                    return Ok(None);
                }
                KeyCode::Char(c) => {
                    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        match c {
                            's' => break, // Ctrl+S to save
                            'c' => {
                                disable_raw_mode().ok();
                                return Ok(None);
                            }
                            _ => {}
                        }
                    } else {
                        state.insert_char(c);
                    }
                }
                KeyCode::Enter => {
                    state.new_line();
                }
                KeyCode::Backspace => {
                    state.backspace();
                }
                KeyCode::Delete => {
                    state.delete_char();
                }
                KeyCode::Left => {
                    state.move_cursor_left();
                }
                KeyCode::Right => {
                    state.move_cursor_right();
                }
                KeyCode::Up => {
                    state.move_cursor_up();
                }
                KeyCode::Down => {
                    state.move_cursor_down();
                }
                KeyCode::Home => {
                    state.home();
                }
                KeyCode::End => {
                    state.end();
                }
                KeyCode::Tab => {
                    state.insert_char('\t');
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;
    execute!(io::stdout(), crossterm::cursor::Hide)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    let content = state.get_content();
    if content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}

fn ui(f: &mut Frame, state: &CommitEditorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(15), Constraint::Length(8)].as_ref())
        .split(f.size());

    let editor_lines: Vec<Line> = state
        .lines
        .iter()
        .enumerate()
        .map(|(line_num, line)| {
            let mut spans = vec![];
            
            for (col, ch) in line.chars().enumerate() {
                let is_cursor = line_num == state.cursor_line && col == state.cursor_col;
                let style = if is_cursor {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                spans.push(Span::styled(ch.to_string(), style));
            }

            if line_num == state.cursor_line && state.cursor_col >= line.len() {
                spans.push(Span::styled(
                    " ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            let line_num_span = Span::styled(
                format!("{:3} │ ", line_num + 1),
                Style::default().fg(Color::DarkGray),
            );
            let mut final_spans = vec![line_num_span];
            final_spans.extend(spans);

            Line::from(final_spans)
        })
        .collect();

    let editor = Paragraph::new(editor_lines)
        .block(Block::default().title("Commit Message").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(editor, chunks[0]);

    let status = if state.dirty { "Modified" } else { "Clean" };
    let status_color = if state.dirty { Color::Yellow } else { Color::Green };

    let help_text = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(status, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Ctrl+S", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Save message  "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel  "),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]),
        Line::from(vec![
            Span::styled("↑↓←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Navigate  "),
            Span::styled("Home/End", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Line start/end"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" New line  "),
            Span::styled("Backspace", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Delete  "),
            Span::styled("Delete", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Del forward"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(help, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_editor_new() {
        let editor = CommitEditorState::new(None);
        assert_eq!(editor.lines.len(), 1);
        assert!(editor.is_empty());
    }

    #[test]
    fn test_commit_editor_insert_char() {
        let mut editor = CommitEditorState::new(None);
        editor.insert_char('a');
        editor.insert_char('b');
        assert_eq!(editor.get_content(), "ab");
    }

    #[test]
    fn test_commit_editor_new_line() {
        let mut editor = CommitEditorState::new(None);
        editor.insert_char('a');
        editor.new_line();
        editor.insert_char('b');
        assert_eq!(editor.get_content(), "a\nb");
    }

    #[test]
    fn test_commit_editor_backspace() {
        let mut editor = CommitEditorState::new(None);
        editor.insert_char('a');
        editor.insert_char('b');
        editor.backspace();
        assert_eq!(editor.get_content(), "a");
    }

    #[test]
    fn test_commit_editor_cursor_movement() {
        let mut editor = CommitEditorState::new(None);
        editor.insert_char('a');
        editor.insert_char('b');
        assert_eq!(editor.cursor_col, 2);
        
        editor.move_cursor_left();
        assert_eq!(editor.cursor_col, 1);
        
        editor.move_cursor_right();
        assert_eq!(editor.cursor_col, 2);
    }

    #[test]
    fn test_commit_editor_home_end() {
        let mut editor = CommitEditorState::new(None);
        editor.insert_char('a');
        editor.insert_char('b');
        editor.insert_char('c');
        
        editor.home();
        assert_eq!(editor.cursor_col, 0);
        
        editor.end();
        assert_eq!(editor.cursor_col, 3);
    }

    #[test]
    fn test_commit_editor_from_initial() {
        let editor = CommitEditorState::new(Some("Initial\nmessage".to_string()));
        assert_eq!(editor.lines.len(), 2);
        assert_eq!(editor.lines[0], "Initial");
        assert_eq!(editor.lines[1], "message");
    }
}
