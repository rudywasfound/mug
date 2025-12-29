# Resume Feature Quick Start

## Installation

The resume feature is built into MUG. No additional setup needed.

## Basic Commands

### View Current Operations
```bash
mug resume
```
Shows all ongoing and paused operations with progress percentages.

### Get Full Details
```bash
mug resume show <op-id>
```
Display detailed progress, timestamps, and metadata for a specific operation.

### Pause an Operation
```bash
mug resume pause <op-id>
```
Saves the operation state for later resumption.

### Continue a Paused Operation
```bash
mug resume continue <op-id>
```
Shows how to resume the operation using the original command.

### Remove Old Operations
```bash
mug resume cleanup --days 30
```
Delete completed/failed operations older than 30 days.

### Filter by Status
```bash
mug resume list --paused      # Only paused operations
mug resume list --running     # Only running operations
mug resume list --completed   # Only completed operations
mug resume list --failed      # Only failed operations
```

## Common Workflows

### Scenario: Long-running pack creation gets interrupted

```bash
# Start pack creation
$ mug pack create output/
# (operation gets interrupted)

# Check what happened
$ mug resume
ID: op-abc12345efgh
  Type: pack
  Status: paused
  Progress: 45.2% (452)

# Resume it later
$ mug resume continue op-abc12345efgh
Resuming operation: op-abc1234 (pack)
Previous checkpoint: compressing
Progress: 452/1000 items

⚠️  Resume functionality is operation-specific
Run the original command with --resume op-abc1234 to continue

# The actual resume would look like:
$ mug pack create output/ --resume op-abc12345efgh
```

### Scenario: Clean up old operations from long project history

```bash
# See all operations
$ mug resume
ID: op-111 [pack] 100.0% (completed)
ID: op-222 [fetch] 100.0% (completed)
ID: op-333 [clone] 90.0% (failed)

# Delete operations older than 7 days
$ mug resume cleanup --days 7
✓ Cleaned up 42 old operations

# Verify cleanup
$ mug resume list --completed
# (only recent operations remain)
```

### Scenario: Monitor ongoing operations

```bash
# Check progress repeatedly
$ watch mug resume

# Or check specific operation
$ mug resume show op-abc12345efgh
Progress:
  Items: 452/1000
  Percentage: 45.2%
  Bytes: 1048576000/2097152000

# See more details including checkpoints
$ mug resume show op-abc12345efgh | grep Step
  Current Step: compressing
```

## Compression Reference

### What compression does MUG use?

MUG uses **Zstandard (zstd)** for native pack files and **Flate2/zlib** for Git compatibility.

**Zstandard advantages:**
- 5-10x faster than zlib
- Better compression ratios
- Used in all pack operations

**Flate2/zlib advantages:**
- Git compatible
- Used when importing from Git
- Automatic fallback for legacy data

### Do I need to configure compression?

No. Compression is automatic:
- **Pack operations**: Use fast zstd (level 3) for real-time feedback
- **Batch operations**: Use optimized zstd (level 10) for space
- **Git compatibility**: Auto-detect and fallback as needed

## Troubleshooting

### "Operation not found"
- Check operation ID is correct: `mug resume` to list
- Operation may have been deleted: `mug resume list --completed`
- Try full operation ID instead of shortened version

### Operation status isn't updating
- Verify operation is "running": `mug resume show <id>`
- Check for error messages: look for `error_message` in details
- May be genuinely stalled - consider pausing and checking logs

### Want to forcefully stop an operation?
```bash
# Pause it (saves state for resumption)
mug resume pause <op-id>

# Or delete it (removes from history)
mug resume delete <op-id>
```

## Tips & Best Practices

1. **Regular cleanup**: Run `mug resume cleanup` monthly to keep database lean
2. **Monitor large operations**: Use `watch mug resume` to monitor pack operations
3. **Save operation IDs**: Note the ID if you might need to resume later
4. **Check before resuming**: Use `show` to verify state before continuing
5. **Use filters**: `--paused` to quickly find operations needing attention

## Integration with Other Commands

The resume feature integrates with:

- `mug pack create` - Track and resume pack file creation
- `mug clone` - Resume interrupted clones
- `mug fetch` - Resume partial downloads
- `mug push` - Track push operations
- `mug rebase` - Multi-step rebase operations
- `mug merge` - Conflict resolution state

*Note: Resume support for these commands is being added incrementally.*

## Example Session

```bash
# Initialize repo
$ mug init .
Initialized empty MUG repository in "."

# Check for operations (should be none)
$ mug resume
No operations found

# Later, with a long operation...
$ mug resume
Resumable Operations:

ID: op-a1b2c3d4e5f6g7h8
  Type: pack
  Status: running
  Progress: 23.5% (235)
  Step: chunking
  Updated: 2025-12-29T14:30:45Z

# Get details
$ mug resume show op-a1b2c3d4e5f6g7h8
Operation Details:
  ID: op-a1b2c3d4e5f6g7h8
  Type: pack
  Status: running
  Progress: 23.5% (235/1000 items)
  Bytes: 524288000/2097152000
  Current Step: chunking

# Pause it
$ mug resume pause op-a1b2c3d4e5f6g7h8
✓ Operation paused

# Later resume
$ mug resume continue op-a1b2c3d4e5f6g7h8
Resuming operation: op-a1b2c3 (pack)
Previous checkpoint: chunking

# Clean old operations
$ mug resume cleanup --days 30
✓ Cleaned up 5 old operations (older than 30 days)
```

## Related Documentation

- **RESUME_FEATURE.md** - Comprehensive feature documentation
- **COMPRESSION_ARCHITECTURE.md** - Deep dive into compression system
- **IMPLEMENTATION_SUMMARY.md** - Technical implementation details
