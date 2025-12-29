# MUG Resume Feature

## Overview

The resume feature allows users to track and manage long-running operations that can be paused, resumed, or cleaned up. This is particularly useful for operations like:

- Pack file creation (`mug pack create`)
- Clone/fetch operations (`mug clone`, `mug fetch`)
- Push operations (`mug push`)
- Rebase operations (`mug rebase`)
- Merge operations (`mug merge`)

Instead of restarting entire operations from scratch, users can now:

1. **Check the status** of ongoing operations
2. **View detailed progress** including items processed and bytes transferred
3. **Pause** a running operation when needed
4. **Resume** from the last checkpoint
5. **Delete** operation history
6. **Clean up** old completed/failed operations

## Usage

### View All Operations

List all resumable operations in your repository:

```bash
mug resume
```

This displays:
- Operation ID (shortened)
- Operation type (pack, clone, fetch, etc.)
- Current status (running, paused, completed, failed)
- Progress percentage and item count
- Current step/checkpoint
- Last updated timestamp

### Filter Operations by Status

```bash
# Show only paused operations
mug resume list --paused

# Show only running operations
mug resume list --running

# Show only completed operations
mug resume list --completed

# Show only failed operations
mug resume list --failed
```

You can combine multiple filters to see multiple statuses at once.

### View Operation Details

Get detailed information about a specific operation:

```bash
mug resume show <operation-id>
```

This shows:
- Full operation ID
- Type and status
- Creation time, start time, and last update
- Detailed progress metrics (items, bytes, percentage)
- Current step and total steps
- Any error messages (if failed)
- Custom metadata

### Resume a Paused Operation

Resume a previously paused operation:

```bash
mug resume continue <operation-id>
```

This will display the operation details and instructions for how to continue it using the original command with a `--resume` flag.

### Pause a Running Operation

Pause a currently running operation to resume later:

```bash
mug resume pause <operation-id>
```

The operation state is saved to the database, preserving:
- Current checkpoint
- Progress metrics
- All processed items
- Custom metadata

### Delete an Operation

Remove an operation from the history:

```bash
mug resume delete <operation-id>
```

This removes the operation from the database but doesn't affect any data that was already processed.

### Clean Up Old Operations

Automatically remove old completed or failed operations:

```bash
# Delete operations older than 30 days (default)
mug resume cleanup

# Delete operations older than 7 days
mug resume cleanup --days 7

# Delete operations older than 90 days
mug resume cleanup --days 90
```

## Implementation Details

### Architecture

The resume feature is built with a trait-based design that allows operations to be resumable:

```
Operation Manager
├── Create new operations
├── Update status/progress
├── Retrieve operation state
├── Persist to database
└── Cleanup old entries
```

### Database Schema

Operations are stored in the MUG database under the "operations" key with the following structure:

```json
{
  "id": "op-{uuid}",
  "op_type": "pack|clone|fetch|push|rebase|merge|custom",
  "status": "running|paused|completed|failed",
  "created_at": "RFC3339 timestamp",
  "started_at": "RFC3339 timestamp",
  "last_updated": "RFC3339 timestamp",
  "state": {
    "checkpoint": "arbitrary JSON string",
    "current_step": "step name",
    "total_steps": 42,
    "error_message": "error details if failed",
    "metadata": {
      "custom_key": "custom_value"
    }
  },
  "progress": {
    "processed": 100,
    "total": 1000,
    "bytes_processed": 1048576,
    "total_bytes": 10485760
  }
}
```

### Compression Handling

MUG uses a dual-compression approach:

1. **Zstd** (primary): For native MUG pack files
   - 5-10x faster than zlib
   - Better compression ratios
   - Used in `ZstdCompressor`

2. **Flate2/zlib** (fallback): For Git compatibility
   - Used in `FlateCompressor`
   - Maintains compatibility with Git objects
   - Used in `src/remote/git_compat.rs`

The compression selection is transparent based on the file format being processed.

### Checkpoint System

Each operation maintains a checkpoint that includes:

- **Current Step**: Named checkpoint (e.g., "initialized", "chunking", "compressing")
- **Step Count**: Track progress through multi-step operations
- **Serialized State**: Custom JSON state for operation-specific data
- **Metadata**: Arbitrary key-value pairs for operation-specific information

### Progress Tracking

Operations track two types of progress:

1. **Item Progress**: Number of items processed out of total
2. **Byte Progress**: Number of bytes transferred out of total

This allows accurate ETA calculations and bandwidth monitoring.

## Integration with Long-Running Commands

### Extending Commands to Support Resume

To add resume support to a command, integrate with `OperationManager`:

```rust
use mug::core::resume::{OperationManager, OperationType};

let manager = OperationManager::new(repo.get_db().clone());

// Create operation
let op = manager.create(
    OperationType::Pack,
    serde_json::to_string(&checkpoint)?,
    metadata,
)?;

// Update progress
manager.update_progress(
    &op.id,
    processed_items,
    Some(total_items),
    processed_bytes,
    Some(total_bytes),
)?;

// Update checkpoint
manager.update_checkpoint(
    &op.id,
    serde_json::to_string(&new_checkpoint)?,
    "compressing".to_string(),
    Some(total_steps),
)?;

// Mark as complete
manager.complete(&op.id)?;
```

### Example: Pack Command Integration

The pack command can be extended to track creation progress:

```bash
mug pack create output/
# Creates operation, saves progress as files are packed

mug resume
# Shows pack operation at 45% completion

# (interrupted)

mug resume continue <op-id>
# Shows how to resume with: mug pack create output/ --resume <op-id>
```

## Best Practices

1. **Frequent Checkpoints**: Update checkpoints at regular intervals (e.g., every 100 items)
2. **Clear Step Names**: Use descriptive step names for debugging
3. **Include Metadata**: Store relevant context in metadata (source URL, target commit, etc.)
4. **Regular Cleanup**: Periodically run `mug resume cleanup` to keep the database lean
5. **Error Context**: When failing, include detailed error messages for debugging

## Technical Specification

### OperationManager Public API

```rust
pub fn create(
    op_type: OperationType,
    checkpoint: String,
    metadata: HashMap<String, String>,
) -> Result<Operation>

pub fn get(&self, op_id: &str) -> Result<Option<Operation>>

pub fn update_status(&self, op_id: &str, status: OperationStatus) -> Result<()>

pub fn update_progress(
    &self,
    op_id: &str,
    processed: u64,
    total: Option<u64>,
    bytes_processed: u64,
    total_bytes: Option<u64>,
) -> Result<()>

pub fn update_checkpoint(
    &self,
    op_id: &str,
    checkpoint: String,
    current_step: String,
    total_steps: Option<usize>,
) -> Result<()>

pub fn complete(&self, op_id: &str) -> Result<()>

pub fn fail(&self, op_id: &str, error: &str) -> Result<()>

pub fn list(
    &self,
    status_filter: Option<OperationStatus>,
) -> Result<Vec<Operation>>

pub fn get_latest_pausable(&self, op_type: &str) -> Result<Option<Operation>>

pub fn get_running(&self, op_type: &str) -> Result<Option<Operation>>

pub fn delete(&self, op_id: &str) -> Result<()>

pub fn cleanup_old(&self, days_old: i64) -> Result<usize>
```

### Module Location

- **Core Module**: `src/core/resume.rs`
- **Main Handler**: `src/main.rs` (Commands enum and handler)
- **Database Integration**: Uses existing `MugDb` from `src/core/database.rs`

## Future Enhancements

1. **Operation Hooks**: Pre/post-operation callbacks for custom logging
2. **Progress Estimation**: Automatic ETA calculation based on historical data
3. **Operation Replay**: Ability to replay operation logs for debugging
4. **Concurrent Operations**: Track multiple concurrent operations
5. **Operation Dependencies**: Chain dependent operations
6. **Progress Webhooks**: Send progress updates to external systems
7. **Selective Resume**: Resume from specific checkpoint, not just latest

## Testing

The resume module includes unit tests for:

- Progress percentage calculation
- Operation type/status string representations
- Zero-item progress handling
- Operation state creation

Run tests with:

```bash
cargo test core::resume
```

## Troubleshooting

### Operation Not Found

If you get "Operation not found":
1. Check the operation ID is correct (use `mug resume` to list)
2. Verify the operation hasn't been deleted
3. Check it's in the correct repository

### Progress Not Updating

If progress seems stuck:
1. Verify the operation is still "running" status
2. Check for error messages with `mug resume show <id>`
3. Consider pausing and deleting if the operation is genuinely stalled

### Database Corruption

If operations are corrupted:
1. Run `mug resume cleanup --days 0` to remove all operations
2. Verify repository integrity with `mug verify`
3. Run garbage collection with `mug gc`
