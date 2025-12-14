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
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

#[derive(Debug, Clone)]
pub struct ConflictHunk {
    pub file_path: String,
    pub current_lines: Vec<String>,
    pub incoming_lines: Vec<String>,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkResolution {
    Current,
    Incoming,
    Both,
    Skip,
}

impl HunkResolution {
    pub fn to_string(&self) -> &'static str {
        match self {
            HunkResolution::Current => "Keep Current",
            HunkResolution::Incoming => "Accept Incoming",
            HunkResolution::Both => "Keep Both",
            HunkResolution::Skip => "Skip",
        }
    }

    pub fn next(&self) -> HunkResolution {
        match self {
            HunkResolution::Current => HunkResolution::Incoming,
            HunkResolution::Incoming => HunkResolution::Both,
            HunkResolution::Both => HunkResolution::Skip,
            HunkResolution::Skip => HunkResolution::Current,
        }
    }

    pub fn prev(&self) -> HunkResolution {
        match self {
            HunkResolution::Current => HunkResolution::Skip,
            HunkResolution::Incoming => HunkResolution::Current,
            HunkResolution::Both => HunkResolution::Incoming,
            HunkResolution::Skip => HunkResolution::Both,
        }
    }
}

pub struct MergeConflictState {
    hunks: Vec<(ConflictHunk, HunkResolution)>,
    current_hunk: usize,
    show_diff: bool,
}

impl MergeConflictState {
    pub fn new(hunks: Vec<ConflictHunk>) -> Self {
        let hunks_with_resolution = hunks
            .into_iter()
            .map(|h| (h, HunkResolution::Current))
            .collect();

        MergeConflictState {
            hunks: hunks_with_resolution,
            current_hunk: 0,
            show_diff: false,
        }
    }

    pub fn next_hunk(&mut self) {
        if self.current_hunk < self.hunks.len().saturating_sub(1) {
            self.current_hunk += 1;
        }
    }

    pub fn prev_hunk(&mut self) {
        if self.current_hunk > 0 {
            self.current_hunk -= 1;
        }
    }

    pub fn cycle_resolution(&mut self) {
        if let Some((_, resolution)) = self.hunks.get_mut(self.current_hunk) {
            *resolution = resolution.next();
        }
    }

    pub fn reverse_cycle_resolution(&mut self) {
        if let Some((_, resolution)) = self.hunks.get_mut(self.current_hunk) {
            *resolution = resolution.prev();
        }
    }

    pub fn toggle_diff(&mut self) {
        self.show_diff = !self.show_diff;
    }

    pub fn get_resolved_content(&self, hunk: &ConflictHunk, resolution: HunkResolution) -> Vec<String> {
        let mut result = hunk.context_before.clone();
        
        match resolution {
            HunkResolution::Current => {
                result.extend(hunk.current_lines.clone());
            }
            HunkResolution::Incoming => {
                result.extend(hunk.incoming_lines.clone());
            }
            HunkResolution::Both => {
                result.extend(hunk.current_lines.clone());
                result.extend(hunk.incoming_lines.clone());
            }
            HunkResolution::Skip => {
                result.push("<<<<<<< CURRENT".to_string());
                result.extend(hunk.current_lines.clone());
                result.push("=======".to_string());
                result.extend(hunk.incoming_lines.clone());
                result.push(">>>>>>> INCOMING".to_string());
            }
        }
        
        result.extend(hunk.context_after.clone());
        result
    }
}

pub fn run_merge_conflict_resolver(hunks: Vec<ConflictHunk>) -> Result<Vec<(ConflictHunk, HunkResolution)>> {
    enable_raw_mode().map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;
    let mut stdout = io::stdout();
    
    execute!(stdout, crossterm::cursor::Hide)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    let mut state = MergeConflictState::new(hunks);

    loop {
        terminal
            .draw(|f| ui(f, &state))
            .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

        if let Event::Key(key) = event::read()
            .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    break;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.prev_hunk();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.next_hunk();
                }
                KeyCode::Right | KeyCode::Tab => {
                    state.cycle_resolution();
                }
                KeyCode::Left | KeyCode::BackTab => {
                    state.reverse_cycle_resolution();
                }
                KeyCode::Char('c') => {
                    if let Some((_, res)) = state.hunks.get_mut(state.current_hunk) {
                        *res = HunkResolution::Current;
                    }
                }
                KeyCode::Char('i') => {
                    if let Some((_, res)) = state.hunks.get_mut(state.current_hunk) {
                        *res = HunkResolution::Incoming;
                    }
                }
                KeyCode::Char('b') => {
                    if let Some((_, res)) = state.hunks.get_mut(state.current_hunk) {
                        *res = HunkResolution::Both;
                    }
                }
                KeyCode::Char('s') => {
                    if let Some((_, res)) = state.hunks.get_mut(state.current_hunk) {
                        *res = HunkResolution::Skip;
                    }
                }
                KeyCode::Char('d') => {
                    state.toggle_diff();
                }
                KeyCode::Enter => {
                    break;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;
    execute!(io::stdout(), crossterm::cursor::Show)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    Ok(state.hunks)
}

fn ui(f: &mut Frame, state: &MergeConflictState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(20), Constraint::Length(10)].as_ref())
        .split(f.size());

    let hunks_list: Vec<ListItem> = state
        .hunks
        .iter()
        .enumerate()
        .map(|(i, (hunk, resolution))| {
            let is_selected = i == state.current_hunk;
            let resolution_str = resolution.to_string();
            let file_name = hunk.file_path.split('/').last().unwrap_or(&hunk.file_path);

            let content = format!(
                "[{}] {} | Current: {} lines, Incoming: {} lines | Resolution: {}",
                i + 1,
                file_name,
                hunk.current_lines.len(),
                hunk.incoming_lines.len(),
                resolution_str
            );

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                match resolution {
                    HunkResolution::Current => Style::default().fg(Color::Blue),
                    HunkResolution::Incoming => Style::default().fg(Color::Green),
                    HunkResolution::Both => Style::default().fg(Color::Yellow),
                    HunkResolution::Skip => Style::default().fg(Color::Red),
                }
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list = List::new(hunks_list)
        .block(Block::default().title("Merge Conflicts").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, chunks[0]);

    let help_text = vec![
        Line::from("Controls:"),
        Line::from(vec![
            Span::styled("j/↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Next conflict  "),
            Span::styled("k/↑", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Previous"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::raw(" Current  "),
            Span::styled("i", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Incoming  "),
            Span::styled("b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Both  "),
            Span::styled("s", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Skip"),
        ]),
        Line::from(vec![
            Span::styled("Tab/→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Next resolution  "),
            Span::styled("d", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(" Toggle diff"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Apply all  "),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
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
    fn test_hunk_resolution_cycle() {
        let mut res = HunkResolution::Current;
        res = res.next();
        assert_eq!(res, HunkResolution::Incoming);
        res = res.next();
        assert_eq!(res, HunkResolution::Both);
        res = res.next();
        assert_eq!(res, HunkResolution::Skip);
        res = res.next();
        assert_eq!(res, HunkResolution::Current);
    }

    #[test]
    fn test_conflict_state_navigation() {
        let hunks = vec![
            ConflictHunk {
                file_path: "file1.rs".to_string(),
                current_lines: vec!["current1".to_string()],
                incoming_lines: vec!["incoming1".to_string()],
                context_before: vec![],
                context_after: vec![],
            },
            ConflictHunk {
                file_path: "file2.rs".to_string(),
                current_lines: vec!["current2".to_string()],
                incoming_lines: vec!["incoming2".to_string()],
                context_before: vec![],
                context_after: vec![],
            },
        ];

        let mut state = MergeConflictState::new(hunks);
        assert_eq!(state.current_hunk, 0);

        state.next_hunk();
        assert_eq!(state.current_hunk, 1);

        state.prev_hunk();
        assert_eq!(state.current_hunk, 0);
    }

    #[test]
    fn test_get_resolved_content_current() {
        let hunk = ConflictHunk {
            file_path: "test.rs".to_string(),
            current_lines: vec!["current line".to_string()],
            incoming_lines: vec!["incoming line".to_string()],
            context_before: vec!["before".to_string()],
            context_after: vec!["after".to_string()],
        };

        let state = MergeConflictState::new(vec![hunk.clone()]);
        let resolved = state.get_resolved_content(&hunk, HunkResolution::Current);

        assert!(resolved.contains(&"before".to_string()));
        assert!(resolved.contains(&"current line".to_string()));
        assert!(!resolved.contains(&"incoming line".to_string()));
        assert!(resolved.contains(&"after".to_string()));
    }

    #[test]
    fn test_get_resolved_content_both() {
        let hunk = ConflictHunk {
            file_path: "test.rs".to_string(),
            current_lines: vec!["current".to_string()],
            incoming_lines: vec!["incoming".to_string()],
            context_before: vec![],
            context_after: vec![],
        };

        let state = MergeConflictState::new(vec![hunk.clone()]);
        let resolved = state.get_resolved_content(&hunk, HunkResolution::Both);

        assert!(resolved.contains(&"current".to_string()));
        assert!(resolved.contains(&"incoming".to_string()));
    }
}
