use regex::Regex;
use std::fs;
use std::path::Path;

use crate::core::error::Result;

/// Manages .mugignore patterns for excluding files from version control
#[derive(Debug, Clone)]
pub struct IgnoreRules {
    patterns: Vec<IgnorePattern>,
}

#[derive(Debug, Clone)]
struct IgnorePattern {
    #[allow(dead_code)]
    pattern: String,
    regex: Regex,
    negated: bool, // ! prefix means include
}

impl IgnoreRules {
    /// Creates a new empty ignore rules set
    pub fn new() -> Self {
        IgnoreRules {
            patterns: Vec::new(),
        }
    }

    /// Loads ignore rules from a .mugignore file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut rules = IgnoreRules::new();

        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();

                // Skip empty lines and comments
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                rules.add_pattern(trimmed)?;
            }
        }

        Ok(rules)
    }

    /// Loads rules from project root .mugignore
    pub fn load_from_repo(repo_root: &Path) -> Result<Self> {
        let mugignore_path = repo_root.join(".mugignore");
        Self::from_file(mugignore_path)
    }

    /// Adds a pattern to the rules
    pub fn add_pattern(&mut self, pattern: &str) -> Result<()> {
        let negated = pattern.starts_with('!');
        let pattern_str = if negated { &pattern[1..] } else { pattern };

        let regex = self.pattern_to_regex(pattern_str)?;

        self.patterns.push(IgnorePattern {
            pattern: pattern_str.to_string(),
            regex,
            negated,
        });

        Ok(())
    }

    /// Converts .mugignore pattern to regex
    /// Supports: *.ext, dir/, exact paths, ** for recursive
    fn pattern_to_regex(&self, pattern: &str) -> Result<Regex> {
        if pattern.is_empty() {
            return Err(crate::core::error::Error::Custom("Empty pattern".to_string()));
        }

        // Convert glob to regex
        let pattern = pattern.trim_end_matches('/');

        let regex_pattern = if pattern == "**" {
            ".*".to_string()
        } else if pattern.starts_with("**/") {
            // Match any depth
            format!("(^|.*/){}$", regex::escape(&pattern[3..]))
        } else if pattern.ends_with("/**") {
            // Match directory and everything in it
            format!("^{}(/.*)?$", regex::escape(&pattern[..pattern.len() - 3]))
        } else if pattern.contains('*') {
            // Simple glob conversion
            let escaped = regex::escape(pattern)
                .replace("\\*", ".*")
                .replace("\\?", ".");
            format!("^{}$", escaped)
        } else {
            // Exact match or directory prefix
            format!("^{}(/.*)?$", regex::escape(pattern))
        };

        Regex::new(&regex_pattern)
            .map_err(|e| crate::core::error::Error::Custom(format!("Invalid regex pattern: {}", e)))
    }

    /// Checks if a path should be ignored
    pub fn should_ignore(&self, path: &str) -> bool {
        let mut ignored = false;

        for pattern in &self.patterns {
            if pattern.regex.is_match(path) {
                ignored = !pattern.negated; // negated patterns re-include
            }
        }

        ignored
    }

    /// Creates default .mugignore content
    pub fn default_content() -> &'static str {
        "# MUG ignore file - patterns for files to exclude from version control
# See https://mug.local/docs/mugignore

# Dependencies and packages
node_modules/
target/
venv/
__pycache__/
*.pyc
*.pyo
*.pyd
.Python

# IDE and editor files
.vscode/
.idea/
*.swp
*.swo
*~
.DS_Store
Thumbs.db

# Build artifacts
*.o
*.so
*.dylib
*.dll
*.exe
*.out
build/
dist/

# Environment variables
.env
.env.local
.env.*.local

# Logs
*.log
logs/

# Temporary files
tmp/
temp/
.tmp/
*.tmp

# OS files
.DS_Store
Thumbs.db
.AppleDouble
.LSOverride

# Git-related (if migrating)
.git/
.gitignore

# Test coverage
coverage/
.coverage
htmlcov/

# Package managers
package-lock.json
yarn.lock
Gemfile.lock
poetry.lock
"
    }

    /// Returns the number of patterns loaded
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

impl Default for IgnoreRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_glob_patterns() {
        let mut rules = IgnoreRules::new();
        rules.add_pattern("*.log").unwrap();
        rules.add_pattern("target/").unwrap();
        rules.add_pattern("node_modules/").unwrap();

        assert!(rules.should_ignore("debug.log"));
        assert!(rules.should_ignore("app.log"));
        assert!(rules.should_ignore("target/debug"));
        assert!(rules.should_ignore("node_modules/express"));
        assert!(!rules.should_ignore("main.rs"));
    }

    #[test]
    fn test_ignore_negated_patterns() {
        let mut rules = IgnoreRules::new();
        rules.add_pattern("*.log").unwrap();
        rules.add_pattern("!important.log").unwrap();

        assert!(rules.should_ignore("debug.log"));
        assert!(!rules.should_ignore("important.log"));
    }

    #[test]
    fn test_ignore_directory_recursive() {
        let mut rules = IgnoreRules::new();
        rules.add_pattern("target/**").unwrap();

        assert!(rules.should_ignore("target/debug"));
        assert!(rules.should_ignore("target/release"));
        assert!(!rules.should_ignore("src/main.rs"));
    }

    #[test]
    fn test_ignore_nested_glob() {
        let mut rules = IgnoreRules::new();
        rules.add_pattern("**/node_modules").unwrap();

        assert!(rules.should_ignore("node_modules"));
        assert!(rules.should_ignore("src/node_modules"));
        assert!(rules.should_ignore("deeply/nested/node_modules"));
    }

    #[test]
    fn test_default_content_not_empty() {
        let content = IgnoreRules::default_content();
        assert!(!content.is_empty());
        assert!(content.contains("node_modules/"));
        assert!(content.contains("target/"));
    }
}
