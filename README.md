# 🥣 MUG - A Modern Version Control System

A fast, Rust-powered version control system that combines Git-like functionality with modern architecture.

> "Mug your files into history"

> [!NOTE]
> Fun fact:
> Mug means "fool" in british english

MUG is built on modern technologies:
- **Content-addressable store** using SHA256 hashes for automatic deduplication
- **Sled embedded database** for instant status and log operations
- **Serde serialization** for clean, type-safe data formats
- **Pure Rust** with zero external C dependencies
- **Zero tree-walking overhead** through hash-based indexing

## Features

MUG is a **hybrid VCS** combining distributed version control with novel research-driven features.

### Core Capabilities (35+ commands)

#### Repository Operations
- `mug init` - Initialize a new repository
- `mug add` - Stage files (add to index)
- `mug remove` - Unstage files
- `mug commit` - Create a commit
- `mug log` - View commit history
- `mug show` - Show commit details
- `mug status` - Show working tree status

#### Branching & Merging
- `mug branch` - Create branches
- `mug branches` - List all branches
- `mug checkout` - Switch branches
- `mug merge` - Merge branches with fast-forward detection
- `mug rebase` - Rebase current branch onto another (interactive or standard)

#### File Operations
- `mug rm` - Remove files
- `mug mv` - Move/rename files
- `mug restore` - Restore files
- `mug grep` - Parallel regex search across files

#### Commit History Control
- `mug reset` - Reset operations (soft/mixed/hard)
- `mug diff` - Show diffs between commits
- `mug cherry-pick` - Cherry-pick a commit onto current branch
- `mug cherry-pick-range` - Cherry-pick a range of commits
- `mug bisect-start` - Start a bisect session to find bad commits
- `mug bisect-good` - Mark commit as good during bisect
- `mug bisect-bad` - Mark commit as bad during bisect

#### Tag Management
- `mug tag` - Create annotated tags
- `mug tags` - List tags
- `mug delete-tag` - Delete tags

#### Stash Operations
- `mug stash` - Save work-in-progress
- `mug stash-list` - List stashes
- `mug stash-pop` - Apply and remove stash

#### Remote & Sync Operations
- `mug remote add` - Add remote repositories
- `mug remote list` - List remotes
- `mug remote remove` - Remove remotes
- `mug remote set-default` - Set default remote
- `mug remote update-url` - Update remote URLs
- `mug push` - Push to remote
- `mug pull` - Pull from remote
- `mug fetch` - Fetch from remote
- `mug clone` - Clone repository
- `mug serve` - Start HTTP server for remote operations

#### Hook System
- 6 hook types: pre-commit, post-commit, pre-push, post-push, pre-merge, post-merge
- Install hooks from scripts
- Hook execution with stdout/stderr capture
- Enable/disable hooks dynamically

#### Configuration & Metadata
- `.mugignore` - Pattern-based file exclusion with glob support
- `.mugattributes` - File attributes (merge strategy, line endings, diffs)
- `.mug/config.json` - Repository configuration

### Hybrid VCS Features

#### Git Migration
- `mug migrate <git-repo> <mug-repo>` - Import complete Git repositories
- Supports both loose objects and pack files
- Extracts full commit metadata (author, message, tree, parents)
- Converts Git branches and HEAD to MUG format
- Tested on repositories with 20+ commits

#### Cryptographic Signing
- `mug keys generate` - Create Ed25519 keypairs
- `mug keys import <seed>` - Import keys from base64 seed
- `mug keys list` - List all signing keys
- `mug keys current` - Show active key
- Immutable audit trail with Ed25519 signatures
- Prevents commit forgery and proves authorship

#### Temporal Branching (Non-Linear History)
- `mug temporal create <name> <commit>` - Create branch at any commit (not just tip)
- `mug temporal list` - Show all temporal branches
- `mug temporal show <branch>` - Visualize temporal branch structure with DAG
- `mug temporal merge <target> <source>` - Merge branches at any point in history
- Solves real-world problems: security patches, backports, multi-version support

#### Centralized Large File Storage
- `mug store set-server <url>` - Configure central object server
- `mug store set-threshold <MB>` - Set file size threshold (default 10MB)
- `mug store config` - Show current storage configuration
- `mug store cache-stats` - View LRU cache metrics
- `mug store clear-cache` - Clear local cache
- Hybrid approach: commits locally, large files on server
- LRU cache (1GB default) for transparent performance
- Solves distributed VCS bloat with binary files

## Quick Start

```bash
# Build from source
cargo build --release
./target/release/mug --help

# Initialize a repository
mug init

# Configure your identity
mug config set user.name "Your Name"
mug config set user.email "you@example.com"

# Make your first commit
mug add .
mug commit -m "Initial commit"
```

## Usage Examples

### Basic Workflow

```bash
mug init
mug add .
mug commit -m "Initial commit" -a "Your Name"
```

### Work with Branches and Rebase

```bash
mug branch feature/new-feature
mug checkout feature/new-feature
mug add .
mug commit -m "Add feature"
mug checkout main

# Standard rebase
mug rebase feature/new-feature

# Interactive rebase with TUI
mug rebase -i feature/new-feature
# Use j/k to navigate, p/s/r/d to choose actions, Enter to execute
```

### Push to a Remote

```bash
mug remote add origin https://example.com/repo.git
mug push -r origin -b main
```

### Stash Work and Switch Branches

```bash
mug stash
mug checkout other-branch
mug checkout main
mug stash-pop
```

### Use Hooks

```bash
echo "cargo fmt" > .mug/hooks/pre-commit
mug hook enable pre-commit
```

## Architecture

### Core Components

**Object Store** (`src/store.rs`)
- Content-addressable blob storage
- Tree snapshots for directory structures
- Automatic deduplication via SHA256

**Index** (`src/index.rs`)
- Staging area for changes
- Tracks file paths and their hashes
- Persisted to Sled database

**Commits** (`src/commit.rs`)
- Immutable commit objects
- Metadata: author, message, timestamp, parent
- Full history traversal support

**Branches** (`src/branch.rs`)
- Named references to commits
- HEAD management (attached/detached)
- Fast branch switching

**Remote & Sync** (`src/remote.rs`, `src/sync.rs`)
- Remote configuration and management
- Push/pull/fetch/clone operations
- Connection testing and URL validation

**Hook System** (`src/hooks.rs`)
- Automatic hook discovery
- Flexible execution model
- Enable/disable controls

**Ignore & Attributes** (`src/ignore.rs`, `src/attributes.rs`)
- Pattern-based file exclusion
- Per-file configuration attributes

**Database** (`src/database.rs`)
- Sled-backed lightweight embedded DB
- Separate trees for commits, branches, index, HEAD
- Flush-on-demand persistence

### Data Flow

```
Working Directory
       |
       v
   Hash (SHA256)
       |
       v
Object Store (content-addressable)
       |
       v
    Index (staging)
       |
       v
   Commit (immutable snapshot)
       |
       v
  Branch Ref (named pointer)
       |
       v
Sled Database (persistence)
```

## Performance

MUG is optimized for speed:

- **Status**: O(1) indexed lookup instead of O(n) tree walk
- **Commits**: Linked list traversal with minimal metadata
- **Storage**: Automatic deduplication via content addressing
- **Branches**: Instant creation and switching
- **Parallel operations**: Regex grep with Rayon parallelization

## Implementation Statistics

- **~3,600 lines** of well-documented Rust code
- **100+ unit tests** across 15 modules
- **26 feature modules** with comprehensive test coverage
- **Zero compiler warnings**

## Status

**Alpha 1 (Current)** - Stable with all core and hybrid features implemented

### Core Features
- ✅ Repository initialization
- ✅ Complete staging and commit workflow
- ✅ Branch creation, switching, and merging
- ✅ Interactive and standard rebase
- ✅ Remote operations (push, pull, fetch, clone)
- ✅ HTTP server mode for remote access
- ✅ Full hook system
- ✅ Tag management
- ✅ Stash operations
- ✅ Cherry-pick and bisect

### Hybrid VCS Features
- ✅ Git repository migration (loose objects + pack files)
- ✅ Cryptographic signing (Ed25519)
- ✅ Temporal branching (non-linear history)
- ✅ Centralized large file storage with LRU cache

## Design Philosophy

1. **Minimal metadata** - Only track what's necessary
2. **Content-addressed** - Use hashes as the source of truth
3. **Fast by default** - No tree walking, clever indexing
4. **Simple interface** - Clean commands, clear semantics
5. **Complete feature set** - All essential VCS operations
6. **Research-driven** - Novel branching, cryptographic signing, hybrid storage

## Documentation

- **FEATURE_SUMMARY.md** - Complete feature overview and CLI reference
- **HYBRID_VCS.md** - Architecture design decisions and innovation details
- **RESEARCH_VCS_MODELS.md** - VCS types analysis and comparison
- **MIGRATION_COMPLETE.md** - Git migration implementation details

## Development

Run tests:

```bash
cargo test
```

Build documentation:

```bash
cargo doc --open
```

Format code:

```bash
cargo fmt
```

Check for issues:

```bash
cargo clippy
```

## Security

If you discover a security vulnerability, please email atsharma623@gmail.com with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if available)

Do not open public issues for security vulnerabilities.

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Contributing

We welcome contributions! Here's how you can help:

### Getting Started
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit with clear messages
8. Push to your fork
9. Open a Pull Request

### Contribution Areas
- **Performance**: Optimization and caching improvements
- **Features**: Advanced merge/rebase, sparse checkout
- **Documentation**: Examples, tutorials, API docs
- **Testing**: Additional test coverage and edge cases
- **Bug Fixes**: Issues marked as help-wanted

## License

This project is licensed under the **GNU Lesser General Public License v3.0** - see [LICENSE](LICENSE) file for details.

By contributing, you agree that your contributions will be licensed under its LGPL v3 License.
