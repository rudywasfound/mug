# MUG: Hybrid VCS - Feature Summary

## Overview

MUG is a research-driven version control system that combines:
1. Distributed VCS (full history, offline work, Git migration)
2. Novel Features (temporal branches, cryptographic signing)
3. Centralized Storage (solution for large files)

Built in Rust for performance and safety.

## Core Features Implemented

Git Compatibility:
- Migration: `mug migrate /path/to/git /path/to/mug`
- Imports all commits with full metadata
- Parses zlib-compressed Git objects
- Extracts author, messages, parent relationships
- Creates proper BranchRef and HEAD pointers
- Tested on 33+ commit repositories

Cryptographic Signing:
```bash
mug keys generate
mug keys import <seed>
```
- All commits can be signed with Ed25519
- Immutable audit trail
- Proves authorship, prevents forgery
- Base64-encoded keys for portability

Temporal Branching:
```bash
mug temporal create main <commit>
mug temporal list
mug temporal show <branch>
mug temporal merge <target> <source>
```
- Branches fork/merge at any point in history
- Solves real-world workflows: backports, security patches
- Explicit merge point tracking
- DAG visualization with merge markers

Centralized Large File Storage:
```bash
mug store set-server <url>
mug store set-threshold <MB>
mug store cache-stats
mug store clear-cache
```
- Hybrid model: local commits + central files
- Files < threshold (10MB default): local `.mug/objects`
- Files >= threshold: streamed from central server
- LRU cache (1GB default) for performance
- Works offline for commits, streams files on-demand

## Architecture

MUG Repository Structure:
├── .mug/
│   ├── db/                  # RocksDB: commits, branches, metadata
│   ├── objects/             # Local object store (small files)
│   ├── cache/               # LRU cache for remote objects
│   ├── crypto/              # Ed25519 keys
│   └── temporal/            # Temporal branch tracking
├── .mugignore               # Ignore patterns
└── (working directory)

Technology Stack:
- Language: Rust
- Compression: flate2 (zlib)
- Cryptography: ed25519-dalek
- Database: sled (RocksDB-compatible)
- Serialization: serde_json
- Parallel: rayon
- Web: actix-web (for server)

## CLI Reference

Basic Commands:
```bash
mug init .
mug add .
mug commit -m "message"
mug log
mug status
```

Branching:
```bash
mug branch feature
mug branches
mug checkout main
mug temporal create fix <hash>
```

Cryptography:
```bash
mug keys generate
mug keys import <seed>
mug keys list
mug keys current
```

Temporal:
```bash
mug temporal create <name> <commit>
mug temporal list
mug temporal show <branch>
mug temporal merge <target> <source>
```

Storage:
```bash
mug store config
mug store set-server <url>
mug store set-threshold 50
mug store cache-stats
mug store clear-cache
```

Migration:
```bash
mug migrate /git/repo /mug/repo
```

## Performance Characteristics

mug init: <10ms (directory creation)
mug add: O(n) (parallel file processing)
mug commit: <100ms (Ed25519 signing fast)
mug log: O(depth) (follows parent chain)
mug temporal create: <10ms (DB write only)
Git migration: <1s (33 commits, decompress overhead)

Tested on:
- Repo Size: 10MB (mug itself)
- Commits: 33 commits with full history
- Files: 60+ Rust source files
- Database: sled (embedded)

## CLI Commands (35+)

Core:
- init, add, remove, commit, log, show, status

Branching:
- branch, branches, checkout, merge, rebase

File Operations:
- rm, mv, restore, grep, diff

History:
- reset, cherry-pick, cherry-pick-range, bisect-start, bisect-good, bisect-bad

Tags:
- tag, tags, delete-tag

Stash:
- stash, stash-pop, stash-list

Remote:
- remote add/list/remove/set-default/update-url, push, pull, fetch, clone, serve

Keys:
- keys generate/list/import/current

Temporal:
- temporal create/list/show/merge

Store:
- store set-server/set-threshold/config/cache-stats/clear-cache

Configuration:
- config set/get/list

Maintenance:
- migrate, verify, gc, reflog, update-ref

Pack:
- pack create/stats/dedup/verify

## Features Status

Implemented:
- Full Git compatibility and migration
- Cryptographic signing with Ed25519
- Temporal branching (non-linear history)
- Centralized large file storage with LRU cache
- 35+ CLI commands
- Hook system (pre/post commit, push, merge)
- Interactive rebase with TUI
- Bisect for bug finding
- Stash operations
- Tag management
- Cherry-pick and cherry-pick-range
- Remote operations (push, pull, fetch, clone)
- HTTP server mode
- Repository verification and garbage collection

## Design Philosophy

1. Minimal metadata - Only track what's necessary
2. Content-addressed - Use hashes as the source of truth
3. Fast by default - No tree walking, clever indexing
4. Simple interface - Clean commands, clear semantics
5. Complete feature set - All essential VCS operations
6. Research-driven - Novel branching, cryptographic signing, hybrid storage

## Use Cases

Git Migration:
- Import existing Git repositories to MUG format
- Preserve full history and metadata
- Supports both loose objects and pack files

Secure Collaboration:
- Cryptographic signing of commits
- Immutable audit trail
- Proves authorship

Complex Workflows:
- Temporal branching for backports
- Non-linear history for multi-version support
- Security patches across versions

Large File Management:
- Hybrid local/central storage
- Transparent LRU caching
- Prevents repository bloat

## Limitations

Server Implementation: Basic HTTP protocol, not full Git wire protocol
Pack File Optimization: Manual pack creation, no automatic optimization
Shallow Clone: Not yet implemented
Submodules: Not yet implemented

## Future Enhancements

Automatic pack file creation on GC
Shallow clone support
Submodule support
Advanced merge strategies (ours, theirs, recursive)
Signed push with cryptographic verification
Web UI for repository browsing
