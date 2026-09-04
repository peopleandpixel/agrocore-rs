---
name: Bug Report
about: Report a bug or unexpected behavior
title: "[BUG] "
labels: ["bug", "triage"]
assignees: []
---

## Bug Description
A clear and concise description of what the bug is.

## Steps to Reproduce
1. Go to '...'
2. Click on '...'
3. Scroll down to '...'
4. See error

## Expected Behavior
A clear and concise description of what you expected to happen.

## Actual Behavior
What actually happened (error message, stack trace, screenshot).

## Environment
- **OS**: [e.g., Ubuntu 24.04, Windows 11, macOS 15]
- **Rust version**: [output of `rustc --version`]
- **Docker version**: [output of `docker --version`]
- **Deployment**: [Docker Compose / Local cargo run / Kubernetes / Other]
- **AgroCore version**: [git tag / commit hash / `cargo metadata`]

## Configuration
Relevant environment variables (redact secrets):
```bash
# Example
AGROCORE_DATABASE_URL=postgresql://...
AGROCORE_NATS_URL=nats://...
```

## Logs / Error Output
```
Paste relevant logs here (use code blocks)
```

## Additional Context
- Screenshots (Admin UI, TUI Dashboard)
- Related issues/PRs
- Workaround (if any)
- Impact: [Critical / High / Medium / Low] – affects production? data loss?