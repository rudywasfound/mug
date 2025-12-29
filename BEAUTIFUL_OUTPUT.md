# Beautiful Unicode Output for MUG

MUG now features gorgeous terminal output inspired by Jujutsu VCS, complete with Unicode symbols and vibrant colors.

## Features

### Color Support
- **Bright Cyan** - Headers and labels
- **Bright Green** - Successful operations and additions
- **Red** - Errors and deletions
- **Yellow** - Warnings and modifications
- **Magenta** - Special operations like renames
- **White** - Regular text content

### Unicode Symbols
- `◆` - Current HEAD commit
- `◉` - Regular commits
- `●` - Current branch
- `○` - Other branches
- `│` - Vertical connectors
- `─` - Horizontal lines
- `╭╮╰╯` - Rounded boxes
- `✓` - Success
- `✘` - Error
- `⚠️` - Warning
- `🌿` - Branch indicator
- `📝` - Changes indicator
- `✏️` - Modified files
- `➕` - Added files
- `🗑` - Deleted files

### ASCII Fallback
All Unicode characters have ASCII equivalents for compatibility with terminals that don't support Unicode:
- `◆` → `*`
- `◉` → `o`
- `│` → `|`
- `─` → `-`
- etc.

## Usage Examples

### Log Output
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Commit History
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
◆ abc1234 Update docs [main]
│  Author: Your Name
│  Date:   2025-12-29

◉ def5678 Add feature [develop]
│  Author: Another Dev
│  Date:   2025-12-28
┴
```

### Status Output
```
╭──────────────────────────────────────────────────────────────────╮
│ 🌿 On branch: main
│
│ 📝 Changes:
│   ✏️  src/main.rs
│   ➕ docs/API.md
│   🗑 old_file.rs
╰──────────────────────────────────────────────────────────────────╯
```

### Branch List
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Branches
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
● main (current)
○ develop
○ feature/new-ui
○ hotfix/security
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Progress Bar
```
████████████████░░░░░░░░░░░░░░  60%
```

### Diff Output
```
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ src/lib.rs (+3 -1) @@
 context line
-old line that was removed
+new line that was added
 another context line
```

### Error Messages
```
✘ error: File not found: src/missing.rs
```

### Success Messages
```
✓ success: Changes committed with hash abc1234
```

### Warning Messages
```
⚠ warning: This operation cannot be undone
```

### Merge Conflicts
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  Merge Conflict in src/main.rs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
<<<<<<< HEAD (ours)
fn our_implementation() {}
=======
fn their_implementation() {}
>>>>>>> (theirs)
```

## Configuration

The formatter automatically detects terminal capabilities, but you can control output explicitly:

```rust
// Use Unicode and colors
let formatter = UnicodeFormatter::new(true, true);

// ASCII-only mode (for piping to files or old terminals)
let formatter = UnicodeFormatter::new(false, false);

// Unicode but no colors
let formatter = UnicodeFormatter::new(true, false);
```

## Color Palette

| Color | Hex | Usage |
|-------|-----|-------|
| Bright Cyan | `#00FFFF` | Headers, labels, connectors |
| Bright Green | `#00FF00` | Success, additions, current item |
| Red | `#FF0000` | Errors, deletions |
| Yellow | `#FFFF00` | Modifications, warnings |
| Magenta | `#FF00FF` | Special operations, renames |
| White | `#FFFFFF` | Content text |

## Formatter Methods

### Log Formatting
```rust
formatter.format_log(&commits)
```

### Status Formatting
```rust
formatter.format_status(branch, changes)
```

### Branch List Formatting
```rust
formatter.format_branch_list(current, branches)
```

### Progress Bar
```rust
formatter.format_progress_bar(current, total)
```

### Diff Formatting
```rust
formatter.format_diff(&hunks)
```

### Error/Success/Warning Messages
```rust
formatter.format_error("Error message")
formatter.format_success("Success message")
formatter.format_warning("Warning message")
```

### Merge Conflict Display
```rust
formatter.format_merge_conflict(file, ours, theirs)
```

## Terminal Compatibility

MUG's beautiful output works on:
- ✓ Linux (all terminals)
- ✓ macOS (Terminal, iTerm2, etc.)
- ✓ Windows (Windows Terminal, ConEmu, etc.)
- ✓ Web-based terminals
- ✓ SSH sessions
- ✓ Legacy terminals (with ASCII fallback)

## Dependencies

The enhanced formatter uses:
- `colored` - Terminal color support
- `ratatui` - Already in dependencies for TUI components
- Standard library formatting

No additional heavy dependencies added.
