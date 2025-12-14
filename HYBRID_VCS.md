# Hybrid VCS Architecture

## Overview

MUG combines distributed version control with novel features and centralized storage for large files.

## Core Architecture

Object Store:
- Content-addressable blob storage using SHA256 hashes
- Automatic deduplication
- Local `.mug/objects/` directory
- Files below threshold (10MB default) stored locally

Index:
- Staging area for changes
- Tracks file paths and hashes
- Persisted to Sled database
- O(1) lookup on file changes

Commit Log:
- Immutable commit objects
- Stores author, message, timestamp, parent
- Full history traversal
- SHA256 hash-based references

Branches:
- Named references to commits
- HEAD management (attached/detached)
- Fast branch switching
- Stored in Sled database

Database:
- Sled embedded key-value store
- Separate trees: commits, branches, index, HEAD
- Flush-on-demand persistence
- No external database required

## Distributed Features

Full History Locally:
- All commits available offline
- No central dependency for basic operations
- Fast log and status operations
- O(1) index lookups

Remote Operations:
- Push: Send commits to remote
- Pull: Fetch and merge from remote
- Fetch: Retrieve without merging
- Clone: Full repository copy

## Novel Features

Temporal Branching:
- Create branches at any commit in history
- Not just at branch tips
- Enables backports and security patches
- Non-linear history visualization
- Merge branches at any point in history

Cryptographic Signing:
- Ed25519 keypairs for commits
- Immutable audit trail
- Proves authorship
- Prevents commit forgery
- Base64-encoded seeds for portability

## Hybrid Storage

Problem:
- Binary files bloat distributed repos
- Large datasets slow down clones
- Developers need rapid local access
- Central servers expensive to distribute

Solution:
- Hybrid model with local + central storage
- Small files (< 10MB): local `.mug/objects/`
- Large files: streamed from central server
- LRU cache (1GB default) for transparency
- Works offline for commits, streams files on-demand

Configuration:
```bash
mug store set-server https://store.example.com
mug store set-threshold 10
```

## Technology Stack

Language: Rust
- Memory safety without garbage collection
- Zero-cost abstractions
- Strong type system

Compression: flate2
- zlib-compatible compression
- Fast decompression for Git compatibility

Cryptography: ed25519-dalek
- Fast Ed25519 signing
- Industry-standard algorithm
- Constant-time operations

Database: sled
- Embedded key-value store
- ACID transactions
- LSM tree for performance
- No external dependencies

Serialization: serde_json
- Human-readable format
- Type-safe serialization
- Standard Rust ecosystem

Concurrency: rayon
- Data parallelism
- Work-stealing scheduler
- Efficient parallel grep

Web: actix-web
- High-performance web framework
- Async/await support
- HTTP server for remote access

## Data Flow

Working Directory
    ↓
File Operations (add, rm, mv)
    ↓
Hash File (SHA256)
    ↓
Object Store (content-addressable)
    ↓
Index (staging area)
    ↓
Commit (immutable snapshot)
    ↓
Branch Reference (named pointer)
    ↓
Sled Database (persistence)
    ↓
Remote (push/pull/fetch)

## Performance Optimizations

Status Command:
- O(1) index lookup instead of O(n) tree walk
- Instant on large repositories
- No filesystem scanning

Commit Lookup:
- Linked list traversal
- Minimal metadata overhead
- Direct hash references

Storage:
- Automatic deduplication via content addressing
- Compression for history
- No duplicate content storage

Branch Operations:
- Instant branch creation
- O(1) branch switching
- No working directory copies

Parallel Operations:
- Rayon parallelization for grep
- Multi-threaded file processing
- Efficient CPU utilization

## Comparison with Alternatives

Git:
- More features (submodules, sparse checkout)
- Larger ecosystem
- Slower status on large repos
- Centralized large file support separate

Mercurial:
- Simpler internals
- Slower index operations
- Less mature

Pijul (CRDT-based):
- Advanced merge strategies
- Complex implementation
- Smaller ecosystem

MUG:
- Modern architecture
- Hybrid local/central storage
- Cryptographic signing built-in
- Temporal branching for complex workflows
- Fast by default
- Research-driven design

## Future Enhancements

Automatic Pack Files:
- Background packing on GC
- Automatic compression

Shallow Clones:
- Partial history for large repos
- Faster initial clone

Submodule Support:
- Nested repositories
- Dependency management

Advanced Merge Strategies:
- ours, theirs, recursive
- Custom merge drivers

Web UI:
- Repository browsing
- Commit visualization
- Merge conflict resolution

Signed Push:
- Cryptographic verification of push operations
- Proof of origin

## Design Decisions

Content Addressing:
- Immutable by hash
- Automatic deduplication
- Eliminates data corruption

Sled Database:
- Embedded, no external server
- ACID transactions
- Good performance

Ed25519 Signing:
- Fast and secure
- Modern standard
- Constant-time operations

Temporal Branching:
- Enables complex workflows
- Solves real-world problems
- Non-linear history support

## Implementation Statistics

Lines of Code: 3,600+
Rust Modules: 26
Unit Tests: 100+
CLI Commands: 35+
Documentation: 10+ markdown files
Zero Compiler Warnings: Yes

## Security Considerations

Content Hashing:
- SHA256 for integrity
- Prevents accidental corruption
- Detects tampering

Ed25519 Signing:
- Cryptographic proof of authorship
- Immutable audit trail
- Prevents forgery

Database Encryption:
- Future enhancement
- Optional at-rest encryption

Hook Execution:
- Subprocess isolation
- Limited permissions
- Captured output

## Integration Points

Git Repositories:
- Full migration support
- Preserve history and metadata

HTTP Remotes:
- Standard HTTP push/pull
- No special protocol

SSH Remotes:
- SSH key support
- Secure connections

Local Remotes:
- Filesystem-based remotes
- Useful for testing

## Backward Compatibility

Version Management:
- Database versioning
- Format evolution support
- Graceful upgrades

Git Compatibility:
- Import existing repositories
- Preserve metadata
- Support both formats
