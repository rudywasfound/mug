use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::Result;

/// Hook types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    PreCommit,
    PostCommit,
    PrePush,
    PostPush,
    PreMerge,
    PostMerge,
}

impl HookType {
    pub fn name(&self) -> &'static str {
        match self {
            HookType::PreCommit => "pre-commit",
            HookType::PostCommit => "post-commit",
            HookType::PrePush => "pre-push",
            HookType::PostPush => "post-push",
            HookType::PreMerge => "pre-merge",
            HookType::PostMerge => "post-merge",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HookType::PreCommit => "Runs before creating a commit",
            HookType::PostCommit => "Runs after a commit is created",
            HookType::PrePush => "Runs before pushing to remote",
            HookType::PostPush => "Runs after pushing to remote",
            HookType::PreMerge => "Runs before merging branches",
            HookType::PostMerge => "Runs after merging branches",
        }
    }
}

/// Hook configuration and execution
#[derive(Debug, Clone)]
pub struct Hook {
    pub name: String,
    pub hook_type: HookType,
    pub path: PathBuf,
    pub enabled: bool,
}

impl Hook {
    pub fn new(name: String, hook_type: HookType, path: PathBuf) -> Self {
        Hook {
            name,
            hook_type,
            path,
            enabled: true,
        }
    }

    /// Execute the hook script
    pub fn execute(&self, args: &[&str]) -> Result<HookResult> {
        if !self.enabled {
            return Ok(HookResult::skipped());
        }

        if !self.path.exists() {
            return Err(crate::error::Error::Custom(format!(
                "Hook not found: {}",
                self.path.display()
            )));
        }

        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&self.path, perms)?;
        }

        // Execute the hook
        let output = Command::new(&self.path).args(args).output().map_err(|e| {
            crate::error::Error::Custom(format!("Failed to execute hook {}: {}", self.name, e))
        })?;

        Ok(HookResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        })
    }

    /// Disable this hook
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable this hook
    pub fn enable(&mut self) {
        self.enabled = true;
    }
}

/// Result of hook execution
#[derive(Debug, Clone)]
pub struct HookResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl HookResult {
    fn skipped() -> Self {
        HookResult {
            success: true,
            stdout: "Hook skipped".to_string(),
            stderr: String::new(),
            exit_code: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn failed(&self) -> bool {
        !self.success
    }
}

/// Hook manager for managing all hooks in a repository
pub struct HookManager {
    #[allow(dead_code)]
    repo_root: PathBuf,
    hooks_dir: PathBuf,
}

impl HookManager {
    /// Create a new hook manager
    pub fn new(repo_root: &Path) -> Result<Self> {
        let hooks_dir = repo_root.join(".mug/hooks");
        fs::create_dir_all(&hooks_dir)?;

        Ok(HookManager {
            repo_root: repo_root.to_path_buf(),
            hooks_dir,
        })
    }

    /// Install a hook from a script
    pub fn install(&self, name: &str, hook_type: HookType, script: &str) -> Result<Hook> {
        let hook_filename = format!("{}-{}", hook_type.name(), name);
        let hook_path = self.hooks_dir.join(&hook_filename);

        // Determine shebang based on script content
        let full_script = if script.starts_with("#!") {
            script.to_string()
        } else {
            format!("#!/bin/bash\nset -e\n{}", script)
        };

        fs::write(&hook_path, full_script)?;

        Ok(Hook::new(name.to_string(), hook_type, hook_path))
    }

    /// Create a hook from a file
    pub fn install_from_file(
        &self,
        name: &str,
        hook_type: HookType,
        file_path: &Path,
    ) -> Result<Hook> {
        let script = fs::read_to_string(file_path)?;
        self.install(name, hook_type, &script)
    }

    /// Get a hook by name and type
    pub fn get_hook(&self, name: &str, hook_type: HookType) -> Result<Option<Hook>> {
        let hook_filename = format!("{}-{}", hook_type.name(), name);
        let hook_path = self.hooks_dir.join(&hook_filename);

        if hook_path.exists() {
            Ok(Some(Hook::new(name.to_string(), hook_type, hook_path)))
        } else {
            Ok(None)
        }
    }

    /// List all hooks
    pub fn list_hooks(&self) -> Result<Vec<Hook>> {
        let mut hooks = Vec::new();

        if !self.hooks_dir.exists() {
            return Ok(hooks);
        }

        for entry in fs::read_dir(&self.hooks_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip disabled hooks
                    if filename.ends_with(".disabled") {
                        continue;
                    }

                    if let Some((hook_type_str, name)) = parse_hook_filename(filename) {
                        if let Some(hook_type) = string_to_hook_type(hook_type_str) {
                            hooks.push(Hook::new(name.to_string(), hook_type, path));
                        }
                    }
                }
            }
        }

        hooks.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(hooks)
    }

    /// List hooks of a specific type
    pub fn list_hooks_by_type(&self, hook_type: HookType) -> Result<Vec<Hook>> {
        Ok(self
            .list_hooks()?
            .into_iter()
            .filter(|h| h.hook_type == hook_type)
            .collect())
    }

    /// Delete a hook
    pub fn delete_hook(&self, name: &str, hook_type: HookType) -> Result<()> {
        let hook_filename = format!("{}-{}", hook_type.name(), name);
        let hook_path = self.hooks_dir.join(&hook_filename);

        if hook_path.exists() {
            fs::remove_file(&hook_path)?;
        }

        Ok(())
    }

    /// Execute all hooks of a type
    pub fn trigger(&self, hook_type: HookType, args: &[&str]) -> Result<Vec<HookResult>> {
        let hooks = self.list_hooks_by_type(hook_type)?;
        let mut results = Vec::new();

        for hook in hooks {
            match hook.execute(args) {
                Ok(result) => {
                    if !result.is_success() {
                        eprintln!("Hook {} failed: {}", hook.name, result.stderr);
                    }
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("Error executing hook {}: {}", hook.name, e);
                }
            }
        }

        Ok(results)
    }

    /// Trigger and fail if any hook fails
    pub fn trigger_strict(&self, hook_type: HookType, args: &[&str]) -> Result<()> {
        let results = self.trigger(hook_type, args)?;

        for result in results {
            if !result.is_success() {
                return Err(crate::error::Error::Custom(format!(
                    "Hook failed: {}",
                    result.stderr
                )));
            }
        }

        Ok(())
    }

    /// Disable a hook
    pub fn disable_hook(&self, name: &str, hook_type: HookType) -> Result<()> {
        let hook_filename = format!("{}-{}.disabled", hook_type.name(), name);
        let original_path = self
            .hooks_dir
            .join(format!("{}-{}", hook_type.name(), name));
        let disabled_path = self.hooks_dir.join(&hook_filename);

        if original_path.exists() {
            fs::rename(&original_path, &disabled_path)?;
        }

        Ok(())
    }

    /// Enable a hook
    pub fn enable_hook(&self, name: &str, hook_type: HookType) -> Result<()> {
        let disabled_path = self
            .hooks_dir
            .join(format!("{}-{}.disabled", hook_type.name(), name));
        let original_path = self
            .hooks_dir
            .join(format!("{}-{}", hook_type.name(), name));

        if disabled_path.exists() {
            fs::rename(&disabled_path, &original_path)?;
        }

        Ok(())
    }
}

/// Parse hook filename into (hook_type, name)
fn parse_hook_filename(filename: &str) -> Option<(&str, &str)> {
    // Hook filename format: "pre-commit-name" or "post-merge-name" etc.
    // We need to find the hook type (which contains dashes) and the name

    // Remove .disabled suffix if present
    let clean_name = if filename.ends_with(".disabled") {
        &filename[..filename.len() - 9]
    } else {
        filename
    };

    // Try to find the hook type by checking each known type
    for hook_type in &[
        "pre-commit",
        "post-commit",
        "pre-push",
        "post-push",
        "pre-merge",
        "post-merge",
    ] {
        if let Some(rest) = clean_name.strip_prefix(hook_type) {
            if rest.starts_with('-') {
                let name = &rest[1..];
                return Some((hook_type, name));
            }
        }
    }

    None
}

/// Convert string to HookType
fn string_to_hook_type(s: &str) -> Option<HookType> {
    match s {
        "pre-commit" => Some(HookType::PreCommit),
        "post-commit" => Some(HookType::PostCommit),
        "pre-push" => Some(HookType::PrePush),
        "post-push" => Some(HookType::PostPush),
        "pre-merge" => Some(HookType::PreMerge),
        "post-merge" => Some(HookType::PostMerge),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hook_type_name() {
        assert_eq!(HookType::PreCommit.name(), "pre-commit");
        assert_eq!(HookType::PostCommit.name(), "post-commit");
        assert_eq!(HookType::PrePush.name(), "pre-push");
    }

    #[test]
    fn test_hook_result() {
        let result = HookResult {
            success: true,
            stdout: "Output".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
        };

        assert!(result.is_success());
        assert!(!result.failed());
    }

    #[test]
    fn test_hook_manager_install() {
        let dir = TempDir::new().unwrap();
        let manager = HookManager::new(dir.path()).unwrap();

        let script = "#!/bin/bash\necho 'test'";
        let hook = manager
            .install("test", HookType::PreCommit, script)
            .unwrap();

        assert_eq!(hook.name, "test");
        assert_eq!(hook.hook_type, HookType::PreCommit);
        assert!(hook.path.exists());
    }

    #[test]
    fn test_hook_manager_list() {
        let dir = TempDir::new().unwrap();
        let manager = HookManager::new(dir.path()).unwrap();

        manager
            .install("test1", HookType::PreCommit, "#!/bin/bash\necho 'test'")
            .unwrap();
        manager
            .install("test2", HookType::PostCommit, "#!/bin/bash\necho 'test'")
            .unwrap();

        let hooks = manager.list_hooks().unwrap();
        assert_eq!(hooks.len(), 2);
    }

    #[test]
    fn test_hook_manager_list_by_type() {
        let dir = TempDir::new().unwrap();
        let manager = HookManager::new(dir.path()).unwrap();

        manager
            .install("test1", HookType::PreCommit, "#!/bin/bash\necho 'test'")
            .unwrap();
        manager
            .install("test2", HookType::PreCommit, "#!/bin/bash\necho 'test'")
            .unwrap();
        manager
            .install("test3", HookType::PostCommit, "#!/bin/bash\necho 'test'")
            .unwrap();

        let pre_commit = manager.list_hooks_by_type(HookType::PreCommit).unwrap();
        assert_eq!(pre_commit.len(), 2);

        let post_commit = manager.list_hooks_by_type(HookType::PostCommit).unwrap();
        assert_eq!(post_commit.len(), 1);
    }

    #[test]
    fn test_hook_manager_delete() {
        let dir = TempDir::new().unwrap();
        let manager = HookManager::new(dir.path()).unwrap();

        manager
            .install("test", HookType::PreCommit, "#!/bin/bash\necho 'test'")
            .unwrap();
        assert_eq!(manager.list_hooks().unwrap().len(), 1);

        manager.delete_hook("test", HookType::PreCommit).unwrap();
        assert_eq!(manager.list_hooks().unwrap().len(), 0);
    }

    #[test]
    fn test_hook_manager_disable() {
        let dir = TempDir::new().unwrap();
        let manager = HookManager::new(dir.path()).unwrap();

        manager
            .install("test", HookType::PreCommit, "#!/bin/bash\necho 'test'")
            .unwrap();

        manager.disable_hook("test", HookType::PreCommit).unwrap();

        // Disabled hooks shouldn't appear in list
        assert_eq!(manager.list_hooks().unwrap().len(), 0);
    }

    #[test]
    fn test_parse_hook_filename() {
        assert_eq!(
            parse_hook_filename("pre-commit-test"),
            Some(("pre-commit", "test"))
        );
        assert_eq!(
            parse_hook_filename("post-merge-verify"),
            Some(("post-merge", "verify"))
        );
        assert_eq!(
            parse_hook_filename("pre-commit-test.disabled"),
            Some(("pre-commit", "test"))
        );
    }

    #[test]
    fn test_string_to_hook_type() {
        assert_eq!(string_to_hook_type("pre-commit"), Some(HookType::PreCommit));
        assert_eq!(string_to_hook_type("post-push"), Some(HookType::PostPush));
        assert_eq!(string_to_hook_type("invalid"), None);
    }
}
