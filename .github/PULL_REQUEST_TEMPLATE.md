# Pull Request Template

## Description
Brief summary of changes. Link related issue(s): `Fixes #123`, `Relates to #456`.

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update
- [ ] Refactor / Code quality / Performance
- [ ] Test improvements
- [ ] CI/CD / Build / Release
- [ ] Dependency update

## Affected Crates
- [ ] api
- [ ] admin-ui
- [ ] asset-registry
- [ ] backup-service
- [ ] dashboard
- [ ] domain
- [ ] geometry-service
- [ ] infrastructure
- [ ] i18n-shared
- [ ] lpis-providers
- [ ] logging
- [ ] messaging
- [ ] reporting-service
- [ ] scheduler
- [ ] shared
- [ ] weather-service

## Testing
### Local Quality Gates (ALL must pass)
```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace --features=mocks
cargo audit
```

### Test Results
- [ ] All existing tests pass
- [ ] New unit tests added for changed logic
- [ ] New integration tests added (if API/DB changes)
- [ ] LPIS provider tests pass (if provider changed)
- [ ] Manual testing done: [describe what you tested]

## Screenshots / Demo (if UI changes)
| Before | After |
|--------|-------|
| ![before](url) | ![after](url) |

## Breaking Changes
- [ ] No breaking changes
- [ ] Yes – describe:
  - API: [endpoint changes, DTO changes]
  - DB: [migration needed, run `sqlx migrate run`]
  - Config: [new required env vars, renamed vars]
  - UI: [route changes, component API changes]

## Migration Guide (if breaking)
Steps for users upgrading from previous version:
1. 
2. 
3. 

## Checklist
- [ ] Code follows project style (rustfmt, clippy clean)
- [ ] Self-review completed
- [ ] Comments added for complex logic
- [ ] Documentation updated (README, CHANGELOG, doc comments)
- [ ] CHANGELOG.md updated (Keep a Changelog format)
- [ ] Translations updated (if UI text changed) – all 10 languages
- [ ] DCO sign-off: `git commit -s` (Developer Certificate of Origin)
- [ ] No `unwrap()`/`expect()`/stubs in production code
- [ ] No direct `tracing`/`async_nats`/`tokio::time::interval` in business logic
- [ ] Security implications considered (see SECURITY.md)

## Additional Notes
Anything else reviewers should know? Deployment notes? Rollback plan?