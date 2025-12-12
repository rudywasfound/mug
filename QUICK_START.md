# Quick Start Guide

## Installation

```bash
# Build from source
cargo build --release

# Binary is at ./target/release/mug
./target/release/mug --help
```

## Basic Workflow

```bash
# Initialize a new repository
mug init

# Configure your identity
mug config set user.name "Your Name"
mug config set user.email "you@example.com"

# Check what changed
mug status

# Stage all changes
mug add .

# Create a commit
mug commit -m "Initial commit"

# View history
mug log
mug log --oneline

# Show specific commit
mug show <commit-hash>
```

## Working with Branches

```bash
# Create a branch
mug branch feature/my-feature

# List all branches
mug branches

# Switch to a branch
mug checkout feature/my-feature

# Make changes and commit
mug add .
mug commit -m "Add new feature"

# Switch back to main
mug checkout main

# Merge the feature branch
mug merge feature/my-feature

# Or rebase instead
mug rebase feature/my-feature
```

## Interactive Rebase

```bash
# Start interactive rebase
mug rebase -i main

# In the TUI:
# - j/k to navigate
# - p = pick (use commit)
# - s = squash (merge with previous)
# - r = reword (edit message)
# - d = drop (discard commit)
# - Enter to execute
```

## Remote Operations

```bash
# Add a remote
mug remote add origin https://example.com/repo.git

# List remotes
mug remote list

# Push to remote
mug push origin main

# Pull from remote
mug pull origin main

# Fetch updates without merging
mug fetch origin

# Clone a repository
mug clone https://example.com/repo.git my-repo
```

## Server Mode

```bash
# Start an HTTP server
mug serve --host 0.0.0.0 --port 8080 --repos /path/to/repos

# From another machine
mug clone http://192.168.1.100:8080/myrepo
```

## File Operations

```bash
# Rename or move a file
mug mv old_name.rs new_name.rs

# Remove a file
mug rm unwanted.rs

# Restore a file to HEAD state
mug restore modified.rs
```

## Search in Files

```bash
# Parallel grep search (regex supported)
mug grep "TODO"
mug grep "function_name" src/
```

## Stash (Save Work in Progress)

```bash
# Save uncommitted changes
mug stash

# Switch to another branch
mug checkout other-branch

# Switch back and restore changes
mug checkout main
mug stash-pop

# List all stashes
mug stash-list
```

## Tags

```bash
# Create a tag
mug tag v1.0.0 -m "Version 1.0.0"

# List all tags
mug tags

# Delete a tag
mug delete-tag v1.0.0
```

## Resetting Commits

```bash
# Soft reset: keep changes staged
mug reset soft HEAD~1

# Mixed reset: keep changes unstaged (default)
mug reset mixed HEAD~1

# Hard reset: discard all changes
mug reset hard HEAD~1
```

## Cherry-pick

```bash
# Pick a single commit onto current branch
mug cherry-pick <commit-hash>

# Cherry-pick a range of commits
mug cherry-pick-range <start-commit> <end-commit>
```

## Bisect (Find Breaking Commit)

```bash
# Start bisect session
mug bisect-start

# Mark current commit as bad
mug bisect-bad

# Checkout a known good commit and mark it
mug checkout v1.0.0
mug bisect-good

# MUG will binary search between good/bad
# Test the code, then mark as good/bad
mug bisect-good    # or bisect-bad
```

## Git Compatibility

```bash
# Migrate a Git repository to MUG
mug migrate /path/to/git/repo /path/to/new/mug/repo

# Preserves:
# - All commit history
# - Branch references
# - Tags
```

## Configuration

```bash
# Set config values
mug config set key value

# Get config value
mug config get key

# List all config
mug config list
```

## Ignore Patterns

Create a `.mugignore` file:

```
# Comments start with #
*.log              # Log files
node_modules/      # Dependencies
target/            # Build artifacts
.env               # Secrets
!important.log     # Re-include specific files
```

## Attributes

Create a `.mugattributes` file:

```
*.bin merge=binary diff=binary
*.jpg line_ending=binary
* export-ignore
```

## Hooks

Create executable scripts in `.mug/hooks/`:

```bash
# Pre-commit hook
echo "cargo fmt" > .mug/hooks/pre-commit
chmod +x .mug/hooks/pre-commit
```

Hook types:
- `pre-commit` - Before creating commit
- `post-commit` - After successful commit
- `pre-push` - Before pushing
- `post-push` - After pushing
- `pre-merge` - Before merging
- `post-merge` - After merging

## Verify Repository

```bash
# Check repository integrity
mug verify
```

## Garbage Collection

```bash
# Clean up unreferenced objects
mug gc
```

## View Differences

```bash
# Show diff between commits
mug diff --from HEAD~2 --to HEAD

# Show diff of specific commit
mug show <commit-hash>
```

## Tips & Tricks

1. **Use `-a` flag for author override**: `mug commit -m "Fix" -a "John Doe"`
2. **Use `--oneline` for compact log**: `mug log --oneline`
3. **Push all branches**: `mug push origin` (no branch specified)
4. **Create tracking branches**: `mug branch --track feature origin/feature`
5. **View HEAD position**: `mug log --oneline -n 1`

## Troubleshooting

```bash
# Check repository status
mug status

# Verify database integrity
mug verify

# Check configuration
mug config list

# View recent commits
mug log --oneline -n 20
```

For more detailed documentation, see [DOCS.md](DOCS.md).
