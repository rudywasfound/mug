use crate::core::error::Result;
use crate::core::rebase::RebaseCommit;
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

/// Action for each commit in interactive rebase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseAction {
    Pick,
    Squash,
    Reword,
    Drop,
}

impl RebaseAction {
    pub fn to_string(&self) -> &'static str {
        match self {
            RebaseAction::Pick => "pick",
            RebaseAction::Squash => "squash",
            RebaseAction::Reword => "reword",
            RebaseAction::Drop => "drop",
        }
    }

    pub fn next(&self) -> RebaseAction {
        match self {
            RebaseAction::Pick => RebaseAction::Squash,
            RebaseAction::Squash => RebaseAction::Reword,
            RebaseAction::Reword => RebaseAction::Drop,
            RebaseAction::Drop => RebaseAction::Pick,
        }
    }

    pub fn prev(&self) -> RebaseAction {
        match self {
            RebaseAction::Pick => RebaseAction::Drop,
            RebaseAction::Squash => RebaseAction::Pick,
            RebaseAction::Reword => RebaseAction::Squash,
            RebaseAction::Drop => RebaseAction::Reword,
        }
    }
}

/// Interactive rebase state
pub struct RebaseState {
    pub commits: Vec<(RebaseCommit, RebaseAction)>,
    pub selected: usize,
}

impl RebaseState {
    pub fn new(commits: Vec<RebaseCommit>) -> Self {
        let commits_with_actions = commits
            .into_iter()
            .map(|c| (c, RebaseAction::Pick))
            .collect();

        RebaseState {
            commits: commits_with_actions,
            selected: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.selected < self.commits.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn cycle_action(&mut self) {
        if let Some((_, action)) = self.commits.get_mut(self.selected) {
            *action = action.next();
        }
    }

    pub fn reverse_cycle_action(&mut self) {
        if let Some((_, action)) = self.commits.get_mut(self.selected) {
            *action = action.prev();
        }
    }
}

/// Run interactive rebase TUI
pub fn run_interactive_rebase(commits: Vec<RebaseCommit>) -> Result<Vec<(RebaseCommit, RebaseAction)>> {
    // Setup terminal
    enable_raw_mode().map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;
    let mut stdout = io::stdout();
    
    // Alternative approach without EnterAltScreen/LeaveAltScreen
    execute!(stdout, crossterm::cursor::Hide)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    let mut state = RebaseState::new(commits);

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
                    state.select_prev();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.select_next();
                }
                KeyCode::Char('p') => {
                    state.commits[state.selected].1 = RebaseAction::Pick;
                }
                KeyCode::Char('s') => {
                    state.commits[state.selected].1 = RebaseAction::Squash;
                }
                KeyCode::Char('r') => {
                    state.commits[state.selected].1 = RebaseAction::Reword;
                }
                KeyCode::Char('d') => {
                    state.commits[state.selected].1 = RebaseAction::Drop;
                }
                KeyCode::Tab | KeyCode::Right => {
                    state.cycle_action();
                }
                KeyCode::BackTab | KeyCode::Left => {
                    state.reverse_cycle_action();
                }
                KeyCode::Enter => {
                    break;
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode().map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;
    execute!(io::stdout(), crossterm::cursor::Show)
        .map_err(|e| crate::core::error::Error::Custom(e.to_string()))?;

    Ok(state.commits)
}

fn ui(f: &mut Frame, state: &RebaseState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(15), Constraint::Length(7)].as_ref())
        .split(f.size());

    // Commits list
    let commits_list: Vec<ListItem> = state
        .commits
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.selected;
            let (commit, action) = item;
            let action_str = action.to_string();
            let hash = commit.hash[..8.min(commit.hash.len())].to_string();

            let content = format!("{} {} {}", action_str, hash, commit.message);

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                match action {
                    RebaseAction::Pick => Style::default().fg(Color::Green),
                    RebaseAction::Squash => Style::default().fg(Color::Yellow),
                    RebaseAction::Reword => Style::default().fg(Color::Cyan),
                    RebaseAction::Drop => Style::default().fg(Color::Red),
                }
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list = List::new(commits_list)
        .block(Block::default().title("Interactive Rebase").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, chunks[0]);

    // Help text
    let help_text = vec![
        Line::from("Controls:"),
        Line::from(vec![
            Span::styled("j/↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Next  "),
            Span::styled("k/↑", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Prev  "),
            Span::styled("→/Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Cycle action"),
        ]),
        Line::from(vec![
            Span::styled("p", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" pick  "),
            Span::styled("s", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" squash  "),
            Span::styled("r", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" reword  "),
            Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" drop"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Execute  "),
            Span::styled("q/Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
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
    fn test_rebase_action_cycle() {
        let mut action = RebaseAction::Pick;
        action = action.next();
        assert_eq!(action, RebaseAction::Squash);
        action = action.next();
        assert_eq!(action, RebaseAction::Reword);
        action = action.next();
        assert_eq!(action, RebaseAction::Drop);
        action = action.next();
        assert_eq!(action, RebaseAction::Pick);
    }

    #[test]
    fn test_rebase_action_prev() {
        let action = RebaseAction::Pick;
        assert_eq!(action.prev(), RebaseAction::Drop);
        assert_eq!(action.prev().prev(), RebaseAction::Reword);
    }

    #[test]
    fn test_rebase_state_navigation() {
        let commits = vec![
            RebaseCommit {
                hash: "abc123".to_string(),
                message: "First".to_string(),
                author: "Alice".to_string(),
            },
            RebaseCommit {
                hash: "def456".to_string(),
                message: "Second".to_string(),
                author: "Bob".to_string(),
            },
        ];

        let mut state = RebaseState::new(commits);
        assert_eq!(state.selected, 0);

        state.select_next();
        assert_eq!(state.selected, 1);

        state.select_next();
        assert_eq!(state.selected, 1); // Stay at end

        state.select_prev();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_rebase_state_action_cycling() {
        let commits = vec![RebaseCommit {
            hash: "abc123".to_string(),
            message: "Test".to_string(),
            author: "Alice".to_string(),
        }];

        let mut state = RebaseState::new(commits);
        assert_eq!(state.commits[0].1, RebaseAction::Pick);

        state.cycle_action();
        assert_eq!(state.commits[0].1, RebaseAction::Squash);

        state.cycle_action();
        assert_eq!(state.commits[0].1, RebaseAction::Reword);

        state.reverse_cycle_action();
        assert_eq!(state.commits[0].1, RebaseAction::Squash);
    }
}
