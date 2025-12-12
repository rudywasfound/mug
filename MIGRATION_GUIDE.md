# Git to MUG Migration Guide

Migrate your existing Git repositories to MUG using the built-in migration tool.

## Quick Start

### Via HTTP API

```bash
# 1. Start MUG server
./target/release/mug serve --host 0.0.0.0 --port 8080 --repos /path/to/mug-repos

# 2. Trigger migration
curl -X POST \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"git_path": "/path/to/your/git/repo"}' \
  http://localhost:8080/repo/myrepo/migrate-from-git

# Response:
# {
#   "success": true,
#   "message": "Migrated X branches to MUG. Next: implement commit/object import.",
#   "repo": "myrepo"
# }
```

### Via CLI (Future)

```bash
# Direct migration from command line
mug migrate /path/to/git/repo /path/to/mug/repo

# Or within a MUG repository
cd /path/to/mug/repo
mug import-git /path/to/git/repo
```

## What Gets Migrated

✅ **Branch References** - All branches are imported with their names  
✅ **Repository Structure** - .git/config, refs, HEAD state  
⏳ **Commit History** - TODOs: Parse and import commit objects  
⏳ **Objects** - TODOs: Convert blobs, trees to MUG format  
⏳ **Tags** - TODOs: Import annotated and lightweight tags  

## Migration Process

### Step 1: Verify Source Repository

```bash
# Check it's a Git repository
ls -la /path/to/repo/.git/

# List branches
ls /path/to/repo/.git/refs/heads/
```

### Step 2: Trigger Migration

```bash
# Via HTTP
curl -X POST \
  -H "Authorization: Bearer token" \
  -H "Content-Type: application/json" \
  -d '{"git_path": "/full/path/to/git/repo"}' \
  http://localhost:8080/repo/newname/migrate-from-git
```

### Step 3: Verify Migration

```bash
# Check repository was created
ls -la /path/to/mug-repos/newname/

# List branches (once import is fully implemented)
curl -H "Authorization: Bearer token" \
  http://localhost:8080/repo/newname/list-branches
```

## API Endpoint

### POST /repo/{name}/migrate-from-git

Migrate a Git repository to MUG.

**Parameters:**
- `name` - New MUG repository name
- `git_path` - Absolute path to source Git repository

**Request:**
```json
{
  "git_path": "/full/path/to/git/repository"
}
```

**Response Success:**
```json
{
  "success": true,
  "message": "Migrated X branches to MUG...",
  "repo": "myrepo"
}
```

**Response Error:**
```json
{
  "error": "Not a Git repository"
}
```

**Authentication:** Bearer token with write permission

**Status Codes:**
- `200` - Migration started successfully
- `400` - Invalid Git path or missing parameters
- `401` - Missing/invalid authentication token
- `403` - Permission denied

## Remaining Work

The migration infrastructure is in place. These functions need implementation:

### git_compat Module Todos

```rust
// In src/remote/git_compat.rs

// Parse Git objects and convert to MUG format
fn import_git_commits(git_path: &Path, repo: &Repository) -> Result<()> {
    // TODO: Walk git/objects directory
    // TODO: Read commit objects
    // TODO: Convert to MUG Commit format
    // TODO: Store in repository
}

fn import_git_blobs(git_path: &Path, repo: &Repository) -> Result<()> {
    // TODO: Extract blob objects from Git
    // TODO: Store in MUG object store
}

fn import_git_trees(git_path: &Path, repo: &Repository) -> Result<()> {
    // TODO: Parse Git tree objects
    // TODO: Create corresponding MUG trees
}
```

### Complete Implementation

Once complete, the migration will support:

1. **Full commit history** with all metadata
2. **All object types** (blobs, trees, commits, tags)
3. **Branch preservation** with proper heads
4. **Author information** intact
5. **Timestamps** from original commits
6. **Merge commits** with parent tracking
7. **Tags** (annotated and lightweight)

## Use Cases

### 1. Single Repository Migration

```bash
# Migrate one repo from GitHub/GitLab to internal MUG server
curl -X POST -H "Authorization: Bearer token" \
  -d '{"git_path": "/tmp/github-repo"}' \
  http://git.company.com:8080/repo/my-project/migrate-from-git
```

### 2. Batch Migration Script

```bash
#!/bin/bash
for repo in /mnt/git-backups/*/.git; do
  repo_dir=$(dirname "$repo")
  repo_name=$(basename "$repo_dir")
  
  curl -X POST \
    -H "Authorization: Bearer $TOKEN" \
    -d "{\"git_path\": \"$repo_dir\"}" \
    http://localhost:8080/repo/$repo_name/migrate-from-git
done
```

### 3. CI/CD Integration

```yaml
# GitHub Actions example (future)
- name: Migrate to MUG
  run: |
    mug migrate $GITHUB_WORKSPACE $MUG_REPO_PATH
```

## Performance

Migration times depend on repository size:

| Repo Size | Branches | Est. Time |
|-----------|----------|-----------|
| < 100MB | < 10 | < 1 second |
| 100MB - 1GB | 10-50 | 1-5 seconds |
| 1GB - 10GB | 50-100 | 5-30 seconds |
| > 10GB | 100+ | 30+ seconds |

Current implementation (TODOs pending): Tests with actual Git objects will determine real performance.

## Troubleshooting

### "Not a Git repository"

```bash
# Verify .git directory exists
test -d /path/to/repo/.git && echo "Valid Git repo" || echo "Not a Git repo"

# Check it's not a bare repository
test -f /path/to/repo/.git/config && echo "Regular repo" || echo "Bare repo"
```

### "Permission denied"

```bash
# Ensure MUG server can read the Git repository
chmod -R +r /path/to/git/repo
chmod +x /path/to/git/repo/.git
```

### Migration hangs

```bash
# Check server logs
RUST_LOG=debug ./target/release/mug serve ...

# Timeout after waiting
# Check if large repository is processing
du -sh /path/to/git/repo
```

## Comparison: Git vs MUG

### Git Model
```
working directory
    ↓
staging area (index)
    ↓
commit (loose or packed objects)
```

### MUG Model
```
working directory
    ↓
staging area (Sled database)
    ↓
commit (content-addressable store)
```

### Key Differences

| Aspect | Git | MUG |
|--------|-----|-----|
| Storage | File-based loose/packed | Content-addressable hashing |
| Index | .git/index file | Sled embedded database |
| Objects | Loose files + packs | Single object store |
| Deduplication | Manual (gc) | Automatic (SHA256) |
| Performance | O(n) status | O(1) status |

## Post-Migration

After migration completes:

### 1. Verify Data

```bash
# Check repository structure
ls -la /path/to/mug-repos/myrepo/.mug/

# Verify branches imported
mug branches -r /path/to/mug-repos/myrepo
```

### 2. Test Operations

```bash
# Clone via MUG protocol
mug clone http://token@ip:8080/repo/myrepo

# Or with Git (once full Git protocol is implemented)
git clone http://token@ip:8080/repo/myrepo
```

### 3. Update Team Workflows

```bash
# Replace Git remote
git remote set-url origin http://ip:8080/repo/myrepo

# Or switch to MUG client
mug remote add origin http://ip:8080/repo/myrepo
```

## Support

For issues or questions:
1. Check repository exists: `ls /path/to/repo/.git`
2. Verify server is running: `curl http://localhost:8080/health`
3. Review logs: `RUST_LOG=debug ./target/release/mug serve ...`
4. Check permissions on source repo
5. Ensure write access to target directory

## See Also

- [IMPROVEMENTS.md](IMPROVEMENTS.md) - Remote server features
- [REMOTE_SERVER_USAGE.md](REMOTE_SERVER_USAGE.md) - API documentation
- [QUICK_START.md](QUICK_START.md) - Quick reference
