# MUG - Complete Documentation

A fast, Rust-powered version control system that pokes fun at itself.

## Quick Start

### Installation
```bash
cargo install --path .
# or
cargo build --release
./target/release/mug --help
```

### Initialize Repository
```bash
mug init
```

Creates `.mug/` directory with:
- `.mug/db/` - Embedded Sled database
- `.mug/objects/` - Object storage
- `.mugignore` - Default ignore patterns
- Default `main` branch

### Basic Workflow
```bash
# Stage files
mug add .

# Check status
mug status

# Commit changes
mug commit -m "Initial commit" --author "Your Name"

# View history
mug log
mug log --oneline

# Show commit details
mug show <commit-id>
```

## Commands

### Repository
- `init [path]` - Initialize a new repository
- `status` - Show working directory status
- `log [--oneline]` - Show commit history

### Staging
- `add <path>` - Stage files (use "." for all)
- `remove <path>` - Unstage files
- `rm <paths...>` - Remove and unstage files
- `mv <from> <to>` - Move/rename files
- `restore <paths...>` - Restore files to HEAD state

### Commits
- `commit -m <msg> [--author <name>]` - Create commit
- `show <commit>` - Display commit details
- `reset [soft|mixed|hard] [commit]` - Reset to commit

### Branches
- `branch <name>` - Create branch
- `branches` - List all branches
- `checkout <branch>` - Switch branch
- `merge <branch>` - Merge branch into current

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

## Hooks

MUG supports custom hooks for automating workflows. Hooks are stored in `.mug/hooks/`.

### Hook Types

- **pre-commit** - Runs before creating a commit (e.g., lint checks, formatting)
- **post-commit** - Runs after commit creation (e.g., notifications)
- **pre-push** - Runs before pushing to remote (e.g., validation)
- **post-push** - Runs after push completes (e.g., cleanup)
- **pre-merge** - Runs before merging branches (e.g., conflict detection)
- **post-merge** - Runs after merge completes (e.g., dependency updates)

### Hook File Format

Hooks are executable scripts with naming convention: `<hook-type>-<name>`

Example: `.mug/hooks/pre-commit-lint`

```bash
#!/bin/bash
# Run linters on staged files
cargo clippy
cargo fmt --check
```

### Hook Management

Hooks are automatically discovered from `.mug/hooks/`. You can:
- Create hooks manually as executable scripts
- Install hooks programmatically via HookManager
- Disable hooks by renaming with `.disabled` suffix
- List all active hooks with `hook list` (future CLI command)

### Execution

Hooks are triggered automatically at the appropriate lifecycle points:
- `pre-commit` triggers before `mug commit`
- `post-commit` triggers after successful commit
- `pre-merge` triggers before `mug merge`
- And so on...

Hooks that fail will prevent the operation (strict mode).

## Architecture

### Core Modules

**database.rs** - Sled-backed key-value store
- Trees: HEAD, BRANCHES, INDEX, COMMITS, REMOTES
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
- Merge operations

**store.rs** - Object storage
- Blob storage for file contents
- Tree storage for directory structures
- Content-addressed objects

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
- Merge strategies (ours, theirs, union, binary)
- Export ignore marking

**remote.rs** - Remote repository management
- Protocol detection (HTTP, HTTPS, SSH)
- Remote configuration storage
- Default remote tracking

**hash.rs** - SHA-1 hashing
- File content hashing
- Git-compatible hash format
- Short hash generation

### File Structure
```
.mug/
├── db/                 # Sled database (persisted)
│   ├── data.db
│   └── *.log
├── objects/           # Object storage (git-like)
│   ├── [hash]/content
│   └── [hash]/content
└── config.json        # Repository config

.mugignore           # Ignore patterns
.mugattributes       # File attributes
```

## Configuration

### .mugignore Patterns
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

### .mugattributes
```
# Pattern  [key=value]...

*.bin line_ending=binary diff=binary merge=binary
*.jpg line_ending=binary diff=binary
* export-ignore      # Exclude from exports
```

### config.json
```json
{
  "user_name": "John Doe",
  "user_email": "john@example.com",
  "default_branch": "main",
  "custom_key": "custom_value"
}
```

## Remote Operations

### Setup
```bash
# Add remote
mug remote add origin https://github.com/user/repo

# List remotes
mug remote list

# Set default
mug remote set-default origin
```

### Push/Pull
```bash
# Pull changes
mug pull origin main

# Push changes
mug push origin main

# Push to default remote
mug push
```

## Advanced Usage

### Commit Modes
```bash
# Soft reset - keep changes staged
mug reset soft <commit>

# Mixed reset - keep changes unstaged (default)
mug reset mixed <commit>

# Hard reset - discard changes
mug reset hard <commit>
```

### Search
```bash
# Parallel grep across files
mug grep "TODO"
mug grep "function_name" src/
```

### File Operations
```bash
# Move/rename file
mug mv old.rs new.rs

# Remove file
mug rm deleted.rs

# Restore file to HEAD state
mug restore path/to/file.rs
```

## Performance Characteristics

- **Index operations** - O(1) lookups, O(n) scans
- **Status** - O(n) file walks with parallelization
- **Commit** - O(n) tree building
- **Search** - Parallel with rayon
- **Storage** - Append-only objects, no repacking

## Testing

Run all tests:
```bash
cargo test --lib
```

Test coverage:
- 42 tests across all modules
- Index: validation, sorting, persistence
- Config: defaults, save/load
- Attributes: pattern matching, parsing
- Ignore: glob patterns, negation
- And more...

## What's Ready

✅ Repository initialization
✅ Basic add/commit/log workflow
✅ Branch creation and switching/merging
✅ Status tracking with ignore support
✅ File attribute management (.mugattributes)
✅ Configuration management (.mugconfig)
✅ Parallel file search with regex
✅ Index staging area with validation
✅ File operations (move, remove, restore)
✅ Commit browsing and diffs
✅ Reset with soft/mixed/hard modes
✅ Tag creation, listing, deletion
✅ Stash: save, list, pop work-in-progress
✅ Simple merge (fast-forward detection)
✅ **Full hook system** (6 hook types)
✅ **Remote management** (add/remove/list/update)
✅ **Sync operations** (push/pull/fetch/clone)

## What's NOT Implemented

These features are not yet available:
- Network transport (currently simulated)
- Interactive rebase
- Conflict resolution (three-way merge algorithm)
- Cherry-pick
- Bisect
- Submodules
- Signed commits

## Architecture Decisions

- **Sled** for persistence - fast, ordered, embedded
- **SHA-1** hashing - git-compatible
- **Object storage** - content-addressed, deduplication
- **Rayon** for parallelism - data-parallel operations
- **Clap** for CLI - ergonomic command parsing
- **Thiserror** for error handling - structured errors

## Compatibility

- Rust 2024 edition
- All dependencies latest versions
- No breaking changes to APIs
- Backward compatible with existing repos

## Development

```bash
# Build
cargo build

# Release
cargo build --release

# Test
cargo test --lib

# Lint
cargo clippy

# Format
cargo fmt
```

## Contributing

Current gaps for production:
1. Complete remote push/pull
2. Implement merge conflict resolution
3. Add interactive rebase
4. Full test coverage for complex scenarios
5. Performance benchmarking

See code comments and TODOs for specific implementation notes.
