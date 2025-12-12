# Git Migration Examples

Quick reference for migrating Git repos to MUG.

## CLI Command

```bash
# Migrate Git repo to MUG
mug migrate /path/to/git/repo /path/to/mug/repo

# Example with absolute paths
mug migrate /home/user/my-project /var/mug/repos/my-project

# Example with relative paths
cd /var/mug/repos
mug migrate ../github-backup/project ./project
```

### Output

```
✓ Migration complete
Migrated 3 branches to MUG. Next: implement commit/object import.
```

## HTTP API

```bash
# Start server
./target/release/mug serve --host 0.0.0.0 --port 8080 --repos /var/mug/repos

# Migrate via HTTP
curl -X POST \
  -H "Authorization: Bearer token" \
  -H "Content-Type: application/json" \
  -d '{"git_path": "/home/user/git-repo"}' \
  http://localhost:8080/repo/myrepo/migrate-from-git
```

## Batch Migration Script

```bash
#!/bin/bash

# Migrate all Git repos in a directory
cd /var/mug/repos

for repo_dir in ~/git-backups/*/; do
  repo_name=$(basename "$repo_dir")
  echo "Migrating $repo_name..."
  
  mug migrate "$repo_dir" "./$repo_name"
  
  if [ $? -eq 0 ]; then
    echo "✓ $repo_name migrated"
  else
    echo "✗ $repo_name failed"
  fi
done
```

## Real-World Scenarios

### 1. Single Repo from GitHub

```bash
# Clone GitHub repo
git clone https://github.com/user/repo.git /tmp/repo

# Migrate to MUG
mug migrate /tmp/repo /var/mug/repos/repo

# Verify
ls -la /var/mug/repos/repo/.mug/
```

### 2. Organization Migration

```bash
#!/bin/bash

# Download all org repos
mkdir -p /tmp/github-export
cd /tmp/github-export

# List of repos (or use GitHub API)
repos=("repo1" "repo2" "repo3")

for repo in "${repos[@]}"; do
  git clone https://github.com/myorg/$repo.git
done

# Migrate all to MUG server
for repo_dir in */; do
  repo_name=$(basename "$repo_dir")
  echo "Migrating $repo_name..."
  
  mug migrate "$repo_dir" "/var/mug/repos/$repo_name"
done

echo "All repos migrated!"
```

### 3. Local Development Setup

```bash
# Migrate your local projects to MUG
cd ~/projects

for project in my-project another-project third-project; do
  echo "Migrating $project..."
  mug migrate "./$project" ~/mug-repos/$project
done

# Update local remotes
cd my-project
git remote set-url origin http://localhost:8080/repo/my-project
```

## Verification Steps

After migration:

```bash
# 1. Check repository structure
ls -la /path/to/mug/repo/.mug/

# 2. Expected directories
# - .mug/objects/      (object store)
# - .mug/db/          (database)
# - .mugignore        (ignore rules)

# 3. Verify branches (once import is complete)
mug branches -r /path/to/mug/repo

# 4. Test clone/push/pull operations
mug clone http://ip:8080/repo/name
```

## Troubleshooting

### "Not a Git repository"

```bash
# Verify source is a Git repo
test -d /path/to/repo/.git && echo "Valid" || echo "Invalid"

# Check it's not a bare repository
test -f /path/to/repo/.git/config && echo "Regular repo"
```

### "Permission denied"

```bash
# Ensure MUG server can read Git repo
chmod -R +r /path/to/repo
chmod +x /path/to/repo/.git

# Check write access to destination
mkdir -p /var/mug/repos
touch /var/mug/repos/test-write
```

### "Invalid path"

```bash
# Use absolute paths (safer)
mug migrate /full/path/to/git /full/path/to/mug

# Or convert relative paths first
cd /tmp
git clone /home/user/repo my-repo
mug migrate "$PWD/my-repo" "/var/mug/repos/my-repo"
```

## Performance Tips

```bash
# For large repos, use local paths first
# (faster than network access)

# 1. Copy to local SSD
rsync -av /network/share/large-repo /tmp/large-repo

# 2. Migrate locally
time mug migrate /tmp/large-repo /var/mug/repos/large-repo

# 3. Monitor progress
watch -n 1 'du -sh /var/mug/repos/large-repo/.mug/'
```

## Integration with CI/CD

### GitHub Actions

```yaml
name: Migrate to MUG

on: [workflow_dispatch]

jobs:
  migrate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install MUG
        run: |
          curl -L https://releases.example.com/mug-latest-linux -o /tmp/mug
          chmod +x /tmp/mug
      
      - name: Migrate
        run: |
          /tmp/mug migrate $GITHUB_WORKSPACE /mug/repos/${{ github.repository }}
        env:
          MUG_HOST: ${{ secrets.MUG_HOST }}
```

### GitLab CI

```yaml
migrate:
  stage: deploy
  image: mug:latest
  script:
    - mug migrate $CI_PROJECT_DIR /mug/repos/$CI_PROJECT_NAME
  only:
    - web
```

## Automation with cron

```bash
# Backup and migrate Git repos daily
0 2 * * * /home/user/scripts/backup-and-migrate.sh

# Script: backup-and-migrate.sh
#!/bin/bash
set -e

cd /var/backups
git clone https://github.com/user/repo.git repo-$(date +%Y%m%d)
mug migrate repo-$(date +%Y%m%d) /var/mug/repos/repo
```

## Next Steps

Once migration completes:

1. **Test operations** - Clone, fetch, push
2. **Update CI/CD** - Point to new MUG server
3. **Communicate** - Notify team of new location
4. **Decommission** - Remove old Git server when ready
5. **Archive** - Keep Git backups in storage

See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for detailed information.
