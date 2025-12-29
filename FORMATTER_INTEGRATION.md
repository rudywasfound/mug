# Beautiful Output Integration Guide

This guide explains how to integrate MUG's new beautiful formatter into your commands.

## Quick Start

The formatter is located in `src/ui/formatter.rs` and is already exported from the UI module.

### Basic Usage

```rust
use mug::ui::{UnicodeFormatter, CommitInfo};

// Create formatter (auto-detects terminal capabilities)
let formatter = UnicodeFormatter::new(true, true);

// Create commit info
let commits = vec![
    CommitInfo {
        hash: "abc1234567890".to_string(),
        author: "Your Name".to_string(),
        date: "2025-12-29 14:32:15".to_string(),
        message: "Add new feature".to_string(),
        is_head: true,
        branch: Some("main".to_string()),
    },
];

// Format and print
println!("{}", formatter.format_log(&commits));
```

## Integration Points

### 1. Log Command (`mug log`)

Replace plain text output in `commands/log.rs`:

```rust
let formatter = UnicodeFormatter::new(use_unicode_flag, use_colors_flag);
println!("{}", formatter.format_log(&commits));
```

### 2. Status Command (`mug status`)

Replace plain text output in `commands/status.rs`:

```rust
let changes = vec![
    ("src/main.rs".to_string(), 'M'),
    ("new_file.rs".to_string(), 'A'),
];
println!("{}", formatter.format_status(current_branch, &changes));
```

### 3. Branch Command (`mug branches`)

Replace plain text output in `commands/branch.rs`:

```rust
println!("{}", formatter.format_branch_list(current_branch, &branches));
```

### 4. Diff Command (`mug diff`)

Use the diff formatter:

```rust
use mug::ui::{DiffHunk, DiffLine};

let hunks = vec![
    DiffHunk {
        file: "src/main.rs".to_string(),
        added: 5,
        removed: 2,
        lines: vec![
            DiffLine::Context("fn main() {".to_string()),
            DiffLine::Removed("    old_code();".to_string()),
            DiffLine::Added("    new_code();".to_string()),
            DiffLine::Context("}".to_string()),
        ],
    }
];

println!("{}", formatter.format_diff(&hunks));
```

### 5. Clone/Push/Fetch Progress

Use the progress bar formatter:

```rust
for chunk in 0..=total_chunks {
    println!("{}", formatter.format_progress_bar(chunk, total_chunks));
    // Process chunk
}
```

### 6. Error Messages

Use the message formatters in error handling:

```rust
match operation {
    Ok(result) => println!("{}", formatter.format_success("Operation completed")),
    Err(e) => eprintln!("{}", formatter.format_error(&e.to_string())),
}
```

### 7. Merge Conflicts

Display conflicts beautifully:

```rust
println!("{}", formatter.format_merge_conflict("file.rs", our_content, their_content));
```

## Available Methods

### Log Formatting
```rust
pub fn format_log(&self, commits: &[CommitInfo]) -> String
```
Formats commit history with symbols, colors, and proper spacing.

**CommitInfo fields:**
- `hash: String` - Commit hash (first 8 chars shown)
- `author: String` - Author name and email
- `date: String` - Commit date/time
- `message: String` - Commit message
- `is_head: bool` - Whether this is the current HEAD
- `branch: Option<String>` - Branch name (optional)

### Status Formatting
```rust
pub fn format_status(&self, branch: &str, changes: &[(String, char)]) -> String
```
Displays branch and file changes in a beautiful box.

**Change types:**
- `'M'` - Modified
- `'A'` - Added
- `'D'` - Deleted
- `'R'` - Renamed
- `'?'` - Unknown

### Branch List Formatting
```rust
pub fn format_branch_list(&self, current: &str, branches: &[String]) -> String
```
Shows all branches with current branch highlighted.

### Progress Bar
```rust
pub fn format_progress_bar(&self, current: u64, total: u64) -> String
```
Displays a progress bar for long operations.

### Diff Formatting
```rust
pub fn format_diff(&self, hunks: &[DiffHunk]) -> String
```
Shows file diffs with added/removed/context lines colored appropriately.

### Message Formatters
```rust
pub fn format_error(&self, error: &str) -> String
pub fn format_success(&self, message: &str) -> String
pub fn format_warning(&self, message: &str) -> String
```

### Merge Conflict Display
```rust
pub fn format_merge_conflict(&self, file: &str, ours: &str, theirs: &str) -> String
```

## Configuration

### Control Output Format

```rust
// Full unicode and colors
let formatter = UnicodeFormatter::new(true, true);

// ASCII-only (for piping to files)
let formatter = UnicodeFormatter::new(false, false);

// Unicode without colors (for limited terminals)
let formatter = UnicodeFormatter::new(true, false);
```

### Command-Line Flags (Recommended)

Add CLI flags to control output:

```rust
#[derive(Parser)]
struct LogArgs {
    #[arg(long, help = "Use ASCII instead of Unicode")]
    ascii: bool,
    
    #[arg(long, help = "Disable colored output")]
    no_color: bool,
}

// Usage
let formatter = UnicodeFormatter::new(!args.ascii, !args.no_color);
```

## Examples in Commands

### Example: `log` Command

```rust
use crate::ui::{UnicodeFormatter, CommitInfo};

pub fn execute(args: &LogArgs) -> Result<()> {
    let repo = Repository::open(".")?;
    let commits = repo.log()?;
    
    // Convert to CommitInfo
    let commit_infos: Vec<CommitInfo> = commits
        .into_iter()
        .map(|commit| CommitInfo {
            hash: commit.hash,
            author: commit.author,
            date: commit.date,
            message: commit.message,
            is_head: commit.is_head,
            branch: repo.get_branch_for_commit(&commit.hash).ok(),
        })
        .collect();
    
    let formatter = UnicodeFormatter::new(
        !args.ascii,
        !args.no_color,
    );
    
    println!("{}", formatter.format_log(&commit_infos));
    Ok(())
}
```

### Example: `status` Command

```rust
use crate::ui::UnicodeFormatter;

pub fn execute() -> Result<()> {
    let repo = Repository::open(".")?;
    let branch = repo.current_branch()?;
    let changes = repo.status()?;
    
    // Convert to (String, char) tuples
    let change_list: Vec<(String, char)> = changes
        .into_iter()
        .map(|change| (change.path, change.kind))
        .collect();
    
    let formatter = UnicodeFormatter::new(true, true);
    println!("{}", formatter.format_status(&branch, &change_list));
    Ok(())
}
```

## Testing

All formatter methods are tested. Run tests with:

```bash
cargo test ui::formatter
```

See `tests` module in `src/ui/formatter.rs` for examples.

## Performance

- No additional overhead compared to plain text formatting
- Color codes are only added if `use_colors` is true
- Unicode detection can be done once at startup

## Compatibility

The formatter includes automatic fallback:
- ✓ Modern terminals (full Unicode + colors)
- ✓ SSH sessions
- ✓ CI/CD pipelines (detects piped output)
- ✓ Legacy terminals (ASCII mode)
- ✓ Windows Terminal
- ✓ Web-based terminals

## Tips & Best Practices

1. **Detect terminal capabilities at startup**
   ```rust
   let use_colors = atty::is(atty::Stream::Stdout);
   ```

2. **Cache the formatter** - Create it once in your main command handler

3. **Use `.bold()`** for emphasis with the colored crate:
   ```rust
   use colored::Colorize;
   println!("{}", formatter.format_success("Done").bold());
   ```

4. **Test in ASCII mode** - Always test with `--ascii` flag to ensure readability

5. **Keep messages concise** - Unicode/colors don't make long text readable

## Contributing

If you add new output types:

1. Add corresponding method to `UnicodeFormatter`
2. Include both Unicode and ASCII variants
3. Use consistent color scheme (see BEAUTIFUL_OUTPUT.md)
4. Add unit tests
5. Document in this guide
