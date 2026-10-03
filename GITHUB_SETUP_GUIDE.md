# GitHub Repository Settings – Manual Setup Guide

> **Only YOU as the repository owner can make these settings.** I have prepared all files – you just have to click through the GitHub settings.

---

## 1. Repository Basics (Settings → General)

### Description (About section → ⚙️ Edit)
```
Farm operations platform – fields, tasks, livestock, finance, compliance, weather, equipment & admin in one local-first, open-source system. 8 European LPIS providers (SIGPAC, BRP, RPG, iLPIS, SIAN, LPIS-DE, LPIS-PL, INVEKOS) built-in. Rust 2024 · Actix Web · Leptos · PostgreSQL/PostGIS · NATS.
```

### Website (About section → ⚙️ Edit)
- **Website**: `https://github.com/peopleandpixel/agrocore-rs` (or your custom domain if you have one)
- If you have a demo instance: `https://demo.agrocore.rs` (later)

### Topics (About section → ⚙️ Edit → Topics)
**Copy this list exactly:**
```
rust, agriculture, farm-management, postgis, lpis, sigpac, brp, rpg, ilpis, sian, actix-web, leptos, open-source, gpl-3, local-first, docker, nats, precision-agriculture, multi-language, i18n
```
> **Tip**: Max 20 topics – these 19 are optimal for discoverability.

### Social Preview Image
1. Settings → General → Social preview → **Upload an image**
2. Choose: `docs/agrocore_RS.png` (already in the repo)
3. The image is shown as a preview on Twitter/X, LinkedIn, Slack, Discord, Matrix

---

## 2. Features (Settings → General → Features)

**Enable:**
- ✅ **Issues** (already on)
- ✅ **Projects** (optional, for roadmap boards)
- ✅ **Wiki** (disable – we use docs/ + Discussions)
- ✅ **Discussions** ⭐ **IMPORTANT** – for community Q&A, show & tell, farmer feedback
- ✅ **Sponsorships** (already via FUNDING.yml, but make it visible here)

**Disable:**
- ❌ Wiki (we use the docs/ folder)

---

## 3. Branches (Settings → Branches)

### Branch Protection Rules → Add rule
**Branch name pattern**: `main`

**Required:**
- ✅ Require a pull request before merging
  - ✅ Require approvals: **1** (you're on your own → 1 is enough)
  - ✅ Dismiss stale PR approvals when new commits are pushed
  - ✅ Require review from code owners (if CODEOWNERS exists)
- ✅ Require status checks to pass before merging
  - ✅ Require branches to be up to date before merging
  - **Status checks** (will appear after the first CI run):
    - `Quality Gates`
    - `Tests`
    - `Build Admin UI WASM`
    - `Docker Build`
    - `Security Audit`
- ✅ Require conversation resolution before merging
- ✅ Require signed commits (optional, but recommended)
- ✅ Require linear history
- ✅ Do not allow bypassing the above settings

**Optional (later):**
- ✅ Require deployments to succeed (if you have staging/prod environments)

---

## 4. Security & Analysis (Settings → Security & analysis)

**Enable EVERYTHING:**
- ✅ Dependency graph
- ✅ Dependabot alerts
- ✅ Dependabot security updates (auto-PRs for security fixes)
- ✅ Dependabot version updates (requires `.github/dependabot.yml` – see below)
- ✅ Code scanning alerts (CodeQL – GitHub runs Rust analysis automatically)
- ✅ Secret scanning alerts
- ✅ Secret scanning push protection

---

## 5. Dependabot Configuration (Create `.github/dependabot.yml`)

```yaml
# .github/dependabot.yml
version: 2
updates:
  # Rust dependencies
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "06:00"
    open-pull-requests-limit: 10
    labels:
      - "dependencies"
      - "rust"
    groups:
      rust-minor:
        patterns:
          - "*"
        update-types:
          - "minor"
          - "patch"
      rust-major:
        patterns:
          - "*"
        update-types:
          - "major"

  # GitHub Actions
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "06:00"
    labels:
      - "dependencies"
      - "github-actions"

  # Docker base images
  - package-ecosystem: "docker"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "06:00"
    labels:
      - "dependencies"
      - "docker"
```

---

## 6. Environments (Settings → Environments)

**Create two environments:**
1. **staging** – for preview deployments
2. **production** – for releases

**Protection rules for `production`:**
- ✅ Required reviewers: `@peopleandpixel` (you)
- ✅ Wait timer: 5 minutes
- ✅ Deployment branches: `main` only

---

## 7. Pages (Settings → Pages) – for documentation / landing pages

**Source**: `Deploy from a branch`
**Branch**: `main` / `docs` (or `gh-pages` if you build separate docs)
**Folder**: `/docs` (for GitHub Pages from the `docs/` folder)

> This lets you use `https://peopleandpixel.github.io/agrocore-rs/` for docs/marketing.

---

## 8. Actions (Settings → Actions → General)

**Actions permissions:**
- ✅ Allow all actions and reusable workflows

**Workflow permissions:**
- ✅ Read and write permissions (for the release action, Docker push, etc.)
- ✅ Allow GitHub Actions to create and approve pull requests (for Dependabot)

---

## 9. Collaborators & Teams (Settings → Collaborators)

**If you add co-maintainers:**
- Role: `Maintain` or `Admin`
- Teams: create a `maintainers` team with write access

---

## 10. Notifications (Personal Settings → Notifications)

**Recommended for you:**
- ✅ Automatically watch repositories you contribute to
- ✅ Email notifications for: security alerts, Dependabot alerts, failed workflows
- ✅ Install the GitHub Mobile App (push for critical alerts)

---

## 11. Repository Secrets (Settings → Secrets and variables → Actions)

**For CI/CD (already referenced in ci-cd.yml):**
| Secret Name | Value | Description |
|-------------|-------|-------------|
| `DOCKERHUB_USERNAME` | `dein_dockerhub_user` | For `docker push` |
| `DOCKERHUB_TOKEN` | `dckr_pat_xxx` | Docker Hub Access Token (Read/Write) |

**Optional (later):**
| Secret Name | Value |
|-------------|-------|
| `CARGO_REGISTRY_TOKEN` | For `cargo publish` (if you publish crates on crates.io) |
| `GHCR_TOKEN` | `ghp_xxx` – if you use GHCR instead of Docker Hub (already in GITHUB_TOKEN) |

---

## 12. Custom Properties (Settings → Properties) – Optional

**Repository properties** (for organization-level reporting):
- `project-type`: `application`
- `domain`: `agriculture`
- `tech-stack`: `rust,actix,leptos,postgis,nats`
- `license`: `gpl-3`
- `maturity`: `beta` (0.x versions)
- `target-audience`: `farmers,advisors,cooperatives`

---

## 13. Webhooks (Settings → Webhooks) – Optional

**For external integrations:**
- Discord/Slack notifications on releases
- Netlify/Vercel deploy hook for the docs site
- Custom deployment webhook

---

## ✅ Checklist – Everything done?

| Category | Item | Done? |
|-----------|------|-------|
| **Basics** | Description set | ☐ |
| **Basics** | Website set | ☐ |
| **Basics** | **19 topics** set | ☐ |
| **Basics** | Social preview image (`docs/agrocore_RS.png`) | ☐ |
| **Features** | **Discussions enabled** | ☐ |
| **Features** | Wiki disabled | ☐ |
| **Branches** | Branch protection `main` with 5 checks | ☐ |
| **Security** | All 7 security features on | ☐ |
| **Security** | `.github/dependabot.yml` committed | ☐ |
| **Environments** | `staging` + `production` created | ☐ |
| **Pages** | GitHub Pages enabled from `/docs` | ☐ |
| **Actions** | Write permissions + PR approval | ☐ |
| **Secrets** | `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` | ☐ |
| **Files** | All new files committed & pushed | ☐ |

---

## 🚀 After the push – what happens automatically?

1. **CI/CD runs** → badges in the README turn green
2. **Dependabot PRs** appear on Mondays
3. **CodeQL scans** run weekly
4. **Discussions** tab is there → community can ask questions
5. **Sponsor button** appears in the header
6. **Issue templates** kick in on "New Issue"
7. **PR template** kicks in on "New Pull Request"

---

## 📝 Next commits (copy-paste ready)

```bash
cd /home/jens/RustroverProjects/agrocore-rs

# Add all new files
git add README.md CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md \
  .github/FUNDING.yml .github/dependabot.yml \
  .github/ISSUE_TEMPLATE/ .github/PULL_REQUEST_TEMPLATE.md

# Commit with DCO sign-off
git commit -s -m "chore: optimize GitHub presence

- README: updated badges, LPIS table, one-liner, screenshots placeholder
- CONTRIBUTING: full guide + LPIS provider step-by-step
- SECURITY: policy, supported versions, hardening guide
- CODE_OF_CONDUCT: Contributor Covenant 2.1 + farm context
- FUNDING: GitHub Sponsors + Open Collective
- Issue templates: bug, feature, LPIS provider, general
- PR template: quality gates, breaking changes, DCO
- dependabot.yml: weekly cargo/actions/docker updates
- Branch protection ready, Discussions ready, Security features ready"

# Push
git push origin main
```

---

## 🎯 Afterwards: open the first Discussions

Go to **Discussions** → **New discussion** and create these 3 categories:

| Category | Title | Body |
|----------|-------|------|
| **Announcements** | 📢 Welcome to AgroCore RS Discussions! | This is the community space for farmers, advisors, developers, and contributors. Ask questions, share setups, request features, report bugs. |
| **Show & Tell** | 🚜 Show us your AgroCore setup! | Running AgroCore on your farm? Share screenshots, docker-compose tweaks, custom reports, hardware integrations. |
| **Ideas** | 💡 Feature requests & ideas | What would make your farm office work easier? New LPIS country? Mobile app? Specific report? Tell us! |

---

**Done!** 🎉 Your repository is now professionally set up for:
- **Developers** (clear contribution guides, CI/CD, security)
- **Farmers** (Discussions, multilingual, LPIS focus)
- **Sponsors** (FUNDING.yml, sponsor button)
- **Search engines** (topics, description, social preview)
