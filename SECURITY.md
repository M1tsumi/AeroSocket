# Security Policy

## Supported Versions

The AeroSocket team provides security updates for the following versions:

| Version | Supported | Security Updates |
|---------|-----------|------------------|
| 1.x.x   | ✅ Yes    | Yes              |
| 0.3.x   | ✅ Yes    | Yes              |
| 0.2.x   | ⚠️ Limited | Critical only   |
| 0.1.x   | ❌ No     | No               |

*Versions older than 0.2.x are no longer supported for security updates.*

## Reporting a Vulnerability

If you discover a security vulnerability in AeroSocket, please report it responsibly.

### How to Report

**Do NOT open a public issue.** Instead, send your report privately to:

quefep on discord

### What to Include

Please include the following information in your report:

1. **Vulnerability Description**
   - Clear description of the vulnerability
   - Potential impact and severity
   - Affected versions

2. **Proof of Concept**
   - Minimal reproduction case
   - Steps to reproduce
   - Any relevant code snippets

3. **Environment Information**
   - Operating system
   - Rust version
   - AeroSocket version
   - Related dependencies

4. **Additional Context**
   - Any mitigations you've found
   - Whether you believe this is exploitable
   - Timeline for disclosure (if applicable)

### Response Timeline

We commit to the following response times:

- **Critical (CVSS 9.0+)**: Within 24 hours
- **High (CVSS 7.0-8.9)**: Within 48 hours  
- **Medium (CVSS 4.0-6.9)**: Within 72 hours
- **Low (CVSS 0.1-3.9)**: Within 5 business days

### Disclosure Process

1. **Acknowledgment**: We'll confirm receipt within the response timeline
2. **Assessment**: We'll evaluate the vulnerability and determine severity
3. **Coordination**: We'll work with you to develop and test a fix
4. **Disclosure**: We'll coordinate public disclosure with you
5. **Credit**: We'll credit you in the security advisory (with your permission)

## Security Features

AeroSocket includes several security features by design:

### Built-in Protections

- **Memory Safety**: Rust's ownership model prevents common vulnerabilities
- **Input Validation**: All WebSocket frames are validated
- **Size Limits**: Configurable limits on frame and message sizes
- **Timeout Protection**: Handshake and idle timeouts prevent resource exhaustion
- **TLS Support**: Secure connections with modern cryptography

### Security Best Practices

- **Secure Defaults**: TLS enabled by default when available
- **Least Privilege**: Minimal permissions required
- **Defense in Depth**: Multiple layers of security checks
- **Regular Audits**: Regular security audits and dependency scanning

## Security Advisories

We maintain a list of security advisories in our [GitHub Security Advisories](https://github.com/M1tsumi/AeroSocket/security/advisories).

### Recent Advisories

*No current advisories. This section will be updated as needed.*

## Security Best Practices for Users

### Deployment

1. **Use TLS**: Always use TLS in production environments
2. **Keep Updated**: Use the latest supported version
3. **Configure Limits**: Set appropriate frame and message size limits
4. **Monitor Logs**: Monitor for unusual activity or errors
5. **Network Security**: Use firewalls and network segmentation

### Development

1. **Validate Input**: Always validate user input
2. **Error Handling**: Handle errors gracefully without leaking information
3. **Authentication**: Implement proper authentication and authorization
4. **Rate Limiting**: Implement rate limiting to prevent abuse
5. **Audit Code**: Regularly audit your WebSocket code

### Configuration

1. **Secure Headers**: Use secure WebSocket headers
2. **Origin Validation**: Validate WebSocket origins
3. **Subprotocol Negotiation**: Use secure subprotocol negotiation
4. **Extension Limits**: Limit enabled extensions
5. **Connection Limits**: Set reasonable connection limits

## Threat Model

### Attack Vectors We Consider

1. **Protocol Attacks**: Malformed WebSocket frames, protocol violations
2. **Resource Exhaustion**: Memory/CPU exhaustion, connection flooding
3. **Data Injection**: Malicious payload injection, command injection
4. **Authentication Bypass**: Unauthorized access, privilege escalation
5. **Information Disclosure**: Sensitive data leakage, error messages
6. **Denial of Service**: Service disruption, availability attacks

### Mitigation Strategies

1. **Input Validation**: Strict validation of all WebSocket frames
2. **Resource Limits**: Configurable limits on resources and connections
3. **Secure Defaults**: Secure-by-default configuration
4. **Error Handling**: Secure error handling without information leakage
5. **Monitoring**: Comprehensive logging and monitoring
6. **Testing**: Regular security testing and fuzzing

## Security Team

The AeroSocket security team includes:

- **Security Lead**: security@aerosocket.rs
- **Core Maintainers**: Review and merge security fixes
- **External Auditors**: Regular third-party security audits

## Security Rewards

We offer security rewards for responsibly disclosed vulnerabilities:

- **Critical**: $500 - $2,000 USD
- **High**: $200 - $500 USD  
- **Medium**: $100 - $200 USD
- **Low**: $50 - $100 USD

Rewards are at the discretion of the security team and depend on:
- Vulnerability severity and impact
- Quality of the report
- Availability of existing mitigations
- Responsiveness during disclosure

## Security Dependencies

We regularly audit our dependencies for security issues:

- **Automated Scanning**: Daily automated security scans
- **Manual Review**: Weekly manual security reviews
- **Dependency Updates**: Prompt updates for security issues
- **Vulnerability Monitoring**: Continuous monitoring for new vulnerabilities

## Security Testing

Our security testing includes:

- **Static Analysis**: Automated static code analysis
- **Dynamic Analysis**: Runtime security testing
- **Fuzz Testing**: Comprehensive fuzz testing of protocol handling
- **Penetration Testing**: Regular penetration testing
- **Security Audits**: Third-party security audits

## Contact Information

For security-related matters:

- **Security Issues**: security@aerosocket.rs (placeholder - configure when available)
- **Security Questions**: security@aerosocket.rs (placeholder - configure when available)
- **PGP Key**: Available on our website
- **Security Team**: security@aerosocket.rs (placeholder - configure when available)

For non-security matters, please use our regular channels:
- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Discord**: Community Discord Server

---

Thank you for helping keep AeroSocket secure! 🛡️
