/// Interactive branch selector - inline with shell visible
use colored::Colorize;
use std::io::{self, Write};

pub struct BranchSelector {
    branches: Vec<String>,
    current: String,
}

impl BranchSelector {
    pub fn new(branches: Vec<String>, current: String) -> Self {
        BranchSelector {
            branches,
            current,
        }
    }

    pub fn display_with_numbers(&self) {
        println!();
        println!("{}", "Select a branch:".bright_cyan().bold());
        
        for (idx, branch) in self.branches.iter().enumerate() {
            let number = (idx + 1).to_string().bright_yellow().bold();
            
            if branch == &self.current {
                println!("  {} {} {} {}", 
                    number,
                    "●".bright_green(),
                    branch.bright_green().bold(),
                    "(current)".bright_green().italic()
                );
            } else {
                println!("  {} {} {}",
                    number,
                    "○".cyan(),
                    branch.white()
                );
            }
        }
        println!();
    }

    pub fn prompt_user(&self) -> Option<String> {
        // Display branches with numbers
        self.display_with_numbers();
        
        // Prompt user
        print!("{} ", "Enter branch number or name (or press Enter to skip):".bright_cyan());
        io::stdout().flush().ok()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok()?;
        
        let input = input.trim();
        
        if input.is_empty() {
            return None;
        }
        
        // Try parsing as number
        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= self.branches.len() {
                return Some(self.branches[num - 1].clone());
            } else {
                println!("{}", "Invalid number!".red());
                return None;
            }
        }
        
        // Try matching by name
        if let Some(matched) = self.branches.iter().find(|b| b.contains(input) || input.contains(b.as_str())) {
            return Some(matched.clone());
        }
        
        println!("{}", "Branch not found!".red());
        None
    }
}

/// Simple interactive branch selector with inline display
pub fn select_branch_interactive(branches: Vec<String>, current: String) -> Option<String> {
    if branches.is_empty() {
        return None;
    }

    let selector = BranchSelector::new(branches, current);
    selector.prompt_user()
}
