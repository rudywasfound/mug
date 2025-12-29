# Unicode Output Enhancement for MUG

## Overview

MUG currently outputs basic ASCII text. By adding Unicode box-drawing characters, tree symbols, and emojis, we can make output significantly more beautiful and readable—matching the elegance of tools like Jujutsu.

## Current State vs. Target

### Current MUG Output

```
Branches:
* main
  feature-x
  hotfix-bug

Commit History:
abc1234 (HEAD -> main) Update docs
def5678 (feature-x) Add new feature
ghi9012 Fix critical bug
```

### Target Output (with Unicode)

```
┌─ Branches ──────────────────────────────────────────
│
├─ ● main
│
├─ ○ feature-x
│
└─ ○ hotfix-bug


┌─ Commit History ─────────────────────────────────────
│
├─ ◆ abc1234 (HEAD -> main) Update docs
│  │
│  └─ def5678 (feature-x) Add new feature
│     │
│     └─ ghi9012 Fix critical bug
│
└─ ⚓ (root)
```

## Components to Enhance

### 1. Log Output (`mug log`)

#### Current
```
abc1234 Update docs
def5678 Add new feature
```

#### Enhanced
```
◆  abc1234  (HEAD -> main)  Update docs
│
◆  def5678  (feature-x)     Add new feature
│
◆  ghi9012                  Fix critical bug
~
```

With merge/branch:
```
@  xyz9999  (working)       (no message)
│
◆  abc1234  (main)          Update docs
├─╮
│ ◆  def5678  (feature-x)    Add new feature
│ │
◆ │  ghi9012  (hotfix-bug)   Fix critical bug
├─╯
◆  aaa0000  (root)          Initial commit
```

### 2. Status Output (`mug status`)

#### Current
```
On branch main
Changes not staged for commit:
  modified: src/main.rs
  deleted: docs/old.md
Untracked files:
  test.rs
```

#### Enhanced
```
┌─ Status ──────────────────────────────────────────────────┐
│ 📍 On branch main                                          │
├───────────────────────────────────────────────────────────┤
│ 📝 Changes not staged for commit:                          │
│   ✏️  modified    src/main.rs                              │
│   🗑  deleted     docs/old.md                              │
│                                                            │
│ 📦 Untracked files:                                        │
│   ❓ test.rs                                               │
└───────────────────────────────────────────────────────────┘

●●● 2 modified, 1 deleted, 1 untracked
```

### 3. Branch Visualization (`mug branches`)

#### Current
```
  main (tracked)
* feature-work
  development
```

#### Enhanced
```
┌─ Branches ──────────────────────────────────────────┐
│                                                     │
│ ◆ ● main                       (tracked)            │
│ │                                                   │
│ └─ ○ feature-work                (current)          │
│    │                                                │
│    └─ ○ development              (local)            │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### 4. Diff Output (`mug diff`)

#### Current
```
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello!");
     let x = 5;
```

#### Enhanced
```
┌─ Diff: src/main.rs ──────────────────────────────────┐
│                                                      │
│ fn main() {                                          │
│ ➕ println!("Hello!");                               │
│   let x = 5;                                         │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### 5. Merge Conflict Display

#### Current
```
<<<<<<<< HEAD
our changes
=======
their changes
>>>>>>>
```

#### Enhanced
```
┌─ Conflict in src/main.rs ────────────────┐
│                                          │
│ ⬅️  (current) ────────────────────────   │
│ our changes                              │
│                                          │
│ ➡️  (incoming) ───────────────────────   │
│ their changes                            │
│                                          │
└──────────────────────────────────────────┘
```

### 6. Progress Bars

#### Current
```
Packing objects... [================>          ] 50%
```

#### Enhanced
```
┌─ Packing Objects ────────────────────────────┐
│ ███████████░░░░░░░░░░░░░░░░░░ 50% (1/2 GB)  │
│ ⏱  ETA: 2m 30s                               │
└──────────────────────────────────────────────┘
```

### 7. Error/Warning Messages

#### Current
```
error: file not found
warning: ambiguous reference
```

#### Enhanced
```
❌ error: file not found
   → src/nonexistent.rs

⚠️  warning: ambiguous reference
   → multiple branches match 'feature'
   Use: mug checkout feature-1
```

## Unicode Characters to Use

### Commit/Object States
```
◆  Regular commit
◉  Main commit (current branch)
@  Working copy commit
~  Virtual commit (root)
○  Empty commit
✓  Completed
✗  Failed
⚠️  Warning/Conflict
```

### Branch/Merge Symbols
```
├─ Branch from parent
│  Line continuation
├─╮ Merge point (multiple parents)
│ ╭┴─ Branch merge
└─╯ Merge end
◆  Commit on branch
→  Points to
⚓  Root/anchor
●  Current (active)
○  Inactive branch
```

### File Operations
```
✏️  Modified
➕  Added
🗑  Deleted
🏷  Renamed
↔️  Moved
🔗  Copied
```

### Status Indicators
```
📍 Location/branch
📝 Changes
📦 Staging area
❓ Untracked
✓  Staged
⚡ Unstaged
🔒 Committed
```

### Actions
```
⬅️  Incoming/their
➡️  Outgoing/ours
↑  Push
↓  Pull/Fetch
🔄  Rebase
📌 Tag
🌿 Branch
💾 Save/Stash
```

### Special
```
⊗  Removed/deleted
✈️  Remote
🗂  Directory
📄 File
🔑 Key
🔐 Encrypted
⊖ Conflict
```

## Implementation

### Create Module: `src/ui/formatter.rs`

```rust
pub mod formatter {
    pub struct UnicodeFormatter {
        pub use_colors: bool,
        pub use_unicode: bool,
        pub width: usize,  // Terminal width
    }

    impl UnicodeFormatter {
        pub fn format_commit_graph(&self, commits: &[Commit]) -> String
        pub fn format_status(&self, status: &Status) -> String
        pub fn format_branch_list(&self, branches: &[Branch]) -> String
        pub fn format_diff(&self, diff: &Diff) -> String
        pub fn format_error(&self, error: &str, context: &str) -> String
        pub fn format_progress_bar(&self, current: u64, total: u64) -> String
        pub fn format_table(&self, headers: &[&str], rows: Vec<Vec<&str>>) -> String
    }

    // Helper functions for common elements
    pub fn box_drawing(title: &str, content: &str, width: usize) -> String
    pub fn tree_branch(indent: usize) -> &'static str
    pub fn commit_symbol(is_current: bool, is_merge: bool) -> &'static str
    pub fn file_status_icon(kind: FileChangeKind) -> &'static str
    pub fn colorize(text: &str, color: Color) -> String
}
```

### Cargo Dependencies

```toml
[dependencies]
# Already exists:
ratatui = "0.27"           # TUI library with Unicode support
crossterm = "0.28"         # Terminal control

# New additions:
unicode-width = "0.1"      # Measure Unicode string widths
textwrap = "0.16"          # Word wrapping
colored = "2.1"            # Color output (simple)
# OR
owo-colors = "3.5"         # More elegant coloring
```

### Configuration

Add to `.mug/config`:
```toml
[ui]
use_unicode = true         # Enable Unicode characters
use_colors = true          # Enable colors
use_emojis = false         # Use emoji (optional, more playful)
auto_detect = true         # Auto-detect terminal capabilities
terminal_width = 0         # 0 = auto-detect

[colors]
branch_current = "green"
branch_inactive = "gray"
added = "green"
removed = "red"
modified = "yellow"
commit_hash = "cyan"
```

### Usage Examples

#### Example 1: Enhanced Log

```rust
fn cmd_log(repo: &Repository, args: LogArgs) -> Result<()> {
    let commits = repo.log()?;
    let formatter = UnicodeFormatter::new(&config);
    
    let output = formatter.format_commit_graph(&commits);
    println!("{}", output);
    
    Ok(())
}
```

#### Example 2: Enhanced Status

```rust
fn cmd_status(repo: &Repository) -> Result<()> {
    let status = repo.status()?;
    let formatter = UnicodeFormatter::new(&config);
    
    let output = formatter.format_status(&status);
    println!("{}", output);
    
    Ok(())
}
```

#### Example 3: Progress Display

```rust
fn show_progress(current: u64, total: u64, label: &str) {
    let formatter = UnicodeFormatter::new(&config);
    let bar = formatter.format_progress_bar(current, total);
    
    eprint!("\r{}: {}", label, bar);
    
    if current >= total {
        eprintln!("\n✓ Complete!");
    }
}
```

## Color Scheme

### Recommended Palette

```
Primary Colors:
- Green (#00FF00)  → Success, current branch, added files
- Red (#FF0000)    → Errors, deleted files, conflicts
- Yellow (#FFFF00) → Warnings, modified files
- Cyan (#00FFFF)   → Hashes, links, metadata
- Blue (#0000FF)   → Branches, tags, headers
- Gray (#808080)   → Muted, old, inactive

Backgrounds:
- Dark theme: Dark gray background (#1E1E1E)
- Light theme: Light gray background (#F0F0F0)
```

### Example Color Mapping

```rust
pub enum LogColor {
    CommitHash,      // Cyan
    BranchName,      // Green
    AuthorName,      // Blue
    Timestamp,       // Gray
    Message,         // White
    Tag,             // Yellow
}

pub enum StatusColor {
    Added,           // Green (➕)
    Deleted,         // Red (🗑)
    Modified,        // Yellow (✏️)
    Untracked,       // Gray (❓)
    Header,          // Blue
}
```

## Terminal Compatibility

### Detection Strategy

```rust
pub fn detect_capabilities() -> TerminalCapabilities {
    let supports_unicode = 
        env::var("LANG").ok().map(|l| l.contains("UTF")).unwrap_or(false) ||
        env::var("LC_ALL").ok().map(|l| l.contains("UTF")).unwrap_or(false) ||
        cfg!(windows);  // Windows 10+ supports Unicode
    
    let supports_colors = 
        atty::isnt(atty::Stream::Stdout).not() &&
        env::var("NO_COLOR").is_err();
    
    let terminal_width =
        term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    
    TerminalCapabilities {
        supports_unicode,
        supports_colors,
        supports_hyperlinks: env::var("TERM_PROGRAM").is_ok(),
        width: terminal_width,
    }
}
```

### Fallbacks

If terminal doesn't support Unicode:
```
Unicode: ◆ → [*]
Unicode: ├─ → |--
Unicode: ✓ → OK
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_graph_formatting() {
        let formatter = UnicodeFormatter::new_test();
        let commits = vec![/* test data */];
        let output = formatter.format_commit_graph(&commits);
        
        assert!(output.contains("◆"));
        assert!(output.contains("│"));
    }

    #[test]
    fn test_progress_bar_formatting() {
        let formatter = UnicodeFormatter::new_test();
        let bar = formatter.format_progress_bar(50, 100);
        
        assert!(bar.contains("███"));
        assert!(bar.contains("50%"));
    }

    #[test]
    fn test_unicode_width_calculation() {
        let formatter = UnicodeFormatter::new_test();
        assert_eq!(formatter.measure_width("hello"), 5);
        assert_eq!(formatter.measure_width("café"), 4);  // é is 1 width
    }

    #[test]
    fn test_fallback_ascii_mode() {
        let mut formatter = UnicodeFormatter::new_test();
        formatter.use_unicode = false;
        
        let output = formatter.format_status(&status);
        assert!(!output.contains("◆"));
        assert!(output.contains("[*]"));  // ASCII fallback
    }
}
```

## Implementation Roadmap

### Phase 1: Core Infrastructure (Week 1)
- [ ] Create `src/ui/formatter.rs` module
- [ ] Add Unicode character constants
- [ ] Implement basic formatting functions
- [ ] Terminal capability detection

### Phase 2: Log Output (Week 2)
- [ ] Enhance `mug log` with Unicode graph
- [ ] Test with various commit patterns (linear, merge, branching)
- [ ] Color support

### Phase 3: Status & Branches (Week 3)
- [ ] Enhance `mug status` output
- [ ] Enhance `mug branches` display
- [ ] Add file operation icons

### Phase 4: Diffs & Conflicts (Week 4)
- [ ] Enhance `mug diff` display
- [ ] Conflict visualization
- [ ] Syntax highlighting (optional)

### Phase 5: Polish & Testing (Week 5)
- [ ] Progress bars
- [ ] Error/warning formatting
- [ ] Comprehensive testing
- [ ] Documentation

## Examples: Before & After

### Example 1: Branch Navigation

**Before:**
```
* main
  feature-1
  feature-2
    develop
```

**After:**
```
┌─ Branches ────────────────────────────┐
│                                       │
│ 🌿 ● main                  (current)  │
│                                       │
│ 🌿 ○ feature-1                        │
│                                       │
│ 🌿 ○ feature-2                        │
│    │                                  │
│    └─ 🌿 ○ develop        (stale)    │
│                                       │
└───────────────────────────────────────┘
```

### Example 2: Detailed Log

**Before:**
```
abc1234 (HEAD -> main, origin/main) Update docs
def5678 (feature-x) Add feature
ghi9012 Fix bug
```

**After:**
```
◆  abc1234  (HEAD -> main)  ▍▍▍▍▍▍▍▍ Update documentation
│             (origin/main)
│
◉  def5678  (feature-x)     ▍▍▍▍▍▍ Add new feature
│
◆  ghi9012                  ▍▍▍ Fix critical bug
│
~  000000   (root)
```

### Example 3: Complex Merge

**Before:**
```
*   merge-commit
|\
| * feature-branch
|/
* main-branch
```

**After:**
```
@  abc1234  (working)       Merge feature
├─╮
│ ◆  def5678  (feature)     Add feature
│ │
◆ │  ghi9012  (main)        Update code
├─╯
◆  jkl3456                  Initial commit
~  000000   (root)
```

## Benefits

1. **Visual Clarity**: Graph structure is immediately visible
2. **Less Ambiguity**: Clear symbols for different states
3. **Professionalism**: Looks polished like modern tools
4. **Better UX**: Status/errors are easier to scan
5. **Learning**: Visual hierarchy helps users understand concepts
6. **Accessibility**: Works in limited-color terminals with ASCII fallback

## References

- [Jujutsu Log Output](https://jj-vcs.github.io/jj/latest/)
- [Box Drawing Characters](https://en.wikipedia.org/wiki/Box-drawing_character)
- [Unicode in Rust](https://doc.rust-lang.org/std/string/)
- [Terminal Graphics](https://www.blinry.org/unicode-box-drawing/)
- [Ratatui Documentation](https://docs.rs/ratatui/)

## Future Enhancements

1. **Syntax Highlighting**: Color code by language
2. **Interactive TUI**: Use ratatui for interactive selection
3. **Hyperlinks**: OSC 8 hyperlinks in compatible terminals
4. **Animations**: Subtle animations for long operations
5. **Themes**: User-defined color schemes
6. **Accessibility**: High-contrast mode for vision impairment
