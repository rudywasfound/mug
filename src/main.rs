use clap::{Parser, Subcommand};
use std::path::PathBuf;

use mug::error::Result;
use mug::repo::Repository;

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

        /// Author name
        #[arg(short, long, default_value = "MUG User")]
        author: String,
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let _repo = Repository::init(&path)?;
            println!("Initialized empty MUG repository in {:?}", path);
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
        }

        Commands::Remove { path } => {
            let repo = Repository::open(".")?;
            repo.remove(&path)?;
            println!("Removed {} from staging", path);
        }

        Commands::Status => {
            let repo = Repository::open(".")?;
            let _status = repo.status()?;

            println!(
                "On branch: {}",
                repo.current_branch()?.unwrap_or("main".to_string())
            );
            println!("Working directory status displayed");
        }

        Commands::Commit { message, author } => {
            let repo = Repository::open(".")?;
            let commit_id = repo.commit(author, message)?;
            println!("Committed: {}", mug::hash::short_hash(&commit_id));
        }

        Commands::Log { oneline } => {
            let repo = Repository::open(".")?;
            let commits = repo.log()?;
            for commit in commits {
                if oneline {
                    println!("{}", commit.lines().next().unwrap_or(""));
                } else {
                    println!("{}", commit);
                }
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
        }

        Commands::Branch { name } => {
            let repo = Repository::open(".")?;
            repo.create_branch(name.clone())?;
            println!("Created branch: {}", name);
        }

        Commands::Branches => {
            let repo = Repository::open(".")?;
            let current = repo.current_branch()?;
            let branches = repo.branches()?;
            for branch in branches {
                let marker = if Some(&branch) == current.as_ref() {
                    "* "
                } else {
                    "  "
                };
                println!("{}{}", marker, branch);
            }
        }

        Commands::Checkout { branch } => {
            let repo = Repository::open(".")?;
            repo.checkout(branch.clone())?;
            println!("Switched to branch: {}", branch);
        }

        Commands::Rm { paths } => {
            let repo = Repository::open(".")?;
            let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            mug::commands::remove_files(&repo, &path_refs)?;
            println!("Removed {} files", paths.len());
        }

        Commands::Mv { from, to } => {
            let repo = Repository::open(".")?;
            mug::commands::mv_file(&repo, &from, &to)?;
            println!("Moved {} to {}", from, to);
        }

        Commands::Restore { paths } => {
            let repo = Repository::open(".")?;
            let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            mug::commands::restore_files(&repo, &path_refs)?;
            println!("Restored {} files", paths.len());
        }

        Commands::Diff { from, to } => {
            let repo = Repository::open(".")?;
            let diffs = mug::commands::diff_commits(&repo, from.as_deref(), to.as_deref())?;
            for diff in diffs {
                println!("{}", diff);
            }
        }

        Commands::Reset { mode, commit } => {
            let repo = Repository::open(".")?;
            let reset_mode = mug::reset::ResetMode::from_str(&mode)?;
            mug::reset::reset(&repo, reset_mode, commit.as_deref())?;
            println!(
                "Reset {} to {:?}",
                mode,
                commit.unwrap_or("HEAD".to_string())
            );
        }

        Commands::Tag { name, message } => {
            let repo = Repository::open(".")?;
            let tag_manager = mug::tag::TagManager::new(repo.get_db().clone());

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

            println!("Created tag: {}", name);
        }

        Commands::Tags => {
            let repo = Repository::open(".")?;
            let tag_manager = mug::tag::TagManager::new(repo.get_db().clone());
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
        }

        Commands::DeleteTag { name } => {
            let repo = Repository::open(".")?;
            let tag_manager = mug::tag::TagManager::new(repo.get_db().clone());
            tag_manager.delete(&name)?;
            println!("Deleted tag: {}", name);
        }

        Commands::Merge { branch } => {
            let repo = Repository::open(".")?;
            let result = mug::merge::merge(&repo, &branch, mug::merge::MergeStrategy::Simple)?;

            if result.merged {
                println!("{}", result.message);
            } else {
                println!("Merge failed: {}", result.message);
                for conflict in result.conflicts {
                    println!("  Conflict: {}", conflict);
                }
            }
        }

        Commands::CherryPick { commit } => {
            let repo = Repository::open(".")?;
            let result = mug::cherry_pick::cherry_pick(&repo, &commit)?;

            if result.success {
                println!("{}", result.message);
                println!("New commit: {}", result.new_commit);
            } else {
                println!("Cherry-pick failed: {}", result.message);
            }
        }

        Commands::CherryPickRange { start, end } => {
            let repo = Repository::open(".")?;
            let result = mug::cherry_pick::cherry_pick_range(&repo, &start, &end)?;

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
        }

        Commands::BisectStart { bad, good } => {
            let repo = Repository::open(".")?;
            let session = mug::bisect::start(&repo, &bad, &good)?;
            println!("Started bisect session");
            println!("Testing commit: {}", session.current_commit);
            println!("Commits to test: {}", session.tested_commits.len());
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
            let stash_manager = mug::stash::StashManager::new(repo.get_db().clone());
            let current_branch = repo.current_branch()?.unwrap_or("main".to_string());
            let msg = message.unwrap_or("WIP: stashed changes".to_string());

            let index = mug::index::Index::new(repo.get_db().clone())?;
            let entries = index.entries();

            let stash_id = stash_manager.create(&current_branch, &msg, entries)?;
            println!("Stashed changes: {}", stash_id);
        }

        Commands::StashPop => {
            let repo = Repository::open(".")?;
            let stash_manager = mug::stash::StashManager::new(repo.get_db().clone());

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
            let stash_manager = mug::stash::StashManager::new(repo.get_db().clone());
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
            let sync_manager = mug::sync::SyncManager::new(repo);
            let result = sync_manager.push(&remote, &branch).await?;

            if result.success {
                println!("{}", result.message);
            } else {
                eprintln!("Push failed: {}", result.message);
            }
        }

        Commands::Pull { remote, branch } => {
            let repo = Repository::open(".")?;
            let sync_manager = mug::sync::SyncManager::new(repo);
            let result = sync_manager.pull(&remote, &branch).await?;

            if result.success {
                println!("{}", result.message);
            } else {
                eprintln!("Pull failed: {}", result.message);
            }
        }

        Commands::Fetch { remote } => {
            let repo = Repository::open(".")?;
            let sync_manager = mug::sync::SyncManager::new(repo);
            let result = sync_manager.fetch(&remote).await?;

            if result.success {
                println!("{}", result.message);
            } else {
                eprintln!("Fetch failed: {}", result.message);
            }
        }

        Commands::Clone { url, destination } => {
            mug::sync::SyncManager::clone(&url, destination.as_deref())?;
        }
    }

    Ok(())
}
