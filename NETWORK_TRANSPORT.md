# MUG Network Transport Implementation

## Overview

MUG v1.0.0-alpha.1 now includes HTTP/HTTPS transport for remote operations. This document describes the network layer architecture and implementation.

## Architecture

### Client-Server Model

```
Client                           Server
(Local Repository)               (Remote Repository)

  push()  -----[HTTP POST]----->  /repo/{name}/push
  pull()  -----[HTTP GET]------->  /repo/{name}/pull
  fetch() -----[HTTP GET]------->  /repo/{name}/fetch
  clone() -----[HTTP GET]------->  /repo/{name}/clone
```

### Protocol

All network operations use JSON over HTTP/HTTPS for serialization and transport.

#### Request/Response Format

**PushRequest**
```json
{
  "repo": "repository_name",
  "branch": "main",
  "commits": [...],
  "blobs": [...],
  "trees": [...],
  "head": "HEAD"
}
```

**PushResponse**
```json
{
  "success": true,
  "message": "Push successful",
  "head": "main"
}
```

Similar structures for Pull, Fetch, and Clone operations.

## Implementation Details

### Client Component (`src/client.rs`)

The `RemoteClient` handles outgoing network requests:

```rust
pub struct RemoteClient {
    client: Client,  // reqwest HTTP client
}
```

**Supported Operations:**

1. **push()** - Push commits to remote
   - Extracts commits from local repository
   - Sends PushRequest via POST to `/repo/{name}/push`
   - Returns PushResponse with success status

2. **pull()** - Pull commits from remote
   - Sends PullRequest via GET to `/repo/{name}/pull`
   - Receives commits, blobs, trees
   - Merges with local branch

3. **fetch()** - Fetch branch information
   - Gets all available branches from remote
   - No merge operation (like Git fetch)

4. **clone()** - Clone a remote repository
   - Creates local directory and initializes repository
   - Configures remote URL as "origin"

5. **test_connection()** - Health check
   - Tests connectivity to `/health` endpoint

### Server Component (`src/server.rs`)

The HTTP server handles incoming requests:

```rust
pub struct ServerState {
    repos_dir: PathBuf,
    auth: Arc<Mutex<ServerAuth>>,
}
```

**Endpoints:**

- `GET /health` - Health check
- `POST /repo/{name}/push` - Accept push
- `GET /repo/{name}/pull` - Provide pull data
- `GET /repo/{name}/fetch` - Provide branch list
- `GET /repo/{name}/clone` - Provide full repository

**Authentication:**

All endpoints require Bearer token in Authorization header:
```
Authorization: Bearer {token}
```

Tokens are validated against permission model (read/write).

### Sync Manager (`src/sync.rs`)

The `SyncManager` coordinates local and remote operations:

```rust
pub struct SyncManager {
    repo: Repository,
}
```

**Methods:**

- `push(remote_name, branch)` - Push to remote
- `pull(remote_name, branch)` - Pull from remote
- `fetch(remote_name)` - Fetch branch info
- `test_connection(remote_name)` - Check connectivity

All methods are async and use the HTTP client internally.

## Transport Protocols

### HTTP/HTTPS

Currently implemented with reqwest HTTP client:

- **TLS**: Rustls (no OpenSSL dependency)
- **Serialization**: JSON via serde
- **Connection**: Keep-alive support
- **Errors**: HTTP status codes + JSON error responses

### SSH (Planned for v1.1.0)

Reserved for future implementation. Currently returns error:
```
"SSH support coming in v1.1.0"
```

## Integration with Commands

### CLI Integration

Network operations are accessed via main.rs CLI:

```bash
mug push -r origin -b main    # Push to remote
mug pull -r origin -b main    # Pull from remote
mug fetch -r origin            # Fetch from remote
mug clone <url>                # Clone repository
```

Commands use `#[tokio::main]` for async runtime.

## Error Handling

Network errors are caught and converted to user-friendly messages:

```rust
match client.push(&remote, &repo, branch, "").await {
    Ok(response) => { /* handle success */ }
    Err(e) => Ok(SyncResult::failed(format!("Push failed: {}", e)))
}
```

HTTP errors (4xx, 5xx) are converted to Error::Custom.

## URL Parsing

Remote URLs are parsed to extract repository names:

```
https://example.com/repo       -> "repo"
https://example.com/repo/      -> "repo"
https://example.com/repo.git   -> "repo"
git@github.com:user/myrepo     -> "myrepo"
/path/to/repo                  -> "repo"
```

## Testing

Unit tests for URL parsing:

```bash
cargo test -- --nocapture client::tests
```

## Performance Considerations

### Compression

Currently not implemented. Planned optimizations:
- gzip compression for large payloads
- Binary protocol option for performance

### Batching

Multiple commits can be pushed/pulled in single request.

### Parallel Operations

Uses tokio async runtime for concurrent requests.

## Security Considerations

### HTTPS

- Use HTTPS URLs for production: `https://example.com/repo`
- TLS certificate validation via rustls
- No hostname verification skipping (secure by default)

### Authentication

- Bearer token validation on server
- Tokens checked against per-repository permissions
- No plain-text password storage

### Data Validation

- All JSON payloads validated against schema
- Content hash verification (SHA256)
- Commit signature support planned

## Known Limitations

1. **No SSH support** (v1.0.0-alpha.1)
   - HTTP/HTTPS only
   - SSH coming in v1.1.0

2. **No compression** 
   - Full data sent without compression
   - gzip support planned

3. **No proxy support**
   - Direct connections only
   - HTTP proxy support planned

4. **No authentication UI**
   - Tokens must be provided manually
   - OAuth coming in v1.1.0

## Future Enhancements

### v1.1.0

- SSH transport with key management
- OAuth2 authentication
- gzip compression
- HTTP/2 support

### v1.2.0

- Git protocol support
- Smart HTTP protocol
- Shallow cloning
- Repository mirroring

### v2.0.0

- End-to-end encryption
- P2P transport option
- Custom transport plugins

## Running the Server

```bash
# Start MUG server on localhost:3000
mug server --host 127.0.0.1 --port 3000 --repo-dir /tmp/repos

# Push to local server
mug remote add origin http://127.0.0.1:3000/myrepo
mug push -r origin -b main
```

## Example Workflow

```bash
# Local repository
mug init myrepo
cd myrepo
echo "hello" > file.txt
mug add .
mug commit -m "initial" -a "User"

# Add remote
mug remote add origin https://example.com/myrepo

# Push
mug push -r origin -b main

# On another machine
mug clone https://example.com/myrepo
cd myrepo
```

## Troubleshooting

### Connection Refused

```
Push failed: error trying to connect: ...
```

- Check server is running
- Verify URL is correct
- Check firewall rules

### 401 Unauthorized

```
Push failed: Permission denied
```

- Check Bearer token is provided
- Verify token has write permission
- Re-authenticate if needed

### 404 Not Found

```
Push failed: Repository not found
```

- Check repository name in URL
- Verify repository exists on server
- Create repository if needed

## References

- Protocol definitions: `src/protocol.rs`
- Client implementation: `src/client.rs`
- Server implementation: `src/server.rs`
- Sync operations: `src/sync.rs`
