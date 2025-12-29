# MUG Beautiful Formatter - Quick Reference

## Import
```rust
use mug::ui::{UnicodeFormatter, CommitInfo, DiffHunk, DiffLine};
```

## Create Formatter
```rust
// With Unicode and colors
let fmt = UnicodeFormatter::new(true, true);

// ASCII only (no Unicode chars, no colors)
let fmt = UnicodeFormatter::new(false, false);

// Unicode without colors
let fmt = UnicodeFormatter::new(true, false);
```

## Format Commit Log

```rust
let commits = vec![CommitInfo {
    hash: "abc1234567890".to_string(),
    author: "Your Name".to_string(),
    date: "2025-12-29 14:32:15".to_string(),
    message: "Your message".to_string(),
    is_head: true,
    branch: Some("main".to_string()),
}];

println!("{}", fmt.format_log(&commits));
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━
Commit History
━━━━━━━━━━━━━━━━━━━━
◆ abc1234 Your message [main]
│  Author: Your Name
│  Date: 2025-12-29 14:32:15
┴
```

## Format Repository Status

```rust
let changes = vec![
    ("src/main.rs".to_string(), 'M'),      // Modified
    ("new.rs".to_string(), 'A'),           // Added
    ("old.rs".to_string(), 'D'),           // Deleted
    ("renamed.rs".to_string(), 'R'),       // Renamed
];

println!("{}", fmt.format_status("main", &changes));
```

**Output:**
```
╭──────────────────────────╮
│ 🌿 On branch: main
│
│ 📝 Changes:
│   ✏️  src/main.rs
│   ➕  new.rs
│   🗑  old.rs
│   ↻  renamed.rs
╰──────────────────────────╯
```

## Format Branch List

```rust
let branches = vec![
    "main".to_string(),
    "develop".to_string(),
    "feature/ui".to_string(),
];

println!("{}", fmt.format_branch_list("main", &branches));
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━
Branches
━━━━━━━━━━━━━━━━━━━━
● main (current)
○ develop
○ feature/ui
━━━━━━━━━━━━━━━━━━━━
```

## Format Progress Bar

```rust
println!("{}", fmt.format_progress_bar(50, 100));
```

**Output:**
```
███████████████░░░░░░░░░░░░░░░  50%
```

## Format Diff

```rust
let hunks = vec![DiffHunk {
    file: "src/main.rs".to_string(),
    added: 2,
    removed: 1,
    lines: vec![
        DiffLine::Context("fn main() {".to_string()),
        DiffLine::Removed("    old();".to_string()),
        DiffLine::Added("    new();".to_string()),
        DiffLine::Context("}".to_string()),
    ],
}];

println!("{}", fmt.format_diff(&hunks));
```

**Output:**
```
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ src/main.rs (+2 -1) @@
 fn main() {
-    old();
+    new();
 }
```

## Messages

### Success
```rust
println!("{}", fmt.format_success("Operation completed"));
// ✓ success: Operation completed
```

### Error
```rust
eprintln!("{}", fmt.format_error("File not found"));
// ✘ error: File not found
```

### Warning
```rust
println!("{}", fmt.format_warning("This is irreversible"));
// ⚠ warning: This is irreversible
```

## Merge Conflict

```rust
let conflict = fmt.format_merge_conflict(
    "src/file.rs",
    "our_code();",
    "their_code();",
);
println!("{}", conflict);
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  Merge Conflict in src/file.rs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
<<<<<<< HEAD (ours)
our_code();
=======
their_code();
>>>>>>> (theirs)
```

## CommitInfo Structure

| Field | Type | Description |
|-------|------|-------------|
| `hash` | `String` | Full commit hash (first 8 chars displayed) |
| `author` | `String` | Author name (with optional email) |
| `date` | `String` | Commit date/time |
| `message` | `String` | Commit message |
| `is_head` | `bool` | `true` if this is HEAD |
| `branch` | `Option<String>` | Branch name (optional) |

## Change Type Characters

| Char | Meaning | Icon |
|------|---------|------|
| `'M'` | Modified | ✏️ |
| `'A'` | Added | ➕ |
| `'D'` | Deleted | 🗑 |
| `'R'` | Renamed | ↻ |
| `'?'` | Unknown | ? |

## DiffLine Enum

```rust
pub enum DiffLine {
    Added(String),       // Green
    Removed(String),     // Red
    Context(String),     // White
}
```

## Color Mapping

| Color | Hex | Usage |
|-------|-----|-------|
| Bright Cyan | `#00FFFF` | Headers, labels, borders |
| Bright Green | `#00FF00` | Success, current, additions |
| Red | `#FF0000` | Errors, deletions |
| Yellow | `#FFFF00` | Modifications, warnings |
| Magenta | `#FF00FF` | Renames, special ops |
| White | `#FFFFFF` | Regular text |

## Unicode Symbols Reference

| Symbol | Name | ASCII |
|--------|------|-------|
| `◆` | Full diamond (HEAD) | `*` |
| `◉` | Full circle (commit) | `o` |
| `●` | Current branch | `*` |
| `○` | Other branch | `o` |
| `│` | Vertical line | `\|` |
| `─` | Horizontal line | `-` |
| `┌┐└┘` | Box corners | `+` |
| `╭╮╰╯` | Rounded corners | `+` |
| `✓` | Check mark | `>` |
| `✘` | Cross | `x` |
| `⚠` | Warning | `!` |
| `~` | Tilde | `~` |
| `█` | Full block | `=` |
| `░` | Light shade | ` ` |

## Example: Complete Status Display

```rust
use mug::ui::UnicodeFormatter;

fn show_status() {
    let fmt = UnicodeFormatter::new(true, true);
    let branch = "feature/beautiful-output";
    let changes = vec![
        ("src/ui/formatter.rs".to_string(), 'M'),
        ("examples/demo.rs".to_string(), 'A'),
        ("old.rs".to_string(), 'D'),
    ];
    
    println!("{}", fmt.format_status(branch, &changes));
}
```

## Testing

All formatters have unit tests. Run:
```bash
cargo test ui::formatter
```

## Demo

See complete working examples:
```bash
cargo run --example formatter_demo
```

This shows all formatters in action with realistic data.

## Tips

1. **Detect terminal color support:**
   ```rust
   let use_colors = std::env::var("TERM") != Ok("dumb".to_string());
   let fmt = UnicodeFormatter::new(true, use_colors);
   ```

2. **Cache the formatter** - Create once, reuse many times

3. **Test ASCII mode** - Always verify output with `UnicodeFormatter::new(false, false)`

4. **Handle piped output** - Auto-disable colors when stdout is piped

5. **Use with error handling:**
   ```rust
   match operation() {
       Ok(_) => println!("{}", fmt.format_success("Done")),
       Err(e) => eprintln!("{}", fmt.format_error(&e.to_string())),
   }
   ```

## Files

- **Implementation:** `src/ui/formatter.rs`
- **Module exports:** `src/ui/mod.rs`
- **Examples:** `examples/formatter_demo.rs`
- **Documentation:** `BEAUTIFUL_OUTPUT.md`
- **Integration guide:** `FORMATTER_INTEGRATION.md`
