# MUG Remote Server - Quick Start

## Build & Run

```bash
# Build release binary
cargo build --release

# Start server on all IPs
./target/release/mug serve --host 0.0.0.0 --port 8080 --repos /path/to/repos

# Test it's running
curl http://localhost:8080/health
# {"status":"ok"}
```

## API Quick Reference

| Endpoint | Method | Purpose | Body Required |
|----------|--------|---------|---|
| `/health` | GET | Health check | No |
| `/repo/{name}/info` | GET | Repository info | No |
| `/repo/{name}/list-branches` | GET | List branches | No |
| `/repo/{name}/fetch` | POST | Fetch branches | Yes |
| `/repo/{name}/pull` | POST | Pull changes | Yes |
| `/repo/{name}/push` | POST | Push changes | Yes |
| `/repo/{name}/clone` | POST | Clone repository | Yes |

## Example Requests

```bash
# Health check
curl http://localhost:8080/health

# Get repo info
curl -H "Authorization: Bearer token123" \
  http://localhost:8080/repo/myrepo/info

# Fetch branches
curl -X POST \
  -H "Authorization: Bearer token123" \
  -H "Content-Type: application/json" \
  -d '{"repo":"myrepo","branch":null}' \
  http://localhost:8080/repo/myrepo/fetch
```

## Docker Deployment

```bash
# Build image
docker build -t mug:latest .

# Run container
docker run -p 8080:8080 -v /data/repos:/repos mug:latest \
  serve --host 0.0.0.0 --port 8080 --repos /repos

# Or use docker-compose
docker-compose up
```

## Network Access

From another machine on the network:

```bash
# Get IP of server machine
ip addr show

# Test from client machine
curl http://192.168.1.100:8080/health

# Access with token
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://192.168.1.100:8080/repo/myrepo/info
```

## Key Features

✅ **Git-like operations** - clone, fetch, pull, push  
✅ **HTTP-based** - firewall friendly, no SSH setup  
✅ **Token auth** - simple Bearer token authentication  
✅ **Network accessible** - via IP address from anywhere  
✅ **Content-addressable** - automatic deduplication  
✅ **Fast** - O(1) status checks, efficient transport  

## Troubleshooting

```bash
# Check if server is running
ps aux | grep mug

# Check port is listening
netstat -tlnp | grep 8080

# Check logs (if running with logging)
RUST_LOG=debug ./target/release/mug serve --host 0.0.0.0 --port 8080 --repos /repos

# Test connectivity
curl -v http://localhost:8080/health
```

## Documentation

- **IMPROVEMENTS.md** - What was added/fixed
- **REMOTE_SERVER_USAGE.md** - Complete API documentation
- **IMPROVEMENTS_SUMMARY.txt** - Detailed change summary

## Next Steps

1. Complete object store iteration (see TODOs in code)
2. Implement token management system
3. Add branch enumeration logic
4. Deploy to production with HTTPS

See REMOTE_SERVER_USAGE.md for full details.
