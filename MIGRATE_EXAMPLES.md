# Migration Examples

## Basic Migration

Migrate a Git Repository:
```bash
mug migrate ~/my-git-repo ~/my-mug-repo
```

This command:
- Reads all commits from Git repo
- Decompresses objects
- Extracts metadata
- Creates MUG repository
- Preserves full history

## Verify Migration

After Migration:
```bash
cd ~/my-mug-repo
mug log --oneline
```

Check Status:
```bash
mug status
```

Verify Integrity:
```bash
mug verify
```

## Configure After Migration

Set User Information:
```bash
mug config set user.name "Your Name"
mug config set user.email "you@example.com"
```

## Add Remote

Add Original Git Repository as Remote:
```bash
mug remote add origin ~/my-git-repo
```

Or Remote URL:
```bash
mug remote add origin https://github.com/user/repo.git
```

## Continue Development

Make Changes:
```bash
mug branch feature/new
mug checkout feature/new
echo "content" > file.txt
mug add .
mug commit -m "Add feature"
```

Switch Back:
```bash
mug checkout main
```

## Migration with Large Repository

For Large Repository:
```bash
time mug migrate /path/to/large/repo /path/to/mug/repo
```

Monitor Progress:
- Watch for completion
- Verify final commit count
- Check object store size

## Backup Original

Before Migration:
```bash
cp -r ~/my-git-repo ~/my-git-repo.backup
```

This Ensures:
- Safe rollback
- Original preserved
- Data integrity

## Test Migration First

Create Test Directory:
```bash
mkdir ~/mug-test
mug migrate ~/my-git-repo ~/mug-test/repo
```

Verify in Test:
```bash
cd ~/mug-test/repo
mug log -n 5
```

Once Confident:
```bash
mug migrate ~/my-git-repo ~/production/repo
```

## Migration Examples

Simple Repo:
```bash
mug migrate ~/simple ~/simple-mug
```

Complex Repo (with branches):
```bash
mug migrate ~/complex ~/complex-mug
```

Large Repo (100+ commits):
```bash
mug migrate ~/large ~/large-mug
```

Remote URL (over HTTPS):
```bash
cd ~/temp
git clone https://github.com/user/repo.git
mug migrate ~/temp/repo ~/final-mug
```

## Dual VCS During Migration

Keep Both Running:
```bash
cd ~/my-git-repo
git status

cd ~/my-mug-repo
mug status
```

Sync Changes:
```bash
# Git to MUG
cd ~/my-git-repo
git pull

cd ~/my-mug-repo
mug pull origin main
```

## Full Example

Starting Point (Git Repo):
```bash
ls -la ~/myproject/.git
```

Migrate:
```bash
mug migrate ~/myproject ~/myproject-mug
```

Verify:
```bash
cd ~/myproject-mug
mug log --oneline -n 10
mug status
mug verify
```

Configure:
```bash
mug config set user.name "John Doe"
mug config set user.email "john@example.com"
```

Add Remote:
```bash
mug remote add origin https://example.com/repo.git
```

Create Feature:
```bash
mug branch feature/awesome
mug checkout feature/awesome
# Make changes
mug add .
mug commit -m "Awesome feature"
mug checkout main
mug merge feature/awesome
```

Push:
```bash
mug push origin main
```

## Troubleshooting

Migration Fails - Check Source:
```bash
cd ~/my-git-repo
git log -n 1
```

Missing Objects - Verify Source:
```bash
cd ~/my-git-repo
git verify
```

Slow Migration - Check Size:
```bash
du -sh ~/my-git-repo
```

After Migration Issues - Verify:
```bash
mug verify
```

## Batch Migration

Migrate Multiple Repos:
```bash
for repo in ~/repos/*; do
  name=$(basename "$repo")
  echo "Migrating $name..."
  mug migrate "$repo" ~/mug-repos/"$name"
done
```

## Cleanup After Migration

Once Confident:
```bash
rm -rf ~/my-git-repo
rm -rf ~/my-git-repo.backup
```

Or Keep Original:
```bash
mv ~/my-git-repo ~/archive/my-git-repo
```

## Integration Example

Setup New Workflow:
```bash
mkdir ~/projects
cd ~/projects

# Migrate existing
mug migrate ~/backup/old-repo ~/projects/project1-mug

# Or create new
cd project1-mug
mug config set user.name "Developer"
```

## Performance Notes

Small Repo (< 100 commits):
- < 1 second

Medium Repo (100-1000 commits):
- 1-10 seconds

Large Repo (1000+ commits):
- 10-60 seconds

Factors:
- Number of commits
- File sizes
- Object compression
- Disk speed

## Success Indicators

After Migration:
1. `mug log` shows all commits
2. `mug verify` reports no errors
3. All branches present
4. `mug status` works
5. Can create new commits

## Next Steps

After Successful Migration:
1. Configure user identity
2. Add remotes if needed
3. Create feature branches
4. Continue development
5. Archive Git repository

## Reference

Full Migration Details: See MIGRATION_COMPLETE.md
Architecture: See HYBRID_VCS.md
Command Reference: See DOCS.md
