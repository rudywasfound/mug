/// Unicode output formatter for beautiful terminal output with colors
use colored::Colorize;
use std::fmt::Write;

pub struct UnicodeFormatter {
    pub use_unicode: bool,
    pub use_colors: bool,
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub is_head: bool,
    pub branch: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub file: String,
    pub added: usize,
    pub removed: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub enum DiffLine {
    Added(String),
    Removed(String),
    Context(String),
}

impl UnicodeFormatter {
    pub fn new(use_unicode: bool, use_colors: bool) -> Self {
        UnicodeFormatter {
            use_unicode,
            use_colors,
        }
    }

    fn colorize(&self, text: &str, color: &str) -> String {
        if self.use_colors {
            match color {
                "green" => text.green().to_string(),
                "red" => text.red().to_string(),
                "yellow" => text.yellow().to_string(),
                "blue" => text.blue().to_string(),
                "cyan" => text.cyan().to_string(),
                "magenta" => text.magenta().to_string(),
                "white" => text.white().to_string(),
                "bright_green" => text.bright_green().to_string(),
                "bright_yellow" => text.bright_yellow().to_string(),
                "bright_cyan" => text.bright_cyan().to_string(),
                _ => text.to_string(),
            }
        } else {
            text.to_string()
        }
    }

    pub fn format_log(&self, commits: &[CommitInfo]) -> String {
        let mut output = String::new();

        // Header
        let header = if self.use_unicode { "━".repeat(70) } else { "-".repeat(70) };
        writeln!(&mut output, "{}", self.colorize(&header, "cyan")).unwrap();
        writeln!(
            &mut output,
            "{}",
            self.colorize("Commit History", "bright_cyan").bold().to_string()
        )
        .unwrap();
        writeln!(&mut output, "{}", self.colorize(&header, "cyan")).unwrap();

        for (i, commit) in commits.iter().enumerate() {
            let is_last = i == commits.len() - 1;

            // Commit symbol with line connector
            let symbol = if commit.is_head {
                if self.use_unicode {
                    self.colorize("◆", "bright_yellow")
                } else {
                    self.colorize("*", "bright_yellow")
                }
            } else {
                if self.use_unicode {
                    self.colorize("◉", "cyan")
                } else {
                    self.colorize("o", "cyan")
                }
            };

            let branch_info = if let Some(ref branch) = commit.branch {
                format!(" {}", self.colorize(&format!("[{}]", branch), "green"))
            } else {
                String::new()
            };

            // Main commit line
            let short_hash = &commit.hash[..8.min(commit.hash.len())];
            let hash_colored = self.colorize(short_hash, "yellow");
            let message_colored = self.colorize(&commit.message, "white").bold().to_string();

            writeln!(
                &mut output,
                "{} {} {}{}",
                symbol, hash_colored, message_colored, branch_info
            )
            .unwrap();

            // Author and date lines
            let pipe = if self.use_unicode {
                self.colorize("│", "cyan")
            } else {
                self.colorize("|", "cyan")
            };

            let author_label = self.colorize("Author:", "bright_cyan");
            let author_value = self.colorize(&commit.author, "white");
            writeln!(&mut output, "{}  {} {}", pipe, author_label, author_value).unwrap();

            let date_label = self.colorize("Date:", "bright_cyan");
            let date_value = self.colorize(&commit.date, "white");
            writeln!(&mut output, "{}  {} {}", pipe, date_label, date_value).unwrap();

            // Separator
            if !is_last {
                writeln!(&mut output, "{}", pipe).unwrap();
            } else {
                let tilde = if self.use_unicode { "┴" } else { "=" };
                writeln!(&mut output, "{}", self.colorize(tilde, "cyan")).unwrap();
            }

            if i < commits.len() - 1 {
                writeln!(&mut output).unwrap();
            }
        }

        output
    }

    pub fn format_status(&self, branch: &str, changes: &[(String, char)]) -> String {
        let mut output = String::new();

        let width = 70;
        let corner_tl = if self.use_unicode { "╭" } else { "+" };
        let corner_tr = if self.use_unicode { "╮" } else { "+" };
        let corner_bl = if self.use_unicode { "╰" } else { "+" };
        let corner_br = if self.use_unicode { "╯" } else { "+" };
        let h_line = if self.use_unicode { "─" } else { "-" };
        let v_line = if self.use_unicode { "│" } else { "|" };

        let border = format!(
            "{}{}{}",
            self.colorize(corner_tl, "cyan"),
            self.colorize(&h_line.repeat(width - 2), "cyan"),
            self.colorize(corner_tr, "cyan")
        );

        writeln!(&mut output, "{}", border).unwrap();

        // Branch info
        let branch_icon = if self.use_unicode { "🌿" } else { "*" };
        let branch_label = self.colorize("On branch:", "bright_cyan");
        let branch_value = self.colorize(branch, "bright_green").bold().to_string();
        writeln!(
            &mut output,
            "{} {} {} {}",
            self.colorize(v_line, "cyan"),
            branch_icon,
            branch_label,
            branch_value
        )
        .unwrap();

        // Changes section
        if !changes.is_empty() {
            writeln!(&mut output, "{}", self.colorize(v_line, "cyan")).unwrap();

            let changes_icon = if self.use_unicode { "📝" } else { "*" };
            let changes_label = self.colorize("Changes:", "bright_cyan");
            writeln!(
                &mut output,
                "{} {} {}",
                self.colorize(v_line, "cyan"),
                changes_icon,
                changes_label
            )
            .unwrap();

            for (path, kind) in changes {
                let icon = match kind {
                    'M' => self.colorize("✏️ ", "yellow"),
                    'A' => self.colorize("➕ ", "bright_green"),
                    'D' => self.colorize("🗑 ", "red"),
                    'R' => self.colorize("↻", "magenta"),
                    _ => self.colorize("?", "white"),
                };

                let file_colored = match kind {
                    'M' => self.colorize(path, "yellow"),
                    'A' => self.colorize(path, "bright_green"),
                    'D' => self.colorize(path, "red"),
                    _ => self.colorize(path, "white"),
                };

                writeln!(
                    &mut output,
                    "{}   {} {}",
                    self.colorize(v_line, "cyan"),
                    icon,
                    file_colored
                )
                .unwrap();
            }
        } else {
            writeln!(&mut output, "{}", self.colorize(v_line, "cyan")).unwrap();
            let clean = self.colorize("nothing to commit, working tree clean", "bright_green");
            writeln!(
                &mut output,
                "{} {}",
                self.colorize(v_line, "cyan"),
                clean
            )
            .unwrap();
        }

        // Bottom border
        let border = format!(
            "{}{}{}",
            self.colorize(corner_bl, "cyan"),
            self.colorize(&h_line.repeat(width - 2), "cyan"),
            self.colorize(corner_br, "cyan")
        );

        writeln!(&mut output, "{}", border).unwrap();

        output
    }

    pub fn format_branch_list(&self, current: &str, branches: &[String]) -> String {
        let mut output = String::new();

        // Header
        let header = if self.use_unicode { "━".repeat(50) } else { "-".repeat(50) };
        writeln!(&mut output, "{}", self.colorize(&header, "cyan")).unwrap();
        writeln!(
            &mut output,
            "{}",
            self.colorize("Branches", "bright_cyan").bold().to_string()
        )
        .unwrap();
        writeln!(&mut output, "{}", self.colorize(&header, "cyan")).unwrap();

        for branch in branches {
            let is_current = branch == current;

            let symbol = if is_current {
                self.colorize("●", "bright_green")
            } else {
                self.colorize("○", "bright_cyan")
            };

            let branch_name = if is_current {
                self.colorize(branch, "bright_green").bold().to_string()
            } else {
                self.colorize(branch, "white").to_string()
            };

            let indicator = if is_current {
                self.colorize("(current)", "bright_green")
            } else {
                String::new()
            };

            if indicator.is_empty() {
                writeln!(&mut output, "{} {}", symbol, branch_name).unwrap();
            } else {
                writeln!(&mut output, "{} {} {}", symbol, branch_name, indicator).unwrap();
            }
        }

        writeln!(
            &mut output,
            "{}",
            self.colorize(&header, "cyan")
        )
        .unwrap();

        output
    }

    pub fn format_progress_bar(&self, current: u64, total: u64) -> String {
        let percent = if total > 0 {
            (current as f64 / total as f64 * 100.0) as u64
        } else {
            0
        };

        let bar_width = 30;
        let filled = (percent as usize * bar_width) / 100;
        let empty = bar_width - filled;

        let filled_bar = if self.use_unicode {
            "█".repeat(filled)
        } else {
            "=".repeat(filled)
        };

        let empty_bar = if self.use_unicode {
            "░".repeat(empty)
        } else {
            " ".repeat(empty)
        };

        let filled_colored = self.colorize(&filled_bar, "bright_green");
        let empty_colored = self.colorize(&empty_bar, "cyan");
        let percent_str = self.colorize(&format!("{}%", percent), "bright_yellow");

        format!("{}{}  {}", filled_colored, empty_colored, percent_str)
    }

    pub fn format_diff(&self, hunks: &[DiffHunk]) -> String {
        let mut output = String::new();

        for hunk in hunks {
            // File header
            writeln!(
                &mut output,
                "{}",
                self.colorize(&format!("diff --git a/{} b/{}", &hunk.file, &hunk.file), "bright_cyan")
            )
            .unwrap();

            writeln!(
                &mut output,
                "{}",
                self.colorize(&format!("--- a/{}", &hunk.file), "red")
            )
            .unwrap();

            writeln!(
                &mut output,
                "{}",
                self.colorize(&format!("+++ b/{}", &hunk.file), "bright_green")
            )
            .unwrap();

            // Stats
            let stats = format!(
                "@@ {} (+{} -{}) @@",
                &hunk.file, hunk.added, hunk.removed
            );
            writeln!(
                &mut output,
                "{}",
                self.colorize(&stats, "bright_cyan")
            )
            .unwrap();

            // Diff lines
            for line in &hunk.lines {
                match line {
                    DiffLine::Added(text) => {
                        writeln!(
                            &mut output,
                            "{}",
                            self.colorize(&format!("+{}", text), "bright_green")
                        )
                        .unwrap();
                    }
                    DiffLine::Removed(text) => {
                        writeln!(
                            &mut output,
                            "{}",
                            self.colorize(&format!("-{}", text), "red")
                        )
                        .unwrap();
                    }
                    DiffLine::Context(text) => {
                        writeln!(
                            &mut output,
                            "{}",
                            self.colorize(&format!(" {}", text), "white")
                        )
                        .unwrap();
                    }
                }
            }

            writeln!(&mut output).unwrap();
        }

        output
    }

    pub fn format_merge_conflict(&self, file: &str, ours: &str, theirs: &str) -> String {
        let mut output = String::new();

        let header = if self.use_unicode { "━".repeat(60) } else { "-".repeat(60) };
        writeln!(
            &mut output,
            "{}",
            self.colorize(&header, "red")
        )
        .unwrap();

        writeln!(
            &mut output,
            "{}",
            self.colorize(
                &format!("⚠️  Merge Conflict in {}", file),
                "bright_yellow"
            )
        )
        .unwrap();

        writeln!(
            &mut output,
            "{}",
            self.colorize(&header, "red")
        )
        .unwrap();

        writeln!(&mut output, "{}", self.colorize("<<<<<<< HEAD (ours)", "red")).unwrap();
        writeln!(&mut output, "{}", self.colorize(ours, "cyan")).unwrap();
        writeln!(&mut output, "{}", self.colorize("=======", "yellow")).unwrap();
        writeln!(&mut output, "{}", self.colorize(theirs, "magenta")).unwrap();
        writeln!(&mut output, "{}", self.colorize(">>>>>>> (theirs)", "red")).unwrap();

        output
    }

    pub fn format_error(&self, error: &str) -> String {
        let cross = if self.use_unicode { "✘" } else { "x" };
        let error_icon = self.colorize(&format!("{} error:", cross), "red");
        format!("{} {}", error_icon, self.colorize(error, "white"))
    }

    pub fn format_success(&self, message: &str) -> String {
        let check = if self.use_unicode { "✓" } else { ">" };
        let success_icon = self.colorize(&format!("{} success:", check), "bright_green");
        format!("{} {}", success_icon, self.colorize(message, "white"))
    }

    pub fn format_warning(&self, message: &str) -> String {
        let warning = if self.use_unicode { "⚠" } else { "!" };
        let warning_icon = self.colorize(&format!("{} warning:", warning), "bright_yellow");
        format!("{} {}", warning_icon, self.colorize(message, "white"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_log() {
        let formatter = UnicodeFormatter::new(true, true);
        let commits = vec![
            CommitInfo {
                hash: "abc1234567890".to_string(),
                author: "Test Author".to_string(),
                date: "2025-12-29".to_string(),
                message: "Update docs".to_string(),
                is_head: true,
                branch: Some("main".to_string()),
            },
            CommitInfo {
                hash: "def5678901234".to_string(),
                author: "Another Author".to_string(),
                date: "2025-12-28".to_string(),
                message: "Add feature".to_string(),
                is_head: false,
                branch: None,
            },
        ];

        let output = formatter.format_log(&commits);
        assert!(output.contains("Update docs"));
        assert!(output.contains("Add feature"));
    }

    #[test]
    fn test_format_progress() {
        let formatter = UnicodeFormatter::new(true, true);
        let bar = formatter.format_progress_bar(50, 100);
        assert!(bar.contains("50%"));
    }

    #[test]
    fn test_format_status() {
        let formatter = UnicodeFormatter::new(true, true);
        let changes = vec![
            ("src/main.rs".to_string(), 'M'),
            ("docs/README.md".to_string(), 'A'),
        ];
        let output = formatter.format_status("main", &changes);
        assert!(output.contains("On branch"));
        assert!(output.contains("Changes"));
    }

    #[test]
    fn test_ascii_fallback() {
        let formatter = UnicodeFormatter::new(false, false);
        let commits = vec![CommitInfo {
            hash: "abc1234".to_string(),
            author: "Author".to_string(),
            date: "2025-12-29".to_string(),
            message: "Message".to_string(),
            is_head: true,
            branch: None,
        }];

        let output = formatter.format_log(&commits);
        assert!(output.contains("Message"));
    }

    #[test]
    fn test_format_error() {
        let formatter = UnicodeFormatter::new(true, true);
        let error = formatter.format_error("File not found");
        assert!(error.contains("error"));
        assert!(error.contains("File not found"));
    }

    #[test]
    fn test_format_success() {
        let formatter = UnicodeFormatter::new(true, true);
        let success = formatter.format_success("Changes committed");
        assert!(success.contains("success"));
        assert!(success.contains("Changes committed"));
    }
}
