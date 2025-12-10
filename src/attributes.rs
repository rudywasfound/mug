use std::fs;
use std::path::Path;

use crate::error::Result;

/// File attributes manager (.mugattributes)
/// Manages special handling for files (line endings, merging strategies, etc.)
#[derive(Debug, Clone)]
pub struct Attributes {
    patterns: Vec<AttributeRule>,
}

#[derive(Debug, Clone)]
struct AttributeRule {
    pattern: String,
    attributes: FileAttributes,
}

/// Attributes applied to files matching patterns
#[derive(Debug, Clone, Default)]
pub struct FileAttributes {
    /// Line ending handling: auto, lf, crlf, binary
    pub line_ending: Option<String>,
    /// Merge strategy: ours, theirs, union, binary
    pub merge: Option<String>,
    /// Diff algorithm: binary, text, auto
    pub diff: Option<String>,
    /// Export ignore: whether file should be excluded from exports
    pub export_ignore: bool,
}

impl Attributes {
    /// Creates a new empty attributes set
    pub fn new() -> Self {
        Attributes {
            patterns: Vec::new(),
        }
    }

    /// Loads attributes from .mugattributes file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut attrs = Attributes::new();

        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();

                // Skip comments and empty lines
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                attrs.parse_line(trimmed)?;
            }
        }

        Ok(attrs)
    }

    /// Loads attributes from project root .mugattributes
    pub fn load_from_repo(repo_root: &Path) -> Result<Self> {
        let attrs_path = repo_root.join(".mugattributes");
        Self::load_from_file(attrs_path)
    }

    /// Parses a single attribute line
    fn parse_line(&mut self, line: &str) -> Result<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(());
        }

        let pattern = parts[0].to_string();
        let mut attributes = FileAttributes::default();

        for part in &parts[1..] {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "line_ending" => attributes.line_ending = Some(value.to_string()),
                    "merge" => attributes.merge = Some(value.to_string()),
                    "diff" => attributes.diff = Some(value.to_string()),
                    _ => {}
                }
            } else if part.starts_with('-') {
                // Unset attribute
                let attr_name = &part[1..];
                match attr_name {
                    "line_ending" => attributes.line_ending = None,
                    "merge" => attributes.merge = None,
                    "diff" => attributes.diff = None,
                    "export_ignore" | "export-ignore" => attributes.export_ignore = false,
                    _ => {}
                }
            } else if *part == "export-ignore" || *part == "export_ignore" {
                attributes.export_ignore = true;
            }
        }

        self.patterns.push(AttributeRule { pattern, attributes });

        Ok(())
    }

    /// Gets attributes for a file path
    pub fn get_attributes(&self, path: &str) -> FileAttributes {
        let mut result = FileAttributes::default();

        for rule in &self.patterns {
            if self.matches_pattern(&rule.pattern, path) {
                if let Some(ref le) = rule.attributes.line_ending {
                    result.line_ending = Some(le.clone());
                }
                if let Some(ref m) = rule.attributes.merge {
                    result.merge = Some(m.clone());
                }
                if let Some(ref d) = rule.attributes.diff {
                    result.diff = Some(d.clone());
                }
                if rule.attributes.export_ignore {
                    result.export_ignore = true;
                }
            }
        }

        result
    }

    /// Pattern matching (simple glob-like)
    fn matches_pattern(&self, pattern: &str, path: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with("/*") {
            let dir = &pattern[..pattern.len() - 2];
            return path.starts_with(dir) && path != dir;
        }

        if pattern.ends_with("/**") {
            let dir = &pattern[..pattern.len() - 3];
            return path.starts_with(dir);
        }

        if pattern.starts_with("*.") {
            let ext = &pattern[1..];
            return path.ends_with(ext);
        }

        path == pattern
    }

    /// Creates default .mugattributes content
    pub fn default_content() -> &'static str {
        "# MUG attributes file - configure merge and diff strategies
# See https://mug.local/docs/mugattributes

# Auto line-ending handling
* line_ending=auto

# Binary files
*.bin line_ending=binary diff=binary merge=binary
*.exe line_ending=binary diff=binary merge=binary
*.dll line_ending=binary diff=binary merge=binary
*.so line_ending=binary diff=binary merge=binary
*.dylib line_ending=binary diff=binary merge=binary

# Images
*.jpg line_ending=binary diff=binary
*.png line_ending=binary diff=binary
*.gif line_ending=binary diff=binary
*.pdf line_ending=binary diff=binary

# Archives
*.zip line_ending=binary
*.tar line_ending=binary
*.gz line_ending=binary

# Documents
*.doc line_ending=binary
*.docx line_ending=binary
*.xls line_ending=binary
*.xlsx line_ending=binary

# Export ignores
.* export-ignore
*.log export-ignore
"
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching_star() {
        let attrs = Attributes::new();
        assert!(attrs.matches_pattern("*", "any_file.txt"));
    }

    #[test]
    fn test_pattern_matching_extension() {
        let attrs = Attributes::new();
        assert!(attrs.matches_pattern("*.txt", "file.txt"));
        assert!(attrs.matches_pattern("*.txt", "path/to/file.txt"));
        assert!(!attrs.matches_pattern("*.txt", "file.rs"));
    }

    #[test]
    fn test_pattern_matching_directory() {
        let attrs = Attributes::new();
        assert!(attrs.matches_pattern("build/*", "build/file.o"));
        assert!(!attrs.matches_pattern("build/*", "src/file.rs"));
    }

    #[test]
    fn test_pattern_matching_recursive() {
        let attrs = Attributes::new();
        assert!(attrs.matches_pattern("node_modules/**", "node_modules/pkg"));
        assert!(attrs.matches_pattern("node_modules/**", "node_modules/pkg/index.js"));
    }

    #[test]
    fn test_get_attributes() {
        let mut attrs = Attributes::new();
        attrs
            .parse_line("*.bin line_ending=binary diff=binary merge=binary")
            .unwrap();

        let file_attrs = attrs.get_attributes("data.bin");
        assert_eq!(file_attrs.line_ending, Some("binary".to_string()));
        assert_eq!(file_attrs.diff, Some("binary".to_string()));
        assert_eq!(file_attrs.merge, Some("binary".to_string()));
    }

    #[test]
    fn test_export_ignore() {
        let mut attrs = Attributes::new();
        attrs.parse_line("* export-ignore").unwrap();

        let file_attrs = attrs.get_attributes(".gitignore");
        assert!(file_attrs.export_ignore);
    }

    #[test]
    fn test_default_content_not_empty() {
        let content = Attributes::default_content();
        assert!(!content.is_empty());
        assert!(content.contains("*.bin"));
        assert!(content.contains("export-ignore"));
    }
}
