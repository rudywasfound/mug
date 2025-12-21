<h1 align="center">MUG – Version Control System</h1>
<p align="center">
  <img src="https://github.com/rudywasfound/mug/blob/master/mug-vcs.png?raw=true" alt="Mug VCS Logo" width="400"/>
</p>

<p align="center">
  A Rust-powered version control system with Git-like functionality and modern architecture.
</p>
<p align="center">
  <img src="https://img.shields.io/badge/version-1.0.0--alpha.1-blue" alt="Version">
  <img src="https://img.shields.io/badge/release-december-brightgreen">
  <img src="https://img.shields.io/badge/license-LGPL--3.0-green" alt="License">
</p>


## Features

MUG is a hybrid VCS combining distributed version control with centralized features!

### Core Commands (35+ commands)

Repository Operations:
- `mug init` - Initialize a new repository
- `mug add` - Stage files
- `mug remove` - Unstage files
- `mug commit` - Create a commit
- `mug log` - View commit history
- `mug show` - Show commit details
- `mug status` - Show working tree status

Branching & Merging:
- `mug branch` - Create branches
- `mug branches` - List all branches
- `mug checkout` - Switch branches
- `mug merge` - Merge branches with fast-forward detection
- `mug rebase` - Rebase current branch onto another

File Operations:
- `mug rm` - Remove files
- `mug mv` - Move/rename files
- `mug restore` - Restore files
- `mug grep` - Regex search across files

Commit History Control:
- `mug reset` - Reset operations (soft/mixed/hard)
- `mug diff` - Show diffs between commits
- `mug cherry-pick` - Cherry-pick a commit onto current branch
- `mug cherry-pick-range` - Cherry-pick a range of commits
- `mug bisect-start` - Start a bisect session
- `mug bisect-good` - Mark commit as good during bisect
- `mug bisect-bad` - Mark commit as bad during bisect

Tag Management:
- `mug tag` - Create annotated tags
- `mug tags` - List tags
- `mug delete-tag` - Delete tags

Stash Operations:
- `mug stash` - Save work-in-progress
- `mug stash-list` - List stashes
- `mug stash-pop` - Apply and remove stash

Remote & Sync Operations:
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

Hook System:
- 6 hook types: pre-commit, post-commit, pre-push, post-push, pre-merge, post-merge
- Install hooks from scripts
- Hook execution with stdout/stderr capture
- Enable/disable hooks dynamically

Configuration & Metadata:
- `.mugignore` - Pattern-based file exclusion
- `.mugattributes` - File attributes
- `.mug/config.json` - Repository configuration

### Hybrid VCS Features

Git Migration:
- `mug migrate <git-repo> <mug-repo>` - Import Git repositories
- Supports both loose objects and pack files
- Extracts full commit metadata
- Converts Git branches and HEAD to MUG format

Cryptographic Signing:
- `mug keys generate` - Create Ed25519 keypairs
- `mug keys import <seed>` - Import keys from base64 seed
- `mug keys list` - List all signing keys
- `mug keys current` - Show active key
- Immutable audit trail with Ed25519 signatures

Temporal Branching:
- `mug temporal create <name> <commit>` - Create branch at any commit
- `mug temporal list` - Show all temporal branches
- `mug temporal show <branch>` - Visualize temporal branch structure
- `mug temporal merge <target> <source>` - Merge branches at any point

Centralized Large File Storage:
- `mug store set-server <url>` - Configure central object server
- `mug store set-threshold <MB>` - Set file size threshold
- `mug store config` - Show current storage configuration
- `mug store cache-stats` - View LRU cache metrics
- `mug store clear-cache` - Clear local cache

## Quick Start

```bash
cargo build --release
./target/release/mug --help

mug init
mug config set user.name "Your Name"
mug config set user.email "you@example.com"
mug add .
mug commit -m "Initial commit"
```

## Usage Examples

Basic Workflow:
```bash
mug init
mug add .
mug commit -m "Initial commit" -a "Your Name"
```

Work with Branches:
```bash
mug branch feature/new-feature
mug checkout feature/new-feature
mug add .
mug commit -m "Add feature"
mug checkout main
mug rebase feature/new-feature
```

Push to Remote:
```bash
mug remote add origin https://example.com/repo.git
mug push -r origin -b main
```

Stash Work:
```bash
mug stash
mug checkout other-branch
mug checkout main
mug stash-pop
```

Use Hooks:
```bash
echo "cargo fmt" > .mug/hooks/pre-commit
mug hook enable pre-commit
```

## Architecture

Object Store:
- Content-addressable blob storage
- Tree snapshots for directory structures
- Automatic deduplication via SHA256

Index:
- Staging area for changes
- Tracks file paths and their hashes
- Persisted to Sled database

Commits:
- Immutable commit objects
- Metadata: author, message, timestamp, parent
- Full history traversal support

Branches:
- Named references to commits
- HEAD management
- Fast branch switching

Remote & Sync:
- Remote configuration and management
- Push/pull/fetch/clone operations
- Connection testing and URL validation

Hook System:
- Automatic hook discovery
- Flexible execution model
- Enable/disable controls

Ignore & Attributes:
- Pattern-based file exclusion
- Per-file configuration attributes

Database:
- Sled-backed lightweight embedded DB
- Separate trees for commits, branches, index, HEAD
- Flush-on-demand persistence

## Performance

- Status: O(1) indexed lookup instead of O(n) tree walk
- Commits: Linked list traversal with minimal metadata
- Storage: Automatic deduplication via content addressing
- Branches: Instant creation and switching
- Parallel operations: Regex grep with Rayon parallelization

## Implementation Statistics

- ~3,600 lines of Rust code
- 100+ unit tests across 15 modules
- 26 feature modules with comprehensive test coverage
- Zero compiler warnings

## Status

Alpha 1 (Current) - Stable with all core and hybrid features implemented

Core Features:
- Repository initialization
- Complete staging and commit workflow
- Branch creation, switching, and merging
- Interactive and standard rebase
- Remote operations (push, pull, fetch, clone)
- HTTP server mode for remote access
- Full hook system
- Tag management
- Stash operations
- Cherry-pick and bisect

Hybrid VCS Features:
- Git repository migration (loose objects + pack files)
- Cryptographic signing (Ed25519)
- Temporal branching (non-linear history)
- Centralized large file storage with LRU cache

## Design Philosophy

1. Minimal metadata - Only track what's necessary
2. Content-addressed - Use hashes as the source of truth
3. Fast by default - No tree walking, clever indexing
4. Simple interface - Clean commands, clear semantics
5. Complete feature set - All essential VCS operations
6. Research-driven - Novel branching, cryptographic signing, hybrid storage

## Documentation

- FEATURE_SUMMARY.md - Complete feature overview and CLI reference
- HYBRID_VCS.md - Architecture design decisions and innovation details
- RESEARCH_VCS_MODELS.md - VCS types analysis and comparison
- MIGRATION_COMPLETE.md - Git migration implementation details

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

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code.

## Contributing
 I need you please!!!
We welcome contributions! Here's how you can help:

Getting Started:
1. Fork the repository
2. Create a feature branch (git checkout -b feature/amazing-feature)
3. Make your changes
4. Run tests: cargo test
5. Run linter: cargo clippy
6. Format code: cargo fmt
7. Commit with clear messages
8. Push to your fork
9. Open a Pull Request

Contribution Areas:
- Performance: Optimization and caching improvements
- Features: Advanced merge/rebase, sparse checkout
- Documentation: Examples, tutorials, API docs
- Testing: Additional test coverage and edge cases
- Bug Fixes: Issues marked as help-wanted

## License

This project is licensed under the GNU Lesser General Public License v3.0 - see LICENSE file for details.

By contributing, you agree that your contributions will be licensed under its LGPL v3 License.
