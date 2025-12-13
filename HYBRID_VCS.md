# Hybrid VCS Architecture - MUG

MUG is not just a Git clone. It's a **hybrid version control system** combining Git's proven model with novel research-driven features for modern development workflows.

## Core Innovation

Traditional VCS systems (Git) enforce linear history constraints. MUG breaks these constraints with research-backed approaches:

### 1. **Cryptographic Verification by Default**

Every commit is signed with Ed25519 keys, creating an immutable audit trail.

```bash
# Generate a signing key
mug keys generate

# Your commits are automatically verified
mug log  # Shows signature verification status
```

**Why it matters:**
- Prevents commit forgery
- Cryptographically proves authorship
- Essential for distributed trust in open-source
- Blocks supply chain attacks at the VCS level

### 2. **Temporal Branching**

Unlike Git's linear branch model, temporal branches can fork and merge at **any point** in history, not just the tip.

```bash
# Create a temporal branch from a specific commit
mug temporal create feature abc123def

# Merge branches at arbitrary history points
mug temporal merge main feature

# Visualize the complete temporal DAG
mug temporal show feature
```

**Why it matters:**
- Resolves conflicts at multiple history points simultaneously
- Enables "time-traveling" fixes that apply across history
- Supports workflows where teams need to patch multiple release branches
- Matches real-world development: security fixes, backports, etc.

### 3. **Content-Addressed Commits**

All commits reference content by cryptographic hash (like IPFS/Git), but with **explicit merkle DAG** visualization.

```bash
# Every commit is a DAG node
commit 1e600e9 (abc123...)
  ├─ tree: 0ff0afd...
  ├─ parent: 488231f...
  └─ signature: Ed25519...
```

### 4. **Schema-Based Commits** (Upcoming)

Define structured commit types with validation:

```rust
[commit "bug-fix"]
required_fields = ["issue_id", "test_coverage"]
```

Enforces data integrity at commit time.

### 5. **Conflict-Free Merging** (Roadmap)

CRDT-based text merging (like Yjs) for automatic conflict resolution in many scenarios.

## Architecture

```
MUG Repository
├── .mug/objects/          # Content-addressed blob storage
├── .mug/commits/          # Commit metadata with signatures
├── .mug/temporal/         # Temporal branch tracking
├── .mug/crypto/           # Ed25519 keys (local)
└── .mug/db/              # RocksDB for indexing
```

## CLI Commands

### Crypto Keys
```bash
mug keys generate           # Create signing key
mug keys import <seed>     # Import from base64 seed
mug keys list              # List available keys
mug keys current           # Show active signing key
```

### Temporal Branches
```bash
mug temporal create <name> <commit>   # Create at specific commit
mug temporal list                     # Show all temporal branches
mug temporal show <branch>            # Visualize branch structure
mug temporal merge <target> <source>  # Merge branches
```

## Research Foundation

This hybrid approach draws from:

- **ETH Zurich VCS Research**: Novel branching and merging strategies
- **Cryptographic Git (CryptDB)**: Proving commit authenticity
- **IPFS/Content Addressing**: Hash-based integrity
- **CRDT Literature**: Conflict-free data structures

## Performance

- **Signing**: O(1) per commit with Ed25519
- **Temporal merge**: O(n) where n = merge conflicts
- **Visualization**: O(edges) in DAG

## Compatibility

MUG can import Git repositories:
```bash
mug migrate /path/to/git/repo /path/to/mug/repo
```

But adds capabilities Git cannot match.

## Vision

MUG is building a VCS that:
1. **Proves authorship** - Cryptography, not trust
2. **Handles real workflows** - Temporal branching for backports, security patches
3. **Scales to research** - Schema-based commits, CRDT merging
4. **Teaches fundamentals** - Educational codebase in Rust

This isn't Git 2.0. It's a research vehicle for rethinking what version control should be.


