# GitHub Repository Settings – Manual Setup Guide

> **Diese Einstellungen kannst nur DU als Repository-Owner vornehmen.** Ich habe alle Dateien vorbereitet – du musst nur in den GitHub-Settings klicken.

---

## 1. Repository Basics (Settings → General)

### Description (About section → ⚙️ Edit)
```
Farm operations platform – fields, tasks, livestock, finance, compliance, weather, equipment & admin in one local-first, open-source system. 8 European LPIS providers (SIGPAC, BRP, RPG, iLPIS, SIAN, LPIS-DE, LPIS-PL, INVEKOS) built-in. Rust 2024 · Actix Web · Leptos · PostgreSQL/PostGIS · NATS.
```

### Website (About section → ⚙️ Edit)
- **Website**: `https://github.com/peopleandpixel/agrocore-rs` (or your custom domain if you have one)
- Wenn du eine Demo-Instanz hast: `https://demo.agrocore.rs` (später)

### Topics (About section → ⚙️ Edit → Topics)
**Kopiere diese Liste genau:**
```
rust, agriculture, farm-management, postgis, lpis, sigpac, brp, rpg, ilpis, sian, actix-web, leptos, open-source, gpl-3, local-first, docker, nats, precision-agriculture, multi-language, i18n
```
> **Tipp**: Max 20 Topics – diese 19 sind optimal für Auffindbarkeit.

### Social Preview Image
1. Settings → General → Social preview → **Upload an image**
2. Wähle: `docs/agrocore_RS.png` (bereits im Repo)
3. Das Bild wird bei Twitter/X, LinkedIn, Slack, Discord, Matrix als Preview angezeigt

---

## 2. Features (Settings → General → Features)

**Aktiviere:**
- ✅ **Issues** (schon an)
- ✅ **Projects** (optional, für Roadmap-Boards)
- ✅ **Wiki** (deaktivieren – wir nutzen docs/ + Discussions)
- ✅ **Discussions** ⭐ **WICHTIG** – für Community-Q&A, Show & Tell, Farmer-Feedback
- ✅ **Sponsorships** (schon über FUNDING.yml, aber hier sichtbar machen)

**Deaktiviere:**
- ❌ Wiki (nutzt docs/ Ordner)

---

## 3. Branches (Settings → Branches)

### Branch Protection Rules → Add rule
**Branch name pattern**: `main`

**Required:**
- ✅ Require a pull request before merging
  - ✅ Require approvals: **1** (du alleine → 1 reicht)
  - ✅ Dismiss stale PR approvals when new commits are pushed
  - ✅ Require review from code owners (wenn CODEOWNERS existiert)
- ✅ Require status checks to pass before merging
  - ✅ Require branches to be up to date before merging
  - **Status checks** (werden nach erstem CI-Run erscheinen):
    - `Quality Gates`
    - `Tests`
    - `Build Admin UI WASM`
    - `Docker Build`
    - `Security Audit`
- ✅ Require conversation resolution before merging
- ✅ Require signed commits (optional, aber empfohlen)
- ✅ Require linear history
- ✅ Do not allow bypassing the above settings

**Optional (später):**
- ✅ Require deployments to succeed (wenn du Staging/Prod Environments hast)

---

## 4. Security & Analysis (Settings → Security & analysis)

**Aktiviere ALLES:**
- ✅ Dependency graph
- ✅ Dependabot alerts
- ✅ Dependabot security updates (auto-PRs für Security-Fixes)
- ✅ Dependabot version updates (braucht `.github/dependabot.yml` – siehe unten)
- ✅ Code scanning alerts (CodeQL – GitHub führt Rust-Analysis automatisch durch)
- ✅ Secret scanning alerts
- ✅ Secret scanning push protection

---

## 5. Dependabot Configuration (Erstelle `.github/dependabot.yml`)

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

**Erstelle zwei Environments:**
1. **staging** – für Preview-Deployments
2. **production** – für Releases

**Protection rules für `production`:**
- ✅ Required reviewers: `@peopleandpixel` (du)
- ✅ Wait timer: 5 minutes
- ✅ Deployment branches: `main` only

---

## 7. Pages (Settings → Pages) – für Documentation / Landingpages

**Source**: `Deploy from a branch`
**Branch**: `main` / `docs` (oder `gh-pages` wenn du separate docs baust)
**Folder**: `/docs` (für GitHub Pages aus `docs/` Ordner)

> Damit kannst du `https://peopleandpixel.github.io/agrocore-rs/` für Docs/Marketing nutzen.

---

## 8. Actions (Settings → Actions → General)

**Actions permissions:**
- ✅ Allow all actions and reusable workflows

**Workflow permissions:**
- ✅ Read and write permissions (für Release-Action, Docker Push, etc.)
- ✅ Allow GitHub Actions to create and approve pull requests (für Dependabot)

---

## 9. Collaborators & Teams (Settings → Collaborators)

**Falls du Co-Maintainer hinzufügst:**
- Role: `Maintain` oder `Admin`
- Teams: erstelle `maintainers` Team mit Write-Zugriff

---

## 10. Notifications (Personal Settings → Notifications)

**Für dich empfohlen:**
- ✅ Automatically watch repositories you contribute to
- ✅ Email notifications für: Security alerts, Dependabot alerts, Failed workflows
- ✅ GitHub Mobile App installieren (Push für Critical Alerts)

---

## 11. Repository Secrets (Settings → Secrets and variables → Actions)

**Für CI/CD (bereits in ci-cd.yml referenziert):**
| Secret Name | Wert | Beschreibung |
|-------------|------|--------------|
| `DOCKERHUB_USERNAME` | `dein_dockerhub_user` | Für `docker push` |
| `DOCKERHUB_TOKEN` | `dckr_pat_xxx` | Docker Hub Access Token (Read/Write) |

**Optional (später):**
| Secret Name | Wert |
|-------------|------|
| `CARGO_REGISTRY_TOKEN` | Für `cargo publish` (wenn du Crates auf crates.io veröffentlichst) |
| `GHCR_TOKEN` | `ghp_xxx` – wenn du statt Docker Hub GHCR nutzt (schon in GITHUB_TOKEN) |

---

## 12. Custom Properties (Settings → Properties) – Optional

**Repository properties** (für Organization-Level Reporting):
- `project-type`: `application`
- `domain`: `agriculture`
- `tech-stack`: `rust,actix,leptos,postgis,nats`
- `license`: `gpl-3`
- `maturity`: `beta` (0.x versions)
- `target-audience`: `farmers,advisors,cooperatives`

---

## 13. Webhooks (Settings → Webhooks) – Optional

**Für externe Integrationen:**
- Discord/Slack Notifications bei Releases
- Netlify/Vercel Deploy Hook für Docs-Seite
- Custom Deployment Webhook

---

## ✅ Checklist – Alles erledigt?

| Kategorie | Item | Done? |
|-----------|------|-------|
| **Basics** | Description gesetzt | ☐ |
| **Basics** | Website gesetzt | ☐ |
| **Basics** | **19 Topics** gesetzt | ☐ |
| **Basics** | Social Preview Image (`docs/agrocore_RS.png`) | ☐ |
| **Features** | **Discussions aktiviert** | ☐ |
| **Features** | Wiki deaktiviert | ☐ |
| **Branches** | Branch Protection `main` mit 5 Checks | ☐ |
| **Security** | Alle 7 Security Features an | ☐ |
| **Security** | `.github/dependabot.yml` committed | ☐ |
| **Environments** | `staging` + `production` erstellt | ☐ |
| **Pages** | GitHub Pages aus `/docs` aktiviert | ☐ |
| **Actions** | Write permissions + PR approval | ☐ |
| **Secrets** | `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` | ☐ |
| **Files** | Alle neuen Files committed & gepusht | ☐ |

---

## 🚀 Nach dem Push – Was passiert automatisch?

1. **CI/CD läuft** → Badges in README werden grün
2. **Dependabot PRs** erscheinen montags
3. **CodeQL Scans** laufen wöchentlich
4. **Discussions** Tab ist da → Community kann Fragen stellen
5. **Sponsor Button** erscheint im Header
6. **Issue Templates** greifen bei "New Issue"
7. **PR Template** greift bei "New Pull Request"

---

## 📝 Nächste Commits (kopierfertig)

```bash
cd /home/jens/RustroverProjects/agrocore-rs

# Alle neuen Dateien hinzufügen
git add README.md CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md \
  .github/FUNDING.yml .github/dependabot.yml \
  .github/ISSUE_TEMPLATE/ .github/PULL_REQUEST_TEMPLATE.md

# Commit mit DCO sign-off
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

## 🎯 Danach: Erste Discussions eröffnen

Gehe zu **Discussions** → **New discussion** und erstelle diese 3 Kategorien:

| Category | Title | Body |
|----------|-------|------|
| **Announcements** | 📢 Welcome to AgroCore RS Discussions! | This is the community space for farmers, advisors, developers, and contributors. Ask questions, share setups, request features, report bugs. |
| **Show & Tell** | 🚜 Show us your AgroCore setup! | Running AgroCore on your farm? Share screenshots, docker-compose tweaks, custom reports, hardware integrations. |
| **Ideas** | 💡 Feature requests & ideas | What would make your farm office work easier? New LPIS country? Mobile app? Specific report? Tell us! |

---

**Fertig!** 🎉 Dein Repository ist jetzt professionell aufgestellt für:
- **Entwickler** (klare Contribution-Guides, CI/CD, Security)
- **Landwirte** (Discussions, mehrsprachig, LPIS-Fokus)
- **Sponsoren** (FUNDING.yml, Sponsor Button)
- **Suchmaschinen** (Topics, Description, Social Preview)