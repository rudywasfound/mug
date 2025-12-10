use std::collections::HashMap;
use similar::TextDiff;

pub struct Diff {
    pub path: String,
    pub old_hash: String,
    pub new_hash: String,
    pub lines_added: usize,
    pub lines_removed: usize,
}

pub struct DiffStats {
    pub files_changed: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
}

/// Compare two snapshots and produce a diff
pub fn diff_snapshots(
    old_tree: &HashMap<String, String>,
    new_tree: &HashMap<String, String>,
) -> Vec<Diff> {
    let mut diffs = Vec::new();

    // Find changed and new files
    for (path, new_hash) in new_tree {
        if let Some(old_hash) = old_tree.get(path) {
            if old_hash != new_hash {
                diffs.push(Diff {
                    path: path.clone(),
                    old_hash: old_hash.clone(),
                    new_hash: new_hash.clone(),
                    lines_added: 0,
                    lines_removed: 0,
                });
            }
        } else {
            // New file
            diffs.push(Diff {
                path: path.clone(),
                old_hash: String::new(),
                new_hash: new_hash.clone(),
                lines_added: 0,
                lines_removed: 0,
            });
        }
    }

    // Find deleted files
    for (path, old_hash) in old_tree {
        if !new_tree.contains_key(path) {
            diffs.push(Diff {
                path: path.clone(),
                old_hash: old_hash.clone(),
                new_hash: String::new(),
                lines_added: 0,
                lines_removed: 0,
            });
        }
    }

    diffs
}

/// Calculate statistics from diffs
pub fn diff_stats(diffs: &[Diff]) -> DiffStats {
    DiffStats {
        files_changed: diffs.len(),
        lines_added: diffs.iter().map(|d| d.lines_added).sum(),
        lines_removed: diffs.iter().map(|d| d.lines_removed).sum(),
    }
}

/// Perform a detailed text diff between two content strings
pub fn text_diff(old_content: &str, new_content: &str) -> Vec<String> {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut result = Vec::new();

    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Delete => {
                result.push(format!("- {}", change.value()));
            }
            similar::ChangeTag::Insert => {
                result.push(format!("+ {}", change.value()));
            }
            similar::ChangeTag::Equal => {
                result.push(format!("  {}", change.value()));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_snapshots() {
        let mut old_tree = HashMap::new();
        old_tree.insert("file1.txt".to_string(), "hash1".to_string());
        old_tree.insert("file2.txt".to_string(), "hash2".to_string());

        let mut new_tree = HashMap::new();
        new_tree.insert("file1.txt".to_string(), "hash1_new".to_string());
        new_tree.insert("file3.txt".to_string(), "hash3".to_string());

        let diffs = diff_snapshots(&old_tree, &new_tree);
        assert_eq!(diffs.len(), 3); // modified, deleted, new
    }
}
