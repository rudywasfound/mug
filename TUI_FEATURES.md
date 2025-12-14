# TUI Features

## Interactive Rebase

Start Interactive Rebase:
```bash
mug rebase -i <branch>
```

Navigation:
- j: Move down
- k: Move up
- Page Down: Scroll down
- Page Up: Scroll up

Actions:
- p: pick - Use commit
- s: squash - Merge with previous
- r: reword - Edit commit message
- d: drop - Discard commit
- e: edit - Pause for amending

Execute:
- Enter: Apply selected actions
- Esc: Cancel rebase
- q: Quit

## Merge Conflict Resolution

View Conflicts:
```bash
mug status
```

Edit Files:
- Locate conflict markers
- Remove `<<<<<<`, `======`, `>>>>>>`
- Keep desired changes

Mark Resolved:
```bash
mug add <file>
```

Complete Merge:
```bash
mug commit -m "Merge message"
```

## Stash Management

Create Stash:
```bash
mug stash -m "Description"
```

List Stashes:
```bash
mug stash-list
```

Apply Stash:
```bash
mug stash-pop
```

## Branch Management TUI

List Branches:
```bash
mug branches
```

Create Branch:
```bash
mug branch <name>
```

Delete Branch:
```bash
mug branch -d <name>
```

Switch Branch:
```bash
mug checkout <name>
```

## Status Display

Show Status:
```bash
mug status
```

Displays:
- Current branch
- Staged files
- Modified files
- Untracked files
- Merge conflicts

## Log Visualization

Show History:
```bash
mug log
```

Compact View:
```bash
mug log --oneline
```

Show Specific Commit:
```bash
mug show <commit>
```

## Features

Keyboard Navigation:
- Arrow keys for movement
- Page up/down for scrolling
- Home/End for first/last

Color Output:
- Green: Added files
- Red: Deleted files
- Yellow: Modified files
- White: Untracked files

Context Information:
- Branch name
- Commit hash
- Author and timestamp
- Commit message

## Merge Conflict TUI

Conflict Markers:
```
<<<<<<< HEAD
Your changes
=======
Their changes
>>>>>>> branch-name
```

Resolution:
1. Open file in editor
2. Choose which version to keep
3. Remove conflict markers
4. Save file
5. `mug add <file>`
6. `mug commit`

## Interactive Commands

Edit Commit Message:
```bash
mug rebase -i HEAD~1
# Select 'r' for reword
```

Squash Commits:
```bash
mug rebase -i HEAD~3
# Change 'pick' to 's' for commits to squash
```

Drop Commits:
```bash
mug rebase -i HEAD~5
# Change 'pick' to 'd' for commits to drop
```

Reorder Commits:
```bash
mug rebase -i HEAD~5
# Move lines in the editor
```

## Display Options

Show Detailed Log:
```bash
mug log
```

Show Oneline Log:
```bash
mug log --oneline
```

Show Recent Commits:
```bash
mug log --oneline -n 10
```

Show Branch Commits Only:
```bash
mug log main..HEAD
```

## Error Display

Clear Error Messages:
- File not found
- Invalid branch
- Merge conflicts
- Permission denied

Error Recovery:
- Suggested commands
- Helpful hints
- Recovery options

## Output Formatting

Commit Display:
```
commit abc1234567890
Author: Name <email@example.com>
Date: 2024-01-15 14:30:45

    Commit message goes here
```

Status Display:
```
On branch: main
Staged: 3 files
Modified: 2 files
Untracked: 1 file
```

## Advanced Features

Terminal Compatibility:
- 256 color support
- UTF-8 output
- Window resizing support
- Terminal size detection

Performance:
- Fast rendering
- No flickering
- Efficient updates
- Smooth scrolling

Accessibility:
- Clear text labels
- Logical navigation
- Keyboard-only operation
- Terminal-friendly

## Tips

Use Rebase for Clean History:
```bash
mug rebase -i main
```

Squash Before Merging:
```bash
mug rebase -i main
# Mark commits to squash
```

Check Status Before Commit:
```bash
mug status
```

Use Oneline for Quick View:
```bash
mug log --oneline
```

## Known Limitations

Mouse Support:
- Not currently supported
- Keyboard navigation only

Copy/Paste:
- Terminal-dependent
- Use standard copy/paste

Large Commits:
- Display may be slow
- Pagination recommended

## Future Enhancements

Mouse Support:
- Click to select
- Drag to navigate
- Context menus

Diff Highlighting:
- Syntax highlighting
- Line-by-line diffs
- Word diffs

Branch Visualization:
- ASCII graph
- DAG display
- Merge tree

Search:
- Find commits
- Filter by author
- Search message
