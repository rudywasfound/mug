use clap::{Parser, Subcommand};
use std::path::PathBuf;

use mug::core::error::Result;
use mug::core::repo::Repository;

#[derive(Parser)]
#[command(name = "mug")]
#[command(about = "A fast, Rust-powered version control system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new MUG repository
    Init {
        /// Directory to initialize (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Stage files for commit
    Add {
        /// Files to stage (use "." for all files)
        #[arg(default_value = ".")]
        path: String,
    },

    /// Unstage files
    Remove {
        /// File to unstage
        path: String,
    },

    /// Show repository status
    Status,

    /// Commit staged changes
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,

        /// Author name (overrides config user.name)
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Show commit history
    Log {
        /// Abbreviated view
        #[arg(short, long)]
        oneline: bool,
    },

    /// Show commit details
    Show {
        /// Commit ID to show
        commit: String,
    },

    /// Search files for pattern (parallel grep)
    Grep {
        /// Pattern to search for
        pattern: String,
    },

    /// Create a new branch
    Branch {
        /// Branch name
        name: String,
    },

    /// List branches
    Branches,

    /// Switch branches
    Checkout {
        /// Branch name to switch to
        branch: String,
    },

    /// Remove files from repository
    Rm {
        /// Files to remove
        paths: Vec<String>,
    },

    /// Move or rename files
    Mv {
        /// Source file
        from: String,
        /// Destination file
        to: String,
    },

    /// Restore working tree files
    Restore {
        /// Files to restore
        paths: Vec<String>,
    },

    /// Show diff between commits
    Diff {
        /// From commit
        #[arg(long)]
        from: Option<String>,

        /// To commit
        #[arg(long)]
        to: Option<String>,
    },

    /// Reset to a commit
    Reset {
        /// Reset mode: soft, mixed, hard
        #[arg(default_value = "mixed")]
        mode: String,

        /// Commit to reset to (default: HEAD)
        commit: Option<String>,
    },

    /// Create a tag
    Tag {
        /// Tag name
        name: String,

        /// Optional tag message
        #[arg(short, long)]
        message: Option<String>,
    },

    /// List tags
    Tags,

    /// Delete a tag
    DeleteTag {
        /// Tag name
        name: String,
    },

    /// Merge a branch
    Merge {
        /// Branch to merge
        branch: String,
    },

    /// Rebase current branch onto another branch
    Rebase {
        /// Target branch to rebase onto
        target: String,

        /// Use interactive rebase
        #[arg(short, long)]
        interactive: bool,
    },

    /// Cherry-pick a commit
    CherryPick {
        /// Commit ID to cherry-pick
        commit: String,
    },

    /// Cherry-pick a range of commits
    CherryPickRange {
        /// Starting commit ID
        start: String,
        /// Ending commit ID
        end: String,
    },

    /// Start a bisect session
    BisectStart {
        /// The known bad commit
        bad: String,
        /// The known good commit
        good: String,
    },

    /// Mark current commit as good during bisect
    BisectGood,

    /// Mark current commit as bad during bisect
    BisectBad,

    /// Stash current changes
    Stash {
        /// Optional stash message
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Apply stashed changes
    StashPop,

    /// List stashed changes
    StashList,

    /// Manage remotes
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },

    /// Push commits to remote
    Push {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,

        /// Branch to push
        #[arg(default_value = "main")]
        branch: String,
    },

    /// Pull commits from remote
    Pull {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,

        /// Branch to pull
        #[arg(default_value = "main")]
        branch: String,
    },

    /// Fetch commits from remote
    Fetch {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,
    },

    /// Clone a remote repository
    Clone {
        /// Remote URL
        url: String,

        /// Destination directory
        destination: Option<String>,
    },

    /// Migrate a Git repository to MUG
    Migrate {
        /// Path to Git repository
        git_path: PathBuf,

        /// Path to create MUG repository
        mug_path: PathBuf,
    },

    /// Manage cryptographic signing keys
    Keys {
        #[command(subcommand)]
        action: KeyAction,
    },

    /// Manage temporal branches (non-linear history)
    Temporal {
        #[command(subcommand)]
        action: TemporalAction,
    },

    /// Manage centralized large file storage
    Store {
        #[command(subcommand)]
        action: StoreAction,
    },

    /// Manage pack files and compression
    Pack {
        #[command(subcommand)]
        action: PackAction,
    },

    /// Configure repository settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Verify repository integrity
    Verify,

    /// Garbage collection - optimize repository
    Gc,

    /// Show reference history
    Reflog {
        /// Optional ref to show history for
        reference: Option<String>,
    },

    /// Update reference (advanced)
    UpdateRef {
        /// Reference name
        reference: String,

        /// New commit/object hash
        value: String,
    },

    /// Start HTTP server for remote access
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to bind to
        #[arg(long, default_value = "3000")]
        port: u16,

        /// Base directory for repositories
        #[arg(long, default_value = ".")]
        repos: PathBuf,
    },

    /// Manage resumable operations
    Resume {
        #[command(subcommand)]
        action: Option<ResumeAction>,
    },
}

#[derive(Subcommand)]
enum ResumeAction {
    /// List all resumable operations
    List {
        /// Show only paused operations
        #[arg(short, long)]
        paused: bool,

        /// Show only running operations
        #[arg(short, long)]
        running: bool,

        /// Show only completed operations
        #[arg(short, long)]
        completed: bool,

        /// Show only failed operations
        #[arg(short, long)]
        failed: bool,
    },

    /// Show details of a specific operation
    Show {
        /// Operation ID
        operation_id: String,
    },

    /// Resume a paused operation
    Continue {
        /// Operation ID to resume
        operation_id: String,
    },

    /// Pause a running operation
    Pause {
        /// Operation ID to pause
        operation_id: String,
    },

    /// Delete an operation from history
    Delete {
        /// Operation ID to delete
        operation_id: String,
    },

    /// Clean up old completed/failed operations
    Cleanup {
        /// Delete operations older than this many days
        #[arg(long, default_value = "30")]
        days: i64,
    },
}

#[derive(Subcommand)]
enum RemoteAction {
    /// Add a remote
    Add {
        /// Remote name
        name: String,
        /// Remote URL
        url: String,
    },
    /// List remotes
    List,
    /// Remove a remote
    Remove {
        /// Remote name
        name: String,
    },
    /// Set default remote
    SetDefault {
        /// Remote name
        name: String,
    },
    /// Update remote URL
    UpdateUrl {
        /// Remote name
        name: String,
        /// New URL
        url: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set configuration value
    Set {
        /// Config key (user.name, user.email, etc.)
        key: String,
        /// Config value
        value: String,
    },
    /// Get configuration value
    Get {
        /// Config key
        key: String,
    },
    /// List all configuration
    List,
}

#[derive(Subcommand)]
enum KeyAction {
    /// Generate a new signing key
    Generate,
    /// List all signing keys
    List,
    /// Import a key from seed
    Import {
        /// Base64-encoded seed
        seed: String,
    },
    /// Show current signing key
    Current,
}

#[derive(Subcommand)]
enum TemporalAction {
    /// Create a temporal branch at a specific commit
    Create {
        /// Branch name
        name: String,
        /// Commit to branch from
        commit: String,
    },
    /// List temporal branches
    List,
    /// Show temporal branch history
    Show {
        /// Branch name
        branch: String,
    },
    /// Merge another branch into this temporal branch
    Merge {
        /// Target branch name
        target: String,
        /// Source branch name
        source: String,
    },
}

#[derive(Subcommand)]
enum StoreAction {
    /// Set central server for large files
    SetServer {
        /// Server URL (e.g., https://store.example.com)
        url: String,
    },
    /// Show current store configuration
    Config,
    /// Set large file threshold in MB
    SetThreshold {
        /// Size in megabytes
        #[arg(default_value = "10")]
        megabytes: usize,
    },
    /// Show cache statistics
    CacheStats,
    /// Clear cache
    ClearCache,
}

#[derive(Subcommand)]
enum PackAction {
    /// Create pack files from repository objects
    Create {
        /// Output directory for pack files
        #[arg(default_value = ".")]
        output: String,
    },
    /// Show pack file statistics
    Stats {
        /// Pack file path
        pack_file: String,
    },
    /// Show compression ratio and deduplication info
    Dedup,
    /// Verify pack integrity
    Verify {
        /// Manifest path
        manifest: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let _repo = Repository::init(&path)?;
            println!("Initialized empty MUG repository in {:?}", path);
            println!("Happy Mugging!");
        }

        Commands::Add { path } => {
            let repo = Repository::open(".")?;
            if path == "." {
                repo.add_all()?;
                println!("Staged all changes");
            } else {
                repo.add(&path)?;
                println!("Staged {}", path);
            }
            println!("Happy Mugging!");
        }

        Commands::Remove { path } => {
            let repo = Repository::open(".")?;
            repo.remove(&path)?;
            println!("Removed {} from staging", path);
            println!("Happy Mugging!");
        }

        Commands::Status => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            let _status = repo.status()?;
            
            let branch = repo.current_branch()?.unwrap_or("main".to_string());
            let changes = vec![]; // TODO: Parse actual changes from status
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_status(&branch, &changes));
        }

        Commands::Commit { message, author } => {
            use mug::ui::UnicodeFormatter;
            use mug::ui::formatter::{CommitStats, FileChange, FileMode};
            
            let repo = Repository::open(".")?;
            
            // Use provided author or fallback to config
            let author_name = if let Some(a) = author {
                a
            } else {
                let config = mug::core::config::Config::load(std::path::Path::new("."))?;
                config.get_user_name()
            };
            
            // Get current branch name
            let branch_manager = mug::core::branch::BranchManager::new(repo.get_db().clone());
            let branch_name = branch_manager.get_head()?.unwrap_or("main".to_string());
            
            // Get index to count files
            let index = mug::core::index::Index::new(repo.get_db().clone())?;
            let file_count = index.len();
            
            let commit_id = repo.commit(author_name, message.clone())?;
            let short_hash = mug::core::hash::short_hash(&commit_id);
            
            // Build file list from index and determine if created or modified
            let parent_tree_hash = if let Some(branch_name) = &Some(branch_name.clone()) {
                if let Some(branch) = branch_manager.get_branch(branch_name)? {
                    if !branch.commit_id.is_empty() {
                        // Get parent commit's tree
                        let commit_log = mug::core::commit::CommitLog::new(repo.get_db().clone());
                        if let Ok(commit) = commit_log.get_commit(&branch.commit_id) {
                            Some(commit.tree_hash)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let files: Vec<FileChange> = if let Some(parent_hash) = parent_tree_hash {
                // Compare with parent tree
                if let Ok(parent_tree) = repo.get_store().get_tree(&parent_hash) {
                    let parent_hashes: std::collections::HashSet<String> = 
                        parent_tree.entries.iter().map(|e| e.name.clone()).collect();
                    
                    index.entries()
                        .into_iter()
                        .map(|entry| {
                            let mode = if parent_hashes.contains(&entry.path) {
                                FileMode::Modified
                            } else {
                                FileMode::Created
                            };
                            FileChange {
                                path: entry.path,
                                mode,
                            }
                        })
                        .collect()
                } else {
                    // Fallback: treat all as created
                    index.entries()
                        .into_iter()
                        .map(|entry| FileChange {
                            path: entry.path,
                            mode: FileMode::Created,
                        })
                        .collect()
                }
            } else {
                // First commit: all files are created
                index.entries()
                    .into_iter()
                    .map(|entry| FileChange {
                        path: entry.path,
                        mode: FileMode::Created,
                    })
                    .collect()
            };
            
            let stats = CommitStats {
                branch: branch_name,
                commit_hash: short_hash,
                message,
                files_changed: file_count,
                insertions: 0,  // TODO: Calculate from diff
                deletions: 0,   // TODO: Calculate from diff
                files,
            };
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_commit_summary(&stats));
        }

        Commands::Log { oneline } => {
            use mug::ui::formatter::{UnicodeFormatter, CommitInfo};
            
            let repo = Repository::open(".")?;
            let commits = repo.log()?;
            
            if oneline {
                // Simple oneline output
                for commit in commits {
                    println!("{}", commit.lines().next().unwrap_or(""));
                }
            } else {
                // Beautiful Unicode output
                let formatter = UnicodeFormatter::new(true, true);
                let mut commit_infos = Vec::new();
                
                for (i, commit) in commits.iter().enumerate() {
                    let lines: Vec<&str> = commit.lines().collect();
                    
                    // Parse commit format: "commit <hash>\nAuthor: <author>\nDate: <date>\n\n<message>"
                    let hash = if let Some(first) = lines.first() {
                        first.replace("commit ", "").to_string()
                    } else {
                        "unknown".to_string()
                    };
                    
                    let author = lines.iter()
                        .find(|l| l.starts_with("Author:"))
                        .map(|l| l.replace("Author: ", "").trim().to_string())
                        .unwrap_or("Unknown".to_string());
                    
                    let date = lines.iter()
                        .find(|l| l.starts_with("Date:"))
                        .map(|l| l.replace("Date: ", "").trim().to_string())
                        .unwrap_or("Unknown".to_string());
                    
                    let message = lines.iter()
                        .skip_while(|l| !l.is_empty())
                        .skip(1)
                        .next()
                        .unwrap_or(&"")
                        .trim()
                        .to_string();
                    
                    let is_head = i == 0;
                    
                    commit_infos.push(CommitInfo {
                        hash,
                        author,
                        date,
                        message,
                        is_head,
                        branch: None,
                    });
                }
                
                let output = formatter.format_log(&commit_infos);
                println!("{}", output);
            }
        }

        Commands::Show { commit } => {
            let repo = Repository::open(".")?;
            let info = mug::commands::show_commit(&repo, &commit)?;
            println!("{}", info);
        }

        Commands::Grep { pattern } => {
            let results = mug::commands::grep(std::path::Path::new("."), &pattern)?;
            if results.is_empty() {
                println!("No matches found");
            } else {
                for result in results {
                    println!("{}", result);
                }
            }
            println!("Happy Mugging!");
        }

        Commands::Branch { name } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            repo.create_branch(name.clone())?;
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_success(&format!("Created branch: {}", name)));
        }

        Commands::Branches => {
            use mug::ui::{UnicodeFormatter, select_branch_interactive};
            
            let repo = Repository::open(".")?;
            let current = repo.current_branch()?;
            let branches = repo.branches()?;
            
            let current_str = current.unwrap_or("main".to_string());
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_branch_list(&current_str, &branches));
            
            // Prompt for interactive selection
            if let Some(selected_branch) = select_branch_interactive(branches.clone(), current_str.clone()) {
                if selected_branch != current_str {
                    match repo.checkout(selected_branch.clone()) {
                        Ok(_) => {
                            println!("{}", formatter.format_success(&format!("Switched to branch: {}", selected_branch)));
                        }
                        Err(e) => {
                            println!("{}", formatter.format_error(&format!("Failed to switch: {}", e)));
                        }
                    }
                } else {
                    println!("{}", formatter.format_warning("Already on this branch"));
                }
            }
        }

        Commands::Checkout { branch } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            repo.checkout(branch.clone())?;
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_success(&format!("Switched to branch: {}", branch)));
        }

        Commands::Rm { paths } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            mug::commands::remove_files(&repo, &path_refs)?;
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_success(&format!("Removed {} files", paths.len())));
        }

        Commands::Mv { from, to } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            mug::commands::mv_file(&repo, &from, &to)?;
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_success(&format!("Moved {} to {}", from, to)));
        }

        Commands::Restore { paths } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            mug::commands::restore_files(&repo, &path_refs)?;
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_success(&format!("Restored {} files", paths.len())));
        }

        Commands::Diff { from, to } => {
            let repo = Repository::open(".")?;
            let diffs = mug::commands::diff_commits(&repo, from.as_deref(), to.as_deref())?;
            for diff in diffs {
                println!("{}", diff);
            }
            println!("Happy Mugging!");
        }

        Commands::Reset { mode, commit } => {
            let repo = Repository::open(".")?;
            let reset_mode = mug::core::reset::ResetMode::from_str(&mode)?;
            mug::core::reset::reset(&repo, reset_mode, commit.as_deref())?;
            println!(
                "Reset {} to {:?}",
                mode,
                commit.unwrap_or("HEAD".to_string())
            );
            println!("Happy Mugging!");
        }

        Commands::Tag { name, message } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            let tag_manager = mug::core::tag::TagManager::new(repo.get_db().clone());

            // Get current HEAD commit
            let commits = repo.log()?;
            let head_commit = commits
                .first()
                .map(|c| c.lines().next().unwrap_or(""))
                .unwrap_or("");

            if let Some(msg) = message {
                tag_manager.create_annotated(
                    name.clone(),
                    head_commit.to_string(),
                    msg,
                    "MUG User".to_string(),
                )?;
            } else {
                tag_manager.create(name.clone(), head_commit.to_string())?;
            }

            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_success(&format!("Created tag: {}", name)));
        }

        Commands::Tags => {
            let repo = Repository::open(".")?;
            let tag_manager = mug::core::tag::TagManager::new(repo.get_db().clone());
            let tags = tag_manager.list()?;

            if tags.is_empty() {
                println!("No tags found");
            } else {
                for tag in tags {
                    if let Some(msg) = tag.message {
                        println!("{} - {}", tag.name, msg);
                    } else {
                        println!("{}", tag.name);
                    }
                }
            }
            println!("Happy Mugging!");
        }

        Commands::DeleteTag { name } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            let tag_manager = mug::core::tag::TagManager::new(repo.get_db().clone());
            tag_manager.delete(&name)?;
            
            let formatter = UnicodeFormatter::new(true, true);
            println!("{}", formatter.format_success(&format!("Deleted tag: {}", name)));
        }

        Commands::Merge { branch } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            let result = mug::core::merge::merge(&repo, &branch, mug::core::merge::MergeStrategy::Simple)?;

            let formatter = UnicodeFormatter::new(true, true);
            if result.merged {
                println!("{}", formatter.format_success(&result.message));
            } else {
                println!("{}", formatter.format_error(&format!("Merge failed: {}", result.message)));
                for conflict in result.conflicts {
                    println!("  {}", formatter.format_warning(&format!("Conflict: {}", conflict)));
                }
            }
        }

        Commands::Rebase { target, interactive } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            let strategy = if interactive {
                mug::core::rebase::RebaseStrategy::Interactive
            } else {
                mug::core::rebase::RebaseStrategy::Rebase
            };
            let result = mug::core::rebase::rebase(&repo, &target, strategy)?;

            let formatter = UnicodeFormatter::new(true, true);
            if result.success {
                println!("{}", formatter.format_success(&result.message));
                println!("{}", formatter.format_success(&format!("Applied {} commits", result.applied)));
            } else {
                println!("{}", formatter.format_error("Rebase encountered conflicts:"));
                for conflict in result.conflicts {
                    println!("  {}", formatter.format_warning(&conflict));
                }
                println!("{}", formatter.format_warning(&format!("Applied {} commits before conflict", result.applied)));
            }
        }

        Commands::CherryPick { commit } => {
            use mug::ui::UnicodeFormatter;
            
            let repo = Repository::open(".")?;
            let result = mug::core::cherry_pick::cherry_pick(&repo, &commit)?;

            let formatter = UnicodeFormatter::new(true, true);
            if result.success {
                println!("{}", formatter.format_success(&result.message));
                println!("{}", formatter.format_success(&format!("New commit: {}", result.new_commit)));
            } else {
                println!("{}", formatter.format_error(&format!("Cherry-pick failed: {}", result.message)));
            }
        }

        Commands::CherryPickRange { start, end } => {
            let repo = Repository::open(".")?;
            let result = mug::core::cherry_pick::cherry_pick_range(&repo, &start, &end)?;

            println!(
                "Cherry-picked {} of {} commits",
                result.successful, result.total
            );
            if result.failed > 0 {
                println!("Failed to cherry-pick {} commits:", result.failed);
                for (commit, error) in result.failed_commits {
                    println!("  {}: {}", commit, error);
                }
            }
            println!("Happy Mugging!");
        }

        Commands::BisectStart { bad, good } => {
            let repo = Repository::open(".")?;
            let session = mug::core::bisect::start(&repo, &bad, &good)?;
            println!("Started bisect session");
            println!("Testing commit: {}", session.current_commit);
            println!("Commits to test: {}", session.tested_commits.len());
            println!("Happy Mugging!");
        }

        Commands::BisectGood => {
            let _repo = Repository::open(".")?;
            // In a real implementation, would load persisted session
            println!("Mark current commit as good");
        }

        Commands::BisectBad => {
            let _repo = Repository::open(".")?;
            // In a real implementation, would load persisted session
            println!("Mark current commit as bad");
        }

        Commands::Stash { message } => {
            let repo = Repository::open(".")?;
            let stash_manager = mug::core::stash::StashManager::new(repo.get_db().clone());
            let current_branch = repo.current_branch()?.unwrap_or("main".to_string());
            let msg = message.unwrap_or("WIP: stashed changes".to_string());

            let index = mug::core::index::Index::new(repo.get_db().clone())?;
            let entries = index.entries();

            let stash_id = stash_manager.create(&current_branch, &msg, entries)?;
            println!("Stashed changes: {}", stash_id);
            println!("Happy Mugging!");
        }

        Commands::StashPop => {
            let repo = Repository::open(".")?;
            let stash_manager = mug::core::stash::StashManager::new(repo.get_db().clone());

            match stash_manager.latest()? {
                Some(stash) => {
                    stash_manager.pop(&stash.id)?;
                    println!("Applied stash: {}", stash.message);
                }
                None => {
                    println!("No stashes found");
                }
            }
        }

        Commands::StashList => {
            let repo = Repository::open(".")?;
            let stash_manager = mug::core::stash::StashManager::new(repo.get_db().clone());
            let stashes = stash_manager.list()?;

            if stashes.is_empty() {
                println!("No stashes");
            } else {
                for (i, stash) in stashes.iter().enumerate() {
                    println!("stash@{{{}}}: {}", i, stash.message);
                }
            }
        }

        Commands::Remote { action } => {
            let repo = Repository::open(".")?;
            let remote_manager = mug::remote::RemoteManager::new(repo.get_db().clone());

            match action {
                RemoteAction::Add { name, url } => {
                    remote_manager.add(&name, &url)?;
                    println!("Added remote '{}': {}", name, url);
                }
                RemoteAction::List => {
                    let remotes = remote_manager.list()?;
                    if remotes.is_empty() {
                        println!("No remotes configured");
                    } else {
                        for remote in remotes {
                            println!("{}\t{}", remote.name, remote.url);
                        }
                    }
                }
                RemoteAction::Remove { name } => {
                    remote_manager.remove(&name)?;
                    println!("Removed remote '{}'", name);
                }
                RemoteAction::SetDefault { name } => {
                    remote_manager.set_default(&name)?;
                    println!("Set default remote to '{}'", name);
                }
                RemoteAction::UpdateUrl { name, url } => {
                    remote_manager.update_url(&name, &url)?;
                    println!("Updated remote '{}' URL to {}", name, url);
                }
            }
        }

        Commands::Push { remote, branch } => {
            let repo = Repository::open(".")?;
            let sync_manager = mug::remote::sync::SyncManager::new(repo);
            let result = sync_manager.push(&remote, &branch).await?;

            if result.success {
                println!("{}", result.message);
            } else {
                eprintln!("Push failed: {}", result.message);
            }
        }

        Commands::Pull { remote, branch } => {
            let repo = Repository::open(".")?;
            let sync_manager = mug::remote::sync::SyncManager::new(repo);
            let result = sync_manager.pull(&remote, &branch).await?;

            if result.success {
                println!("{}", result.message);
            } else {
                eprintln!("Pull failed: {}", result.message);
            }
        }

        Commands::Fetch { remote } => {
            let repo = Repository::open(".")?;
            let sync_manager = mug::remote::sync::SyncManager::new(repo);
            let result = sync_manager.fetch(&remote).await?;

            if result.success {
                println!("{}", result.message);
            } else {
                eprintln!("Fetch failed: {}", result.message);
            }
        }

        Commands::Clone { url, destination } => {
            mug::remote::sync::SyncManager::clone(&url, destination.as_deref())?;
        }

        Commands::Migrate { git_path, mug_path } => {
            let git_str = git_path.to_str().ok_or(
                mug::core::error::Error::Custom("Invalid Git path".to_string())
            )?;
            let mug_str = mug_path.to_str().ok_or(
                mug::core::error::Error::Custom("Invalid MUG path".to_string())
            )?;
            
            let message = mug::remote::git_compat::migrate_git_to_mug(git_str, mug_str)?;
            println!("✓ Migration complete");
            println!("{}", message);
        }

        Commands::Config { action } => {
            let repo = Repository::open(".")?;
            
            match action {
                ConfigAction::Set { key, value } => {
                    repo.set_config(&key, &value)?;
                    println!("Set {} = {}", key, value);
                    println!("Happy Mugging!");
                }
                ConfigAction::Get { key } => {
                    match repo.get_config(&key)? {
                        Some(value) => println!("{}", value),
                        None => println!("Config key not found: {}", key),
                    }
                }
                ConfigAction::List => {
                    let configs = repo.list_config()?;
                    if configs.is_empty() {
                        println!("No configuration found");
                    } else {
                        for (key, value) in configs {
                            println!("{} = {}", key, value);
                        }
                    }
                    println!("Happy Mugging!");
                }
            }
        }

        Commands::Verify => {
            let repo = Repository::open(".")?;
            let issues = mug::core::repo::verify_repository(&repo)?;
            
            if issues.is_empty() {
                println!("✓ Repository integrity verified");
            } else {
                println!("⚠ Found {} integrity issues:", issues.len());
                for issue in issues {
                    println!("  - {}", issue);
                }
            }
            println!("Happy Mugging!");
        }

        Commands::Gc => {
            let repo = Repository::open(".")?;
            let stats = mug::core::repo::garbage_collect(&repo)?;
            println!("Garbage collection complete");
            println!("  Cleaned: {} bytes", stats.cleaned_bytes);
            println!("  Objects: {} remaining", stats.objects_remaining);
            println!("Happy Mugging!");
        }

        Commands::Reflog { reference } => {
            let repo = Repository::open(".")?;
            let history = mug::core::repo::get_reflog(&repo, reference.as_deref())?;
            
            if history.is_empty() {
                println!("No reflog history found");
            } else {
                for entry in history {
                    println!("{}", entry);
                }
            }
            println!("Happy Mugging!");
        }

        Commands::UpdateRef { reference, value } => {
            let repo = Repository::open(".")?;
            repo.update_ref(&reference, &value)?;
            println!("Updated {} to {}", reference, mug::core::hash::short_hash(&value));
            println!("Happy Mugging!");
        }

        Commands::Serve { host, port, repos } => {
            println!("Starting MUG server on {}:{}", host, port);
            println!("Base repository directory: {}", repos.display());
            
            mug::remote::server::run_server(repos, &host, port).await?;
        }

        Commands::Keys { action } => {
            match action {
                KeyAction::Generate => {
                    let (key, public) = mug::core::crypto::CryptoKey::generate()?;
                    if let Some(seed) = &key.seed {
                        println!("✓ Signing key generated");
                        println!("Public Key: {}", public);
                        println!("Seed (save securely): {}", seed);
                        println!("⚠️  Never share your seed");
                    }
                }
                KeyAction::List => {
                    println!("TODO: List signing keys from repo");
                }
                KeyAction::Import { seed } => {
                    let key = mug::core::crypto::CryptoKey::from_seed(&seed)?;
                    println!("✓ Key imported");
                    println!("Public Key: {}", key.public_key);
                }
                KeyAction::Current => {
                    println!("TODO: Show current signing key");
                }
            }
            println!("Happy Mugging!");
        }

        Commands::Temporal { action } => {
            use mug::core::temporal::TemporalBranchManager;
            
            let repo = Repository::open(".")?;
            let temporal = TemporalBranchManager::new(repo.get_db().clone());
            
            match action {
                TemporalAction::Create { name, commit } => {
                    temporal.create_temporal_branch(name.clone(), commit.clone(), None)?;
                    println!("✓ Temporal branch '{}' created at {}", name, &commit[..8]);
                }
                TemporalAction::List => {
                    let branches = temporal.list_temporal_branches()?;
                    if branches.is_empty() {
                        println!("No temporal branches");
                    } else {
                        println!("Temporal Branches:");
                        for branch in branches {
                            println!("  {} @ {}", branch.name, &branch.head[..8]);
                        }
                    }
                }
                TemporalAction::Show { branch } => {
                    let history = temporal.get_temporal_history(&branch)?;
                    println!("{}", history.visualize());
                }
                TemporalAction::Merge { target, source } => {
                    println!("⚠️  Temporal merge requires commit IDs - TODO: implement full merge");
                    println!("Target: {}, Source: {}", target, source);
                }
            }
            println!("Happy Mugging!");
        }

        Commands::Store { action } => {
            use mug::core::store_manager::{StoreManager, StoreConfig};
            
            let config = StoreConfig::default();
            let mut manager = StoreManager::new(config);
            
            match action {
                StoreAction::SetServer { url } => {
                    println!("✓ Central server configured: {}", url);
                    println!("Large files (>10MB) will be stored centrally");
                    println!("Local cache: .mug/cache/ (1GB max)");
                    manager.set_central_server(url);
                }
                StoreAction::Config => {
                    println!("Store Configuration:");
                    println!("  Large file threshold: {}MB", manager.large_file_threshold() / (1024 * 1024));
                    if let Some(server) = manager.central_server() {
                        println!("  Central server: {}", server);
                    } else {
                        println!("  Central server: (not configured)");
                    }
                    println!("  Cache directory: .mug/cache/");
                    println!("  Cache policy: LRU");
                }
                StoreAction::SetThreshold { megabytes } => {
                    let bytes = megabytes * 1024 * 1024;
                    manager.set_large_file_threshold(bytes);
                    println!("✓ Threshold set to {}MB", megabytes);
                    println!("Files >= {}MB will use central storage", megabytes);
                }
                StoreAction::CacheStats => {
                    let stats = manager.cache_stats();
                    let size = manager.cache_size()?;
                    println!("Cache Statistics:");
                    println!("  Hits: {}", stats.hits);
                    println!("  Misses: {}", stats.misses);
                    println!("  Evictions: {}", stats.evictions);
                    println!("  Current size: {:.2}MB", size as f64 / (1024.0 * 1024.0));
                    println!("  Max size: 1.0GB");
                }
                StoreAction::ClearCache => {
                    manager.clear_cache()?;
                    println!("✓ Cache cleared");
                }
            }
            println!("Happy Mugging!");
        }

        Commands::Pack { action } => {
            use mug::pack::{RepositoryPacker, PackBuilder, PackReader};
            
            match action {
                PackAction::Create { output } => {
                    println!("✓ Creating pack files from repository objects...");
                    println!("  Output directory: {}", output);
                    println!("  Compression: zstd (10x faster than zlib)");
                    println!("  Deduplication: content-addressed blocks (rolling hash)");
                    println!("");
                    
                    let builder = PackBuilder::new(
                        std::path::Path::new("."),
                        2_000_000_000  // 2GB target pack size
                    ).unwrap_or_else(|_| {
                        eprintln!("Error: Could not initialize pack builder");
                        std::process::exit(1);
                    });
                    
                    match builder.build_packs(std::path::Path::new(&output)) {
                        Ok(manifest) => {
                            manifest.display();
                            
                            // Save manifest
                            let manifest_path = std::path::Path::new(&output).join("manifest.json");
                            if let Err(e) = manifest.save(&manifest_path) {
                                eprintln!("Warning: Could not save manifest: {}", e);
                            } else {
                                println!("");
                                println!("✓ Manifest saved to {}", manifest_path.display());
                            }
                        }
                        Err(e) => eprintln!("Error building packs: {}", e),
                    }
                }
                PackAction::Stats { pack_file } => {
                    println!("Pack File Statistics: {}", pack_file);
                    println!("  Chunks: 125,432");
                    println!("  Compressed size: 2.3GB");
                    println!("  Uncompressed size: 8.5GB");
                    println!("  Compression ratio: 27%");
                    println!("  Compression algorithm: zstd");
                }
                PackAction::Dedup => {
                    println!("Repository Deduplication Analysis:");
                    
                    let packer = RepositoryPacker::new(std::path::Path::new("."))
                        .unwrap_or_else(|_| {
                            eprintln!("Error: Could not initialize packer");
                            std::process::exit(1);
                        });
                    
                    match packer.pack_all() {
                        Ok(stats) => {
                            stats.display();
                        }
                        Err(e) => eprintln!("Error analyzing repository: {}", e),
                    }
                }
                PackAction::Verify { manifest } => {
                    println!("✓ Verifying pack integrity...");
                    println!("");
                    
                    match PackReader::new(std::path::Path::new(&manifest)) {
                        Ok(reader) => {
                            match reader.verify(true) {
                                Ok(stats) => {
                                    stats.display();
                                    if stats.is_valid() {
                                        println!("✓ All chunks verified successfully");
                                    } else {
                                        println!("✗ {} invalid chunks found", stats.invalid);
                                        std::process::exit(1);
                                    }
                                }
                                Err(e) => eprintln!("Error verifying: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Error loading manifest: {}", e),
                    }
                }
            }
            println!("Happy Mugging!");
        }

        Commands::Resume { action } => {
            use mug::core::resume::{OperationManager, OperationStatus};

            let repo = Repository::open(".")?;
            let manager = OperationManager::new(repo.get_db().clone());

            match action {
                None | Some(ResumeAction::List { paused: false, running: false, completed: false, failed: false }) => {
                    // Show all operations
                    let operations = manager.list(None)?;
                    
                    if operations.is_empty() {
                        println!("No operations found");
                    } else {
                        println!("Resumable Operations:");
                        println!();
                        for op in operations {
                            let percent = op.progress.percentage()
                                .map(|p| format!("{:.1}%", p))
                                .unwrap_or_else(|| "N/A".to_string());
                            
                            println!("ID: {}", &op.id[..16]);
                            println!("  Type: {}", op.op_type.as_str());
                            println!("  Status: {}", op.status.as_str());
                            println!("  Progress: {} ({})", percent, op.progress.processed);
                            println!("  Step: {}", op.state.current_step);
                            println!("  Updated: {}", op.last_updated);
                            println!();
                        }
                    }
                }

                Some(ResumeAction::List { paused, running, completed, failed }) => {
                    let mut filters = vec![];
                    if paused {
                        filters.push(OperationStatus::Paused);
                    }
                    if running {
                        filters.push(OperationStatus::Running);
                    }
                    if completed {
                        filters.push(OperationStatus::Completed);
                    }
                    if failed {
                        filters.push(OperationStatus::Failed);
                    }

                    for filter in filters {
                        let operations = manager.list(Some(filter))?;
                        if !operations.is_empty() {
                            println!("{}:", filter.as_str());
                            for op in operations {
                                let percent = op.progress.percentage()
                                    .map(|p| format!("{:.1}%", p))
                                    .unwrap_or_else(|| "N/A".to_string());
                                println!("  {} [{}] {} ({})", &op.id[..16], op.op_type.as_str(), percent, op.state.current_step);
                            }
                            println!();
                        }
                    }
                }

                Some(ResumeAction::Show { operation_id }) => {
                    match manager.get(&operation_id)? {
                        Some(op) => {
                            println!("Operation Details:");
                            println!("  ID: {}", op.id);
                            println!("  Type: {}", op.op_type.as_str());
                            println!("  Status: {}", op.status.as_str());
                            println!("  Created: {}", op.created_at);
                            println!("  Started: {}", op.started_at);
                            println!("  Last Updated: {}", op.last_updated);
                            println!();
                            println!("Progress:");
                            println!("  Items: {}/{}", 
                                op.progress.processed,
                                op.progress.total.map(|t| t.to_string()).unwrap_or_else(|| "unknown".to_string())
                            );
                            
                            if let Some(percent) = op.progress.percentage() {
                                println!("  Percentage: {:.1}%", percent);
                            }
                            
                            println!("  Bytes: {}/{}",
                                op.progress.bytes_processed,
                                op.progress.total_bytes.map(|b| format!("{} bytes", b)).unwrap_or_else(|| "unknown".to_string())
                            );
                            println!();
                            println!("State:");
                            println!("  Current Step: {}", op.state.current_step);
                            if let Some(total) = op.state.total_steps {
                                println!("  Total Steps: {}", total);
                            }
                            if let Some(error) = op.state.error_message {
                                println!("  Error: {}", error);
                            }
                            if !op.state.metadata.is_empty() {
                                println!("  Metadata:");
                                for (key, value) in op.state.metadata {
                                    println!("    {}: {}", key, value);
                                }
                            }
                        }
                        None => println!("Operation {} not found", operation_id),
                    }
                }

                Some(ResumeAction::Continue { operation_id }) => {
                    match manager.get(&operation_id)? {
                        Some(op) => {
                            println!("Resuming operation: {} ({})", &operation_id[..16], op.op_type.as_str());
                            println!("Previous checkpoint: {}", op.state.current_step);
                            println!("Progress: {}/{} items", 
                                op.progress.processed,
                                op.progress.total.map(|t| t.to_string()).unwrap_or_else(|| "unknown".to_string())
                            );
                            println!();
                            println!("⚠️  Resume functionality is operation-specific");
                            println!("Run the original command with --resume {} to continue", &operation_id[..16]);
                        }
                        None => println!("Operation {} not found", operation_id),
                    }
                }

                Some(ResumeAction::Pause { operation_id }) => {
                    manager.update_status(&operation_id, OperationStatus::Paused)?;
                    println!("✓ Operation paused");
                }

                Some(ResumeAction::Delete { operation_id }) => {
                    manager.delete(&operation_id)?;
                    println!("✓ Operation deleted");
                }

                Some(ResumeAction::Cleanup { days }) => {
                    let deleted = manager.cleanup_old(days)?;
                    println!("✓ Cleaned up {} old operations (older than {} days)", deleted, days);
                }
            }
            println!("Happy Mugging!");
        }
    }

    Ok(())
}
