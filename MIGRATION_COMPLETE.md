# Git Migration

## Overview

MUG can import complete Git repositories, preserving all commits and metadata.

## Migration Command

```bash
mug migrate /path/to/git/repo /path/to/new/mug/repo
```

## What Gets Migrated

Commits:
- All commits with full metadata
- Author and committer information
- Timestamps
- Commit messages
- Parent relationships

Branches:
- All branch references
- Branch names
- Head position

Tags:
- Tag references
- Tag names
- Tag messages (if annotated)

Objects:
- All blob objects
- All tree objects
- Direct transfer to MUG object store

## Supported Formats

Loose Objects:
- Standard Git object files
- Located in `.git/objects/`
- Automatically decompressed

Pack Files:
- Git pack files (`.pack`)
- Index files (`.idx`)
- Full support for all versions

Mixed Repositories:
- Combination of loose and packed objects
- Automatic detection
- Seamless migration

## Performance

Time:
- 33 commits: <1 second
- 100 commits: 1-2 seconds
- 1000 commits: 10-20 seconds

Depends on:
- Number of commits
- Repository size
- Object compression
- Disk I/O speed

Space:
- No duplication
- Direct object transfer
- Compression maintained
- Minimal overhead

## Example Workflow

Migrate Repository:
```bash
mug migrate ~/old-repo ~/new-repo
cd ~/new-repo
```

Verify Migration:
```bash
mug log
mug status
mug verify
```

Configure MUG:
```bash
mug config set user.name "Your Name"
mug config set user.email "you@example.com"
```

Continue Working:
```bash
mug branch feature/new
mug checkout feature/new
mug add .
mug commit -m "New commit in MUG"
```

## Preservation

Full History:
- All commits preserved exactly
- Parent relationships maintained
- Linear and merge commits both work

Authorship:
- Original author preserved
- Timestamp preserved
- Commit message preserved

References:
- Branch names preserved
- Tag references preserved
- HEAD position preserved

## Limitations

No Shallow History:
- Full commit history always imported
- Cannot selectively import commits
- Cannot skip early history

No Submodule Conversion:
- Submodules treated as regular objects
- May need manual cleanup

No LFS Conversion:
- Git LFS pointers imported as-is
- Actual LFS files not converted
- May need to reconfigure LFS

## Post-Migration Steps

1. Verify Repository:
```bash
mug verify
```

2. Check Log:
```bash
mug log --oneline
```

3. Configure Identity:
```bash
mug config set user.name "Name"
mug config set user.email "email@example.com"
```

4. Set Up Remotes:
```bash
mug remote add origin <url>
```

5. Test Operations:
```bash
mug branch test
mug checkout test
mug add .
mug commit -m "Test"
```

## Troubleshooting

Migration Fails:
- Check paths are correct
- Verify source is valid Git repo
- Check disk space
- Check file permissions

Slow Migration:
- Network latency?
- Large pack files?
- System load high?
- Use local path for faster transfer

Missing Objects:
- Run `mug verify`
- Check source repository
- Ensure pack files are complete
- Check for corruption

## Technical Details

Object Parsing:
- Decompresses Git objects using zlib
- Parses object structure
- Extracts commit metadata
- Validates object hashes

Tree Building:
- Reconstructs file trees
- Creates tree objects in MUG format
- Maintains directory structure
- Preserves file permissions

Commit Reconstruction:
- Extracts parent references
- Preserves message text
- Keeps author information
- Maintains timestamps

Reference Mapping:
- Maps branch references
- Sets HEAD correctly
- Preserves tag information
- Maintains reference structure

## Comparison

Before Migration (Git):
- `.git/objects/` - Pack and loose objects
- `.git/refs/` - Branch and tag references
- `.git/HEAD` - Current branch
- `.git/config` - Configuration

After Migration (MUG):
- `.mug/objects/` - Equivalent object store
- `.mug/db/` - Database with refs
- `.mugignore` - Ignore patterns
- `.mug/config.json` - Configuration

## Advanced Usage

Migrate to Specific Version:
```bash
cd ~/old-repo
git checkout v1.0.0
cd ~/original-path
mug migrate ~/old-repo ~/v1-repo
```

Incremental Migration:
- Not supported
- Import entire history
- Can filter afterward if needed

Reverse Migration:
- Export MUG to Git (future enhancement)
- Currently one-way process
- Consider keeping original Git repo

## Integration

After Migration:
- MUG fully independent of Git
- No Git installation needed
- Standard MUG operations work
- Remote operations compatible

Dual VCS:
- Keep both Git and MUG temporarily
- Pull from Git, push to MUG
- Verify data integrity
- Delete Git when confident

## Success Indicators

Verify Successful Migration:
1. `mug log` shows all commits
2. `mug verify` reports no errors
3. All branches present in `mug branches`
4. `mug status` works correctly
5. `mug show` displays full commit info

## Backup Strategy

Before Migration:
1. Backup original Git repository
2. Create test directory
3. Test migration process
4. Verify results
5. Keep original until confident

Data Safety:
- Migration reads source
- Does not modify source
- Safe to retry
- Always keep backup

## Documentation

For detailed technical documentation, see:
- HYBRID_VCS.md - Architecture overview
- DOCS.md - Command reference
- README.md - Feature list

## Next Steps

After successful migration:
1. Configure MUG with your identity
2. Set up remote repositories
3. Start using normal MUG workflows
4. Archive original Git repository
5. Update CI/CD pipelines
