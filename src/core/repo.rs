use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::core::branch::BranchManager;
use crate::core::commit::CommitLog;
use crate::core::database::MugDb;
use crate::core::error::{Error, Result};
use crate::core::hash;
use crate::core::ignore::IgnoreRules;
use crate::core::index::Index;
use crate::core::status::Status;
use crate::core::store::{ObjectStore, TreeEntry};

pub struct Repository {
    root: PathBuf,
    mug_dir: PathBuf,
    db: MugDb,
    store: ObjectStore,
}

impl Repository {
    const MUG_DIR: &'static str = ".mug";
    const OBJECTS_DIR: &'static str = ".mug/objects";
    const DB_DIR: &'static str = ".mug/db";

    /// Initialize a new MUG repository
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        let mug_dir = root.join(Self::MUG_DIR);
        let objects_dir = root.join(Self::OBJECTS_DIR);
        let db_dir = root.join(Self::DB_DIR);

        // Create directories
        fs::create_dir_all(&objects_dir)?;
        fs::create_dir_all(&db_dir)?;

        // Create database
        let db = MugDb::new(db_dir)?;

        // Initialize default branch
        let branch_manager = BranchManager::new(db.clone());
        branch_manager.create_branch("main".to_string(), String::new())?;
        branch_manager.set_head("main".to_string())?;

        db.flush()?;

        // Create default .mugignore file
        let mugignore_path = root.join(".mugignore");
        if !mugignore_path.exists() {
            fs::write(&mugignore_path, IgnoreRules::default_content())?;
        }

        Ok(Repository {
            root,
            mug_dir,
            db,
            store: ObjectStore::new(objects_dir)?,
        })
    }

    /// Open an existing repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        let mug_dir = root.join(Self::MUG_DIR);
        let objects_dir = root.join(Self::OBJECTS_DIR);
        let db_dir = root.join(Self::DB_DIR);

        if !mug_dir.exists() {
            return Err(Error::NotARepository);
        }

        let db = MugDb::new(db_dir)?;
        let store = ObjectStore::new(objects_dir)?;

        Ok(Repository {
            root,
            mug_dir,
            db,
            store,
        })
    }

    /// Check if a repository exists at path
    pub fn is_repo<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().join(Self::MUG_DIR).exists()
    }

    /// Stage a file
    pub fn add(&self, path: &str) -> Result<()> {
        let file_path = self.root.join(path);
        if !file_path.exists() {
            return Err(Error::Custom(format!("File not found: {}", path)));
        }

        let hash = hash::hash_file(&file_path)?;
        self.store.store_file(&file_path)?;

        let mut index = Index::new(self.db.clone())?;
        index.add(path.to_string(), hash)?;

        Ok(())
    }

    /// Stage multiple files (glob patterns)
    pub fn add_all(&self) -> Result<()> {
        let mut index = Index::new(self.db.clone())?;

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.to_string_lossy().contains(".mug") {
                continue;
            }

            if let Ok(rel_path) = path.strip_prefix(&self.root) {
                let path_str = rel_path.to_string_lossy().to_string();
                let hash = hash::hash_file(path)?;
                self.store.store_file(path)?;
                index.add(path_str, hash)?;
            }
        }

        Ok(())
    }

    /// Remove a file from staging
    pub fn remove(&self, path: &str) -> Result<()> {
        let mut index = Index::new(self.db.clone())?;
        index.remove(path)?;
        Ok(())
    }

    /// Get repository status
    pub fn status(&self) -> Result<Status> {
        let index = Index::new(self.db.clone())?;
        Status::from_index_and_wd(&index, &self.root)
    }

    /// Create a commit
    pub fn commit(&self, author: String, message: String) -> Result<String> {
        let index = Index::new(self.db.clone())?;

        if index.is_empty() {
            return Err(Error::Custom(
                "Nothing to commit. Stage files with 'mug add'.".to_string(),
            ));
        }

        // Build tree from index entries
        let mut tree_entries = Vec::new();
        for entry in index.entries() {
            tree_entries.push(TreeEntry {
                name: entry.path,
                hash: entry.hash,
                is_dir: false,
            });
        }

        let tree_hash = self.store.store_tree(tree_entries)?;

        // Get parent commit
        let branch_manager = BranchManager::new(self.db.clone());
        let current_branch = branch_manager.get_head()?;

        let parent_commit_id = if let Some(ref branch_name) = current_branch {
            if let Some(branch) = branch_manager.get_branch(branch_name)? {
                if !branch.commit_id.is_empty() {
                    Some(branch.commit_id)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Create commit
        let commit_log = CommitLog::new(self.db.clone());
        let commit_id = commit_log.create_commit(tree_hash, author, message, parent_commit_id)?;

        // Update branch reference
        if let Some(branch_name) = current_branch {
            branch_manager.update_branch(&branch_name, commit_id.clone())?;
        }

        // Clear staging area
        let mut index = Index::new(self.db.clone())?;
        index.clear()?;

        self.db.flush()?;

        Ok(commit_id)
    }

    /// Get commit log
    pub fn log(&self) -> Result<Vec<String>> {
        let branch_manager = BranchManager::new(self.db.clone());
        let commit_log = CommitLog::new(self.db.clone());

        let head = branch_manager.get_head()?;
        if head.is_none() {
            return Err(Error::NoCommits);
        }

        let branch_name = head.unwrap();
        let branch = branch_manager.get_branch(&branch_name)?;

        if branch.is_none() || branch.as_ref().unwrap().commit_id.is_empty() {
            return Err(Error::NoCommits);
        }

        let history = commit_log.history(branch.unwrap().commit_id)?;

        Ok(history
            .into_iter()
            .map(|c| {
                format!(
                    "commit {}\nAuthor: {}\nDate: {}\n\n    {}\n",
                    hash::short_hash(&c.id),
                    c.author,
                    c.timestamp,
                    c.message
                )
            })
            .collect())
    }

    /// Create a new branch
    pub fn create_branch(&self, name: String) -> Result<()> {
        let branch_manager = BranchManager::new(self.db.clone());
        let head = branch_manager.get_head()?;

        if let Some(branch_name) = head {
            if let Some(branch) = branch_manager.get_branch(&branch_name)? {
                branch_manager.create_branch(name, branch.commit_id)?;
                self.db.flush()?;
                return Ok(());
            }
        }

        Err(Error::NoCommits)
    }

    /// Switch to a branch
    pub fn checkout(&self, branch_name: String) -> Result<()> {
        let branch_manager = BranchManager::new(self.db.clone());

        if branch_manager.get_branch(&branch_name)?.is_none() {
            return Err(Error::BranchNotFound(branch_name));
        }

        branch_manager.set_head(branch_name)?;
        self.db.flush()?;
        Ok(())
    }

    /// List all branches
    pub fn branches(&self) -> Result<Vec<String>> {
        let branch_manager = BranchManager::new(self.db.clone());
        let branches = branch_manager.list_branches()?;
        Ok(branches.into_iter().map(|b| b.name).collect())
    }

    /// Get the current branch
    pub fn current_branch(&self) -> Result<Option<String>> {
        let branch_manager = BranchManager::new(self.db.clone());
        branch_manager.get_head()
    }

    /// Get database reference for advanced operations
    pub fn get_db(&self) -> &MugDb {
        &self.db
    }
}

// Helper function to clone the database (since Sled Db doesn't impl Clone)
impl Clone for Repository {
    fn clone(&self) -> Self {
        Repository {
            root: self.root.clone(),
            mug_dir: self.mug_dir.clone(),
            db: MugDb::new(self.mug_dir.join("db")).expect("Failed to clone database"),
            store: ObjectStore::new(self.mug_dir.join("objects")).expect("Failed to clone store"),
        }
    }
}
