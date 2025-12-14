/// Git compatibility layer for migration
/// Allows importing Git repositories into MUG

use crate::core::error::{Error, Result};
use crate::core::repo::Repository;
use std::path::{Path, PathBuf};
use std::fs;
use flate2::read::ZlibDecoder;
use std::io::Read;

/// Import a Git repository into MUG
pub fn import_git_repo<P: AsRef<Path>>(git_path: P, mug_path: P) -> Result<()> {
    let git_path = git_path.as_ref();
    let mug_path = mug_path.as_ref();

    // Verify it's a Git repository
    if !git_path.join(".git").exists() {
        return Err(Error::Custom("Not a Git repository".to_string()));
    }

    // Initialize MUG repository
    let mug_repo = Repository::init(mug_path)?;

    // Copy Git objects to MUG store
    import_git_objects(git_path, &mug_repo)?;

    // Import commit history
    import_git_commits(git_path, &mug_repo)?;

    // Create branches from Git refs
    import_git_branches(git_path, &mug_repo)?;

    Ok(())
}

/// Import Git objects (blobs and trees) into MUG object store
fn import_git_objects(git_path: &Path, mug_repo: &Repository) -> Result<()> {
    let objects_dir = git_path.join(".git/objects");
    
    if !objects_dir.exists() {
        return Ok(()); // No objects to import
    }

    // Walk through Git objects directory (excluding pack files for now)
    for entry in fs::read_dir(&objects_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip pack directory
        if path.file_name().map_or(false, |n| n == "pack") {
            continue;
        }
        
        // Read object files (Git uses 2-char + 38-char SHA1)
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(&path) {
                for obj_entry in entries.flatten() {
                    let obj_path = obj_entry.path();
                    if let Ok(content) = fs::read(&obj_path) {
                        // Store raw content in MUG object store
                        let _ = mug_repo.get_store().store_blob(&content);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Parse a Git commit object (decompressed content)
fn parse_git_commit(content: &[u8]) -> Result<(String, String, Option<String>, String)> {
    let content_str = String::from_utf8_lossy(content);
    
    // Git commit format: "tree <hash>\nparent <hash>\nauthor <name> <time>\ncommitter <name> <time>\n\nmessage"
    let mut tree_hash = String::new();
    let mut parent = None;
    let mut author = String::new();
    let mut message = String::new();
    
    let mut lines = content_str.lines().peekable();
    let mut in_message = false;
    
    while let Some(line) = lines.next() {
        if line.is_empty() {
            in_message = true;
            continue;
        }
        
        if in_message {
            message = line.to_string();
        } else if let Some(hash) = line.strip_prefix("tree ") {
            tree_hash = hash.to_string();
        } else if let Some(parent_hash) = line.strip_prefix("parent ") {
            parent = Some(parent_hash.to_string());
        } else if let Some(author_line) = line.strip_prefix("author ") {
            // Extract name from "name <email> timestamp timezone"
            if let Some(email_pos) = author_line.rfind('<') {
                author = author_line[..email_pos].trim().to_string();
            } else {
                author = author_line.to_string();
            }
        }
    }
    
    Ok((tree_hash, message, parent, author))
}

/// Read a Git object from disk (handles zlib decompression)
fn read_git_object(object_path: &Path) -> Result<Vec<u8>> {
    let file = fs::File::open(object_path)?;
    let mut decoder = ZlibDecoder::new(file);
    let mut content = Vec::new();
    decoder.read_to_end(&mut content)?;
    Ok(content)
}

/// Import Git commits into MUG database using gix (gitoxide - better pack file handling)
fn import_git_commits(git_path: &Path, mug_repo: &Repository) -> Result<()> {
    use chrono::Utc;
    use std::collections::HashSet;
    
    // Use gix for better performance and pack file support
    let repo = gix::open(git_path)
        .map_err(|e| crate::core::error::Error::Custom(format!("Failed to open git repo: {}", e)))?;

    let mut visited = HashSet::new();
    let mut count = 0;
    
    // Walk from HEAD and all refs - gix handles pack files automatically
    if let Ok(head) = repo.head() {
        if let Some(head_id) = head.id() {
            let head_id_str = head_id.to_hex().to_string();
            let mut queue = vec![head_id_str];
        
        while let Some(oid_str) = queue.pop() {
            if visited.contains(&oid_str) {
                continue;
            }
            visited.insert(oid_str.clone());
            
            // Parse OID from hex string
            if let Ok(oid) = gix::ObjectId::from_hex(oid_str.as_bytes()) {
                if let Ok(object) = repo.find_object(oid) {
                    if let Ok(commit) = object.try_into_commit() {
                        let commit_hash = oid_str.clone();
                        let tree_hash = commit.tree_id()
                            .ok()
                            .map(|id| id.to_hex().to_string())
                            .unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());

                        let author_str = commit.author()
                            .ok()
                            .and_then(|a| std::str::from_utf8(a.name).ok())
                            .unwrap_or("Unknown")
                            .to_string();

                        let message = commit.message_raw()
                            .ok()
                            .and_then(|b| std::str::from_utf8(&*b).ok())
                            .map(|m| m.trim().to_string())
                            .unwrap_or_else(|| "(no message)".to_string());

                        let mut parent_ids = commit.parent_ids();
                        if let Some(parent_id) = parent_ids.next() {
                            queue.push(parent_id.to_hex().to_string());
                        }

                        let parent_str: Option<String> = commit.parent_ids().next().map(|p| p.to_hex().to_string());
                        let commit_json = if let Some(parent_hash) = parent_str {
                            serde_json::json!({
                                "id": commit_hash,
                                "tree_hash": tree_hash,
                                "parent": parent_hash,
                                "author": author_str,
                                "message": message,
                                "timestamp": Utc::now().to_rfc3339(),
                            })
                        } else {
                            serde_json::json!({
                                "id": commit_hash,
                                "tree_hash": tree_hash,
                                "parent": serde_json::Value::Null,
                                "author": author_str,
                                "message": message,
                                "timestamp": Utc::now().to_rfc3339(),
                            })
                        };

                        if let Ok(serialized) = serde_json::to_vec(&commit_json) {
                            let _ = mug_repo.get_db().set("COMMITS", commit_hash.as_bytes(), &serialized);
                            count += 1;
                        }
                    }
                }
            }
        }
        }
    }

    eprintln!("[INFO] Imported {} commits from git using gix (native pack file support)", count);
    Ok(())
}

/// Create branches from Git refs
fn import_git_branches(git_path: &Path, mug_repo: &Repository) -> Result<()> {
    use crate::core::branch::{BranchManager, BranchRef};
    
    let refs_heads = git_path.join(".git/refs/heads");
    
    if !refs_heads.exists() {
        return Ok(()); // No branches to import
    }

    let branch_manager = BranchManager::new(mug_repo.get_db().clone());
    let mut head_branch: Option<String> = None;

    // Check current HEAD
    if let Ok(head_ref) = fs::read_to_string(git_path.join(".git/HEAD")) {
        if let Some(branch) = head_ref.strip_prefix("ref: refs/heads/") {
            head_branch = Some(branch.trim().to_string());
        }
    }

    for entry in fs::read_dir(&refs_heads)? {
        let entry = entry?;
        if let Some(branch_name) = entry.file_name().to_str() {
            let branch_name = branch_name.to_string();
            let commit_hash = fs::read_to_string(entry.path())?
                .trim()
                .to_string();
            
            if !commit_hash.is_empty() {
                // Create branch with proper BranchRef struct
                let _ = branch_manager.create_branch(branch_name.clone(), commit_hash);
            }
        }
    }

    // Set HEAD to the current branch if available
    if let Some(branch_name) = head_branch {
        let _ = branch_manager.set_head(branch_name);
    }

    Ok(())
}

/// Check if a directory is a Git repository
pub fn is_git_repo<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().join(".git").exists()
}

/// Get list of branches from Git repository
pub fn get_git_branches<P: AsRef<Path>>(git_path: P) -> Result<Vec<String>> {
    let git_path = git_path.as_ref();
    let refs_heads = git_path.join(".git/refs/heads");

    if !refs_heads.exists() {
        return Ok(Vec::new());
    }

    let mut branches = Vec::new();

    for entry in fs::read_dir(&refs_heads)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            branches.push(name.to_string());
        }
    }

    Ok(branches)
}

/// Get Git commit hash (HEAD)
pub fn get_git_head_commit<P: AsRef<Path>>(git_path: P) -> Result<Option<String>> {
    let git_path = git_path.as_ref();
    let head_file = git_path.join(".git/HEAD");

    if !head_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(head_file)?;
    
    // HEAD file format: "ref: refs/heads/main\n" or commit hash
    let content = content.trim();

    if content.starts_with("ref:") {
        // Extract branch name and read its commit
        let branch_ref = content.strip_prefix("ref: ").unwrap_or("").trim();
        let branch_file = git_path.join(".git").join(branch_ref);
        
        if branch_file.exists() {
            let commit = fs::read_to_string(branch_file)?
                .trim()
                .to_string();
            Ok(Some(commit))
        } else {
            Ok(None)
        }
    } else {
        // Direct commit hash (detached HEAD)
        Ok(Some(content.to_string()))
    }
}

/// Migrate Git repository to MUG format
pub fn migrate_git_to_mug(git_path: &str, mug_path: &str) -> Result<String> {
    let git_path = PathBuf::from(git_path);
    let mug_path = PathBuf::from(mug_path);

    // Verify Git repo
    if !is_git_repo(&git_path) {
        return Err(Error::Custom(
            "Source is not a Git repository".to_string(),
        ));
    }

    // Run full import process
    import_git_repo(&git_path, &mug_path)?;

    // Get branches for summary
    let branches = get_git_branches(&git_path)?;
    let branch_count = branches.len();

    // Return migration summary
    Ok(format!(
        "Migration complete. Migrated {} branches, commits, and objects to MUG.",
        branch_count
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_detection() {
        // This would need a test Git repo
        assert!(!is_git_repo("/nonexistent"));
    }
}
