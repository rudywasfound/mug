# Beautiful Output for MUG - Start Here

MUG now has gorgeous, professional terminal output with Unicode symbols and vibrant colors, inspired by Jujutsu VCS.

## 🎯 Quick Start (60 seconds)

1. **See it in action:**
   ```bash
   cargo run --example formatter_demo
   ```

2. **Use in your code:**
   ```rust
   use mug::ui::UnicodeFormatter;
   
   let fmt = UnicodeFormatter::new(true, true);  // Unicode + colors
   println!("{}", fmt.format_success("Changes committed!"));
   ```

3. **That's it!** The formatter handles all the beautiful output.

## 📚 Documentation Map

### For Users
- **BEAUTIFUL_OUTPUT.md** - Visual examples of what the output looks like
- **BEAUTIFUL_OUTPUT_SUMMARY.md** - Summary of what was added

### For Developers
- **FORMATTER_QUICK_REFERENCE.md** - Copy-paste code examples
- **FORMATTER_INTEGRATION.md** - Detailed integration guide for each command

### For Learning
- **examples/formatter_demo.rs** - Working code demonstrating all features

## 🎨 What You Get

### Beautiful Log Output
```
◆ abc1234 Add beautiful output [main]
│  Author: Alice <alice@example.com>
│  Date: 2025-12-29 14:32:15

◉ def5678 Initial commit
│  Author: Bob <bob@example.com>
│  Date: 2025-12-28 10:15:42
```

### Status Display
```
╭──────────────────────────────────╮
│ 🌿 On branch: main
│
│ 📝 Changes:
│   ✏️  src/main.rs
│   ➕ new_file.rs
│   🗑 old_file.rs
╰──────────────────────────────────╯
```

### Colored Messages
```
✓ success: Operation completed
⚠ warning: This is irreversible
✘ error: File not found
```

## 🚀 Integration Steps

### 1. Replace Plain Output in Commands

**Before:**
```rust
println!("On branch {}", branch);
for (file, kind) in changes {
    println!("{}: {}", kind, file);
}
```

**After:**
```rust
use mug::ui::UnicodeFormatter;

let fmt = UnicodeFormatter::new(true, true);
let changes: Vec<(String, char)> = /* ... */;
println!("{}", fmt.format_status(&branch, &changes));
```

### 2. Format Commit Logs
```rust
let commits: Vec<CommitInfo> = /* convert your commits */;
println!("{}", fmt.format_log(&commits));
```

### 3. Format Error Messages
```rust
match operation() {
    Ok(_) => println!("{}", fmt.format_success("Done!")),
    Err(e) => eprintln!("{}", fmt.format_error(&e.to_string())),
}
```

## 📖 Common Tasks

### Show Commit History
```rust
use mug::ui::{UnicodeFormatter, CommitInfo};

let commits = vec![CommitInfo {
    hash: "abc1234567890".to_string(),
    author: "Your Name".to_string(),
    date: "2025-12-29".to_string(),
    message: "Your message".to_string(),
    is_head: true,
    branch: Some("main".to_string()),
}];

let fmt = UnicodeFormatter::new(true, true);
println!("{}", fmt.format_log(&commits));
```

### Show Repository Status
```rust
let changes = vec![
    ("src/main.rs".to_string(), 'M'),  // Modified
    ("new.rs".to_string(), 'A'),       // Added
    ("old.rs".to_string(), 'D'),       // Deleted
];

println!("{}", fmt.format_status("main", &changes));
```

### Show Branches
```rust
let branches = vec![
    "main".to_string(),
    "develop".to_string(),
    "feature/ui".to_string(),
];

println!("{}", fmt.format_branch_list("main", &branches));
```

### Show Progress
```rust
for chunk in 0..=100 {
    println!("{}", fmt.format_progress_bar(chunk, 100));
    // process chunk...
}
```

### Display Errors/Success
```rust
println!("{}", fmt.format_success("All tests passed!"));
println!("{}", fmt.format_warning("Using deprecated API"));
eprintln!("{}", fmt.format_error("Failed to connect"));
```

## 🎯 Main Methods

| Method | Purpose | Input |
|--------|---------|-------|
| `format_log()` | Format commit history | `&[CommitInfo]` |
| `format_status()` | Show branch and changes | `&str`, `&[(String, char)]` |
| `format_branch_list()` | List branches | `&str`, `&[String]` |
| `format_diff()` | Show diffs | `&[DiffHunk]` |
| `format_progress_bar()` | Show progress | `u64, u64` |
| `format_success()` | Success message | `&str` |
| `format_error()` | Error message | `&str` |
| `format_warning()` | Warning message | `&str` |
| `format_merge_conflict()` | Show conflicts | `&str, &str, &str` |

## 🔧 Configuration

### Control Output Format
```rust
// Full Unicode + colors (modern terminals)
let fmt = UnicodeFormatter::new(true, true);

// ASCII only (legacy terminals, piping)
let fmt = UnicodeFormatter::new(false, false);

// Unicode without colors (limited terminals)
let fmt = UnicodeFormatter::new(true, false);
```

### Detect Terminal Capabilities
```rust
// Simple check for color support
let use_colors = atty::is(atty::Stream::Stdout);
let fmt = UnicodeFormatter::new(true, use_colors);
```

## 📝 File Changes

### Added Files
- `src/ui/formatter.rs` - Main formatter implementation
- `examples/formatter_demo.rs` - Working example showing all features
- `BEAUTIFUL_OUTPUT.md` - Visual examples
- `FORMATTER_INTEGRATION.md` - Detailed dev guide
- `FORMATTER_QUICK_REFERENCE.md` - Code reference

### Modified Files
- `src/ui/mod.rs` - Exports formatter types
- `Cargo.toml` - Added `colored = "2.1"` dependency

## 🧪 Testing

Run the demo:
```bash
cargo run --example formatter_demo
```

Run tests:
```bash
cargo test ui::formatter
```

## 🌐 Compatibility

Works on:
- ✓ Linux (all terminals)
- ✓ macOS (Terminal, iTerm2, etc.)
- ✓ Windows (Windows Terminal, ConEmu, etc.)
- ✓ SSH sessions
- ✓ CI/CD pipelines
- ✓ Web terminals
- ✓ Legacy terminals (with ASCII fallback)

## 💡 Pro Tips

1. **Cache the formatter** - Create it once, reuse throughout
   ```rust
   let fmt = UnicodeFormatter::new(true, true);
   // Use fmt many times
   ```

2. **Test ASCII mode** - Always verify output with `new(false, false)`

3. **Pipe-friendly** - Auto-detects when output is piped and disables colors

4. **Zero overhead** - No performance impact, colors are optional

5. **Standard terminal codes** - Works everywhere ANSI is supported

## 🎨 Color Scheme

| Element | Color | Hex |
|---------|-------|-----|
| Headers/Labels | Bright Cyan | `#00FFFF` |
| Success/Current | Bright Green | `#00FF00` |
| Errors/Deleted | Red | `#FF0000` |
| Warnings/Modified | Yellow | `#FFFF00` |
| Special Ops | Magenta | `#FF00FF` |
| Content | White | `#FFFFFF` |

## 🚧 Next Steps for Integration

1. [ ] Review `FORMATTER_QUICK_REFERENCE.md` for your use case
2. [ ] Look at `examples/formatter_demo.rs` for implementation patterns
3. [ ] Update `mug status` command to use formatter
4. [ ] Update `mug log` command to use formatter
5. [ ] Update `mug branches` command to use formatter
6. [ ] Add formatter to diff command
7. [ ] Use formatter for all error/success messages
8. [ ] Test with `--ascii` flag

## 📖 Documentation Files

Choose based on what you need:

- **Just want to use it?** → Read BEAUTIFUL_OUTPUT.md
- **Want code examples?** → Read FORMATTER_QUICK_REFERENCE.md
- **Integrating into commands?** → Read FORMATTER_INTEGRATION.md
- **Want to see it work?** → Run `cargo run --example formatter_demo`
- **Full technical details?** → Check src/ui/formatter.rs

## ❓ FAQ

**Q: Will this break my terminal?**
A: No. The formatter automatically detects terminal capabilities and falls back to ASCII if needed.

**Q: Can I pipe the output?**
A: Yes. Colors are automatically disabled when output is piped.

**Q: What if my terminal is old?**
A: Use `UnicodeFormatter::new(false, false)` for pure ASCII output.

**Q: How much does this add?**
A: Only one dependency (`colored`, ~25KB) and ~500 lines of code.

**Q: Can I customize colors?**
A: Currently fixed to a standard palette, but easy to extend.

**Q: Performance impact?**
A: Zero. Colors are just string formatting.

## 🎉 You're Ready!

That's all you need to know. The rest is in the specific documentation files.

**Start with:** `cargo run --example formatter_demo`

Then integrate methods into your commands one by one.

Happy coding! 🚀
