# VCS Architecture Research

## VCS Types and Their Trade-offs

### 1. **Distributed VCS** (Git, Mercurial)
- Every clone = full history
- **Pros**: Offline work, redundancy, no single point of failure
- **Cons**: Large repos slow, full copy of history, LFS overhead
- **Best for**: Open-source, small-medium codebases, teams spread globally

### 2. **Centralized VCS** (SVN, Perforce)
- Single server of truth
- **Pros**: Small working copies, bandwidth efficient, file-locking, fine-grained permissions
- **Cons**: Offline limited, single point of failure, server bottleneck
- **Best for**: Large files, enterprise, strict access control

### 3. **Hybrid VCS** (Perforce, GitLFS + Git)
- Combines distributed + centralized
- **Pros**: Full history locally + efficient large file handling
- **Cons**: Complex, two-tier system, coordination overhead
- **Best for**: Mixed workloads (code + binaries)

### 4. **Monorepo VCS** (Google Piper, Meta Sapling)
- Single massive repository with smart access
- **Pros**: Unified history, atomic cross-project commits
- **Cons**: Requires custom tooling, infrastructure heavy
- **Best for**: Large organizations (Google, Meta scale)

---

## Problem: Large Files in Distributed VCS

### Root Issue
Git stores full blob history:
```
commit A: binary.bin (100MB)
commit B: binary.bin (100MB) - modified
commit C: main.rs (1KB)

.git/objects = 200MB+ for 3 commits
```

Each commit = entire file duplication.

### Current Solutions

#### Git LFS
```
.git/objects: pointers (1KB each)
remote LFS server: actual blobs (100MB)
```
✅ Pros: Works today
❌ Cons: Split system, two servers, requires extra setup

#### Shallow Clone
```
git clone --depth=1 https://repo.git
# Only recent history
```
✅ Pros: Small download
❌ Cons: Can't bisect, offline limited

#### Sparse Checkout
```
git sparse-checkout set src/
# Only some directories
```
✅ Pros: Faster operations
❌ Cons: Still stores full history

---

## Solution: Centralized Large File Store

MUG can implement a **hybrid approach**:

```
MUG Repository
├── .mug/db/           # Distributed: commits, branches, refs
├── .mug/objects/      # Local: small files only
└── .mug/store/        # Optional: large files (threshold 10MB)
    ├── config         # Points to central server
    └── local.cache/   # LRU cache of remote blobs
```

### Implementation Model

#### Tier 1: Local (all commits, small files)
- Distributed model for commits ≤ 10MB
- Full history available offline
- Enables git log, blame, bisect

#### Tier 2: Central (large files, binaries)
- Server-side storage for files > 10MB
- On-demand streaming
- Bandwidth efficient

#### Tier 3: Content Addressing
- All objects have SHA256 hash
- Deduplication across projects
- Supports IPFS-style distribution

---

## MUG Design Decision

### Core Architecture

```rust
enum ObjectSource {
    Local,        // In .mug/objects
    Central,      // Fetch from server
    Distributed,  // P2P/IPFS (future)
}

struct StoreConfig {
    local_threshold: usize,        // < 10MB: local
    central_server: Option<String>, // >= 10MB: remote
    cache_dir: PathBuf,
    cache_size_mb: usize,
}
```

### Workflow

**Scenario: 200MB video file**

```bash
# Clone (distributed part)
mug clone https://repo.mug
# .mug/objects: ~50MB (all commits, metadata)
# Local copy: instant, 50MB

# Checkout video.mp4 (streaming from central)
mug checkout main
# Downloads video.mp4 on-demand: transparent
# Cached locally: future accesses instant

# Bisect (still works!)
mug bisect start bad good
# Only checks commit metadata (available locally)
# Minimal central server traffic
```

---

## Comparison: MUG vs Alternatives

| Feature | Git | Git LFS | SVN | MUG |
|---------|-----|---------|-----|-----|
| Offline commits | ✅ | ✅ | ❌ | ✅ |
| Offline file ops | ✅ | ❌ | ❌ | ✅ |
| Large files | ❌ | ✅* | ✅ | ✅ |
| History locally | ✅ | ✅ | ❌ | ✅ |
| Atomic xacts | ❌ | ❌ | ✅ | ⚙️ (planned) |
| Single command | ✅ | ❌* | ✅ | ✅ |

*= requires setup

---

## Implementation Roadmap

### Phase 1: (Done)
- ✅ Distributed core (commits, refs)
- ✅ Full history locally
- ✅ Git migration

### Phase 2: (Proposed)
- [ ] `mug config store.central` - server setup
- [ ] Large file detection (>10MB)
- [ ] Streaming download on checkout
- [ ] LRU cache management
- [ ] `mug gc` - cleanup old cache

### Phase 3: (Advanced)
- [ ] Server implementation (Rust actix-web)
- [ ] Compression (zstd for large files)
- [ ] Deduplication (content-hash based)
- [ ] IPFS integration (P2P storage)
- [ ] Atomic transactions (ACID commits)

---

## Research References

- **Piper (Google's monorepo)**: https://research.google/pubs/others/
- **Sapling (Meta's VCS)**: https://sapling-scm.com/
- **Git LFS Design**: https://github.com/git-lfs/git-lfs/wiki/Design
- **Content Addressing**: IPFS papers
- **Shallow Cloning**: Git internals, `--shallow` protocol

---

## Next Steps

1. Implement `mug config store.central <url>`
2. Add `file_size_threshold` configuration
3. Lazy-load large blobs on checkout
4. Build simple test server for large files
5. Benchmark vs Git LFS

This creates a **sweet spot**: distributed for code, centralized for binaries.
