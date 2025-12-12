# MUG - Complete Documentation

## Installation

```bash
cargo install --path .
# or
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

### Repository Operations
- `init [path]` - Initialize a new repository
- `status` - Show working directory status
- `log [--oneline]` - Show commit history
- `verify` - Verify repository integrity
- `gc` - Garbage collection

### Staging
- `add <path>` - Stage files (use "." for all)
- `remove <path>` - Unstage files
- `rm <paths...>` - Remove and unstage files
- `mv <from> <to>` - Move/rename files
- `restore <paths...>` - Restore files to HEAD state

### Commits
- `commit -m <msg> [--author <name>] [-a <author>]` - Create commit
- `show <commit>` - Display commit details
- `reset [soft|mixed|hard] [commit]` - Reset to commit

### Branches
- `branch <name>` - Create branch
- `branches` - List all branches
- `checkout <branch>` - Switch branch
- `merge <branch>` - Merge branch into current
- `rebase <branch>` - Rebase current onto branch
- `rebase -i <branch>` - Interactive rebase

### Tags
- `tag <name> [-m <msg>]` - Create tag
- `tags` - List all tags
- `delete-tag <name>` - Delete tag

### Stash
- `stash [-m <msg>]` - Stash changes
- `stash-pop` - Apply and remove stash
- `stash-list` - List all stashes

### Search & Utilities
- `grep <pattern>` - Parallel search in files (regex support)
- `diff [--from <commit>] [--to <commit>]` - Show diff between commits

### Commit History Control
- `cherry-pick <commit>` - Cherry-pick a commit
- `cherry-pick-range <start> <end>` - Cherry-pick range
- `bisect-start` - Start bisect session
- `bisect-good` - Mark current as good
- `bisect-bad` - Mark current as bad

### Configuration
- `config set <key> <value>` - Set config value
- `config get <key>` - Get config value
- `config list` - List all config

### Remote Operations
- `remote add <name> <url>` - Add remote repository
- `remote list` - List all remotes
- `remote remove <name>` - Remove remote
- `remote set-default <name>` - Set default remote
- `remote update-url <name> <url>` - Change remote URL
- `push [remote] [branch]` - Push to remote (default: origin/main)
- `pull [remote] [branch]` - Pull from remote (default: origin/main)
- `fetch [remote]` - Fetch from remote without merging
- `clone <url> [destination]` - Clone remote repository

### Server Mode
- `serve --host <addr> --port <port> --repos <path>` - Start HTTP server
- `migrate <git-repo> <mug-repo>` - Migrate from Git

## Hooks System

Hooks are stored in `.mug/hooks/` and triggered automatically.

### Hook Types

- **pre-commit** - Runs before creating a commit
- **post-commit** - Runs after commit creation
- **pre-push** - Runs before pushing to remote
- **post-push** - Runs after push completes
- **pre-merge** - Runs before merging branches
- **post-merge** - Runs after merge completes

### Creating Hooks

```bash
# Create a pre-commit hook
cat > .mug/hooks/pre-commit << 'EOF'
#!/bin/bash
# Run linters on staged files
cargo clippy
cargo fmt --check
EOF

chmod +x .mug/hooks/pre-commit
```

Hooks are automatically discovered and executed at the appropriate lifecycle points.

## Configuration System

### User Identity

```bash
mug config set user.name "John Doe"
mug config set user.email "john@example.com"
```

### Custom Configuration

```bash
mug config set custom_key custom_value
mug config get custom_key
mug config list
```

Configuration is persisted in `.mug/config.json`.

## File Exclusion (.mugignore)

Create a `.mugignore` file in the repository root:

```
# Comments start with #
*.log              # Ignore all log files
node_modules/      # Ignore directory and contents
target/            # Rust build artifacts
*.tmp              # Temporary files
!important.log     # Re-include specific files

# Directory patterns
**/node_modules    # Ignore at any depth
venv/              # Python virtual env
__pycache__/       # Python cache
```

## File Attributes (.mugattributes)

Create a `.mugattributes` file in the repository root:

```
# Pattern  [key=value]...

*.bin line_ending=binary diff=binary merge=binary
*.jpg line_ending=binary diff=binary
* export-ignore      # Exclude from exports
```

Supported attributes:
- `line_ending` - auto, lf, crlf, binary
- `diff` - text, binary
- `merge` - text, ours, theirs, union, binary
- `export-ignore` - exclude from exports

## Remote Operations

### Setup

```bash
# Add remote
mug remote add origin https://github.com/user/repo

# List remotes
mug remote list

# Set default
mug remote set-default origin

# Update URL
mug remote update-url origin https://github.com/user/newrepo
```

### Sync

```bash
# Pull changes
mug pull origin main

# Push changes
mug push origin main

# Push to default remote
mug push

# Fetch without merging
mug fetch origin
```

## Server Mode

Start an HTTP server to serve repositories:

```bash
mug serve --host 0.0.0.0 --port 8080 --repos /path/to/repos
```

Then clone from another machine:

```bash
mug clone http://server-ip:8080/repo-name local-copy
```

## Git Migration

Import a Git repository into MUG:

```bash
mug migrate /path/to/git/repo /path/to/new/mug/repo
```

This preserves:
- All commit history
- Branch references
- Tags
- Author information

## Architecture

### Core Modules

**database.rs** - Sled-backed key-value store
- Trees: HEAD, BRANCHES, INDEX, COMMITS, REMOTES, STASH, TAGS
- Persistent storage layer

**index.rs** - Staging area management
- Track staged files with hashes and modes
- Methods: add, remove, get, entries, clear, find
- Input validation and error handling

**status.rs** - Working directory analysis
- Compare staged vs working directory
- Respects .mugignore patterns
- File status tracking: Added, Modified, Deleted, Untracked

**commit.rs** - Commit history and storage
- Immutable commit records
- Author, message, timestamp metadata
- Linked commit chain

**branch.rs** - Branch management
- Create, list, switch branches
- HEAD reference tracking
- Merge operations with conflict detection

**store.rs** - Object storage
- Blob storage for file contents
- Tree storage for directory structures
- Content-addressed objects using SHA256

**ignore.rs** - Pattern-based file exclusion
- .mugignore support
- Glob patterns: *.ext, dir/, **/pattern, !negated
- Integration with status scanning

**config.rs** - Repository configuration
- User identity (name, email)
- Branch defaults
- Custom key-value settings
- JSON persistence

**attributes.rs** - File attributes
- .mugattributes support
- Line-ending handling (auto, lf, crlf, binary)
- Merge strategies
- Export ignore marking

**remote.rs** - Remote repository management
- Protocol detection (HTTP, HTTPS, SSH)
- Remote configuration storage
- Default remote tracking

**hash.rs** - SHA-256 hashing
- File content hashing
- Short hash generation
- Content addressing

### File Structure

```
.mug/
├── db/                 # Sled database (persisted)
│   ├── data.db
│   └── *.log
├── objects/           # Object storage
│   ├── [hash]/content
│   └── [hash]/content
├── hooks/             # Custom hooks
│   ├── pre-commit
│   └── post-commit
└── config.json        # Repository config

.mugignore           # Ignore patterns
.mugattributes       # File attributes
```

## Performance Characteristics

- **Index operations** - O(1) lookups, O(n) scans
- **Status** - O(n) file walks with parallelization
- **Commit** - O(n) tree building
- **Search** - Parallel with rayon
- **Storage** - Append-only objects, no repacking

## Reset Modes

```bash
# Soft reset - keep changes staged
mug reset soft <commit>

# Mixed reset - keep changes unstaged (default)
mug reset mixed <commit>

# Hard reset - discard changes
mug reset hard <commit>
```

## Testing

Run all tests:

```bash
cargo test --lib
```

Run specific test:

```bash
cargo test -- --exact test_name
```

## Troubleshooting

### Repository seems corrupt

```bash
# Verify repository integrity
mug verify

# Check status
mug status

# View recent commits
mug log --oneline -n 20
```

### Can't push/pull

```bash
# Check remotes
mug remote list

# Test remote connectivity
mug fetch origin

# Check default remote
mug config get default_remote
```

### Merge conflicts

```bash
# Check status to see conflicts
mug status

# View diff to understand conflict
mug diff

# Abort merge and try rebase instead
mug checkout main
mug rebase feature-branch
```

## Best Practices

1. **Commit frequently** - Small, focused commits are easier to rebase/bisect
2. **Use meaningful messages** - Helps with history navigation
3. **Create branches for features** - Keep main clean
4. **Use interactive rebase** - Clean up commit history before pushing
5. **Set up hooks** - Automate checks and formatting
6. **Keep .mugignore updated** - Prevents committing build artifacts

## Advanced Patterns

### Feature Branch Workflow

```bash
mug checkout main
mug branch feature/my-feature
mug checkout feature/my-feature

# Make commits
mug add .
mug commit -m "Work"

# Update from main
mug fetch origin
mug rebase origin/main

# Merge into main
mug checkout main
mug merge feature/my-feature
mug push origin main
```

### Interactive Rebase Cleanup

```bash
# Before pushing, clean up commits
mug rebase -i main

# Pick first commit, squash/fixup subsequent ones
# Reword commits as needed
```

### Bisect to Find Bad Commit

```bash
mug bisect-start
mug bisect-bad              # Mark current as bad
mug checkout v1.0           # Checkout known good version
mug bisect-good

# MUG will search between good/bad
mug status                  # Test the code
mug bisect-good             # or bisect-bad

# Repeat until found
```

## Limitations

- **Network Transport**: Currently simulated (no actual HTTP/S or SSH calls)
- **Three-Way Merge**: Simplified conflict detection only
- **Signing**: No commit signing support
- **Submodules**: Not implemented
- **Worktrees**: Not implemented

## Getting Help

```bash
mug --help                      # Overall help
mug <command> --help            # Command-specific help
```

See [QUICK_START.md](QUICK_START.md) for simple examples.
