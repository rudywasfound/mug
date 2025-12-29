# Interactive Branch Selection - `mug branches`

The `mug branches` command now supports interactive branch selection and switching.

## Usage

```bash
mug branches
```

This will:
1. Display all branches with the current branch highlighted
2. Enter interactive mode allowing you to navigate and select

## Navigation Controls

| Key | Action |
|-----|--------|
| `TAB` | Move to next branch |
| `↓` (Down Arrow) | Move to next branch |
| `⇧ TAB` (Shift+Tab) | Move to previous branch |
| `↑` (Up Arrow) | Move to previous branch |
| `ENTER` | Switch to selected branch |
| `ESC` | Cancel and exit |

## Example Session

```bash
$ mug branches

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Branches
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
● feature/test (current)
○ main
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Use TAB/↓ to navigate, ENTER to select, ESC to cancel

  → ● feature/test
    ○ main
```

(Press TAB to move to next)

```
  → ● feature/test
    ○ main       ← This is now highlighted
```

(Press ENTER to switch)

```
✓ success: Switched to branch: main
```

## Features

### Beautiful Display
- Lists all branches with Unicode symbols
- Current branch is marked with `●` (bright green)
- Other branches marked with `○` (cyan)
- Styled header and borders

### Interactive Navigation
- Use Tab or arrow keys to navigate
- Currently selected branch is highlighted in bright green
- Arrow indicator (`→`) shows selection
- Smooth, responsive controls

### Smart Switching
- Automatically switches to selected branch
- Shows success message on switch
- Shows warning if selecting current branch
- Shows error message if switch fails

### Keyboard Shortcuts
- **Tab/Down**: Next branch
- **Shift+Tab/Up**: Previous branch
- **Enter**: Select and switch
- **Escape**: Cancel

## Visual Feedback

### Branch List Display
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Branches
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
● feature/awesome (current)
○ main
○ develop
○ hotfix/urgent
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Interactive Selection
```
Use TAB/↓ to navigate, ENTER to select, ESC to cancel

  ○ feature/awesome
  → ● main           ← Selected
    ○ develop
    ○ hotfix/urgent
```

### Result
```
✓ success: Switched to branch: main
```

## Behavior

### On Selection
- If you select a different branch: Switches to it and shows success message
- If you select current branch: Shows warning that you're already on it
- If switch fails: Shows error message

### On Cancellation (ESC)
- Exits interactive mode
- No branch is switched
- Screen is cleared

### Terminal Compatibility
Works on:
- ✓ Linux terminals (all)
- ✓ macOS (Terminal, iTerm2, etc.)
- ✓ Windows Terminal
- ✓ SSH sessions
- ✓ Git Bash
- ✓ Most modern terminals supporting crossterm

## Implementation

Uses `crossterm` library for:
- Raw mode terminal input
- Cross-platform keyboard event handling
- Terminal control sequences
- Proper screen clearing

No external dependencies beyond what MUG already uses.

## Code Example

The interactive selector is accessed via:

```rust
use mug::ui::select_branch_interactive;

let branches = vec!["main", "develop", "feature/ui"];
let current = "main";

if let Some(selected) = select_branch_interactive(branches, current) {
    // User selected a branch
    repo.checkout(selected)?;
}
```

## Tips

1. **Quick navigation**: Use Tab for sequential movement, arrow keys for any direction
2. **Muscle memory**: Works like terminal menus and interactive commands
3. **No mistakes**: Visual feedback confirms which branch you'll switch to before pressing Enter
4. **Always safe**: Can press ESC at any time to cancel

## Limitations

- Interactive mode only works in TTY environments
- Non-interactive environments (piped output) skip the interactive selector
- Requires a terminal that supports raw mode (not available in all environments)

When interactive mode is unavailable, the command still displays the branch list normally.

## Future Enhancements

Potential additions:
- Search/filter branches by name
- Show branch creation date
- Show branch description/notes
- Show last commit info for each branch
- Multi-select branches
- Delete branches from the selector

## Related Commands

- `mug branch <name>` - Create a new branch
- `mug checkout <branch>` - Switch to a branch (non-interactive)
- `mug log` - View commit history
- `mug status` - Show repository status
