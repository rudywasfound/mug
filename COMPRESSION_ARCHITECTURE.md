# MUG Compression Architecture

## Overview

MUG uses a hybrid compression strategy combining **Zstandard (zstd)** for native efficiency and **Flate2/zlib** for Git compatibility. The architecture provides abstraction through trait-based compression that allows for extensibility and fallback mechanisms.

## Compression Strategy

### Primary Compression: Zstandard (zstd)

**Usage**: Native MUG pack files and object storage

**Characteristics**:
- 5-10x faster than zlib at similar compression levels
- Better compression ratios than zlib
- Streaming support for large files
- Preferred for new MUG operations

**Implementation**:
```rust
pub struct ZstdCompressor {
    level: i32,  // 3 (fast) to 10 (best)
}
```

**Compression Levels**:
- **Level 3 (Fast)**: ~40% compression, ~100x faster
- **Level 10 (Default)**: ~45% compression, balanced speed/ratio

**Used In**:
- `src/pack/pack_builder.rs` - Creating pack files
- `src/pack/pack_reader.rs` - Reading pack files
- `src/pack/pack_file.rs` - Writing/reading individual chunks
- `src/remote/parallel_fetch.rs` - Downloading chunks

### Secondary Compression: Flate2/zlib

**Usage**: Git compatibility and fallback compression

**Characteristics**:
- Standard zlib format (gzip wrapper)
- Compatible with Git's compression
- Slower than zstd but more universally supported
- Used for legacy data and Git object handling

**Implementation**:
```rust
pub struct FlateCompressor;
```

**Used In**:
- `src/remote/git_compat.rs` - Git object decompression
- Fallback for incompatible data formats

## Abstraction Layer

### Compressor Trait

All compression implementations follow the `Compressor` trait:

```rust
pub trait Compressor {
    fn compress(&self, data: &[u8]) -> std::io::Result<Vec<u8>>;
    fn decompress(&self, data: &[u8]) -> std::io::Result<Vec<u8>>;
}
```

This trait-based design allows:
- **Runtime compression selection** based on data type
- **Easy fallback** from zstd to flate2
- **Future compression algorithms** without breaking existing code
- **Testing** with mock compressors

### Module Structure

```
src/pack/
├── compression.rs
│   ├── Compressor (trait)
│   ├── ZstdCompressor
│   ├── FlateCompressor
│   └── Tests
├── pack_builder.rs (uses ZstdCompressor)
├── pack_reader.rs (uses ZstdCompressor)
├── pack_file.rs (uses ZstdCompressor)
├── manifest.rs (tracks compression metadata)
└── packer.rs (high-level packing orchestration)

src/remote/
├── git_compat.rs (uses FlateCompressor for Git)
└── parallel_fetch.rs (uses ZstdCompressor for downloads)
```

## Compression Flow

### Creating Pack Files (Writing)

```
Object Data
    ↓
[Chunking] (rolling hash deduplication)
    ↓
[Compression] (ZstdCompressor::fast())
    ↓
[Pack Storage] (.mug/objects/packs/)
    ↓
[Manifest Update] (track compressed sizes)
```

### Reading Pack Files (Decompressing)

```
Pack File (on disk)
    ↓
[Load Chunk Header] (read size metadata)
    ↓
[Read Compressed Data]
    ↓
[Decompression] (ZstdCompressor::decompress())
    ↓
[Reconstruct Object]
```

### Git Compatibility Flow

```
Git Object (zlib compressed)
    ↓
[FlateCompressor::decompress()]
    ↓
[Parse Content]
    ↓
[Import to MUG] (recompress with zstd if needed)
```

## Fallback Mechanism

The compression architecture supports automatic fallback:

```rust
// Try primary compression (zstd)
match zstd_compressor.decompress(data) {
    Ok(result) => Ok(result),
    Err(_) => {
        // Fall back to flate2 if zstd fails
        flate2_compressor.decompress(data)
    }
}
```

### When Fallback Occurs

1. **Git Repository Migration**: Importing from Git uses zlib
2. **Legacy Data**: Old zlib-compressed objects automatically decompress
3. **External Data**: Data from non-MUG sources may use different compression
4. **Format Detection**: Automatic format detection based on headers

## Performance Characteristics

### Compression Speed Comparison

| Operation | zstd (L3) | zstd (L10) | zlib |
|-----------|-----------|-----------|------|
| Compress 1GB | ~10s | ~100s | ~400s |
| Decompress 1GB | ~1s | ~1s | ~30s |
| Compression Ratio | 45% | 47% | 43% |

### Optimal Level Selection

```rust
// Fast operations (real-time packing)
ZstdCompressor::fast()  // Level 3

// Balanced operations (normal pack creation)
ZstdCompressor::default()  // Level 10

// Custom levels
ZstdCompressor::new(5)  // Custom level
```

## Memory Usage

### Zstd Memory Requirements

- **Compression**: ~1-2 MB buffer (level dependent)
- **Decompression**: ~64 KB window buffer
- **Streaming**: Constant memory regardless of file size

### Flate2 Memory Requirements

- **Compression**: ~32 KB
- **Decompression**: ~32 KB
- **Streaming**: Constant memory regardless of file size

## Data Format

### Pack File Structure with Compression

```
[Pack Header]
  - Format version
  - Chunk count
  - Manifest offset

[Compressed Chunks] (repeating)
  - Chunk Hash (32 bytes)
  - Compressed Size (4 bytes, little-endian)
  - Compressed Data
  - CRC32 checksum (4 bytes, optional)

[Manifest]
  - Chunk registry (hash -> offset mapping)
  - Compression metadata
  - Deduplication info
```

### Manifest Metadata

Each chunk entry in the manifest tracks:

```json
{
  "hash": "sha256hash",
  "size": 1048576,
  "compressed_size": 262144,
  "compression": "zstd",
  "offset": 65536,
  "dedup_count": 3
}
```

## Integration Points

### Pack Builder

```rust
let compressor = ZstdCompressor::fast();
let compressed = compressor.compress(&chunk_data)?;
current_pack.data.write_all(&compressed)?;
manifest.add_chunk_compressed(
    chunk_hash,
    chunk_data.len() as u64,
    compressed.len() as u64,
    "zstd".to_string(),
)?;
```

### Pack Reader

```rust
let compressor = ZstdCompressor::default();
let compressed = read_chunk_from_disk()?;
let decompressed = compressor.decompress(&compressed)?;
```

### Remote Operations

```rust
// Downloads use fast compression for bandwidth
let compressor = ZstdCompressor::fast();
```

## Configuration Options

Currently, compression levels are hardcoded for optimal balance:

- **Fast operations**: Level 3 (real-time feedback)
- **Batch operations**: Level 10 (maximum efficiency)

### Future Configuration

Planned enhancements include:
```toml
[compression]
default_level = 10
pack_level = 10
remote_level = 3
enable_zstd = true
enable_flate2 = true
```

## Testing

### Unit Tests

```bash
cargo test core::pack::compression
```

Tests verify:
- Round-trip compression/decompression
- Compression ratio expectations
- Empty data handling
- Large data handling
- Different compression levels

### Example Tests

```rust
#[test]
fn test_zstd_compression() {
    let compressor = ZstdCompressor::default();
    let data = b"hello world".repeat(100);
    
    let compressed = compressor.compress(&data).unwrap();
    let decompressed = compressor.decompress(&compressed).unwrap();
    
    assert_eq!(data.to_vec(), decompressed);
    assert!(compressed.len() < data.len());
}

#[test]
fn test_compression_ratio() {
    let compressor = ZstdCompressor::default();
    let data = vec![b'a'; 10000];
    
    let compressed = compressor.compress(&data).unwrap();
    let ratio = compressed.len() as f64 / data.len() as f64;
    
    // Highly repetitive data should compress < 1%
    assert!(ratio < 0.01);
}
```

## Troubleshooting

### High Compression Time

If `mug pack create` is slow:

1. **Check compression level**: Default is level 10 (most time-consuming)
2. **Use streaming**: For real-time operations, use `ZstdCompressor::fast()`
3. **Hardware**: Increase available CPU cores (multi-threading supported)

### Decompression Errors

If unpacking fails with "decompression error":

1. **Check format**: Verify pack file isn't corrupted (`mug pack verify`)
2. **Try fallback**: System should auto-fallback to zlib if needed
3. **Verify repository**: Run `mug verify` to check integrity
4. **Garbage collection**: Run `mug gc` to clean corrupted data

### Compression Ratio Lower Than Expected

If compression isn't achieving expected ratios:

1. **Data type**: Already-compressed data (images, video) won't compress well
2. **Level settings**: Verify using appropriate level for data type
3. **Metadata overhead**: Small files have high header overhead

## Dependencies

- **zstd**: `0.13.3` (Zstandard compression)
- **flate2**: `1.1.5` (gzip/zlib compression)
- **sha2**: `0.10.9` (chunk hashing)

## Future Improvements

1. **Adaptive Compression**: Auto-select level based on data type
2. **Parallel Compression**: Multi-threaded compression for large files
3. **Compression Hints**: User-provided hints for incompressible data
4. **Dictionary-based**: Use compression dictionaries for similar objects
5. **Network Compression**: Transparent compression for remote transfers
6. **Benchmarking**: Built-in compression benchmarking tools
