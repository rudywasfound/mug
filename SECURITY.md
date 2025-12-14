# Security

## Cryptographic Signing

Ed25519 Keypairs:
```bash
mug keys generate
```
Creates public/private Ed25519 keypair for commit signing.

Import Keys:
```bash
mug keys import <seed>
```
Import keys from base64-encoded seed for portability.

Immutable Audit Trail:
- All commits signed with Ed25519
- Proves authorship
- Prevents commit forgery
- Full history verification

## Integrity Verification

SHA256 Hashing:
- All objects identified by SHA256 hash
- Content-addressed storage
- Automatic corruption detection
- No duplicate content

Repository Verification:
```bash
mug verify
```
Checks:
- Object integrity
- Reference validity
- Database consistency
- Missing objects

## Access Control

File Permissions:
- Respects system file permissions
- Prevents unauthorized file access
- Enforces umask settings

Remote Authentication:
- SSH key support for remotes
- HTTP basic auth support
- SSH agent integration

Hook Execution:
- Subprocess isolation
- Limited permissions
- Controlled environment

## Best Practices

Commit Signing:
```bash
mug config set sign.commits true
```
Enable automatic commit signing with your key.

Backup Keys:
- Store seed securely
- Use password manager
- Never share seed
- Keep backups encrypted

Protect Repositories:
- Use private repositories on servers
- Restrict SSH access
- Use branch protection rules
- Require code review

Secure Remote Access:
- Use SSH over HTTP for authentication
- Enable HTTPS for server mode
- Use firewall rules
- Monitor access logs

## Threat Model

Integrity Threats:
- Network attacks: HTTPS/SSH mitigates
- File corruption: SHA256 detection
- Unauthorized changes: Cryptographic signing
- Data loss: Git and distributed copies

Confidentiality Threats:
- Exposed credentials: Use SSH keys
- Network sniffing: HTTPS/SSH encryption
- File access: System permissions
- Key theft: Secure key storage

Availability Threats:
- Server outages: Distributed copies
- Corruption: Verification and GC
- Storage loss: Backups and remotes
- Denial of service: Rate limiting

## Known Limitations

Database Encryption:
- Not yet implemented
- Future enhancement
- Use OS-level encryption

Shallow Clones:
- Not yet implemented
- Full history always downloaded
- Consider pack files

Network Security:
- Basic HTTP server
- Not production-ready
- Use reverse proxy for security

Hook Sandboxing:
- Hooks execute in subprocess
- No full sandboxing
- Trust hook scripts

## Incident Response

Compromised Key:
1. Generate new key with `mug keys generate`
2. Update configuration
3. Notify collaborators
4. Rotate server access

Unauthorized Changes:
1. Run `mug verify` to check integrity
2. Review log with `mug log`
3. Use `mug reset` to revert
4. Check for unauthorized hooks

Data Corruption:
1. Run `mug verify` to identify issues
2. Use `mug gc` for recovery
3. Restore from backup if needed
4. Check recent commits with `mug reflog`

## Server Security

HTTP Server Mode:
```bash
mug serve --host 127.0.0.1 --port 8080 --repos /path
```

Security Recommendations:
- Bind to localhost (127.0.0.1) only
- Use reverse proxy (nginx) for HTTPS
- Implement authentication layer
- Use firewall rules
- Monitor access logs
- Keep behind VPN

Reverse Proxy Setup (nginx):
```nginx
server {
    listen 443 ssl;
    server_name repo.example.com;
    
    ssl_certificate /path/to/cert;
    ssl_certificate_key /path/to/key;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        auth_basic "Repository Access";
        auth_basic_user_file /etc/nginx/.htpasswd;
    }
}
```

## Compliance

Data Privacy:
- No telemetry collection
- No external connections
- Local operation by default
- Respects system privacy settings

Audit Logging:
- Full commit history
- Author attribution via signing
- Timestamp verification
- Change tracking with diffs

## Cryptography Details

Hash Function:
- Algorithm: SHA256
- Purpose: Object identification and integrity
- Collision resistance: 256-bit security
- Standard: FIPS 180-4

Signing Algorithm:
- Algorithm: Ed25519
- Purpose: Commit signing and authorship
- Security: 128-bit security level
- Performance: Fast constant-time operations

Key Derivation:
- Seed: Base64-encoded bytes
- Format: Portable across systems
- Storage: User-provided location
- Rotation: Generate new key as needed

## Future Enhancements

At-Rest Encryption:
- Encrypt database with AES-256
- Protect sensitive data
- Encryption key management

End-to-End Encryption:
- Encrypt push/pull operations
- Protect in-transit data
- Key exchange protocol

Hardware Security:
- Hardware key storage support
- TPM integration
- Secure enclave support

Multi-Signature:
- Require multiple approvals
- Complex workflows
- Higher security

## Contact

Security Vulnerability Reports:
- Email: atsharma623@gmail.com
- Do NOT open public issues
- Include:
  - Description
  - Reproduction steps
  - Potential impact
  - Suggested fix

## Changelog

Security Updates:
- Check release notes for updates
- Apply patches promptly
- Enable auto-updates if available
