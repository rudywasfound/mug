# MUG Documentation

## Installation

```bash
cargo install --path .
cargo build --release
./target/release/mug --help
```

## Repository Initialization

```bash
mug init
```

Creates `.mug/` directory with:
- `.mug/db/` - Embedded Sled database
- `.mug/objects/` - Object storage
- `.mugignore` - Default ignore patterns
- Default `main` branch

## Commands Reference

Repository Operations:
- `init [path]` - Initialize a new repository
- `status` - Show working directory status
- `log [--oneline]` - Show commit history
- `verify` - Verify repository integrity
- `gc` - Garbage collection

Staging:
- `add <path>` - Stage files (use "." for all)
- `remove <path>` - Unstage files
- `rm <paths...>` - Remove and unstage files
- `mv <from> <to>` - Move/rename files
- `restore <paths...>` - Restore files to HEAD state

Commits:
- `commit -m <msg> [--author <name>] [-a <author>]` - Create commit
- `show <commit>` - Display commit details
- `reset [soft|mixed|hard] [commit]` - Reset to commit

Branches:
- `branch <name>` - Create branch
- `branches` - List all branches
- `checkout <branch>` - Switch branch
- `merge <branch>` - Merge branch into current
- `rebase <branch>` - Rebase current onto branch
- `rebase -i <branch>` - Interactive rebase

Tags:
- `tag <name> [-m <msg>]` - Create tag
- `tags` - List all tags
- `delete-tag <name>` - Delete tag

Stash:
- `stash [-m <msg>]` - Stash changes
- `stash-pop` - Apply and remove stash
- `stash-list` - List all stashes

Search & Utilities:
- `grep <pattern>` - Parallel search in files (regex support)
- `diff [--from <commit>] [--to <commit>]` - Show diff between commits

Commit History Control:
- `cherry-pick <commit>` - Cherry-pick a commit
- `cherry-pick-range <start> <end>` - Cherry-pick range
- `bisect-start` - Start bisect session
- `bisect-good` - Mark current as good
- `bisect-bad` - Mark current as bad

Configuration:
- `config set <key> <value>` - Set config value
- `config get <key>` - Get config value
- `config list` - List all config

Remote Operations:
- `remote add <name> <url>` - Add remote repository
- `remote list` - List all remotes
- `remote remove <name>` - Remove remote
- `remote set-default <name>` - Set default remote
- `remote update-url <name> <url>` - Change remote URL
- `push [remote] [branch]` - Push to remote (default: origin/main)
- `pull [remote] [branch]` - Pull from remote (default: origin/main)
- `fetch [remote]` - Fetch from remote without merging
- `clone <url> [destination]` - Clone remote repository

Server Mode:
- `serve --host <addr> --port <port> --repos <path>` - Start HTTP server
- `migrate <git-repo> <mug-repo>` - Migrate from Git

## Hooks System

Hooks are stored in `.mug/hooks/` and triggered automatically.

Hook Types:
- pre-commit - Runs before creating a commit
- post-commit - Runs after a commit is created
- pre-push - Runs before pushing to remote
- post-push - Runs after pushing to remote
- pre-merge - Runs before merging branches
- post-merge - Runs after merging branches

Create a hook:
```bash
echo "cargo fmt" > .mug/hooks/pre-commit
chmod +x .mug/hooks/pre-commit
```

Enable/disable hooks:
```bash
mug hook enable pre-commit
mug hook disable pre-commit
mug hook list
```

## Configuration Files

### .mugignore

Pattern-based file exclusion using glob patterns.

```
target/
*.log
.DS_Store
build/
```

### .mugattributes

Per-file configuration for merge strategies and attributes.

```
*.json merge=union
*.txt diff=custom
*.binary binary
```

### .mug/config.json

Repository-specific configuration.

```json
{
  "user.name": "Your Name",
  "user.email": "you@example.com"
}
```

## Workflows

### Basic Workflow

```bash
mug init
mug config set user.name "Your Name"
mug add .
mug commit -m "Initial commit"
```

### Branch Workflow

```bash
mug branch feature/new-feature
mug checkout feature/new-feature
mug add .
mug commit -m "Add feature"
mug checkout main
mug merge feature/new-feature
```

### Interactive Rebase

```bash
mug rebase -i main
# j/k - navigate
# p/s/r/d - choose actions
# Enter - execute
```

### Cherry-pick

```bash
mug cherry-pick abc1234
mug cherry-pick-range abc1234 def5678
```

### Bisect

```bash
mug bisect-start --bad abc1234 --good def5678
mug bisect-good
mug bisect-bad
```

### Stash

```bash
mug stash -m "WIP: feature in progress"
mug checkout other-branch
mug checkout main
mug stash-pop
```

### Remote Operations

```bash
mug remote add origin https://example.com/repo.git
mug push origin main
mug pull origin main
mug fetch origin
```

### Clone Repository

```bash
mug clone https://example.com/repo.git
mug clone https://example.com/repo.git my-repo
```

### Git Migration

```bash
mug migrate /path/to/git/repo /path/to/mug/repo
```

### Cryptographic Signing

```bash
mug keys generate
mug keys import <seed>
mug keys list
mug keys current
```

### Temporal Branching

```bash
mug temporal create backport-v1.0 abc1234
mug temporal list
mug temporal show backport-v1.0
mug temporal merge main backport-v1.0
```

### Large File Storage

```bash
mug store set-server https://store.example.com
mug store set-threshold 10
mug store config
mug store cache-stats
mug store clear-cache
```

### Pack Files

```bash
mug pack create ./output
mug pack stats ./pack-file.pack
mug pack dedup
mug pack verify ./manifest.json
```

## Error Messages

NoCommits - No commits have been created yet
BranchNotFound - The specified branch does not exist
CommitNotFound - The commit ID was not found
Conflicts - Working directory has conflicts
NotARepository - Not a valid MUG repository
Database - Internal database error
Custom - Custom error message

## Troubleshooting

Check repository status:
```bash
mug status
```

Verify repository integrity:
```bash
mug verify
```

Clean up repository:
```bash
mug gc
```

Show reflog:
```bash
mug reflog
```

Reset to a previous state:
```bash
mug reset hard HEAD~1
```

## Performance Tips

Use parallelized grep for large repositories:
```bash
mug grep "pattern"
```

Use shallow clones for large projects:
```bash
mug clone --depth 1 <url>
```

Enable compression in pack files:
```bash
mug pack create ./output
```

Use large file storage for binary files:
```bash
mug store set-threshold 5
```

## Advanced Topics

### Object Store

Content-addressable blob storage using SHA256 hashes. Objects are automatically deduplicated.

### Index

Staging area for changes. Tracked in Sled database under `index` tree.

### Commit Log

Immutable commit objects with full history support. Stored in `commits` tree.

### Branch References

Named pointers to commits. Stored in `branches` tree with HEAD management.

### Remote Configuration

Remotes are stored in `.mug/config.json` with URL validation.

### Hook Execution

Hooks execute in a subprocess with full stdout/stderr capture.

## API Reference

See generated documentation with:
```bash
cargo doc --open
```

## Contributing

See CONTRIBUTING.md for contribution guidelines.

## License

GNU Lesser General Public License v3.0
