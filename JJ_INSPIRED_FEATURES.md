# JJ-Inspired Features for MUG

## Overview

This document outlines proposed features inspired by Jujutsu VCS that would make MUG safer, more user-friendly, and reduce data loss. The focus is on three key improvements:

1. **Auto-Snapshots**: Automatically capture file states at regular intervals
2. **Better Rollback**: Make reverting changes easier and safer
3. **Replace Stash with Snapshots**: Eliminate the need for `git stash`

## 1. Auto-Snapshots Feature

### What JJ Does

Jujutsu automatically creates snapshots of your working directory at regular intervals (or on demand). This means you never lose work—even if you discard changes, you can recover them from the snapshot history.

### Implementation for MUG

#### A. Snapshot Architecture

```rust
// src/core/snapshots.rs

pub struct Snapshot {
    pub id: String,                  // UUID
    pub timestamp: DateTime<Local>,
    pub source: SnapshotSource,      // Manual, Auto, Checkpoint
    pub changed_files: Vec<String>,
    pub total_files: usize,
    pub bytes_saved: u64,
    pub description: Option<String>,
    pub is_compressed: bool,
    pub parent_snapshot: Option<String>, // For incremental snapshots
}

pub enum SnapshotSource {
    Manual,           // User ran `mug snapshot`
    Auto,             // Auto-triggered by file watcher
    PreOperation,     // Before major operation (rebase, merge)
    Checkpoint,       // Named checkpoint
}

pub struct SnapshotManager {
    db: MugDb,
    watcher: Option<FileWatcher>,
    config: SnapshotConfig,
}

pub struct SnapshotConfig {
    pub enabled: bool,
    pub auto_interval_secs: u64,      // Default: 300 (5 minutes)
    pub max_snapshots: usize,         // Default: 50
    pub compression: bool,            // Compress snapshots
    pub exclude_patterns: Vec<String>,// .gitignore patterns
}
```

#### B. File Watching (Watchman-style)

Instead of full watchman integration (which adds complexity), implement a lightweight watcher:

```rust
// src/core/watcher.rs

use notify::Watcher;
use std::sync::mpsc;
use std::time::Duration;

pub struct FileWatcher {
    sender: mpsc::Sender<FileChangeEvent>,
    rx: mpsc::Receiver<FileChangeEvent>,
    debounce_ms: u64,
}

pub struct FileChangeEvent {
    pub path: PathBuf,
    pub kind: ChangeKind,
    pub timestamp: Instant,
}

pub enum ChangeKind {
    Created,
    Modified,
    Deleted,
    Renamed,
}

impl FileWatcher {
    pub fn new(repo_path: &Path, debounce_ms: u64) -> Result<Self> {
        // Uses `notify` crate (already in ecosystem)
        // Debounces rapid changes to avoid excessive snapshots
    }

    pub fn is_watching(&self) -> bool {
        // Check if watcher is active
    }

    pub fn next_changes(&self, timeout: Duration) -> Result<Vec<FileChangeEvent>> {
        // Get batched changes since last check
    }
}
```

#### C. Snapshot Commands

```bash
# Manual snapshot
mug snapshot "Work in progress on feature X"

# List snapshots
mug snapshots list
mug snapshots list --limit 10
mug snapshots list --since "2 hours ago"

# View snapshot details
mug snapshots show <snapshot-id>
mug snapshots show <snapshot-id> --diff

# Restore from snapshot
mug snapshots restore <snapshot-id>              # Restores, creates new snapshot first
mug snapshots restore <snapshot-id> --force      # Dangerous: no new snapshot

# Compare snapshots
mug snapshots diff <id1> <id2>

# Delete snapshots
mug snapshots delete <snapshot-id>
mug snapshots cleanup --older-than "7 days"

# Auto-snapshot control
mug snapshots auto-start
mug snapshots auto-stop
mug snapshots auto-status

# Configure auto-snapshot
mug config set snapshots.enabled true
mug config set snapshots.interval 300        # 5 minutes
mug config set snapshots.max-count 50
```

#### D. Storage Strategy

Snapshots are stored efficiently:

```
.mug/snapshots/
├── manifest.json              # Index of all snapshots
├── metadata/
│   └── <snapshot-id>.json    # Metadata for each snapshot
└── data/
    ├── <snapshot-id>.tar.zst # Full snapshot (compressed)
    └── <snapshot-id>.delta   # Incremental (only changes)
```

**Compression**: Zstd compression reduces storage by ~60-70% for typical source code.

**Incremental Snapshots**: After first snapshot, store only diffs:
- Snapshot 1: Full (200MB)
- Snapshot 2: Delta (5MB) - only changed files
- Snapshot 3: Delta (3MB)
- Snapshot 4: Full recompression (120MB) - maintain bounds

#### E. Auto-Cleanup

```rust
pub fn cleanup_old_snapshots(&self, days: u64) -> Result<SnapshotStats> {
    // Keep last N snapshots AND snapshots from last N days
    // Ensures you never lose recent work
    
    // Default: Keep 50 snapshots OR last 30 days, whichever is more
}
```

### Benefits

- **Zero data loss**: Can always recover work even if you do `mug reset --hard`
- **Safe experimentation**: Try risky operations knowing you can go back
- **Incremental storage**: Only changed files use extra space
- **Low overhead**: Background snapshots don't block user

## 2. Better Rollback

### Current Issue

Git makes rollback scary:
- `git revert` creates new commits (doesn't remove history)
- `git reset --hard` loses work permanently
- Accidentally discarding changes is hard to recover from

### JJ Approach

JJ's approach: Make it trivial to see what you're undoing, and easy to undo the undo.

### MUG Implementation

#### A. Enhanced `mug restore`

```bash
# Current (basic)
mug restore <files>

# Enhanced (shows what's being lost)
mug restore <files> --interactive      # Show preview
mug restore <files> --preview          # Dry run
mug restore <files> --create-snapshot  # Auto-snapshot first (default)
```

#### B. Non-Destructive Reset

```bash
# Soft reset (keep changes, move HEAD)
mug reset <commit> --soft

# Mixed reset (keep changes in working, move staging)
mug reset <commit> --mixed

# Hard reset (DISCOURAGED - will warn and recommend snapshot)
mug reset <commit> --hard
# Output: ⚠️  This will discard changes. A snapshot was auto-created.
#         Recover with: mug snapshots restore <snapshot-id>
```

#### C. Change History Tracking

Track what changed in each operation for recovery:

```rust
pub struct OperationChange {
    pub operation_id: String,
    pub timestamp: DateTime<Local>,
    pub operation_type: String,        // "reset", "revert", "restore"
    pub affected_files: Vec<String>,
    pub bytes_changed: u64,
    pub reversible: bool,
    pub command: String,               // Full command that was run
}
```

Commands to explore history:

```bash
# See what each operation changed
mug log changes --limit 20

# Show details of a specific change
mug log changes show <operation-id>

# Undo an operation (like git reflog but better)
mug undo <operation-id>

# See what would be undone
mug undo <operation-id> --preview
```

#### D. Safety Rails

```bash
# Refuse dangerous operations
mug reset --hard --force-unsafe
# ✓ Creating snapshot before dangerous operation...
# ✓ Snapshot created: snap-abc123
# ⚠️  Hard reset to <commit>
# ℹ️  Recover with: mug snapshots restore snap-abc123

# Same for:
# - mug restore --discard
# - mug revert (non-standard)
# - mug reflog expire
```

### Benefits

- **Reversible operations**: Snapshots make almost everything undoable
- **Clear feedback**: Always tell users what's being lost
- **Safety nets**: Auto-create snapshots before dangerous ops
- **History tracking**: Know exactly what changed when

## 3. Replace Stash with Snapshots

### Current Issue

`git stash` is awkward:
```bash
git stash                    # Where did my work go?
git stash list              # Cryptic list
git stash pop               # Did it work?
git stash drop stash@{2}    # Confusing syntax
```

MUG has `mug stash` which is better but still limited.

### JJ Approach

JJ uses snapshots + rebase instead:
- Working directory is always a commit
- Changes automatically become part of working commit
- No separate "stash" concept—just use snapshots

### MUG Implementation

#### A. Unified Change Management

```bash
# Current workflow
mug add file.rs
mug commit -m "Feature X"

# With snapshots (better):
# Edit file.rs
mug snapshot "Work in progress"      # Save working state
mug checkout branch-b                # Switch branches
# ... work on branch-b ...
mug checkout branch-a                # Back to A
mug snapshots restore <snap-id>      # Get work back
mug new                              # Create commit
mug commit -m "Feature X"
```

#### B. Workspace Snapshots

```bash
# Instead of stashing, use snapshots with better semantics
mug snapshot "Paused work on feature X"
mug snapshot list                    # See what you paused
mug snapshots show <id> --diff       # See exactly what's paused
mug snapshots restore <id>           # Resume work
```

#### C. Smart Branch Switching

Make switching branches preserve work automatically:

```bash
# Current Git behavior: ERROR if uncommitted changes
# MUG with snapshots: Auto-snapshot and apply on other branch

mug checkout feature-branch
# ✓ Auto-snapshot created for changes (if any)
# ✓ Switched to feature-branch
# ℹ️  Your changes are in snapshot snap-xyz if you need them
```

#### D. Deprecate `mug stash`

```bash
# Old command
mug stash push -m "Work"             # ← DEPRECATED

# New equivalent
mug snapshot "Work"                  # ← RECOMMENDED
mug snapshots list                   # ← RECOMMENDED (clearer)
mug snapshots restore <id>           # ← RECOMMENDED

# For compatibility:
mug stash <=> mug snapshots         # Alias
```

#### E. Migration Guide

```markdown
# Stash → Snapshots Migration

## Old Way
```bash
git stash push -m "WIP"
git stash list
git stash pop
```

## New Way
```bash
mug snapshot "WIP"
mug snapshots list
mug snapshots restore <id>
```
```

### Benefits

- **Simpler model**: No separate "stash" concept
- **Better UX**: Clearer what snapshots are
- **More flexible**: Snapshots work for any state, not just uncommitted changes
- **Safer**: Can't accidentally lose stashed work

## Implementation Roadmap

### Phase 1: Core Snapshots (Weeks 1-2)

- [ ] Create `src/core/snapshots.rs` module
- [ ] Implement basic snapshot creation/retrieval
- [ ] Add storage to `.mug/snapshots/`
- [ ] Create `mug snapshot` command
- [ ] Create `mug snapshots list/show` commands

### Phase 2: Auto-Snapshots (Weeks 3-4)

- [ ] Implement file watcher using `notify` crate
- [ ] Auto-snapshot on interval
- [ ] Add `mug snapshots auto-start/stop` commands
- [ ] Configuration: `snapshots.enabled`, `snapshots.interval`
- [ ] Cleanup old snapshots

### Phase 3: Safe Operations (Weeks 5-6)

- [ ] Add snapshot on dangerous operations
- [ ] Enhance `mug restore` with safety options
- [ ] Improve `mug reset --hard` warnings
- [ ] Add `mug undo` command
- [ ] Track operation changes

### Phase 4: Deprecate Stash (Week 7)

- [ ] Update `mug stash` to use snapshots internally
- [ ] Add deprecation warnings
- [ ] Create migration documentation
- [ ] Examples and tutorials

## Database Schema

```rust
// snapshots table
pub struct SnapshotRecord {
    pub id: String,                    // UUID
    pub timestamp_secs: i64,           // For sorting
    pub source: String,                // "manual", "auto", "pre_op"
    pub description: Option<String>,
    pub file_count: usize,
    pub bytes_saved: u64,
    pub compression_ratio: f32,        // Actual / Original
    pub parent_id: Option<String>,     // For incremental
    pub is_full: bool,                 // Full vs incremental
    pub metadata_json: String,         // Extra metadata
}
```

## Configuration

```toml
[snapshots]
enabled = true
auto_interval_secs = 300          # 5 minutes
max_snapshots = 50
compression = true
exclude_patterns = [".git", ".mug", "node_modules", "target"]

[safety]
auto_snapshot_before_dangerous = true
warn_on_data_loss = true
require_confirmation_for_reset = true
```

## Testing Strategy

```bash
# Unit tests
cargo test snapshots

# Integration tests
1. Create snapshot, restore, verify files match
2. Auto-snapshot interval works
3. Incremental snapshots save space
4. Cleanup respects retention policies
5. Snapshot survives across operations
```

## Performance Considerations

**Storage**: 
- Typical snapshot: 1-5MB (compressed)
- With 50 snapshots: ~100-250MB total
- Incremental after first: ~1MB per snapshot

**Time**:
- Full snapshot: ~100ms-1s (depends on repo size)
- Incremental: ~10-100ms
- Auto-snapshot runs in background thread

## Future Enhancements

1. **Snapshot Deduplication**: Share identical blocks across snapshots
2. **Remote Snapshots**: Push snapshots to remote server
3. **Snapshot Merging**: Merge snapshots from two branches
4. **Selective Restore**: Restore only specific files from snapshot
5. **Snapshot Signing**: Cryptographically sign snapshots
6. **Export Snapshots**: Download snapshot history

## References

- [JJ Snapshots Documentation](https://jj-vcs.github.io/jj/)
- [Facebook Watchman](https://facebook.github.io/watchman/)
- [Notify Crate](https://docs.rs/notify/latest/notify/)
