# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in MUG, please report it responsibly to ensure the security of the project and its users.

### How to Report

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please email security@example.com with the following information:

1. **Description**: A clear description of the vulnerability
2. **Location**: The affected file(s), function(s), or module(s)
3. **Steps to Reproduce**: Detailed steps to reproduce the vulnerability
4. **Potential Impact**: Description of what an attacker could do with this vulnerability
5. **Suggested Fix**: If you have a proposed fix (optional)
6. **Timeline**: When you discovered it and when you plan to disclose it publicly

### Response Timeline

We aim to:
- Acknowledge receipt of vulnerability reports within 48 hours
- Provide a status update within 7 days
- Release a fix within 30 days for critical vulnerabilities
- Notify reporters before public disclosure

## Security Considerations

### Scope of Security Concerns

We take security seriously in the following areas:

- **Data Integrity**: Ensuring commits and data cannot be tampered with
- **Authentication**: Remote authentication and access control (when implemented)
- **Denial of Service**: Protection against resource exhaustion
- **Injection Attacks**: Protection against code injection vulnerabilities
- **Information Disclosure**: Prevention of unintended information leaks

### Known Limitations

MUG currently has these security-related limitations:

- **Network Transport**: Currently simulated - no actual network calls (future implementation)
- **Authentication**: No authentication or signing support yet
- **Encryption**: No encryption for remote transport (future implementation)
- **Access Control**: No permission/access control mechanisms
- **Audit Logging**: Limited audit trail capabilities

### Current Guarantees

As a local VCS, MUG provides:

- Content-addressable storage for integrity verification
- SHA256 hashing for content verification
- Isolated repository databases per workspace
- No network exposure in alpha releases

## Security Updates

Security updates will be:

1. Released on a dedicated security branch first
2. Tested thoroughly before public release
3. Documented in release notes with CVE references if applicable
4. Backported to supported versions if necessary

## Dependencies

MUG uses the following key dependencies. We monitor their security advisories:

- `sled` - Embedded database
- `serde/serde_json` - Serialization
- `sha2` - Cryptographic hashing
- `regex` - Pattern matching
- `clap` - CLI parsing
- `tokio` - Async runtime
- `actix-web` - Web framework

Security updates to dependencies are applied promptly.

## Best Practices for Users

When using MUG:

1. **Keep Updated**: Always use the latest version to get security fixes
2. **File Permissions**: Ensure `.mug/` directory has appropriate permissions
3. **Hook Security**: Review hook scripts before enabling them - they execute with your permissions
4. **Gitignore Patterns**: Be careful with `.mugignore` patterns to avoid unintended exposure
5. **Local Security**: Protect your local machine and repository access

## Future Security Improvements

Planned security enhancements include:

- GPG/SSH commit signing
- HTTPS/SSH transport with certificate validation
- OAuth authentication for remotes
- Per-repository access control
- Encrypted remote storage
- Audit logging
- Security scanning in CI/CD

## Responsible Disclosure

We follow responsible disclosure principles:

- Vulnerability details are not disclosed publicly until a fix is available
- Reporters are credited unless they request anonymity
- Coordinated disclosure allows time for users to patch
- Public advisories include CVE references where applicable

## Contact

For security matters:
- **Email**: security@example.com
- **Response Time**: Within 48 hours

For other issues, use standard GitHub issues.

---

Last Updated: 2025
