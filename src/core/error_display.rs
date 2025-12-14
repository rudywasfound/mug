use crate::core::error::Error;

pub mod colors {
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

pub fn display_error(error: &Error) {
    let message = match error {
        Error::Io(e) => {
            format!(
                "{}{}IO Error{}: {}",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                e
            )
        }
        Error::Database(msg) => {
            format!(
                "{}{}Database Error{}: {}\n{}Tip:{} Check if .mug directory exists",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                msg,
                colors::CYAN,
                colors::RESET
            )
        }
        Error::NotARepository => {
            format!(
                "{}{}Error:{} Not a mug repository\n{}Tip:{} Run `mug init` to create one",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                colors::CYAN,
                colors::RESET
            )
        }
        Error::NoCommits => {
            format!(
                "{}{}Error:{} No commits yet\n{}Tip:{} Add files and run `mug commit`",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                colors::CYAN,
                colors::RESET
            )
        }
        Error::BranchNotFound(branch) => {
            format!(
                "{}{}Error:{} Branch '{}' not found\n{}Tip:{} Use `mug branches` to list available branches",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                colors::YELLOW,
                colors::CYAN,
                colors::RESET
            )
        }
        Error::CommitNotFound(hash) => {
            format!(
                "{}{}Error:{} Commit '{}' not found\n{}Tip:{} Use `mug log` to see commit history",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                colors::YELLOW,
                colors::CYAN,
                colors::RESET
            )
        }
        Error::ObjectNotFound(hash) => {
            format!(
                "{}{}Error:{} Object '{}' not found",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                colors::YELLOW
            )
        }
        Error::Serialization(e) => {
            format!(
                "{}{}Serialization Error{}: {}\n{}Tip:{} This is likely a bug. Try running `mug gc`",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                e,
                colors::CYAN,
                colors::RESET
            )
        }
        Error::Conflicts => {
            format!(
                "{}{}Conflict:{} Working directory has unresolved conflicts\n{}Tip:{} Use `mug merge` with the TUI resolver",
                colors::MAGENTA,
                colors::BOLD,
                colors::RESET,
                colors::CYAN,
                colors::RESET
            )
        }
        Error::Utf8Error(e) => {
            format!(
                "{}{}Error:{} Invalid UTF8: {}",
                colors::RED,
                colors::BOLD,
                colors::RESET,
                e
            )
        }
        Error::Custom(msg) => {
            if msg.contains("Remote") && msg.contains("not found") {
                let remote = msg
                    .split('\'')
                    .nth(1)
                    .unwrap_or("unknown");
                format!(
                    "{}{}Error:{} Remote '{}' not found\n{}Tip:{} Use `mug remote list` to see remotes, or `mug remote add {} <url>`",
                    colors::RED,
                    colors::BOLD,
                    colors::RESET,
                    colors::YELLOW,
                    colors::CYAN,
                    colors::RESET,
                    remote
                )
            } else if msg.contains("already exists") {
                format!(
                    "{}{}Error:{} {}\n{}Tip:{} Choose a different name or remove the existing one",
                    colors::RED,
                    colors::BOLD,
                    colors::RESET,
                    msg,
                    colors::CYAN,
                    colors::RESET
                )
            } else if msg.contains("permission denied") {
                format!(
                    "{}{}Error:{} {}\n{}Tip:{} Check file permissions",
                    colors::RED,
                    colors::BOLD,
                    colors::RESET,
                    msg,
                    colors::CYAN,
                    colors::RESET
                )
            } else if msg.contains("Connection") || msg.contains("timeout") {
                format!(
                    "{}{}Error:{} {}\n{}Tip:{} Check network connection and remote URL",
                    colors::RED,
                    colors::BOLD,
                    colors::RESET,
                    msg,
                    colors::CYAN,
                    colors::RESET
                )
            } else {
                format!("{}{}{}{}", colors::RED, colors::BOLD, msg, colors::RESET)
            }
        }
    };

    eprintln!("{}", message);
}

pub fn display_success(message: &str) {
    println!("{}✓{} {}", colors::GREEN, colors::RESET, message);
}

pub fn display_warning(message: &str) {
    eprintln!(
        "{}{}⚠ Warning:{} {}",
        colors::YELLOW,
        colors::BOLD,
        colors::RESET,
        message
    );
}

pub fn display_info(message: &str) {
    println!("{}ℹ{} {}", colors::BLUE, colors::RESET, message);
}

pub fn display_section(title: &str) {
    println!(
        "\n{}{}{}\n{}",
        colors::CYAN,
        colors::BOLD,
        title,
        colors::RESET
    );
}

pub fn format_file_path(path: &str) -> String {
    format!("{}{}{}", colors::YELLOW, path, colors::RESET)
}

pub fn format_hash(hash: &str) -> String {
    format!("{}{}{}", colors::CYAN, hash, colors::RESET)
}

pub fn format_branch(branch: &str) -> String {
    format!("{}{}{}", colors::GREEN, branch, colors::RESET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_codes_not_empty() {
        assert!(!colors::RED.is_empty());
        assert!(!colors::GREEN.is_empty());
        assert!(!colors::RESET.is_empty());
    }

    #[test]
    fn test_format_functions() {
        let path = format_file_path("src/main.rs");
        assert!(path.contains("src/main.rs"));
        
        let hash = format_hash("abc123");
        assert!(hash.contains("abc123"));
        
        let branch = format_branch("main");
        assert!(branch.contains("main"));
    }
}
