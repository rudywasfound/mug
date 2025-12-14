# TUI Quick Reference

## Interactive Rebase

Start:
```bash
mug rebase -i HEAD~5
```

Navigation:
- j: Down
- k: Up
- Page Down: Scroll down
- Page Up: Scroll up
- Home: First line
- End: Last line

Actions:
- p: pick (use commit)
- s: squash (merge with previous)
- r: reword (edit message)
- d: drop (remove)
- e: edit (pause for changes)

Execute:
- Enter: Apply
- Esc: Cancel
- q: Quit

## Conflict Resolution

View Conflicts:
```bash
mug status
```

Markers:
```
<<<<<<< HEAD
your changes
=======
their changes
>>>>>>> branch
```

Resolve:
1. Open file in editor
2. Choose one version or combine
3. Remove markers
4. Save

Mark Resolved:
```bash
mug add <file>
```

Complete:
```bash
mug commit
```

## Log View

Show Log:
```bash
mug log
```

Oneline:
```bash
mug log --oneline
```

Recent:
```bash
mug log -n 10
```

Details:
```bash
mug show <commit>
```

## Status View

Current Status:
```bash
mug status
```

Shows:
- Current branch
- Staged files
- Modified files
- Untracked files
- Conflicts

## Common Workflows

Create and Switch Branch:
```bash
mug branch feature
mug checkout feature
```

Commit Changes:
```bash
mug add .
mug commit -m "message"
```

Rebase onto Main:
```bash
mug rebase main
```

Interactive Rebase:
```bash
mug rebase -i main
# Use TUI to select actions
```

Merge Feature:
```bash
mug checkout main
mug merge feature
```

## Keyboard Shortcuts

General:
- Ctrl+C: Cancel
- q: Quit
- ?: Help (if available)
- Enter: Confirm/Execute

Navigation:
- j/k or Arrow Keys: Move
- Page Up/Down: Scroll
- Home/End: Jump

Editing:
- Esc: Cancel edit
- Enter: Confirm edit

## Colors

In Terminal Output:
- Green: New/Added
- Red: Deleted
- Yellow: Modified
- White: Untracked

## Tips

Use Oneline for Quick Overview:
```bash
mug log --oneline -n 20
```

Squash Commits Before Merge:
```bash
mug rebase -i main
# Mark commits to squash with 's'
```

Check Status Before Actions:
```bash
mug status
```

Review Changes Before Commit:
```bash
mug diff
```

## Common Tasks

List All Branches:
```bash
mug branches
```

Delete Branch:
```bash
mug branch -d feature
```

Undo Last Commit:
```bash
mug reset soft HEAD~1
```

View Specific Commit:
```bash
mug show abc1234
```

Show Diff:
```bash
mug diff HEAD~1 HEAD
```

Find Commit:
```bash
mug log --oneline | grep "keyword"
```

## Error Recovery

Abort Rebase:
```bash
mug rebase --abort
```

Abort Merge:
```bash
mug merge --abort
```

Undo Reset:
```bash
mug reflog
mug reset --hard <commit>
```

Recover Branch:
```bash
mug reflog
mug branch <name> <commit>
```

## Performance Tips

Use Oneline for Speed:
```bash
mug log --oneline -n 100
```

Limit Commit Range:
```bash
mug log -n 20
```

Use Grep for Search:
```bash
mug grep "pattern"
```

Check Status for Overview:
```bash
mug status
```

## Merge Conflict TUI

Conflict Markers:
```
<<<<<<< HEAD
Your version
=======
Their version
>>>>>>> branch-name
```

Resolution Steps:
1. Identify conflicts with `mug status`
2. Open file in text editor
3. Review both versions
4. Choose or combine versions
5. Remove conflict markers
6. Save file
7. Run `mug add <file>`
8. Run `mug commit`

## Advanced Commands

Reorder Commits:
```bash
mug rebase -i HEAD~5
# Move lines in editor
```

Extract Commit:
```bash
mug cherry-pick abc1234
```

Split Commit:
```bash
mug rebase -i HEAD~3
# Mark commit with 'e' to edit
# Make changes
# Create new commit
```

Combine Branches:
```bash
mug merge other-branch
```

## Reference

For full documentation: see DOCS.md
For quick start: see QUICK_START.md
For features: see FEATURE_SUMMARY.md
