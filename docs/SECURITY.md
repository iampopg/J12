# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.0.0 | ✅ |
| < 1.0 | ❌ |

## Reporting a Vulnerability

If you discover a security vulnerability in J12 Forensic, please report it by:

1. **Email:** Send details to [your-email@example.com]
2. **GitHub:** Open a private security advisory

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Security Considerations

### Data Protection

J12 Forensic is designed with forensic integrity in mind:

- **Local Storage:** All data is stored locally on the user's machine
- **No Telemetry:** The application does not send any data externally
- **Hash Verification:** All evidence files are hashed (SHA-256/SHA-512)
- **Chain of Custody:** All actions are logged for forensic admissibility

### Authentication

- Default credentials: `admin` / `admin123`
- **Change default credentials in production**
- Session-based authentication
- All sessions stored locally

### AI Privacy

- Local AI (Ollama) recommended for sensitive data
- When using cloud AI providers:
  - Only necessary data is shared
  - No evidence files are uploaded
  - Only text snippets for analysis
- AI provider data handling subject to their privacy policies

### Database Security

- SQLite database stored in user data directory
- No encryption at rest (planned for future)
- WAL mode for concurrent access
- Foreign key constraints enabled

### File Handling

- Evidence files are never modified
- Attachments stored with original hashes
- Temp files cleaned up on exit
- No external file sharing

## Best Practices

1. **Change default credentials** before first use
2. **Use local AI** for sensitive investigations
3. **Verify evidence hashes** before analysis
4. **Export audit logs** for court proceedings
5. **Keep the application updated**
6. **Backup database** regularly

## Known Limitations

- No database encryption at rest
- No multi-user access control
- No network isolation for AI providers
- No automatic security updates

## Security Roadmap

- [ ] Database encryption at rest
- [ ] Multi-user authentication
- [ ] Role-based access control
- [ ] Audit log signing
- [ ] Evidence file encryption
- [ ] Secure deletion of temp files
- [ ] AI data anonymization

---

*Last updated: 2026-08-26*
