# Error Handling

## Error Types

IoError:
- File system errors
- Permission denied
- File not found
- Device I/O errors

DatabaseError:
- Database connection failures
- Corruption detected
- Transaction errors
- Data integrity issues

NotARepository:
- `.mug` directory not found
- Invalid repository state

NoCommits:
- Repository has no commits
- Cannot perform operation without history

BranchNotFound:
- Referenced branch does not exist
- Check branch name spelling

CommitNotFound:
- Commit hash not found in database
- Hash may be incomplete or invalid

ObjectNotFound:
- Object (blob/tree) not in store
- Repository may be corrupted

SerializationError:
- JSON encoding/decoding failure
- Invalid data format

Conflicts:
- Merge conflicts detected
- Working directory has conflicts
- Manual resolution required

Utf8Error:
- Invalid UTF-8 encoding
- File contains invalid characters

CustomError:
- Application-specific error
- Custom error message included

## Common Error Messages

"Not a mug repository"
- Fix: Run `mug init` in the directory

"No commits yet"
- Fix: Create a commit with `mug commit`

"File not found: <path>"
- Fix: Check file path, file may be deleted

"Branch not found: <branch>"
- Fix: Check branch name with `mug branches`

"Commit not found: <hash>"
- Fix: Check commit hash with `mug log`

"Working directory has conflicts"
- Fix: Resolve conflicts manually or run `mug merge --abort`

"Nothing to commit"
- Fix: Stage files with `mug add`

## Error Recovery

Verify Repository:
```bash
mug verify
```
Checks for corrupted objects and missing references.

Repair Database:
```bash
mug gc
```
Garbage collection may help recover corrupted data.

Reset to Previous State:
```bash
mug reset hard HEAD~1
```
Discards recent changes.

View Reflog:
```bash
mug reflog
```
Find previous HEAD positions for recovery.

## Error Handling in Operations

Add Operation:
- Checks file exists
- Verifies file is readable
- Handles permission errors
- Reports missing files

Commit Operation:
- Verifies index not empty
- Checks author configured
- Validates message
- Handles tree building errors

Merge Operation:
- Detects conflicts
- Reports conflicting files
- Prevents unsafe merges
- Requires manual conflict resolution

Push Operation:
- Validates remote URL
- Checks network connection
- Detects remote errors
- Reports rejection reasons

## Debugging

Enable Logging:
```bash
RUST_LOG=debug mug status
```

Check Configuration:
```bash
mug config list
```

Verify Repository:
```bash
mug verify
```

View Recent Changes:
```bash
mug log --oneline -n 20
```

Check Status:
```bash
mug status
```

## Best Practices

Always Stage Before Committing:
```bash
mug add .
mug commit -m "message"
```

Use Branches for Experimental Work:
```bash
mug branch feature/experimental
mug checkout feature/experimental
```

Keep Commits Small:
- Easier to bisect
- Cleaner history
- Simpler to revert

Review Before Pushing:
```bash
mug log origin/main..HEAD
```

Use Stash for Context Switching:
```bash
mug stash
mug checkout other-branch
```

## Conflict Resolution

View Conflicting Files:
```bash
mug status
```

Edit Conflicting Files:
- Manually resolve markers
- Remove conflict indicators

Mark Resolved:
```bash
mug add <file>
```

Complete Merge:
```bash
mug commit -m "Merge branch x"
```

## Prevention Strategies

Use Frequent Commits:
- Small changesets
- Easier merges

Keep Branches Short-lived:
- Reduce merge conflicts
- Faster integration

Pull Before Push:
```bash
mug pull
mug push
```

Use Feature Branches:
- Isolate changes
- Parallel development

Run Tests Before Committing:
```bash
cargo test
mug commit
```

## Disaster Recovery

Lost Commits:
```bash
mug reflog
mug reset hard <commit-hash>
```

Corrupted Repository:
```bash
mug verify
mug gc
```

Deleted Branch:
```bash
mug reflog
mug branch <name> <commit-hash>
```

Failed Merge:
```bash
mug merge --abort
```

Failed Rebase:
```bash
mug rebase --abort
```

## Support

For unrecoverable errors:
1. Backup the `.mug` directory
2. Document the error message
3. Report with repository state if possible
4. Provide reproduction steps
