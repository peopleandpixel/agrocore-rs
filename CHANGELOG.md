# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.49.0] - 2026-10-04

O1 — the Admin UI could not start without a network. Found by switching the network
off and watching the page fail with `net::ERR_INTERNET_DISCONNECTED` for the document
itself.

### A correction to 0.47.0

That release's survey said the UI has no offline path: no pending-change store, no
offline state. That was wrong. `public/sw.js` existed — 6,924 bytes with a complete
strategy, cache-first for static files, a network-first fallback for API calls,
IndexedDB for queued task updates, background sync, push handling. It was not
unfinished; it was unregistered. `serviceWorker.register` appeared nowhere in the
repository.

The claim came from checking localStorage and not checking for a service worker —
the same error as the SIGPAC measurement inherited at the start of this work and the
Clippy gate reported clean at 0.44.0: a statement about absence made without looking.

### Why it would have failed anyway

Five defects, each sufficient on its own:

- `STATIC_ASSETS` named `/styles/tailwind.css` and `/styles/daisyui.css`; neither
  exists, and `cache.addAll` rejects the whole install if any URL fails.
- It cached two API endpoints during install, where an unauthenticated request answers
  401 and `addAll` rejects non-2xx.
- `manifest.json` declared `/icons/icon-*.png`; the directory did not exist.
- `cacheFirst` for every non-API request served the shell from cache forever, so a
  deployed fix never reached a client with the application open.
- Both offline pages loaded Tailwind from the CDN — the pages that exist only for the
  no-network case could not render in it.

### What was done

Twelve files vendored under `public/vendor/`, and `index.html` now has no external
reference at all. Two of them were not in `index.html` and would have been missed by
reading only that file: `src/leaflet.js` fetched the red location marker from
`raw.githubusercontent.com` and its shadow from `cdnjs.cloudflare.com`.

The worker was rewritten and is now registered from `src/main.rs`, after
`mount_to_body` so a registration failure cannot stop the application from starting.
The precache list names only servable paths, `precacheAll` skips rather than fails,
API responses are cached only when they succeeded, navigations are network-first with
the shell as fallback, and cache names are versioned so activation removes the previous
build. nginx serves `sw.js` with `no-cache`, because a worker cached forever can never
be updated.

Icons were generated from `logo_trans.png` with the maskable safe zone — `logo.png`
has an opaque cream background. Two manifest shortcuts referenced icons that do not
exist and were removed rather than pointed at fabricated images.

### O3 — wasm-opt, three wrong turns

`[tools].wasm_opt = "0"` read as "switch it off" is a version string, so it fetched
Binaryen version `0` and 404'd. `[build].wasm_opt = false` read as the boolean form:
Trunk has no such key, and the TOML was silently ignored — the error did not change.
The real control is the HTML attribute `data-wasm-opt="0"` on the
`<link data-trunk rel="rust">` element, where `WasmOptLevel::from_str` maps `"0"` to
`Off`. Set in `index.html`.

wasm-opt is a size optimisation, not a correctness one. Root cause not investigated;
recorded as O3.

### Verified

Zero external references and zero missing files across the three served documents.
Nine tests in `crates/admin-ui/tests/offline_capability_tests.rs`, each checked against
a deliberately broken input: removing the registration, restoring a CDN reference and
deleting a vendored file each fail exactly the corresponding test.

Gates: fmt, clippy -D warnings, the WASM build, and
`cargo test --workspace --features=mocks --no-fail-fast` under `RUSTFLAGS='-D warnings'`.
### The image built, and the artefact contained none of it

The build went green and the image was 105 MB. Opening it showed `sw.js`,
`manifest.json`, the icons and all twelve vendored files absent. Trunk copies nothing
from `public/` on its own -- a file or directory lands in `dist/` only when the HTML
asks for it with `rel="copy-dir"` or `rel="copy-file"`. Nine tests were green, every
reference resolved, every precache URL was servable, and the offline capability was
not in the thing you start.

Fourth time in this work that a check read the source and reported "fine" while the
decisive thing was somewhere else. The image should have been opened before the
claim, not after.

`index.html` now declares the copy for `sw.js`, `manifest.json`, `offline.html`,
`worker-tasks-offline.html`, `icons/` and `vendor/`, and an eleventh test asserts that
every file under `public/` is either copied or the shell document -- it fails when a
`copy-dir` line is removed.

### nginx: three startup failures in one line

Running the image exposed an unrelated defect that pointed straight at the offline
work:

```
[emerg] host not found in upstream "api" in /etc/nginx/conf.d/default.conf:58
```

nginx resolves a literal upstream host at startup and aborts when it cannot. `api` is
a Docker Compose service name, so the image could be built but never run standalone --
which is exactly what is needed to check offline behaviour against a real server.

Two wrong fixes followed before the third was right. `${AGROCORE_API_UPSTREAM:api:8080}`
is envsubst syntax, not nginx, and nginx rejected it with `the closing bracket in
"..." variable is missing`. `env AGROCORE_API_UPSTREAM;` inside `conf.d/*.conf` is
main-context only and that file is included from `http{}`, so it is never legal there.
The working fix is a variable in `proxy_pass`, which is resolved per request instead
of at startup, so an unresolvable API host no longer prevents nginx from starting.
Compose behaviour is unchanged. Verified with `nginx -t` and a started container.

### Cultivar and breed catalogue with tenant scope (migration 0000000009)

`varieties` and `breeds` existed but were unusable. No `tenant_id`, so no tenant could
add its own cultivar and no row-level security could ever apply -- a global reference
table that cannot be extended is only correct while nobody needs something missing.
No `active` flag, so removing an entry would orphan every object referencing it. No
performance reference values, so "Merino for wool" could not be turned into a number.

Global rows must be readable by every tenant while only a platform operator may create
them. RLS cannot express that combination, because every policy is evaluated against a
tenant id. The all-zero UUID is the global scope, which keeps
`tenant_id = $tenant OR tenant_id = $GLOBAL` an ordinary indexed predicate and needs no
special case in any policy. A tenant cannot shadow a global entry; it adds its own row
under its own name, which keeps name resolution unambiguous.

`spatial_objects` gains `planted_at` and `variety_id`, both nullable on purpose:
objects imported from SIGPAC carry no cultivar and no planting date, and forcing one
would either reject the import or invent data. `planted_at` belongs to the individual
object rather than to the cultivar, so a block planted in two different years stays
distinguishable and age is derived rather than stored twice.

The grouping this enables is "objects of this cultivar on this plot" and "every plot
carrying this cultivar". Both are tenant-scoped, so the tenant id leads the index.

`breeds` gains reference performance values: birth and mature weight, productive
lifespan, eggs per year, wool per year, daily gain, litter size, gestation. These are
breed references, not measurements of an individual; actual output belongs in a
measurement series per animal.

Verified against a clean PostGIS instance. All ten migrations apply without error. As
`agrocore_app`: tenant A reads 46 global rows, inserts its own cultivar, is blocked
from inserting a global row (RLS filters it silently rather than raising), and updates
0 rows of tenant B. Tenant B reads the same 46 globals, none of A's, and adds its own.

Known data errors left in place rather than silently corrected: `('Sheep','Merino',
'Spain')` is wrong -- Merino is Australian -- and `('Goat','Nubian','UK')` is doubtful.
A catalogue table is only as good as its sources, so these belong in a seeded
reference dataset with a `source` column, not patched in a migration.

### Reference catalogue, seeded and verified

`crop_species` splits species from cultivars. The initial migration seeded `varieties`
with 46 rows across eight categories, and most were not cultivars at all -- `('Apple',
'Apple')`, `('Wheat', 'Wheat')`, `('Tomato', 'Tomato')`. A species stated twice is not a
variety, and mixing the two made "every cultivar of Vitis vinifera" and "every species we
grow" the same question. 22 such rows moved to `crop_species` with a stable
`species_key`; `varieties` and `breeds` now reference it.

`crop_species` is the one reference value in the system that stays closed. A tenant may
add a cultivar or a breed, but `species_key` is globally unique and cannot be shadowed,
because a tenant-local duplicate would split one species into two in every aggregate
built on it.

Grapevine cultivars come from the GrapeGen06 European Catalogue (Annex 1A, INRA/VIVC):
638 cultivars registered in EU member states, with VIVC accession number, sex, berry
colour and allowed use. 434 are registered in more than one country, which is the column
that answers "may I plant this where I operate" -- origin alone is not a usable proxy.
Portugal 280, Italy 286, France 244, Spain 167, Germany 127.

Olive cultivars: 87 with origin, from the Wikipedia cultivar list.

Breeds: 110 across nine species, origin for every row, harvested from Wikipedia breed
infoboxes and verified rather than guessed.

Wool micron ranges come from the USDA grade tables in the American Sheep Industry
Association's "Wool Grades and the Sheep that Grow the Wool": Merino 17.70-19.14,
Suffolk 36.20-38.09, Lincoln 38.10-40.20. Fibre diameter rather than fleece weight,
because diameter decides the market value while kilograms decide the quantity, and
conflating them produced one misleading number. All 18 breeds in that table are present.

Egg figures carry the strain. Published values for one breed disagree by up to 100
eggs/year -- Rhode Island Red is quoted at 180-220 by the Livestock Conservancy, 250-300
commercially, 150-200 by Oklahoma State. `egg_production_strain` records which
population a figure describes, so a number is never read as universal.

Two origins in the original data were wrong and are corrected: `('Sheep','Merino',
'Spain')` -- Merino was developed in Australia from Spanish stock -- and `('Goat',
'Nubian','UK')` -- the Anglo-Nubian was bred in England, the Nubian breed is Egyptian.

### Four defects found by opening the artefact, not the log

Every one of these produced a green run and wrong data.

`registration_countries` was entirely NULL. The builder wrote `countries` and `synonyms`
while the generator read `registration_countries` and `synonym_names`. A dict lookup that
misses returns None, so every extra column silently became NULL -- and the seed still
applied cleanly.

Breeds were classified by filename substring with a default of `chicken`, so Lusitano,
Appaloosa and Holstein Friesian landed under poultry, and Merino and Suffolk carried the
micron values of breeds they are not. Replaced by derivation from what the article
declares, with anything unclassifiable dropped rather than defaulted.

`ON CONFLICT DO NOTHING` duplicated every row on a second run (749 -> 836 varieties)
because it had no unique index to match on. The fix was an explicit conflict target --
and then a further error, since the index had to include `global_species_key`: Hampshire
is a sheep breed and a pig breed, and Bronze a turkey and a goose.

The breed parser accepted disambiguation pages. "Angus may refer to:" is not a breed,
and the fallback silently classified it as poultry.

### Verified

Eleven migrations apply cleanly on a fresh PostGIS instance. The seed is idempotent --
732 varieties, 110 breeds, 41 species on first and second run. As `agrocore_app`,
tenant A reads the whole catalogue and is blocked from shadowing a species key.

### The seed generator read its inputs from /tmp

`build_catalogue_seed.py` loaded its three datasets from `/tmp`, which is not in the
repository. A fresh checkout therefore produced a seed with **zero** varieties and still
applied it cleanly — no error, no warning, just an empty catalogue. It only surfaced
because the intermediate files were cleaned up while the generator was rerun in the same
session.

The datasets now live in `scripts/reference_data/` with a README naming the source and
licence of each, and the generator resolves paths relative to its own location and exits
with a message if a file is missing or empty. Verified by removing the `/tmp` inputs
(identical output) and by removing a checked-in dataset (hard failure, exit before any
SQL is written).

## [0.47.0] - 2026-10-03

M6, first page — a failed fetch rendered as an empty list.

The survey that prompted it: **43 `LocalResource` fetches across 21 page components
call `fetch_x().await.ok()`**. That turns a failure into `None`, and `None` is
indistinguishable from "this tenant has nothing".

The writes are better than the reads — 72 of 84 `spawn_local` blocks handle their
`Err`, and no write result is discarded — so the defect is on the read path.

`worker_tasks.rs` already separated the three states and became the model.
`orders.rs`, which backs `/tasks`, had `set_load_error` nowhere: its `error` signal
existed but was used only for form validation, so a failed load produced no error
state at all. It now has separate `orders` / `loading` / `load_error` signals, a
retry that bumps a reload counter, and three i18n keys in all ten languages.

The ordering is the fix and it is asserted as such — checking emptiness first
defeats separate error storage, because the error never gets shown. Verified by
swapping the two branches: `error_before_empty` fails.

Two further findings from the same survey, unrelated to this fix and recorded rather
than dropped: `error_boundary.rs` exists and is never mounted (zero `<ErrorBoundary>`
in the application), and 45 hardcoded German strings bypass the ten-language i18n,
20 of them in `backup.rs`.

Gates: fmt, clippy -D warnings, the WASM build, and
`cargo test --workspace --features=mocks --no-fail-fast` under `RUSTFLAGS='-D warnings'`.

## [0.46.0] - 2026-10-03

M5 — the `mocks` test build failed on unused imports. A consequence of the M1 fix
and a mistake rather than a pre-existing defect: replacing thirty `AppState`
literals with `state_with(mock_db)` calls left behind the imports those literals had
needed — `AppState`, `Database`, `EncodingKey`, `Header`, `encode`, `Serialize` —
four or five per file across ten files.

**The gate was wrong, not just the code.** Local runs used
`cargo clippy --workspace --all-targets` without `--features mocks`. Every one of
the eleven targets has `required-features = ["mocks"]`, so Clippy never compiled
them; ten files and 139 tests sat outside the gate I had been reporting as clean.
GitHub sets `RUSTFLAGS: "-D warnings"` workflow-wide, so its test job rejects what
`cargo test` would otherwise have accepted.

Two more lints, both mine: an `if` with identical branches in
`weather_fetch_tests.rs`, and a runtime assertion between two `const`s in
`payload_limit_tests.rs` — which can never fail, so it moved into a `const _: ()`
block checked at compile time.

`field_reassign_with_default` appeared thirty-one times, including in two files
written this week. `crate::common::db_with(|db| db.repo = Some(...))` replaces the
pattern; the builder lives in the shared module because that is where the
reasoning belongs.

Verified with the exact CI commands: `cargo test --workspace --features=mocks
--no-fail-fast` exits 0 with zero failed results under `RUSTFLAGS='-D warnings'`,
as do `cargo clippy --workspace -- -D warnings` and the WASM build. 139 API tests
with mocks pass.

## [0.45.0] - 2026-10-03

M3 — the admin UI's WASM build failed. The UI depends on `agrocore-logging` with
`default-features = false`, so the crate compiles with neither `dev-console` nor
`otlp`, and three constructs in `layers.rs` were only valid with a tracing feature
on.

- `use crate::config::RotationType` and `use crate::error::LoggingError` were
  unconditional while every use of both sits behind a `cfg`.
- `init_logging`'s no-features branch ended in `return Ok(...)`, which is only
  needed while the other `cfg` branches are compiled; clippy's `needless_return`
  fires when it is the only block.

`cargo check` passed in that configuration before the fix. Only `clippy` and the CI
build caught it.

Verified: `RUSTFLAGS='-D warnings' cargo build --target wasm32-unknown-unknown
--release --lib -p admin-ui` exits 0, 1,235,592 byte artefact.

### M4 — a second feature gap, not on the WASM path (open)

`agrocore-domain` does not build without its `sqlx` feature: `entities/site.rs` and
`entities/spatial/types.rs` import `sqlx::postgres` unconditionally. The crate
declares `sqlx` as optional for consumers that do not talk to a database, so the
imports should be gated. Unreachable from the admin UI — it does not depend on
`domain` — but recorded rather than left to be found again.

Gates: fmt, clippy -D warnings, 308 workspace tests, 0 failures.

## [0.44.0] - 2026-10-03

M1 — the `mocks` test feature had no coverage at all. `cargo test -p agrocore-api
--features mocks` did not build: eleven targets, six already broken at 0.41.0.

The cause was mechanical. Thirty `AppState { ... }` literals across ten files, each
listing only the fields that file happened to need, plus a local copy of
`signed_token` in each of the eleven. Every time `AppState` gained a field — metrics
and the backup service most recently — each one stopped compiling and had to be
patched individually. Nobody patched them.

`tests/common/mod.rs` now holds `state_with(MockDatabase)` and `signed_token`.
Thirty literals became thirty one-line calls; eleven token helpers became one. The
measure: deleting `backup_service` from the helper now breaks one site, where before
it broke eleven. That is asserted, not asserted about.

### Four fixtures had drifted from their entities

- `Animal` lost `weight_kg`, `last_weight_date`, `group_id`, `treatments` and
  `grazing_history`; `species` and `status` became strings, `tenant_id` a bare
  `Uuid`, `birth_date` a `NaiveDate`.
- `Site.plots` became a `serde_json::Value`; `tenant_id` a bare `Uuid`.
- `TreatmentRecord.medication` became `String`; `created_at` an `Option`.
- `Equipment` gained `fuel_capacity_liters` and `fuel_type`.

All corrected against the current entity shapes rather than worked around.

### Two things found on the way

`reporting_integration.rs` called `MessagingClient::new_mock()` and then
`set_mock_response`, which does not exist and never did — it was building a queue
message the handler never reads. `new_mock()` itself is a placeholder that panics on
every call, so all eleven files would have failed had they compiled far enough to
reach it. `state_with` sets `messaging: None`, which is what the field became when
the broker was made optional.

The two reporting tests could not be made to pass as written: `export_*` are NATS
request-reply calls and `MessagingClient::request` is a real round trip with no
seam, so with no broker they correctly answer 500. They now assert that, and that
the endpoints require authentication. The happy path needs a NATS server in CI or a
trait seam — recorded as M2, a design change rather than a bug.

### Why this was never caught

CI runs `cargo test --workspace --features=mocks`, and would have been red for
weeks. `origin/main` was at 0.39.0 while the local branch was four commits ahead:
the broken targets were committed locally and CI never saw them.

Gates: fmt, clippy -D warnings, 308 workspace tests, 139 API tests with mocks, 44
infrastructure database tests with RLS enabled.

## [0.43.0] - 2026-10-03

C3 — the import path had no body limit. The audit finding was half right, and the
wrong half is the one that mattered.

`actix_web::web::Json` has always defaulted to 2 MiB (`JsonConfig::DEFAULT_LIMIT`),
so an ordinary endpoint was never unbounded. My first negative test confirmed it:
removing the explicit `JsonConfig` left all five tests green.

What was unbounded is `/sites/import/shapefile`. It carries a file as Base64 — a
third more than the binary — and a municipality's parcel shapefile is genuinely
10–30 MB, so the default rejects legitimate work. A handler in that position cannot
raise the global limit, and re-registering the routes in a scope with a second
`JsonConfig` does not work: they are already mounted by `handlers::configure`, and a
second registration of the same path is shadowed by the first.

So the limit is per-extractor now. `LargeJson<T>` reads the body itself with a 64 MiB
ceiling and is used by exactly the three import handlers; everything else keeps
2 MiB, written out explicitly rather than inherited so a future Actix upgrade cannot
silently change it. The oversize check runs on every chunk, so an oversized body is
rejected without ever being fully buffered.

Two non-obvious details: `Payload::take` in actix-http 3 takes no argument and
applies no limit, so the running total is compared by hand; and the payload stream is
moved into the future rather than borrowed, because `FromRequest::Future` is
`'static`.

### A finding the audit missed: eleven test targets do not compile

`cargo test -p agrocore-api --features mocks` does not build. Six were already broken
at 0.41.0; it is now eleven. Three causes: `AppState` gained four fields and eleven
files still construct it without them, `messaging` became an `Option` and those files
pass a bare `Arc`, and `Equipment` gained two fuel fields.

This means the `mocks` feature has no coverage at all, and every suite added in
0.40–0.43 that uses a repository mock is invisible to a plain `cargo test
--workspace`. Recorded as M1.

### A false pass worth recording

`the_global_limit_applies_to_ordinary_endpoints` first failed with 405, not 413: the
settings group resource registers only GET and PUT, so a POST is rejected before the
payload is read and the test would have passed for the wrong reason. It is the shape
of a test that looks like it covers the size limit and actually covers the routing
table.

Gates: fmt, clippy -D warnings, 308 workspace tests. Five new payload-limit tests,
verified against an unbounded `LargeJson`.

## [0.42.0] - 2026-10-03

Security and correctness batch: C2, C1, D2, J9, I5.

### Metrics endpoints had no authentication (C2)

`/metrics/db` and `/metrics/business` were registered on the `App` in `main`,
outside `handlers::configure`, with no extractor. Anyone who could reach the port
got the whole registry: per-table query counts and durations, pool saturation,
record counts.

Both now require an Admin, and have their own Governor scope — 5 s per request,
burst 12 — because a scrape is a full `gather()` over every metric family and a
dashboard polling at the API rate would spend the tenant-facing budget doing real
work on every request.

The registry carries no tenant label, so this was never a cross-tenant leak. It was
an unauthenticated operational map of the deployment.

### 5xx bodies carried database internals (C1)

`ApiError::error_response` rendered `self.0.to_string()` for every status. For a
500 that is the `Display` of `sqlx::Error`: table, column, violated constraint, and
for a failed decode the Rust type expected. It also changes whenever the schema
does, so clients could not parse it reliably either.

A 5xx body now points at the log. 4xx messages are unchanged — they are authored
and clients read them. `NotImplemented` keeps a specific 501 message, and now
actually returns 501; it returned 500.

### Impersonation was unreachable, and three defects sat behind it (D2)

`impersonate` compared roles against `"admin"` while `generate_jwt` emits
`"Admin"`, so nothing matched and nobody could impersonate anyone. Fixing only the
case would have made a dead endpoint live with the rest intact:

- the admin's token was never revoked;
- nothing was written to the audit log;
- the issued token carried no record of the original caller.

Now Admin-only, self-impersonation refused, audit entry written before the token is
minted, the caller's `jti` revoked, `impersonator_id` in the response.

`stop_impersonation` was open to any authenticated caller — a token-minting
endpoint with no audit trail. Now Admin-only, and it says whose token it returns,
because after impersonation the caller's token belongs to the impersonated user.

### The weather endpoint returned constants (J9)

`GET /api/v1/calculate/weather/fetch` parsed the provider, discarded it, and
answered with a fixed 20.5 °C / 65 % / 1013.25 hPa. The three providers had no
caller in the workspace. Now the provider is selected and called, an unknown name
is a 400 rather than a silent fallback, and a provider failure is reported as a
failure instead of papered over.

`crates/weather-service` was a binary crate with no `lib.rs`, so its providers
could not be called from the API at all. It is now a `[lib]` plus `[[bin]]`.

### The LPIS cache existed and was never enabled (I5)

`BaseClient::new` and `BrpProvider::new` set `cache: None`, and
`create_default_registry` called only the constructors — so the `if let
Some(cache)` branch in `get_cached_or_fetch` was unreachable in every deployment.
Every SIGPAC and BRP listing went to the national WFS service on every call, on
public rate-limited endpoints, for data that does not change within the hour.

Widened beyond the original finding: six more providers — RPG, iLPIS, SIAN, the
German and Polish LPIS services and INVEKOS — held a bare `reqwest::Client` with no
cache field at all. All eight registered providers now share one cache.

A synchronous `LpisCache::memory_if_enabled` was added because
`create_default_registry` runs in `AppState::new` and `LpisCache::new` is async. It
returns `None` for a disabled config or a Redis backend rather than a cache that is
written to but never read.

The six were not routed through `BaseClient` on purpose — each has its own error
type and status handling, and that would have meant rewriting six providers to
change one cache lookup. Cache keys are namespaced per provider, because a French
RPG parcel and a German LPIS parcel for the same bounding box would otherwise
collide on an identical key with different responses.

### Tests

Nine new test files, 36 tests. Every one was checked against a deliberately broken
input: removing `require_admin` fails the C2 role test; restoring the lowercase
role comparison fails four D2 tests; removing the cache from the registry fails
exactly the wiring test and leaves the six mechanics tests green.

Gates: fmt, check, clippy -D warnings, 308 workspace tests, 44 infrastructure
database tests with RLS enabled.

## [0.41.0] - 2026-10-03

G2i fixed: a page can no longer be implemented, wired into `api.rs`, and left
unreachable from the router without any test noticing.

### The check that could not fail

The test for this was removed in 0.40.0 because it passed with the `/water` route
and its import deleted — the one thing it existed to catch. It is back and it
fails.

Two of the three causes were found:

- The embedding scan collected every capitalised name following a `<` or a `(`
  anywhere under `admin-ui/src`. That is 161 names, including `Router`, `Routes`,
  `Icon` and `UserRole`, because `>Icon<` matches as well. Every page counted as
  embedded, so nothing was ever reported.
- A component matched its own declaration: the pattern `Name(` hit the
  `pub fn WaterManagement(` line in the file that declares it.

The third cause was the heuristic itself, so it is gone. Which components are
embedded rather than routed is declared in `EMBEDDED_NOT_ROUTED`, each with a
reason, and the two that are building blocks rather than pages go through
`is_widget`. A list reads as wrong when it is wrong; a heuristic does not.

### Two assertions the same bug had quietly weakened

- `sole_path` only accepts a trimmed, comma-terminated line. Handed an offset
  substring it returns `None` silently — and the route scan used it, so the first
  version of this test reported "no routes were parsed out of lib.rs". It now uses
  `literal_after`, as the rest of the file does.
- `#[test]` on a synchronous test in this file did not compile. `use
  actix_web::test` brings a module named `test` into scope, so the attribute
  resolved to Actix's async test macro and the compiler said "the async keyword is
  missing from the function declaration". The import is now `test as awtest`.

The discrimination is pinned by `the_page_reachability_check_can_still_fail`, on
synthetic input rather than by editing `lib.rs` — a test that mutates the working
tree to prove a point has to restore it, and a run that dies in between leaves the
tree broken for whatever runs next.

42 page components, 36 routes, 4 embedded, 2 widgets.

Gates: fmt, check, clippy -D warnings, workspace tests.

## [0.40.0] - 2026-10-03

G2 complete: every registered API endpoint now has a caller in the Admin UI.

### 27 endpoints that had no page at all

The five pages above shipped as 0.39.0; this release is the 27 endpoints.

| Page | Endpoints |
|---|---|
| `/calculators` | 12 `calculate/*` plus two duplicate registrations |
| `/water` | sources, usage, quotas |
| `/harvest` | seasons, lots, deliveries, cold chain |
| `/agriculture` | olive groves, olive oil records, vineyards, kelter deliveries |
| `/weather/warnings` | frost warnings, active warnings, pest risks |
| `/worker/logs`, `/worker/locations` | work logs, worker positions |
| `/compliance/licenses` | applicator licenses |

The route inventory test that found them reports zero uncovered endpoints.

### Three things that were not obvious

**`/workforce/locations` is not a CRUD collection.** `POST` takes the worker id
from the authenticated user rather than from the request body, so a client cannot
report a position on someone else's behalf. The page shows the current positions
and lets the caller post their own; there is no edit-and-delete table, because
that would imply a capability the endpoint does not have.

**Two calculations are registered twice.** `calculation.rs` and `nutrition.rs` each
mount a fertiliser-amount endpoint, and `calculation.rs` and `specialized.rs` each
mount a profitability one, with near-identical handler bodies. Both paths are live
and both are now reachable.

**The enum wire values are not guessable.** `CropType` and `RiskLevel` carry
lowercase serde renames. `WaterSourceType` and `IrrigationMethod` carry
`strum(serialize_all = "snake_case")` but no serde rename, so they serialise as
the variant name — `"Well"`, `"Drip"` — which is not what the strum attribute
suggests. `LotStatus` is lower-case. Each select was checked against its enum
rather than inferred.

### A test that could not fail was removed

A test was written to catch a page that is implemented and wired into `api.rs`
but never registered in the router. It passed with the `/water` route and its
import deleted, so it was removed rather than shipped. Two causes were found and
fixed on the way — a `<`-scan that swept up 161 names including `Router` and
`Icon`, and a component counting as embedded because its own `pub fn` line matched
— and one was not isolated.

Recorded as G2i. A test that cannot fail is worse than none: it produces
confidence without cover. The two route inventory tests that remain were each
checked against a deliberately broken input.

### A near-miss worth noting

Three subagents were dispatched to build the water, harvest and agriculture pages
in parallel. One overwrote `api.rs` — 2,867 lines down to 602 — while the others
were writing to it. The file was restored from HEAD and the work redone in
sequence. Concurrent edits to one shared source file is the failure mode here,
not the fan-out itself.

## [0.39.0] - 2026-10-03

G2, part one: five Admin UI pages that did nothing now work.

### Five pages were shells

`/trees`, `/groups`, `/buildings`, `/livestock` and `/plot/entities` were in the
router with complete CRUD handlers behind them in the backend. The pages
themselves collected form input and threw it away:

```rust
spawn_local(async move { let _ = (label.get(), count.get()); })
```

Three variations on the same defect, in increasing order of how much damage they
could do:

- `buildings.rs` showed a success toast naming the label, type and plot for a
  write it never performed. A page that claims success for nothing is worse than
  one that visibly does nothing, because it removes the reason to distrust the
  message.
- `plot_subentity.rs` rendered four rows of invented data — a herd called
  "Herde 1", two goats, four cork oaks, a barn — hard-coded in the markup. It
  looked like a working overview and said nothing about the tenant's data. An
  empty plot now shows an empty row, because "no groups" and "four cork oaks"
  must not look alike.

All five now call the API and support list, create, update and delete.

### A create that could not have worked

`CreateAnimalDto` requires `livestock_type` and `status`, and neither has a
default on the Rust side. The client struct omitted both, so the deserialised
request was missing required fields and the call would have been rejected with a
422 the page had no way to surface. The client struct also returned
`serde_json::Value` for the animal, which pushed every field access to a runtime
string lookup; it is typed now.

### 25 i18n keys added

The new pages needed labels, buttons, validation messages and the enum names for
ten languages. Added to `app.yml`, which parses.

Found while doing it: 109 keys referenced by pages across the UI are missing from
`app.yml` altogether, so those places render the key name instead of the label.
Pre-existing, unrelated to this change, recorded as G2h.

### G2 measured again: 32 down to 29

The five entity groups are gone from the uncovered list. What remains has no UI
at all — not a broken one, none: 12 `calculate/*` endpoints, 5 `specialized/*`,
4 `harvest/*`, 3 `water/*`, 3 `weather/*`, two workforce paths and three others.
Broken out as G2a–G2g in `docs/tasks.md`.

G2g is a defect the route test cannot see: four client functions in `api.rs`
(`calculate_material_request`, `calculate_nutrition_demand`, `calculate_water_rate`,
`fetch_specialized_sites`) are complete and working, and no component calls them.
The route test reads path literals, so a bound-but-uncalled endpoint looks covered.

## [0.38.0] - 2026-10-03

J20, J21, and a tenant-deletion bug they surfaced.

### J21 — tenant isolation is now actually verified

`crates/infrastructure/tests/tenant_isolation_rls_tests.rs` adds four tests that
run under `SET ROLE agrocore_app` rather than the migration superuser, so
row-level security is enforced instead of bypassed. The existing
`tenant_isolation_tests.rs` connects as a superuser and therefore verified the
SQL text, not the isolation.

The tests check that a pinned tenant sees only its own rows — using an unfiltered
`SELECT`, because a query with a `WHERE` clause passes even with no policy at all
— that it cannot write or delete another tenant's row, and that it can still write
its own. `rls_is_actually_active_for_the_app_role` runs first and fails if the app
role is not subject to RLS, so the other three cannot pass vacuously.

### A tenant could not be deleted

Writing those tests surfaced a bug that made `DELETE /api/v1/system/tenant` fail
on any tenant that had been used:

1. Three foreign keys on `tenants` (`audit_logs`, `harvest_seasons`,
   `lpis_reference_parcels`) were declared without `ON DELETE CASCADE`, while
   every other tenant-scoped table has one. An audit row is written by ordinary
   use, so a tenant in service for a day could not be deleted — and this is the
   GDPR erasure path, failing exactly when it is needed.
2. Adding the cascades turned that into a different failure: 41 tables are
   cascade children of `tenants` and carry `audit_trigger_function`. Deleting the
   tenant deleted those rows, each delete fired its trigger, and each trigger
   tried to write an `audit_logs` row for a tenant that no longer exists —
   failing against the very foreign key just added.

Migration `0000000008` fixes both and suppresses audit writes during a tenant
delete via a transaction-scoped session flag set by a `BEFORE DELETE` trigger on
`tenants`. Nothing is lost: `audit_logs` has a foreign key to `tenants`, so an
audit row describing the deletion could not outlive the tenant anyway.

Verified against PostgreSQL 17 on a fresh database: without the migration the
delete raises `audit_logs_tenant_id_fkey`; with it the tenant and its data are
removed, the cascade fires, and a neighbouring tenant is untouched.

### J20 — the API contract test can now fail

`crates/api/tests/route_inventory_tests.rs` builds the real Actix application and
probes each candidate path against the running router, so there is no second
copy of the route list to drift. The UI path list is read from the Admin UI
sources for the same reason.

The test this replaces compared a hand-written list of 80 paths against a
hand-written UI list while the handlers registered 187. Nothing compared them, so
renaming or removing a handler left the test green — which is how J1 and J2
unregistered a handler without the contract test noticing.

Confirmed by a negative check: renaming `/livestock` to `/livestock-renamed` makes
the test fail on exactly the affected paths.

### G2 — 32 endpoints have no UI caller

The new test found 32 registered, readable endpoints that the Admin UI never
calls: 12 `calculate/*`, 5 `specialized/*`, 4 `harvest/*`, 3 `water/*`,
3 `weather/*`, 2 workforce paths and 3 others. The backend was built ahead of the
UI and nothing made that visible before.

Recorded as G2 in `docs/tasks.md`. The assertion is marked `#[ignore]` with the
task id so the suite stays green and the failure is attributed to the work that
owns it; the list of paths is in the task.

### UI path corrections found along the way

The Admin UI called four paths the API does not serve:

- `/api/v1/workers` → `/api/v1/workforce/workers`
- `/api/v1/parcels` → `/api/v1/sigpac/parcels`
- `/api/v1/devices` → `/api/v1/iot/devices`
- `/api/v1/gdd/accumulated` → `/api/v1/weather/gdd/accumulated`

`/backups` was registered twice in the Leptos router.

`livestock.rs` and `livestock_new.rs` both opened `web::scope("/livestock")`.
Actix resolves two scopes with the same prefix to the first one only, so one of
the two modules was unreachable while looking correctly wired in `mod.rs`. The
herd-record module now mounts at `/livestock/herds`; `/livestock/animals` is
unchanged.

## [0.37.0] - 2026-10-03

I4 — password hashing no longer occupies the async runtime — and I3 — the
SIGPAC near-point search finally uses an index.

### Argon2 off the async worker (I4)

Argon2 is deliberately slow; that is what makes a stolen hash expensive to attack.
The default parameters cost 50–100 ms of pure CPU. Running that inside an `async`
task holds a Tokio worker for the whole duration, and the runtime has a fixed pool
of workers — so every other request landing on the same worker waits.

That makes authentication a denial-of-service primitive: a handful of concurrent
login attempts occupies every worker without needing many connections, and while
they are being hashed the process serves nothing else.

**Four call sites, not one.** Hashing appeared in `PgUserRepo::create`,
`handlers/system.rs` and `handlers/demo.rs`; verification in the login path. All
four now go through a new `agrocore_infrastructure::password` module, whose
`hash_password` and `verify_password` move the work to the blocking pool.

The module exists so the wrapper cannot be forgotten. A call site that hashes
inline looks correct in review — `hash_password` reads fine until you notice the
missing `spawn_blocking`, and nothing about the code says the difference matters.

The stored hash is parsed before the blocking hop, so a malformed hash is
reported without occupying a thread at all.

Five unit tests, including one that asserts what the fix is actually for: two
concurrent hashes on a *single-threaded* runtime must finish in roughly the time
of one rather than the sum. Run inline, the second call cannot start until the
first finishes, so the test fails. That is the property, not a proxy for it.

### SIGPAC near-point search uses an index (I3)

`handlers/sigpac.rs` filters with `ST_DWithin(geography(geometry), geography(<point>), <radius>)`,
but the only spatial index on `sigpac_parcels` was `USING GIST (geometry)`.

`ST_DWithin(geography, geography, distance)` has no `geometry` overload — the
distance argument is in metres, which only makes sense for the geodetic type.
Wrapping the column in `geography(...)` is a function call, and the planner cannot
match a function call against an index on the raw column. So the query
sequentially scanned every parcel the tenant owns, then filtered, on every
near-point request.

Measured on this schema with 500 parcels:

| | plan | time |
|---|---|---|
| before | `Seq Scan on sigpac_parcels` | 50.0 ms |
| after | `Index Scan using idx_sigpac_parcels_geog` | 0.33 ms |

The ratio grows with the table: a sequential scan reads every row, an index scan
reads only the matching bounding boxes.

Both indexes are kept. `geometry` remains correct for the predicates that stay in
that type — exact `ST_Intersects`, `ST_Contains` — and dropping it would regress
those. The same mismatch was present on `lpis_reference_parcels`, so the index was
added there too rather than leaving the next person to rediscover it.

fmt, check, clippy -D warnings, the full workspace test run and the ignored
database tests all pass.

## [0.36.0] - 2026-10-03

Closes the two remaining P0 tenant-boundary breaks (B2, B3). Both had the same
root cause, and it was not the two handlers the audit named.

### `is_admin()` was treated as a superadmin flag

`is_admin()` is a role *within* a tenant. Six places in `handlers/iot.rs` used it
as if it granted cross-tenant access, and every guard of the shape

```rust
if device.tenant_id != auth.0.tenant_id && !auth.is_admin() {
    return Err(forbidden(...));
}
```

was constant-false: the handler had already called `require_admin()` or
`require_any_role(vec!["admin", "manager"])`, so `is_admin()` was true for every
caller that reached the check, making `!auth.is_admin()` false and the whole
condition false. The guard could never reject anything.

**B3 — `delete_device` deleted across tenants.** The check was dead and the
`DELETE` behind it had no tenant condition of its own, so any tenant admin could
delete any device on the server. The tenant predicate now lives in the statement:

```sql
DELETE FROM iot_devices WHERE device_id = $1 AND tenant_id = $2
```

A handler-body check does not protect the row; anything that reaches the DELETE
without passing the check deletes across tenants.

**Two more holes closed by the same reasoning,** both in the same file and both
found while removing the pattern:

- `create_device` accepted a `tenant_id` from the request, so a tenant admin
  could register a device against another tenant. The row was then invisible to
  that tenant and unreachable through the tenant-scoped queries.
- `list_devices` honoured a `tenant_id` query parameter for admins, so any tenant
  admin could read another tenant's device registry. The parameter is now
  rejected outright.

`is_admin()` no longer appears anywhere in `handlers/iot.rs`. Cross-tenant
visibility needs a separate, globally granted flag — that is a distinct piece of
work, not something a tenant role can stand in for.

### B2 — restore could target a database named by the client

`RestoreRequest.target_database` came unchecked from the request and was passed
straight through `BackupService::restore` to `pg_restore`. The invocation runs
with `--clean`, which drops the objects in the destination before writing: a
caller-chosen target made a restore into a data-destruction primitive against any
database on the server.

- The field is removed from `RestoreRequest`, so it is absent from the OpenAPI
  schema and cannot be sent at all. Leaving it in the DTO would keep advertising a
  capability that no longer exists.
- `BackupService::restore` and `PgDump::restore_from_storage` no longer take a
  database name; the destination is the configured `DATABASE_URL`.
- Backup verification restores into a throwaway database it created itself. That
  is the one legitimate case, and it is now the separately named
  `PgDump::restore_into_named_database`, so the narrow purpose is visible at the
  call site instead of hiding in a general-purpose parameter.
- The handler now rejects a `backup_id` in the body that disagrees with the path
  segment, rather than ignoring it. Silently preferring one of two
  contradicting identifiers is how a client ends up believing it restored A while
  the server restored B.

### Tests

6 tests in `crates/api/tests/tenant_boundary_tests.rs`. They assert on source
text rather than runtime behaviour, which deserves an explanation: both bugs are
about *which code path exists*, not a computed value. A behavioural test would
need a request naming a second real database, and asserting that the second
database was untouched proves only that the fixture was wired correctly — the
same class of test that let the bug through, since the old code had no path a
test could drive without a live second database.

What source assertions cannot catch: a future author reintroducing the same
capability under different wording. That is what code review and the DB-level
tenant-isolation tests are for.


### CI: migration 4 aborted on every run

The CI job connects as `test`, but migration `0000000004_force_rls.sql` granted
`agrocore_app` and `agrocore_auth` to a hardcoded `agrocore` role, so the
migration aborted with `role "agrocore" does not exist` before creating a single
policy. The local docker setup uses `agrocore`, which is why it passed here and
failed only in CI.

Both grants now go to `current_user`, which is the role the migration executes as
and the role that later issues `SET ROLE agrocore_app`. Verified by running all
seven migrations against a fresh database as a separate `test` role: 71 tables,
63 with `FORCE ROW LEVEL SECURITY`, and both application roles granted to `test`.

With that fixed, the test fixture no longer needs its workaround of pre-creating
an `agrocore` group role just to get migration 4 past the GRANT — the workaround
had been hiding this from every local run.

fmt, check, clippy -D warnings, the full workspace test run and the ignored
database tests all pass.

## [0.35.0] - 2026-10-03

Documentation brought back in line with the code, and the demo data made
locale-neutral English.

### Documentation is now English and current

The documentation had drifted: `docs/API.md`, `docs/FLYER.md` and
`docs/TECHNICAL.md` were all stamped `0.8.3` and the README badge `0.15.0`,
while the project was at `0.34.0`. Five files were still written in German.

- `docs/tasks.md`, `docs/TECHNICAL.md`, `docs/optimizations.md`,
  `GITHUB_SETUP_GUIDE.md` and `docs/FLYER.md` translated to English. Task IDs,
  checkbox states, paths, SQL and route references verified byte-identical.
- **`docs/API.md` documented a settings API that no longer exists.** It described
  `GET/PUT /api/v1/settings/lpis` reading from `config/lpis-providers.toml` with
  `https://sigpac.example.com/wfs` as the example URL — the endpoint was removed
  in 0.34.0. Replaced with the actual surface: the key/value API, the five
  settings groups and the database-backed LPIS provider list.
- **`docs/TECHNICAL.md` listed 18 migration files that do not exist.** The real
  set is 7 numbered files; the documented names were invented. Replaced with the
  actual filenames and descriptions, plus the two rules the layout depends on:
  `sqlx::migrate!` rejects unnumbered files (which is why the demo seed lives in
  `scripts/`), and demo data is not a migration.
- The endpoint summary in `docs/API.md` claimed "140+" and summed a table of
  estimates. Now derived from the `#[utoipa::path]` annotations in
  `crates/api/src/openapi.rs` — the same source the served Swagger document uses —
  giving 162 annotated endpoints.
- The handler-module count in `docs/TECHNICAL.md` was 27; there are 33.
- `docs/FLYER.md` Recent Highlights now describe what the project actually does:
  server-side settings, a working backup lifecycle, enforced tenant isolation.
- README database row corrected: 7 migrations and 71 tables, not "15 migrations
  (34 tables)".

### Keeping documentation current

The reason the docs drifted is that nothing required updating them. That is now
written down in `CONTRIBUTING.md`, with a table mapping each kind of change to
the file it must update, and two rules that would have caught this:

- Documentation that describes current state is updated **in the same change that
  alters it**, not batched for a release.
- Counts in the docs are derived from the code, never estimated. An estimated
  endpoint total is wrong within two versions, which is exactly what happened.

### Demo data is locale-neutral English

`scripts/demo_seed.sql` and `crates/api/src/handlers/demo.rs` carried German
names, terms and locale data. The demo is read by people who do not read German,
so a German site label is as opaque as a wrong one.

- Names: Hans Müller → Hans Miller; Nordfeld → North Field; Südhang → South
  Slope; Westweide → West Pasture; Haupthalle → Main Barn.
- Terms: Bodenarten, Rindergülle, Aussaat, Dünger, Wirtschaftsbegriffe
  translated. The cattle breed Fleckvieh became Holstein Friesian and the wheat
  variety Akteur became Cadenza, because a German cultivar name is not
  recognisable to an English reader either.
- **Locale data removed, not just translated:** German IBANs (`DE…`) became
  `GB…`, `+49` phone numbers became `+1`, German addresses became generic ones,
  and the supplier names (Raiffeisen, BayWa) became neutral companies on
  `.example` domains. The demo tenant's language moved from `de` to `en`.
  Machine brands (Fendt, Claas, Amazone) stay — they are real international
  manufacturers named in English farming documents too.

### Demo seed is idempotent for content

`ON CONFLICT (id) DO UPDATE SET updated_at = NOW()` made the seed re-runnable
but only for the timestamp: a corrected demo value survived every subsequent
seed, which is why a stale `MaisTer` product name persisted after the file had
been fixed. The conflict clauses now refresh the descriptive columns per table.

Found while doing it: `inventory_locations` has a `name` column, not `label`, so
a blanket conflict clause failed with `column excluded.label does not exist`.
Verified by running the seed twice — no duplicate rows.

Seed runs clean against a freshly migrated database (71 tables, all 7
migrations). fmt, check, clippy -D warnings, the full workspace test run and the
ignored database tests all pass.

## [0.34.0] - 2026-10-02

Settings API for every resource (F3), and LPIS configuration read from the
database instead of a TOML file (F4, F6, J10).

### Group endpoints (F3)

- New: `GET /api/v1/settings/groups` plus `GET/PUT
  /api/v1/settings/{backup,notification,weather,locale,company}`. Previously a
  client had to know every key name and every type; now it reads
  `GET /api/v1/settings/backup` and gets the group's fields.
- Each group declares its fields with their type. An unknown field is
  rejected instead of landing under a key nothing reads back.
- A wrong type is rejected at the edge. A string under a numeric field would be
  accepted by the database and then silently ignored by every reader using
  `as_u64()`.
- All groups use the same `system_settings` rows as the key/value API. A value
  written through a group is visible through the key/value API and vice versa.
- `null` resets the tenant override to the system default.
- Sensitive values are not returned through a group: the group does not know
  what its value means and cannot redact it selectively.
- The response to a write is the stored state, so a corrected or partially
  rejected value becomes visible.

### LPIS from the database (F4, F6, J10)

- `find_config_file()` walked up from the working directory looking for
  `config/lpis-providers.toml`. In a container, where the working directory is
  `/`, that file does not exist; and it was not tenant-scoped, so every tenant
  shared one provider list.
- New: `crates/api/src/lpis_settings.rs` reads from `system_settings` under
  `lpis.providers.<COUNTRY>.`, with the real endpoints as the fallback.
- `list_lpis_providers` returned `https://{country}.example.com/wfs`. Those
  domains were invented and looked like configuration; now the configured or
  built-in endpoint is reported, and a country without one as disabled.
- `LpisProviderConfig.configured` tells the client whether an entry is
  configured or a built-in default — indistinguishable otherwise without
  comparing domains.
- Credentials do not belong in `system_settings`: a settings row is readable by
  every admin of the tenant.
- The debug `eprintln!` calls are gone with the file path; errors are logged
  structurally.
- 12 LPIS defaults in migration 5, including the real URLs.

### Route ordering

- Actix matches in registration order, and `/{key}` matches any single
  segment. Without ordering discipline `/backup` was read as the setting
  "backup" and `/lpis/providers` as the key "lpis". The groups now sit in the
  same scope ahead of `/{key}`; a second scope with the same prefix would have
  silently swallowed the key/value routes.
- The route test now checks all 16 settings routes individually instead of
  repeating the same URI three times, and it caught the swallowing.

### Toolchain

- The CachyOS `rustc` package no longer starts: `libLLVM.so.23.1` exports
  `_M_mutate` for `wchar_t` but not for `char`, and binds the symbols to an
  `LLVM_23.1` version node the library does not carry. A GCC 16 update made
  `libstdc++` incompatible in that direction. A self-contained toolchain via
  rustup into `~/.rustup` works again.

### Tests

- 8 tests in `crates/api/tests/settings_groups_tests.rs`: shared storage, type
  rejection without an override, unknown field, reset to default, tenant
  isolation of the company profile, real LPIS endpoints, targeted provider
  override, declared types of the defaults.

fmt, check, clippy -D warnings, the full workspace test run and the ignored
database tests all pass.
## [0.33.0] - 2026-10-02

Closes the last two places where the backup API faked success (J7, H3), and makes
`list_backups` real. Together with 0.32.0 no backup function is a dummy any
more.

### Deleting actually deletes (J7)

- `delete_backup` removed nothing. It only checked `get_job_status` and replied
  `200 {"success": true}`; anyone deleting a backup to free space kept paying
  for it, and the retention sweep later found the object again.
- New service method `delete_backup_objects` resolves which objects belong to a
  backup through the manifest. A full backup writes a dump, a checksum and a
  manifest — deleting only the dump would have left the rest behind, and the
  listing would still have shown the backup.
- Deletion covers every configured target. Deleting a replicated backup from
  one target would leave a restorable copy behind.
- The manifest is deleted last, so a partial deletion stays describable: the
  backup's inventory survives until the objects themselves are gone.
- A failing target is collected rather than aborting. A non-empty
  `failed_targets` makes the handler answer with an error instead of success —
  reporting a partial deletion as success is the old deception in a different
  place.
- The in-memory job is discarded. Without that, `GET /backup/backups/{id}`
  keeps answering for a backup that no longer exists.
- Refused while a backup is running: otherwise a job keeps writing to objects
  that are being removed.
- `manifest_object_name` is now shared between writer and deleter. If the two
  disagreed, the manifest would survive.

### Listing reads from storage

- `list_backups` returned `vec![]`. It now reads storage manifests and dump
  objects, so the listing survives a restart — the in-memory job state does
  not.
- Without a manifest the ID is derived from the object name instead of being
  invented with `Uuid::new_v4()`. An invented ID had the UI offer a delete
  button that resolved to nothing.
- The same backup on several targets is one backup, not three. Otherwise a
  replicated backup would appear repeatedly in the listing and could be
  "successfully" deleted repeatedly.
- An unreachable target no longer empties the listing; it is logged.
- `manifest_backed` in the response: without a manifest the ID and type are
  derived, and the client has to be able to tell.

### Backup page (H3)

- New route `/backups` with a navigation entry, visible to admins.
- Configuration: both schedules, timezone, four retention tiers, verification,
  on/off.
- Saving renders the configuration the server returned, not the form. The
  handler answers a write with the stored state, so a rejected or corrected
  value becomes visible here.
- Manual backups: database, configuration, full.
- List with ID, type, status, start, size, target count and actions.
- Restore always runs `dry_run` first and then asks, because restore is not
  reversible and overwrites the current database.
- Deleting reports the number of objects and the space freed.
- Without a manifest the row is marked as such, because the ID is then not
  reliably resolvable.
- New API wrappers: configuration, list, start, detail, status, restore,
  delete.

### Tests

- 5 tests in `crates/backup-service/tests/delete_backup_tests.rs` against a
  real local storage backend, because what has to be proven is that files
  leave the disk: file deleted, unknown backup fails, all targets deleted,
  dump plus checksum plus manifest deleted, unrelated backups survive.

fmt, check, clippy -D warnings, the full workspace test run and the 45 ignored
database tests all pass.
## [0.32.0] - 2026-10-02

Implements F2 on top of F1: the backup configuration is now stored in
`system_settings` instead of being hardcoded. Also closes the gap that became
visible while doing it — the configuration endpoint could read retention and
verification but not change them.

### Backup configuration persisted (F2)

- `get_backup_config` reads nine keys from the `backup.` namespace and falls
  back to the shipped defaults for values never written, instead of hardcoding
  `enabled: true` and `"0 2 * * *"`.
- `update_backup_config` now writes. It previously replied `200
  {"message": "Backup configuration updated"}` without doing anything — an
  admin turning backups off was told the opposite of the truth while the
  schedule kept running.
- The response is the stored configuration, not an acknowledgement. A client
  can therefore tell whether the values were accepted.
- `UpdateBackupConfigRequest` extended with
  `retention_daily/weekly/monthly/yearly` and `verification_enabled`. These
  fields were on the response and could not be changed.
- Partial update is preserved: the write list is built from the fields sent,
  so `{"enabled": false}` does not clear the schedule.
- Validation on schedule length, timezone and retention ranges.
- `targets_count` stays honestly at 0: the backup service owns the targets and
  `system_settings` does not know them. Returning a number that does not match
  registered targets would be the old deception.

### Settings defaults

- The four backup keys `backup.schedule`, `backup.retention_days`,
  `backup.targets` and `backup.verify` are replaced by nine that say what they
  are. `retention_days` was a flat window; the API needs the four retention
  tiers separately.

### Tests

- 5 tests in `crates/api/tests/backup_config_tests.rs`: persistence, the
  `is_default` flip when overriding, partial update, type rejection for
  retention, tenant isolation including reset.
## [0.31.0] - 2026-10-02
Server-side settings (tasks.md F1/H1) plus the cleanup of the schema
deviations that surfaced along the way. All 40 previously skipped
database tests now pass against a fresh database with all migrations
from zero and the demo seed loaded.

### Settings

- New table `system_settings` (migration `0000000005`): typed
  key/value settings, `tenant_id IS NULL` are system defaults that
  every tenant inherits. Uniqueness via `COALESCE(tenant_id, …)`,
  because a plain `UNIQUE` treats NULLs as distinct and would allow
  duplicate defaults.
- 20 shipped defaults, so a fresh installation has a complete
  settings page.
- `SettingsRepository` with `list_effective`, `get`, `set`, `set_many`,
  `reset`, `set_default`, `restore_defaults` and `list_keys`.
- Endpoints `GET/PUT/DELETE /api/v1/settings`, `GET/PUT/DELETE
  /api/v1/settings/{key}` and `POST /api/v1/settings/restore-defaults`, all
  admin-only. A `null` value removes the override instead of storing
  JSON null.
- New module `settings_editor.rs`: the widget follows
  `value_type` instead of one hand-written field per key. A key added
  in the database shows up with a matching editor and no UI change.
  The existing `settings.rs` page only wrote to the browser's
  localStorage and was therefore per device.
- 6 tests in `tests/settings_tests.rs`: tenant isolation, defaults,
  save/reset, type validation, unknown keys.

### Fixed bugs

- **Sites could not be created.** The INSERT named 36 columns but
  supplied only 33 expressions; the placeholders jumped from `$25` to
  `$28`.
- **`sites.center` and `sites.boundary` are `GEOMETRY` but were bound as
  JSONB** — every write failed with `column "center" is of
  type geometry but expression is of type jsonb`. Writes now go through
  `ST_GeomFromGeoJSON`, reads through `ST_AsGeoJSON`.
- **`GeoPoint` does not serialize as GeoJSON.** `{"lng":…,"lat":…}` is
  rejected by PostGIS with `unknown GeoJSON type`; new helpers
  `geo_point_to_geojson` and `boundary_to_geojson` convert explicitly and
  close rings.
- **`sites.lpis_country` is `varchar` but was treated as JSONB**
  (`COALESCE types jsonb and character varying cannot be matched`).
- **Soft-delete was not idempotent.** A second `delete` reported
  success again; `AND is_active` turns the repeat case into `false`.
- **`worker_repo().create()` and the worker task status were dead.**
  `workers` had been created as an HR table (`employee_id`, `firstname`,
  `social_security_number`) and had none of the columns the entity
  declares; `worker_task_statuses` was missing `paused_at`, `resumed_at`,
  `stopped_at`, `done_at` and `updated_at`. Migration
  `0000000006_workforce_schema_alignment.sql` adds both, without
  discarding the existing payroll data, and renames names from `users`.
- **`worker_task_statuses.status` was `TEXT`, the entity expects JSONB** —
  sqlx rejected the decode with `mismatched types`.
- **A foreign key on `worker_locations.worker_id` rejected valid
  rows**, because the workforce module passes a user ID while the FK
  pointed at `workers(id)`.
- **`worker_locations.location` was nullable**, which allowed rows
  without a position that the GPS view shows as a null island.

### Test infrastructure

- The testcontainer fixture creates the `agrocore` role that migration 4
  expects, otherwise the migration aborts earlier.
- Fixture helpers for real foreign-key rows (user, task, unique
  tenant slugs) instead of random UUIDs.
- `rls_tests` and `tenant_pin_tests` generate their tenant IDs and slugs
  uniquely per run, otherwise the second run fails against `tenants_pkey`
  and `tenants_slug_key`. `ON CONFLICT DO NOTHING` is no help under
  FORCE RLS: it triggers a policy check that a plain INSERT does not.
- Two test expectations corrected that contradicted the project:
  `spatial_properties` exists in no migration, and `delete` is
  project-wide a soft-delete (12 repos).
## [0.30.0] - 2026-10-02
Integrates the tenant pin into all database paths and fixes the bugs that
surfaced as a result. Verified against a fresh database with all
migrations from zero and the demo seed loaded.

### Tenant pin

- `TenantPool` pins `app.current_tenant_id` on the same connection that
  runs the query and sets `app.is_superadmin` to `false`.
- Implements the sqlx `Executor` so that 317 call sites in 47 repos stay
  unchanged and the pin cannot be forgotten.
- `begin()` sends an explicit `BEGIN` first, because `SET LOCAL` is a
  no-op outside a transaction block. The other order looks like it works
  and resets the pin on the first statement.
- `unscoped()` uses the nil UUID and thereby denies everything: fail-closed
  for bootstrap work.
- 7 tests in `tests/tenant_pin_tests.rs`, including
  `checkout_does_not_inherit_previous_tenant`.

### Fixed bugs

- **Login was completely broken.** The RLS role lacked grants for 10 tables
  created after the RLS migration; `user_sites` is needed by the login
  join.
- **Auth needs a tightly scoped exception**, because the tenant is only
  read from the user row. Role `agrocore_auth` with SELECT on `users` and
  `user_sites`, policy only for this role.
- **Refresh-token write had no effect.** Unpinned, the UPDATE hit zero
  rows, because `users_update` requires the pin.
- **`#[sqlx(json)]` on `Option<T>` was wrong.** sqlx distinguishes `json`
  (not null) and `json(nullable)`; 29 fields in 10 files were annotated
  incorrectly. Cause of `unexpected null; try decoding as an Option`.
- **Schema drift on `orders`:** `order_type` was VARCHAR instead of JSONB,
  `planned_date`/`deadline_date` were DATE against DateTime, `started_at`
  and `completed_at` were missing from the table. `GET /api/v1/orders` was
  unreachable.
- **37 NUMERIC columns against `f64`.** Every affected entity failed on
  decode, `GET /api/v1/customers` on `vat_rate`. Normalized iteratively
  to DOUBLE PRECISION.
- **The demo seed was substantively wrong:** `seeding` and `fertilizing`
  do not exist as `OrderType`; `{"mode":"Manual"}` wrote PascalCase
  instead of snake_case.
- **Missing NATS aborted the entire API startup**, even though the
  publishers ignore errors anyway. Messaging is now optional,
  `MESSAGING_REQUIRED=1` enforces it.

### Verified

```text
POST /api/v1/auth/login       -> 200
GET  /api/v1/health          -> 200
GET  /api/v1/sites           -> 200
GET  /api/v1/users           -> 200
GET  /api/v1/orders          -> 200
GET  /api/v1/orders/my-tasks -> 200
GET  /api/v1/customers       -> 200
GET  /api/v1/inventory/items -> 200
GET  /api/v1/tasks           -> 200
```

RLS active throughout, connection as `agrocore_app`.
## [0.29.0] - 2026-10-01
Makes the existing row-level security actually effective (`tasks.md` A3).

### Security
- **The connection role was a superuser with BYPASSRLS — RLS was pure decoration** — migration `0000000004_force_rls.sql`. Measured on this installation: `agrocore` has `rolsuper = true` **and** `rolbypassrls = true`. PostgreSQL exempts such roles unconditionally; `FORCE ROW LEVEL SECURITY` changes nothing about that. The 190 policies in the schema were never evaluated.

  Migration `0000000004_force_rls.sql` introduces the role `agrocore_app` (`NOSUPERUSER NOBYPASSRLS NOLOGIN`), grants it access to all RLS tables and sequences, and applies `FORCE ROW LEVEL SECURITY` to all 62 affected tables. The pool switches over in `PgPoolOptions::after_connect` with `SET ROLE agrocore_app` — per physical connection, because it is session state.

  Gated by `AGROCORE_RLS_ENABLED=1`: enabling the policies before the pin would make all repos return zero rows, because `get_current_tenant_id()` returns NULL without `app.current_tenant_id` being set. The switch is therefore a configuration change and not a coordinated release.

- **Two gaps that only became visible once RLS actually took effect:**
  - `sigpac_parcels` had `ENABLE ROW LEVEL SECURITY` but **no policy**. With `FORCE` that would deny every row to everyone, including the owner. Policy added.
  - `tenants` had policies for SELECT, UPDATE and DELETE, but **none for INSERT**. With `FORCE`, `POST /api/v1/system/setup` — the endpoint that creates the first tenant — would have failed on every installation. Added `tenants_insert ... WITH CHECK (true)`: tenant creation is a setup action that happens before a tenant exists, and must therefore not be tenant-filtered.
  - Six tables with a `tenant_id` column had no RLS at all; added for all except `tenants`.

### Added
- **Five RLS tests** (`crates/infrastructure/tests/rls_tests.rs`) against a real database:

```bash
DATABASE_URL=postgresql://agrocore:***@localhost:5432/agrocore \
AGROCORE_RLS_ENABLED=1 \
  cargo test -p agrocore-infrastructure --test rls_tests -- --ignored
```

  What is checked: the application role is neither superuser nor BYPASSRLS; without the pin **no** row is visible (fail-closed, no leaking); with the pin each tenant sees only its own; an unknown UUID or an invalid value yields nothing; `FORCE` is set and every FORCE table also has a policy, because a policy-less table under FORCE would deny every row.

  The test without a pin is the actual proof: a repository that forgets its tenant filter then returns **nothing** instead of the neighbour's data.

### Verified
- The role `agrocore` is a superuser with BYPASSRLS; precisely for that reason the policies could never take hold.
- As `agrocore_app`: a pin to tenant A returns only A, a pin to tenant B only B, unknown UUID and non-UUID each yield 0 rows.
- 62 tables with `relforcerowsecurity` after the migration.
- Migration `0000000004_force_rls.sql` is idempotent and runs through in a fresh database after all previous migrations.
- All gates green, 268 workspace tests.
## [0.28.0] - 2026-10-01
Fixes the refresh token mechanics (D1) and the tenant break on `kelter_deliveries` (B1).

### Security
- **Tenant isolation on `kelter_deliveries` was defeated** — `postgres/kelter_delivery.rs`. All seven methods took `tid: TenantId` and used it in **not one** single query. `find_all` returned globally across all tenants, `delete` deleted arbitrary records.

  **Correction to the audit:** the table already existed in `0000000000`, it just had no `tenant_id` column; the RLS policies of the init schema scope over the associated vineyard (`vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = …)`), which the repository bypasses. Migration `0000000003` therefore adds the column via `ALTER TABLE` instead of recreating the table. Existing rows are backfilled from the vineyard; a row without a vineyard tenant aborts the migration with a clear message instead of being silently assigned to an arbitrary tenant.

  All seven queries now filter by `tenant_id`, `KelterDelivery` has the field in the domain model. The bind order in `find_by_vineyard` was wrong while at it — the placeholders `$1`/`$2` expect the vineyard first, then the tenant.

### Fixed
- **The refresh token mechanics were dead** — `postgres/user.rs:402-420`. All three methods were stubs: `find_by_refresh_token` returned `Ok(None)` (with the comment *Simplified - not fully implemented*), `update_refresh_token` and `invalidate_refresh_token` `Ok(false)` each. The failure stayed silent because `auth.rs:60` only checked `map_err` and an `Ok(false)` is not an error: login answered 200 and handed the client a refresh token that was never in the database, and `/auth/refresh` accordingly always returned 401.

  `find_by_refresh_token` now filters expiry and `is_active` in SQL, so that an expired token is indistinguishable from an unknown one. `invalidate_refresh_token` sets both columns to NULL and reports via `WHERE refresh_token IS NOT NULL` whether anything was actually revoked. Login and refresh now check the return value explicitly and return a 500 instead of handing out an unusable token. The refresh rotates the token, which makes reuse detectable.

- **One test was flaky rather than wrong** — `crates/api/tests/demo_endpoint_auth_tests.rs`. `rejected_values_keep_demo_endpoints_disabled` failed because `cargo` starts tests in parallel in threads and `std::env` is process-wide: the test read the value another test had just set. The tests in the file now take a `Mutex` around the environment access. The production code was correct — the test was only accidentally green.

### Added
- **Four tenant isolation tests** (`crates/infrastructure/tests/tenant_isolation_tests.rs`) — `find_all` returns only own rows, `find_by_id` with the correct UUID of a foreign tenant returns nothing, a cross-tenant `DELETE` affects 0 rows, `find_by_vineyard` stays scoped. Two tenants each with their own vineyard and delivery, cleanup after every test.

```bash
DATABASE_URL=postgresql://agrocore:***@localhost:5432/agrocore \
  cargo test -p agrocore-infrastructure --test tenant_isolation_tests -- --ignored
```

### Tests
- 268 tests in the workspace, 0 failures.
- The four isolation tests run green against a PostgreSQL instance with migrations applied.
## [0.27.0] - 2026-10-01
Closes three security holes (`tasks.md` A4, A5) and makes ten previously
unreachable routes exist (J1, J2).

### Security
- **The server started with a publicly known signing key** — `shared/config.rs`, `api/src/lib.rs`. `validate_jwt_secret()` only compared against the literal `dev-secret` and had **zero callers in the entire workspace**. A deployment without `JWT_SECRET` therefore ran with exactly that key; anyone could forge an admin token with an ordinary HS256 signature and drop themselves into any tenant.

  `run_server` now aborts with an `io::Error` before the server listens. The check no longer returns just `bool`, but `Result<(), JwtSecretError>` with the variants `DevSecret` and `TooShort { length, minimum }`, because the remedy differs. New is also a minimum length of 32 characters — 32 bytes is the width of a SHA-256 digest, the usual lower bound for a symmetric HMAC key. `ALLOW_DEV_SECRET=1` softens only the default-secret check, never the length check.

- **Logout and password change revoked nothing** — `middleware.rs`. `is_revoked()` was defined but had zero calls; only `revoke()` was used. A stolen token, or one intercepted via XSS, stayed valid until expiry. `AuthExtractor` now checks the `jti` after a successful `decode` against `AppState.token_revocation` and rejects with 401 *Token revoked*.

  That required switching the `FromRequest` future from `Ready` to a `Pin<Box<dyn Future>>`, because the revocation list is asynchronous. `from_request` clones headers and the state handle so that the future borrows nothing.

### Fixed
- **`sigpac` and `livestock` were declared as handlers but never registered** — `handlers/mod.rs`. Both had a finished `configure()` and were documented in the OpenAPI specification, but were never called: `/api/v1/sigpac/parcels` (3 routes) and `/api/v1/livestock/animals` (7 routes) did not exist. Additionally `livestock_new::configure` was registered twice; the duplicate entry is removed. The two livestock modules share the prefix `/livestock` with different subpaths, there is no collision.
- **The entire site import was dead code** — `handlers/sites.rs`. `import_sites`, `import_geojson` and `import_shapefile` were fully implemented and use `ImportService` with LPIS registry and geozero shapefile parsing, but had no route. Registered as `POST /sites/import`, `/sites/import/geojson`, `/sites/import/shapefile` — deliberately before `/sites/{id}`, otherwise the id pattern would have tried to parse the path *import* as a UUID. That makes 719 lines of service code and the matching admin UI wrappers reachable for the first time.

### Changed
- **`AuthExtractor::from_request` is no longer synchronous** — the future type is now `Pin<Box<dyn Future<Output = Result<Self, Error>>>>`. Callers that previously pulled the value out of a `Ready` via `.into_inner()` now have to poll the future. Affects the four tests in `middleware.rs`, which got an `extract()` helper.

### Tests
- 268 tests in the workspace, 0 failures.
- 5 new tests for the JWT secret rules in `crates/api/tests/jwt_secret_tests.rs`.
## [0.26.0] - 2026-10-01
Fixes the unauthenticated demo endpoints (`tasks.md` A2) and unifies the
demo password (G2).

### Fixed
- **The demo endpoints were completely unauthenticated** — `handlers/demo.rs`. `POST /api/v1/demo/seed`, `POST /api/v1/demo/reset` and `GET /api/v1/demo/summary` took **no** `AuthExtractor`. `reset` enforces `reset = true` and then runs `DELETE FROM tenants WHERE id = $1` with cascade, where the tenant slug comes from the request body — any unauthenticated client could thus delete an arbitrary tenant along with all its data. `/summary` was open too and exposed tenant and record counts.

  Two independent barriers now: `require_demo_access()` checks `AppState.demo_endpoints_enabled` **and** `auth.require_admin()`. The flag comes from `ALLOW_DEMO_ENDPOINTS` and is **off** by default, a wrong value fails closed (only `1`, `true`, `yes` are recognized, case-insensitive and trimmed). A value that is set but unrecognized is logged.

- **Hardcoded demo admin password removed** — `handlers/demo.rs` hashed `b"demo123"` in plaintext. The password now comes from `DEMO_ADMIN_PASSWORD`; if the variable is missing, a default applies and it is logged.

- **The three demo sources contradicted each other on the password** — `scripts/demo_seed.sql` contained a hash for `demo1234`, `handlers/demo.rs` hashed `demo123`, `scripts/dev.sh` displayed `demo1234`. Anyone going through the API seed could not log in with the displayed password. One source now defines the password: `DEMO_ADMIN_PASSWORD`, default `demo1234-agrocore`. The Argon2id hash in the SQL seed was regenerated and empirically checked against the real verifier — it accepts `demo1234-agrocore` and rejects `demo1234` as well as `demo123`. With 17 characters the default meets the minimum of 12 in force since v0.25.0.

### Changed
- **`scripts/dev.sh`** exports `ALLOW_DEMO_ENDPOINTS` and `DEMO_ADMIN_PASSWORD`, writes both to `.env.dev` and displays the variable in the final output instead of a literal. The API seed call now logs in before seeding and passes the JWT — the demo endpoints require it. The first seed still falls back to SQL, because the demo admin does not exist at that point yet.
- **OpenAPI declarations of the demo endpoints** now carry `security(("bearer_auth"))` as well as 401 and 403 responses. Previously none of the three paths was documented as protected.

### Added
- **Six regression tests** (`crates/api/tests/demo_endpoint_auth_tests.rs`) — the flag is off without the variable set, accepts `1`/`true`/`yes` in any spelling and with whitespace, rejects `0`, `false`, `no`, `off`, `2`, `enabled`, the `enabled` typo and the empty value, and the default password meets the minimum length criterion.

### Tests
- 263 tests in the workspace, 0 failures (258 + 5 new; one test of the new file is a mirror of the parsing rule and is not counted twice in both lists).
## [0.25.0] - 2026-10-01
Fixes the privilege escalation from the audit of 2026-10-01 (`tasks.md` A1, D3) —
the most easily exploitable weakness of the project.

### Fixed
- **Privilege escalation: any user could make themselves admin** — `handlers/users.rs:153-157`. The authorization read:

  ```rust
  if let Err(e) = auth.require_admin()
      && auth.0.user_id != user_id
  { return Err(e.into()); }
  ```

  The admin check was skipped as soon as the target account was the own ID. Since `UpdateUserDto` carries a `roles` field and `PgUserRepo::update` binds it unchecked (`postgres/user.rs:318`), `PUT /api/v1/users/{own_id}` with `{"roles":["Admin"]}` was enough — full admin access on the next login. Not visible, because in the negative test form the condition appears to have a justification of its own.

  Now: role changes and changes to `is_active` require admin, independent of the target. A self-service path with a narrowly defined field list replaces the previous possibility of editing one's own account.

- **Password update without a minimum length** — `dto/user.rs`. `UpdateUserDto.password` had no `validate` annotation, while `CreateUserDto` set `min = 8`. Via the escalation an existing password could be set to an empty string. Both update paths now require 12 to 128 characters; `CreateUserDto` stays at 8, so that existing accounts remain valid.

### Added
- **`PUT /api/v1/users/me`** — `handlers/users.rs:update_own_profile` with `UpdateOwnProfileDto`. Accepts exclusively `firstname`, `lastname`, `password`, `language` and `color`. `roles`, `is_active`, `internal_cost_per_hour` and `external_cost_per_hour` do not exist in the DTO; the mapping to the domain DTO explicitly sets all other fields to `None`, so that `PgUserRepo::update` leaves the columns untouched instead of overwriting them. The route is registered deliberately **before** `/users/{id}` — otherwise the `{id}` pattern would have tried to parse the path "me" as a UUID.
- **Nine regression tests** (`crates/api/tests/privilege_escalation_tests.rs`) — DTO level: the absence of privileged fields in the self-service DTO, the password policy on both update paths (empty, one character, 22 characters, not set), and that the existing email validation was not weakened by the change.

### Tests
- 258 tests in the workspace, 0 failures (249 + 9 new).
## [0.24.0] - 2026-10-01

First part of the code audit from 2026-10-01 (blocks I and J). Fixes the most severe
finding: eight tables were queried by fully implemented repositories but
existed in no migration. A security audit was also documented
(`docs/tasks.md`, blocks A–J).

### Fixed
- **Eight tables were missing from the schema, seven repos were dead at runtime** — `spatial_objects`, `groups`, `trees`, `buildings`, `livestock`, `water_usages`, `animal_treatments`, `animal_grazing_records`. The repos were fully implemented, no migration created the tables, every query failed with `relation "..." does not exist`. Added migration `0000000003_missing_domain_tables.sql`: six tables with indices, GIST geometry indices, RLS policies following the existing pattern, and `ALTER TABLE water_usage RENAME TO water_usages`.
- **`spatial_objects` was queried by every GPS ping, and the error was invisible** — `handlers/workforce.rs:353,364` called `spatial_object_repo().find_containing_point(...)` with `.unwrap_or_default()`. Every location ping ran twice into a nonexistent table, the result was always empty: the location-to-field assignment never worked and went unnoticed. The error is now logged with `warn!` (tenant, coordinates, error text); the ping still proceeds, but the error is visible.
- **INSERT statements did not bind `tenant_id`** — `tree.rs`, `group.rs`, `building.rs`, `livestock.rs`. All SELECTs filter with `WHERE tenant_id = $1`, the INSERT omitted the column — a newly created record would no longer have been findable. Not visible because the method signature `tid: TenantId` looked correct. Fixed four queries and bind orders.
- **Wrong table names in `animal.rs`** — it queried `animal_treatments` and `animal_grazing_records`, but `treatment_records` (migration `:676`) and `grazing_records` (`:665`) exist; the repo also wrote `treatment_date` instead of `date`. Switched to the existing tables, no new schema needed.
- **`animals` was missing three columns the repository reads** — `identifier`, `livestock_type` and `status`. `identifier` is backfilled from `tag_number`. For `livestock_type` a default is not enough, because existing inserts (including the demo seed) only set `species`: a BEFORE trigger derives `livestock_type` from `species` on every INSERT and UPDATE. A first attempt with `SET NOT NULL` broke the demo seed (`null value in column "livestock_type"`).

### Fixed
- **The test fixture started an image without PostGIS** — `crates/infrastructure/tests/common/mod.rs` used `testcontainers_modules::postgres`, which is hardwired to `postgres:11-alpine` and contains no PostGIS. Migration `0000000000` creates the `postgis` extension though, so **all nine** integration tests failed with `extension "postgis" is not available` — they could never have run. Switched to `GenericImage::new("postgis/postgis", "16-3.4")` (the module offers no `with_tag()`), and added backoff plus retries to the connect, because Postgres restarts once during the init phase and resets live connections.

### Added
- **Three regression tests** (`crates/infrastructure/tests/database_setup_tests.rs`) — `test_repository_tables_exist` checks every table queried by a repository, `test_tenant_scoped_tables_have_tenant_id` finds tables without tenant scoping, `test_new_domain_rows_are_tenant_scoped` inserts rows into all four new tables and reads them back through the tenant filter. All three fail if the original state returns.
- **Table list in the migration test extended** — the existing `test_database_migrations_applied` already checked `spatial_objects` and would be green after the migration; added `groups`, `trees`, `buildings`, `livestock` and `water_usages`.

### Changed
- **RLS policies added on the new tables** following the existing pattern via `get_current_tenant_id()`. They do not work for the same reason as the other 190: `app.current_tenant_id` is never set and `FORCE ROW LEVEL SECURITY` is missing (see `tasks.md` A3). The policies exist for consistency, not as a guarantee.

### Verified
- All four migrations run through in order in a fresh PostgreSQL instance.
- Migration `0000000003` ran three times in a row on the same database, without duplicates.
- The demo seed runs through after the migration and creates 3 animals, 3 sites and 6 grazing records.
- All repository queries executed against the new schema: `spatial_objects` (find_by_id, count, GPS ping query), `trees`, `groups`, `buildings`, `livestock` (count and INSERT each), `water_usages`, `animals`, `treatment_records`, `grazing_records`.
- The GPS ping delivers data for the first time; `livestock_type` is correctly derived from `species` (`Cattle -> Cattle`).

### Known Limitations
- Four pre-existing integration tests in `database_setup_tests.rs` still fail, independently of this change: `test_database_migrations_applied` expects a table `spatial_properties` that no migration creates; `test_site_crud_operations` fails with `INSERT has more target columns than expressions`; `test_tenant_creation_and_isolation` creates a tenant per test with the fixed slug `test-tenant` and fails on the unique constraint as soon as more than one test uses the same slug; `test_tenant_scoped_tables_have_tenant_id` finds tables without `tenant_id`. `test_updated_at_trigger` is also red after the fixture fix. See `tasks.md` J19 and J4.
- The error in `workforce.rs` is logged, not fixed. A failed geometry query still yields no location assignment — only now it is visible.
- `work_logs` and around 40 further columns are still missing (`tasks.md` J4).
- `varieties` and `breeds` still have no `tenant_id` column, but their repos filter by it — varieties and breeds are global instead of tenant-isolated (`tasks.md` J5).

## [0.23.0] - 2026-10-01

Backup service phase 8 section 4: streaming pipeline, functional retention, real restore, monitoring metrics, as well as the SFTP and WebDAV backends that were previously only configured.

### Added
- **Streaming backup pipeline** — `pg_dump` reads stdout through a 1 MiB buffer instead of `cmd.output()`. For dumps in the double-digit GB range that was an OOM risk, because the entire dump was in memory at the same time.
- **Streaming restore** — `pg_restore` receives the data directly via stdin. `restore_from_storage` no longer downloads the dump in full via `download_bytes()`.
- **`StorageBackendTrait` extended** — new methods `upload_stream`, `download_stream`, `list_objects`, `delete_object`, `load_manifest` and `save_manifest`. Cloud backends use `object_store::put_multipart` for chunked upload.
- **Prometheus metrics** (`crates/backup-service/src/metrics.rs`) — `backup_duration_seconds`, `backup_size_bytes`, `backup_success_total`, `backup_failed_total` and `backup_restore_total`. Linked to success, failure, duration, size and restore result. 4 unit tests.
- **SFTP backend** (`crates/backup-service/src/sftp_backend.rs`) — password and private key authentication via `russh`/`russh-sftp`, streaming upload in 1 MiB chunks, download, recursive listing, deletion and manifest persistence.
- **WebDAV backend** (`crates/backup-service/src/webdav_backend.rs`) — basic auth, `MKCOL` for collections, streaming upload via `PUT` with chunked transfer encoding, `PROPFIND` for recursive listing including XML parsing, `DELETE`, manifest persistence. 11 unit tests for URL construction, path encoding and response parsing.
- **Encryption for both new backends** — `encrypt_payload`/`decrypt_payload` in `encryption.rs` apply the configured target encryption. The streaming upload encrypts chunk by chunk so that memory usage stays bounded.
- **`dry_run` for restore** — API request (`RestoreRequest.dry_run`) and CLI (`agrocore-backup restore <ID> --dry-run`) check whether a backup is restorable without writing to the database.
- **14 new tests** — 8 storage streaming, 7 retention, 4 manifest, 3 real PostgreSQL integration tests.

### Fixed
- **Retention could not delete anything** — `parse_dump_timestamp` used `rsplit_once('_')` to take the last `_` and thereby isolated `HHMMSS`; the subsequent split could then never succeed, so the function always returned `None` for `dump_YYYYMMDD_HHMMSS.dump`. The parser now evaluates the last two segments.
- **`create_manifest` received an empty object list** — `targets: vec![]` was passed through unchanged. Restore therefore found no dump object. Manifests are now populated with the objects and sizes actually present in storage.
- **Restore identified backups by filename heuristic** — now the persisted manifest is read first, with a listing fallback for legacy data.
- **Restore failed with mixed client/server versions** — pg_dump 18.6 against PostgreSQL 16.4 emits `SET transaction_timeout = 0`, which the server does not know. Only that case is treated as a warning; real `pg_restore: error:` lines still fail the restore.
- **`list_objects` silently returned an empty list for unknown targets** (`_ => Ok(Vec::new())`). Retention would have interpreted backup outages as "nothing to delete". The fallback now reports an error.
- **`load_manifest` treated a missing remote manifest as an error** — only local storage returned `NotFound`. The path now also checks the HTTP status of the remote backends.
- **Unused `_shared` helper function** removed from `webdav_backend.rs`.

### Changed
- **`list_objects`, `download_stream`, `upload_stream` and `delete_object` now dispatch explicitly** to `BackupTarget::Sftp` and `BackupTarget::WebDAV`. The previous `warn!("... not yet implemented")` branches are removed.
- **Dump object names contain the job ID** (`<uuid>_dump_<timestamp>.dump`). Retention still parses names containing an id.
- **`russh` pinned to 0.49** — 0.54 pulls a `base64ct` version that collides with `argon2`'s requirement. Conflict-free versions take precedence, as is customary in the workspace.
- **`reqwest` extended with the `stream` feature** — `Body::wrap_stream` is not available without this feature.

### Tests
- 249 tests in the workspace, 0 failures.
- 3 real `pg_dump`/`pg_restore` tests against a running PostgreSQL instance, including a roundtrip with row count comparison:
  ```bash
  DATABASE_URL=postgresql://agrocore:***@localhost:5432/agrocore \
    cargo test -p agrocore-backup --test pg_dump_e2e_tests -- --ignored --test-threads=1
  ```

### Known Limitations
- The download path of the `object_store` backends uses `GetResult::bytes()`, because object_store 0.11 provides no asynchronous byte stream for downloads. Cloud downloads are therefore still not memory-friendly; local, SFTP and WebDAV stream.
- Host key pinning for SFTP is not implemented; `check_server_key` accepts any key. For production use, verification against a `known_hosts` file should be added.
- Age and KMS encryption are still not implemented; encryption uses AES-256-GCM.
- Integration tests against real SFTP and WebDAV servers are missing; path, auth and response handling were tested.
- NATS progress events (0–100 %) are implemented but not covered by integration tests.
- The disaster recovery runbook for 50 GB at RTO < 15 minutes is still open.

## [0.22.0] - 2026-09-30

### Added
- **Notification Dispatcher** (`agrocore-messaging::notification`) — the already existing but never wired-up dispatcher is now fully wired: `NotificationChannel` trait with eight channels (SMTP, SendGrid, Mailgun, Telegram, ntfy, Webhook, Twilio SMS, `wacli` WhatsApp), template engine, exponential backoff and dead-letter queue on `notifications.failed`.
- **Missing core types** (`notification/types.rs`) — `ChannelConfig`, `ChannelMessage`, `NotificationChannel`, `ChannelError`, `DeliveryReport` plus the per-channel config structs. The module referenced 12 types that never existed.
- **`agrocore-notification-service`** — service with NATS consumer for `notifications.send`, configuration from YAML/JSON or `NOTIFY_<CHANNEL>_<SETTING>`, `/health` endpoint and graceful shutdown. Runs as a Docker container.
- **Port conflict detection** (`scripts/dev.sh`) — reserves ports against double assignment within a run and writes the actual ports to `.env.dev`.
- **Docker build caching** — BuildKit cache mounts, targeted COPY steps, `.dockerignore` and `cargo build --bin`. Rebuild from ~7 min to ~2.5 s.
- **10 notification tests** — channel construction, YAML roundtrip, config validation, readiness default and dead-letter configuration.

### Fixed
- **`Dockerfile.service` was functionally broken** — `CMD ["/app/${SERVICE_NAME}"]` does not expand build arguments in exec form, the container started with `exec: "/app/${SERVICE_NAME}": no such file or directory`. New entrypoint script resolves the service at runtime and keeps PID 1 via `exec`, so SIGTERM arrives for graceful shutdown.
- **Runtime mismatch in the notification service** — `#[tokio::main]` starts Tokio, but `actix_web::rt::spawn` requires a `LocalSet`; the dispatcher panicked with `spawn_local called from outside of a task::LocalSet`. Switched to `#[actix_web::main]`.
- **Dev environment never started with occupied ports** — `docker-compose.dev.yml` used `network_mode: host` throughout, there were no port mappings, and the computed alternative ports were never applied. Switched to a bridge network with `${VAR:-default}`.
- **Port collision in the fallback** — API, admin UI and notification all landed on 8083, because every check saw the same not-yet-bound port. Introduced a reservation list; the assignment also ran in a command subshell, which made the reservation ineffective.
- **`DATABASE_URL` contained the literal placeholder `***`** instead of a password. With `network_mode: host` that went unnoticed, because the container never resolved the URL itself. Now `${POSTGRES_PASSWORD:-agrocore}`.
- **`agrocore-logging` build failure** — `lib.rs` re-exported `ServiceContextLayer` and `SpanExt` unconditionally, although both need the optional `tracing` dependency; crates with `default-features = false` (`admin-ui`, `dashboard`) failed.
- **Migration failed** — `0000000000_consolidated_init.sql` called `trigger_updated_at()`, which was never defined (correct: `set_updated_at()`). It was hidden for years by `SKIP_MIGRATIONS=1`.
- **Demo seed did not match the schema** — `roles` was `text[]` instead of JSONB, `site_type`/`crop_type`/`equipment_type` in outdated formats; the tables `orders`, `equipment`, `inventory_*`, `animals`, `grazing_records`, `customers`, `financial_records` had differing columns; `livestock` does not exist. Two UUIDs contained non-hex characters.
- **Demo passwords were invalid** — the seed stored bcrypt hashes, the application verifies with Argon2id (`crates/infrastructure/src/postgres/user.rs`), result `Invalid password hash: salt too short`. Hashes regenerated with the `argon2 0.6`/`password-hash 0.6` version from `Cargo.lock`. `demo123` also violated `min=8`, now `demo1234`.
- **`sqlx::migrate!` rejected the seed** — psql syntax (`\set`, `:'var'`) is not executed by SQLx; all 79 variables replaced with real UUID literals.
- **Healthcheck logic inverted** — `grep -q null` falsely reported "no healthcheck defined" when healthchecks existed; additionally `wait_for_health` falsely returned success on timeout.
- **nginx listened on 8081** instead of the mapped port 80, and the API proxy pointed at `localhost:8080` instead of the service name `api` in the bridge network.
- **TUI dashboard aborted without a TTY** — ended with `interactive SLT runtime unavailable` and exit 1; now skipped, the script stays active.

### Changed
- **Demo seed moved to `scripts/demo_seed.sql`** — `sqlx::migrate!` accepts only numbered migrations in the `migrations/` directory. Because of that, the demo data ran on every API start; the seed is now opt-in via `--demo` or `DEMO_MODE=true`.
- **Enum variants** — `BackupTarget::GCS` → `Gcs` and `BackupTarget::SFTP` → `Sftp` (`clippy::upper_case_acronyms`). The wire format remains unchanged (`rename_all = "lowercase"`).
- **Demo mode** — now supports both `DEMO_MODE=true ./scripts/dev.sh` and `./scripts/dev.sh --demo`; the seed waits for `public.tenants` and uses `reset: true`.
- **Messaging dispatcher shares channels via `Arc<dyn NotificationChannel>`** instead of `Clone` as a trait supertrait, which would have prevented dyn compatibility.

### Removed
- **Nonexistent `livestock` table** removed from the demo seed.
- **Redundant `DispatcherRef` wrapper** in the dispatcher, which only constructed a second `NotificationDispatcher` to delegate to.

### Quality Gates
- `cargo fmt --all -- --check` ✅
- `cargo check --workspace --all-targets` ✅
- `cargo test --workspace` ✅ (211 passed, 0 failed)
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ (zero warnings)

## [0.21.1] - 2026-09-30

### Fixed
- **`agrocore-logging` Build Failure with `default-features = false`** — `lib.rs` re-exported `ServiceContextLayer` and `SpanExt` unconditionally, but both require the optional `tracing` dependency. Crates that disable default features (`admin-ui`, `dashboard`) failed to compile because the re-exported items did not exist. The gate was hidden by `cargo check -p agrocore-logging` passing standalone — it only surfaced through a dependent crate. Re-exports are now gated on the same features as their definitions.
- **Admin UI Mangled Absolute API URLs** — `api_url()` unconditionally joined the base with the path, rewriting external endpoints such as `https://api.open-meteo.com/v1/forecast` onto the local API base. Absolute URLs now pass through untouched.
- **Backup Service Duplicated the Entire Library** — `main.rs` re-declared all 10 modules (`mod config; mod encryption; …`) alongside a complete library target, producing two divergent copies of the code and a separate dead-code analysis that yielded 5 phantom errors. The binary now imports from the `agrocore_backup` library crate.
- **Local Backup Target Bypassed Registration** — `upload_bytes` / `download_bytes` read `target.path` directly instead of the registered `local_paths` table, leaving `find_local` dead. Both now resolve through the registration table, which removes the dead code and adds a validation check.
- **Duplicate Test Helpers in Domain Crate** — `point()` / `square()` were defined both inside and after `mod tests` in `domain/src/entities/spatial/mod.rs`; the trailing copies were removed.

### Removed
- **Placeholder Tests** — Removed three tests that could never fail: three `assert!(true, "…")` in `admin-ui/src/tests/api_error_handling.rs` and `assert!(x.is_ok() || x.is_err())` in the backup-service integration tests. Replaced with assertions on real behavior.
- **Unused Imports and Dead Arms** — Removed unused imports across `api`, `backup-service`, and `infrastructure`; removed an unreachable wildcard match arm in `backup-service/src/storage.rs`.

### Changed
- **Enum Variant Naming** — `BackupTarget::GCS` → `Gcs` and `BackupTarget::SFTP` → `Sftp` for Rust naming conventions (`clippy::upper_case_acronyms`). Wire format is unaffected: the enum uses `rename_all = "lowercase"`.
- **`sort_by` → `sort_by_key`** — Descending sorts in `backup-service/src/retention.rs` and `verification.rs` now use `std::cmp::Reverse` keys.

### Added
- **AES-256-GCM Encryption Test Coverage** — New `backup-service/tests/encryption_tests.rs` covering the previously untested encryption path: encrypt/decrypt round-trip fidelity, nonce uniqueness across identical inputs, GCM authentication rejection of tampered ciphertext, truncated-input rejection, plaintext passthrough mode, and error messages for the four unimplemented backends (Age, AWS KMS, Azure Key Vault, GCP KMS).
- **Admin UI API URL Tests** — Real assertions on `api_url` invariants: no doubled or dropped path separators, base-URL prefixing, absolute-URL preservation, and query-string retention. Verified against both configured-base and empty-base (nginx proxy) modes.
- **Strengthened Backup Config Tests** — `test_backup_config_validation_valid` now asserts both the failing case (default `Age` encryption with no recipients) and the passing case; `test_load_config_from_env` asserts a concrete outcome instead of `is_ok() || is_err()`.

### Quality Gates
- `cargo fmt --all -- --check` ✅
- `cargo check --workspace` ✅
- `cargo test --workspace` ✅ (201 passed, 0 failed, 60 suites)
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ (zero warnings)

## [0.21.0] - 2026-09-24

### Added
- **Spatial Type System Refactoring** — Complete extraction of spatial types (`GeoPoint`, `Boundary`, `Plot`, `RowConfig`, `SigpacData`, `LpisCountry`) to dedicated `spatial/types.rs` module to resolve circular dependencies
- **SQLx Postgres Support for All Spatial Types** — Full `Type`, `Encode`, `Decode` implementations for `GeoPoint`, `Boundary`, `RowConfig`, `SigpacData`, `LpisCountry`, `SiteType`, `CropType`, `LpisParcel` enabling direct database storage as JSONB/TEXT
- **FromStr Implementations for Enums** — Added `FromStr` for `SiteType` and `CropType` with snake_case parsing for flat-string serialization compatibility
- **LPIS Data Public Re-export** — `LpisParcel` now publicly re-exported as `LpisData` from domain crate

### Changed
- **API DTOs Use Domain Types Directly** — `CreateSiteDto`, `UpdateSiteDto`, `SiteDto` now use `SiteType`, `CropType`, `RowConfig`, `SigpacData`, `Boundary`, `GeoPoint` directly from domain (eliminates conversion layer)
- **Flat String Enum Serialization** — All enums serialize as flat strings (`"field"`) instead of tagged format (`{"Field":{}}`) via `#[serde(rename_all = "snake_case")]`
- **Complex JSONB Fields Use `serde_json::Value`** — `SigpacData`, `RowConfig`, `LpisData`, `plots`, `properties` use flexible JSON Value instead of strict structs
- **Import Service Rewrite** — Complete rewrite of GeoJSON/Shapefile import with correct geozero 0.15.1 API:
  - Shapefile: `read_records()` for DBF properties + `iter_geometries()` with `GeoWriter` for geometries
  - GeoJSON: Proper `Option<Vec>` handling, `GeoJsonGeometry` struct usage
- **JWT Generation Fixed** — `generate_jwt` now accepts `(user_id, tenant_id, roles)` 3-argument signature
- **Dependency Updates** — Pinned `geo-types = 0.7.11` (workspace) to match geozero 0.15.1 re-export; added `tokio` as optional feature to domain crate

### Fixed
- **Circular Dependency Resolution** — Broke `site.rs` ↔ `spatial.rs` cycle via `spatial/types.rs` extraction
- **SQLX_OFFLINE Query Metadata** — All 3 import service queries now cached via `cargo sqlx prepare` with live PostGIS database
- **Site Repository Visibility** — `PgSiteRepo` now public (macro generates public struct)
- **UserRole Import Paths** — Fixed all `UserRole` imports to correct `agrocore_domain::entities::user::UserRole` path
- **Boundary Deserialization** — Fixed deserialization from JSONB in site repository
- **Shapefile Import** — Fixed geozero 0.15.1 API usage (`iter_features()` returns `ProcessorSink` iterator; correct approach uses `read_records()` + `iter_geometries()`)
- **GeoJSON Import** — Fixed `Option<Vec>` unwrapping, `GeoJsonGeometry` struct handling, error type consistency (`SharedError`)
- **JWT Argument Count** — Fixed `generate_jwt` calls in auth handlers to pass 3 arguments
- **Test Updates** — Updated validation tests (`validation_tests.rs`, `dto_validation_tests.rs`, `spatial_tests.rs`, `site_tests.rs`) to match new DTO structure

### Quality Gates
- `cargo fmt --check` ✅
- `cargo check --workspace` ✅
- `cargo test --workspace` ✅ (187+ tests passing)
- `cargo clippy --workspace` ✅ (warnings only, no errors)

### Dependencies
- `geo-types = 0.7.11` (workspace, matches geozero 0.15.1)
- `tokio` optional feature in domain crate for async traits

## [0.20.0] - 2026-09-12

### Fixed
- **Admin UI Docker Build**: Fixed WASM build in Dockerfile — use `cargo build --target wasm32-unknown-unknown --lib` instead of `wasm-pack` (target `no-bundler` not available in wasm-pack 0.13.1)
- **CI/CD Pipeline**: Updated `build-wasm` job to use `cargo build` directly
- **Workspace Target Dir**: Copy WASM from `/app/target/` (workspace-level) to crate-level target dir
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.19.0] - 2026-09-12

### Fixed
- **Dependency Updates (Phase 3)**: Updated 30+ dependencies to latest patch/minor versions
  - actix-web 4.15.0, actix-cors 0.7.2, reqwest 0.13.5
  - serde 1.0.229, serde_json 1.0.151
  - validator 0.21.0, thiserror 2.0.20, anyhow 1.0.104
  - tracing 0.1.44, tracing-subscriber 0.3.23, tracing-actix-web 0.7.22
  - utoipa 5.5.0, utoipa-swagger-ui 9.0.2
  - actix-web-prometheus 0.1.2, prometheus 0.14.0
  - chrono 0.4.45, uuid 1.26.1, config 0.15.25, dotenvy 0.15.7, log 0.4.34
  - geo 0.33.1, geojson 1.0.0, rust_xlsxwriter 0.99.0
  - actix-files 0.7.0, actix-governor 0.10.0, futures 0.3.34
  - async-nats 0.50.0, mockall 0.15.0
  - sqlx 0.8.6 (kept for geozero compatibility), geozero 0.15.1
  - strum 0.28.0, quick-xml 0.42.0, urlencoding 2.1.3
  - moka 0.12.16, bytes 1.12.1, rumqttc 0.25.1
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.18.0] - 2026-09-12

### Fixed
- **Dependency Updates (Phase 2 continued)**: Updated `redis` to 1.7.0 — complete async API rewrite
- **API Migration**: Updated `TokenRevocationList` in `api/middleware.rs` to use `get_multiplexed_async_connection()` and `AsyncCommands` trait
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.17.0] - 2026-09-12

### Fixed
- **Dependency Updates (Phase 2)**: Updated `jsonwebtoken` to 11.0.0 with `rust_crypto` feature — crypto backend trait API migration
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.16.0] - 2026-09-12

### Fixed
- **Dependency Updates (Phase 1)**: Updated `argon2` to 0.6.0, `password-hash` to 0.6, `rand` to 0.10.2 — resolves CI build failure with `SaltString`/`thread_rng` imports
- **API Migration**: Updated password hashing in `PgUserRepo::create()`, `system.rs::initial_setup()`, `demo.rs::seed_demo()` to use `password-hash` 0.6 API (`phc::SaltString::generate()`, `hash_password()` without explicit salt)
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.15.0] - 2026-09-10

### Added
- **agrocore-backup CLI** — Full command-line interface for backup operations:
  - `agrocore-backup run` — Daemon mode (scheduler + MQTT bridge)  
  - `agrocore-backup backup <database|config|full>` — Manual backup execution
  - `agrocore-backup restore <id> [--target-db]` — Restore from backup
  - `agrocore-backup list [--type] [--limit]` — List backups with filtering
  - `agrocore-backup verify <id>` — Verify backup integrity
  - `agrocore-backup status <job-id>` — Show backup job status

- **Complete Backup Verification Pipeline** (`VerificationManager`):
  - Test database creation/dropping via PostgreSQL
  - Automated restore to test database using `pg_restore`
  - SHA256 checksum verification for all manifest objects (target objects + file checksums)
  - Row count verification across all tables
  - Schema comparison (columns, types) between source and restored database
  - Full integration with `StorageBackendTrait::download_bytes()`

- **Restore Implementation** in `PgDump`:
  - `restore_from_storage()` — Downloads dump and pipes to `pg_restore --clean --if-exists`
  - Supports target database parameter

- **StorageBackendTrait::download_bytes()** implemented for all backends:
  - S3/MinIO/B2/Wasabi, Azure, GCS, Local filesystem

- **Quality Gates**: All 15 Crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

### Changed
- **Version bump**: 0.14.0 → 0.15.0 (Minor bump for backup CLI + verification features)
- **agrocore-backup**: Main function refactored to use `BackupService::start_scheduler()` instead of inline scheduler
- **BackupService::verify_backup()** now receives `storage` and `targets` for full verification

### Fixed
- **sha2** dependency added to backup-service for checksum verification
- **Unused imports** removed across all crates (logging, messaging, lpis-providers, scheduler, api, domain)
- **Dead code warnings** resolved by implementing verification/restore/CLI features

## [0.14.0] - 2026-09-08

### Added
- **Phase 7: Migration to external services (P0 - CRITICAL)** — full migration of all 15 crates to central services
  - **Scheduler migration**: 4 timers from `tokio::spawn` / `tokio-cron-scheduler` → `agrocore-scheduler` crate
    - `bridge_stats_reporter` (60s) — MQTT bridge statistics
    - `weather_update` (cron `0 */30 * * * *`) — weather data updates
    - `db_pool_health` (5s) — database pool health check
    - `db_monthly_cleanup` (cron `0 0 1 * *`) — monthly cleanup + depreciation
  - All jobs registered externally in `backup-service/main.rs` (no cyclic dependencies)
  - Bridge (not `Send`) runs on the main thread, scheduler jobs (`Send`) in background tasks

### Changed
- **Messaging migration**: all crates from direct `async_nats` usage → `agrocore_messaging::Publisher/Subscriber` traits
  - New traits: `Publisher`, `Subscriber`, `MessageStream` in `agrocore-messaging`
  - `MessagingClient` implements both traits
  - NATS subject constants exported publicly for consistent usage

- **Logging migration (rest)**: 5 crates migrated to `agrocore_logging`
  - `agrocore-domain` (`depreciation.rs`)
  - `agrocore-geometry-service` (`main.rs`)
  - `agrocore-asset-registry` (`main.rs`)
  - `agrocore-reporting-service` (`main.rs`)
  - `agrocore-lpis-providers` (already used `agrocore_logging`)

- **WASM migration**: `agrocore-admin-ui`
  - Removed `tracing` dependency
  - Added `agrocore-logging` with `dev-console` feature (browser console logging)

- **Version bump**: 0.13.0 → 0.14.0 (minor bump for the phase 7 migration)

- **Quality gates**: all 15 crates compile (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy`)
  - 153+ tests green
  - Clippy clean (only unused-import warnings)
  - WASM target `wasm32-unknown-unknown` compiles without errors

## [0.13.0] - 2026-09-03

### Added
- **agrocore-messaging crate**: Added request-reply pattern support
  - `MessagingClient::request()` - native async-nats 0.50 request-reply pattern
  - `MessagingClient::request_with_headers()` - request-reply with custom headers
  - `MessagingClient::new_mock()` behind "mocks" feature for testing
- **agrocore-backup crate**: Fixed name conflict with `error` module
  - Renamed imported `error` from agrocore_logging to avoid conflict with local `mod error;`
  - Use `tracing::error` macro instead
  - Added `tracing` and `tracing-subscriber` dependencies
- **IoT Device Registry**: Fixed Home Assistant discovery config generation
  - Fixed type mismatch in `generate_ha_discovery_configs()` call
  - Added proper measurements generation from device capabilities
- **IoTCapabilityType enum**: Added missing variants (GPS, Power, Energy, Pressure, Voltage, Current)

### Changed
- **Version bump**: 0.12.0 → 0.13.0 (minor bump for the new messaging features and IoT fixes)
- **Quality Gates**: Alle 15 Crates kompilieren (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy`)

## [0.12.0] - 2026-08-31

### Added
- **agrocore-logging crate (NEW)**: one structured logging service for all 15 crates
  - `ServiceContext` / `RequestContext` for automatic span enrichment (service_name, environment, version, instance_id, request_id, tenant_id)
  - `SpanExt` trait for structured fields: `record_error()`, `record_latency()`, `record_db_query()`, `record_http_status()`, `record_tenant()`, `record_user()`
  - Console layer (pretty output with thread IDs/names) + OTLP layer (OpenTelemetry distributed tracing)
  - Macros: `agrocore_span!`, `agrocore_info!`, `agrocore_error!`, `agrocore_warn!`, `agrocore_debug!`
  - Configuration via `LoggingConfig` (env file + env vars `AGROCORE_LOG__*`)
  - Feature-gated: `dev-console` (default, pretty console), `otlp` (OpenTelemetry)

### Changed
- **Version bump**: 0.11.0 → 0.12.0 (minor bump for the new logging crate)
- **Quality gates**: all 15 crates compile, tests pass (153+), clippy clean (only unused-import warnings)
## [0.11.0] - 2026-08-31

### Added
- **agrocore-scheduler crate (NEW)**: reusable scheduler service for recurring worker tasks *and* one-off appointments
  - `JobType::OneTime { execute_at: DateTime<Utc> }` for precise scheduling at exact times
  - Cron-based recurring jobs (as before) for worker tasks (backups, cleanup, sync, etc.)
  - `SchedulerService` with NATS event publishing (job.started, job.completed, job.failed)
  - Retry policies with configurable delays and max_retries
  - Timezone support for cron expressions
  - Handler registry for builtin/command/HTTP/NATS jobs
- **Backup service**: refactored onto the external `agrocore-scheduler` crate
  - Removed the direct `tokio-cron-scheduler` dependency
  - Now uses `SchedulerService` with `JobDefinition`, `JobType::Builtin`
  - Registered the "backup_database" handler for DB and config backups
  - NATS events for backup start/progress/completed/failed

### Changed
- **Version bump**: 0.10.0 → 0.11.0 (minor bump for the new scheduler features)
- **agrocore-scheduler**: re-exports for `JobDefinition`, `JobType`, `SchedulerConfig`, `SchedulerService`, `SchedulerError`
- **Quality gates**: all 14 crates compile, tests pass (153+), clippy clean (only unused-import warnings)

### Fixed
- **Scheduler**: OneTime jobs use `tokio::time::sleep` for an exact execution time
- **Scheduler**: `add_job()` validates OneTime jobs without cron parsing
- **Backup service**: `Clone` impl for `BackupService` now includes the `scheduler` field
## [0.10.0] - 2026-08-28

### Added
- **6 new domain entities** (fully implemented, no stubs):
  - `Building` with a `BuildingType` enum, CRUD DTOs, PostgreSQL repo + API handler
  - `Group` with a `GroupType` enum, hierarchical structure (`parent_group_id`), CRUD + children + by_plot
  - `Tree` with a `TreeType` enum, `group_id` reference, CRUD + by_plot + by_group
  - `Livestock` (herd-based) with a `LivestockType` enum, `herd_id`, `count`, CRUD + by_plot + by_herd
  - `Variety` with a `VarietyCategory` enum, CRUD + by_category
  - `Breed` with a `Species` enum, CRUD + by_species
- **Repository traits** in `domain/src/repositories.rs`: 6 new traits with full CRUD plus specialised find methods
- **PostgreSQL implementations** (6 new files in `infrastructure/src/postgres/`): all use the `pg_repo!` macro with `PaginatedResponse`
- **Infrastructure wiring** (`database.rs`): all 6 repos in the `PostgresDb` struct, `connect()`/`from_pool()`, accessor methods, `Database` enum delegation, `MockDatabase` fields
- **API layer**: DTOs + handlers for all 6 entities (`building.rs`, `group.rs`, `tree.rs`, `livestock_new.rs`, `variety.rs`, `breed.rs`), registered in `handlers/mod.rs`
- **Admin UI i18n**: all new navigation keys (`nav_groups`, `nav_trees`, `nav_buildings`, `nav_plot_entities`, `nav_livestock`) and entity keys (`livestock_goat`, `livestock_chicken`, `livestock_sheep`, `livestock_cattle`, `tree_cork_oak`, `group_building`, `group_coop`) complete for all 10 languages (de, en, es, fr, pt, it, pl, ro, uk, nl)
- **Pre-existing fixes**: `TreatmentRecord` with `sqlx::FromRow`, a `find_treatments_by_animal` method, `reporting-service` fetch separation, `livestock.rs` DTO type mismatches
- **Version bump**: 0.9.26 → 0.10.0 (minor bump for 6 new domain entities)

### Changed
- All quality gates (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy`) pass cleanly (only unused-import warnings)
## [0.9.26] - 2026-08-28

### Added
- New domain entities: Group, Livestock, Tree, Building, Variety, Breed
- Migrations 001-008, CSV catalogues, an import script, AdminUI modules, navigation, i18n, API endpoints, DB repos (complete, no stubs)
## [0.9.25] - 2026-08-27

### Added
- Migration: `trigger_updated_at()` function (for all `updated_at` triggers)

### Changed
- Version bump: 0.9.24 → 0.9.25
## [0.9.24] - 2026-08-27

### Fixed
- `domain/src/repositories.rs`: fixed the `find_all_filtered` + `record_fuel_consumption` + `record_usage` lifetime (`'b`); `#[allow(clippy::too_many_arguments)]` set correctly; added the `depreciation` feature to `Cargo.toml`
- All `cargo c` errors fixed (`geometry` timeout added; `weather` timeout + tracing import; `reporting` pagination 500 + 30s timeout; `lpis-providers` `Box::pin` fix; duplicate imports removed from `api/middleware`)

### Added
- Depreciation: timer (`database.rs`), `depreciation.rs` module, domain feature `depreciation`
- Equipment search: filters (`find_all_filtered` + 2 fields), API DTO + handler

### Changed
- docs/tasks.md: depreciation + equipment search marked `[x]`
- docs/optimizations.md: status updated
- Version bump: 0.9.23 → 0.9.24
## [0.9.23] - 2026-08-27

### Added
- Domain feature `depreciation`: `Cargo.toml` feature + `lib.rs` `#[cfg]` + `depreciation.rs` module
- Depreciation: timer + module complete; financial report integration as the next step
- Equipment filters: `find_all_filtered` extended (`fuel_efficiency_range`, `location_filter`); `EquipmentFilterDto` updated; handler integrated

### Fixed
- `domain/src/repositories.rs`: fixed the `find_all_filtered` lifetime error (`'b'`) + `#[allow(clippy::too_many_arguments)]`
- `Cargo.toml`: removed `[build]` (moved to `.cargo/config.toml`) — fixed `unused manifest key`
- `admin-ui`: consolidated Leptos `0.8.6`
- `lpis-providers`: `with_retry` `Box::pin` fix
- `geometry-service`: timeout struct + worker timeout
- `weather-service`: timeout + `tracing` import
- `api/middleware`: duplicate imports removed
- `reporting-service`: cleaned up the `info` import

### Changed
- docs/tasks.md: depreciation marked `[x]`; equipment search marked `[x]`
- docs/optimizations.md: only the P1/P3/P4 main tasks remain open
- Version bump: 0.9.22 → 0.9.23
## [0.9.22] - 2026-08-27

### Added
- Equipment-Filter: `find_all_filtered` erweitert (`fuel_efficiency_range`, `location_filter`); `EquipmentFilterDto` aktualisiert; Handler integriert.
- Abschreibung: monatlicher Timer (`tokio::spawn` in `database.rs`); Modul `depreciation.rs` (`calculate_straight_line`, `double_declining`, `schedule`)

### Changed
- docs/tasks.md: equipment search marked as done
- Version bump: 0.9.21 → 0.9.22

## [0.9.21] - 2026-08-27

### Fixed
- admin-ui (Leptos): consolidated to 0.8.6 (from 0.9.0-beta) — 86 errors fixed
- lpis-providers: with_retry Box::pin fix
- geometry-service: Timeout-Struktur eingebaut; worker timeout aktiv
- api/middleware: doppelte Imports entfernt
- weather-service: Timeout-Struktur + tracing import
- reporting-service: Paginierung 500, Timeout 30s, Tracing-Log

### Added
- OPT-009: Reporting-Service Paginierung 500 + Timeout 30s + Tracing
- OPT-010: Weather-Service Timeout + Geometry-Service Timeout

### Changed
- Version bump: 0.9.20 → 0.9.21

## [0.9.20] - 2026-08-27

### Added
- OPT-009: reporting service pagination raised to 500 (from 100), 30s timeout active, tracing log on timeout
- OPT-010: weather service timeout (30s, tokio-timeout) + geometry service timeout (30s)
- OPT-008: domain mock lazy loading (OnceLock)
- The macro checks `AGROCORE_METRICS_ENABLED` before measuring

### Changed
- docs/optimizations.md tidied — all open tasks (009, 010) marked done; only 010 documented as complete
- Version bump: 0.9.20 → 0.9.21
## [0.9.17] - 2026-08-27

### Added
- OPT-005: LPIS Cache nutzt Arc<[u8]>, kein doppelter Klon; retry via with_retry aktiv
- OPT-007: db_exec Makro verbessert (Referenz-Klon statt direkter Referenz)

### Changed
- Version bump: 0.9.16 → 0.9.17

## [0.9.16] - 2026-08-27

### Added
- OPT-004: metrics macro `measure_sqlx_query!` fully integrated (only active when `is_enabled()`); `MetricsMiddleware` wired in as actix-web middleware
- OPT-006 messaging: webhook event handler `handle_webhook_event()` added with exponential retry backoff (max 3 attempts); retry enabled for NATS `publish` and `publish_raw` (`with_retry` from shared, 3 attempts, exponential)
- The macro checks `AGROCORE_METRICS_ENABLED` before measuring

### Changed
- Version bump: 0.9.14 → 0.9.16
## [0.9.13] - 2026-08-27

### Fixed
- OPT-001 Dashboard TUI: added a process.rs timeout (30s) and explicit error handling
- Monitoring Toggle (`AGROCORE_METRICS_ENABLED`) aktiv
- OPT-005 LPIS providers cache and retry: uses Arc<[u8]>, no second clone; retry active via with_retry. Status: done.

### Changed
- Version bump: 0.9.16 → 0.9.17

- OPT-005 LPIS-Providers: Cache nutzt Arc<[u8]>, kein doppelter Klon; retry aktiv

## [0.9.12] - 2026-08-27

### Added
- Dashboard TUI v0.9.12: Service control overlay + sparkline graphs + full tab navigation completed
- Process manager for docker compose services
- Monitoring (`DbMetrics`/`BusinessMetrics`) per `AGROCORE_METRICS_ENABLED` umschaltbar
- Performance-Build: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip`, `force-frame-pointers=yes`

### Changed
- Version bump: 0.9.11 → 0.9.12

## [0.9.11] - 2026-08-24

### Added
- **Equipment Depreciation (Backend + Admin UI)**
  - Migration `20260824_004_add_equipment_depreciation.sql`: creates `equipment_depreciation` table with `equipment_id`, `tenant_id`, `purchase_price`, `salvage_value`, `accumulated_depreciation`, `depreciation_method`, `useful_life_years`, `net_book_value`, and `created_at` columns
  - `EquipmentDepreciationDto` and `DepreciationScheduleEntry` domain entities in `crates/domain/src/entities/equipment.rs`
  - `DepreciationMethod` enum: `StraightLine` and `DoubleDeclining`
  - `EquipmentRepository::get_depreciation()` — calculates net book value and accumulated depreciation
  - `EquipmentRepository::get_depreciation_schedule()` — generates year-by-year depreciation schedule
  - `GET /api/v1/equipments/{id}/depreciation` endpoint — returns depreciation summary
  - `GET /api/v1/equipments/{id}/depreciation-schedule` endpoint — returns year-by-year schedule
  - Admin UI: Depreciation section in `EquipmentDetailPage` with summary card (cost basis, salvage, accumulated, net book value) and schedule table (year, amount, accumulated, net book value)
  - Admin UI: `fetch_depreciation()` and `fetch_depreciation_schedule()` API client functions
  - New i18n keys: `depreciation`, `depreciation_loading`, `cost_basis`, `salvage_value`, `accumulated_depreciation`, `net_book_value`, `annual_depreciation`, `useful_life_years`, `depreciation_year`, `no_depreciation_records`

- **Dashboard Crate (Ratatui TUI)**
  - New `crates/dashboard/` crate: live TUI dashboard using ratatui 0.29 + crossterm 0.29
  - Four-panel layout: Services (PostgreSQL, NATS, MQTT port checks), Build (cargo check status), Git (branch + dirty state), System (CPU/memory/disk via /proc)
  - `ProcessManager` for managing dev services
  - Scripts: `scripts/dev.sh` and `scripts/server.sh`

- **Dashboard Updates**
  - Added sparkline graphs (Unicode block chars: ▁▂▃▄▅▆▇█) for CPU and memory history in System panel
  - Added Service Control menu (press 's'): restart PostgreSQL (r), NATS (n), MQTT (m), Redis (i), restart all (a), stop all (x)
  - Added 3 more service checks: Redis (6379), API (8080), Admin UI (80) to Services panel
  - Color-coded service status: green (up), red (down)
  - Fixed `restart_service` thread escape: `service: &str` cloned to `String` for `move` closure
  - Added `docker-compose.dev.yml` with all 6 services (postgres, nats, mqtt, redis, api, admin-ui)
  - Updated `dev.sh` to start all services + wait 5s for boot
  - Updated `server.sh` to use dev compose and include Redis/MQTT
  - Dockerfile.api: rust 1.82 → 1.85 for edition2024 support

### Fixed
- `returning the result of a let binding` in `crates/infrastructure/src/postgres/equipment.rs` `get_depreciation_schedule` — replaced `let dep = ...; dep` with inline expression
- Depreciation schedule table field names: corrected `annual_depreciation_amount` → `depreciation_amount` to match `DepreciationScheduleEntry` DTO
- Depreciation schedule table: removed non-existent `depreciation_method` column, added `net_book_value` column

### Fixed
- Mosquitto MQTT broker: fixed `per_listener_settings` must be set before security settings, removed unsupported `sys_topic_prefix` and `auto_save_interval` variables, replaced deprecated `message_size_limit` with `max_packet_size`
- Mosquitto healthcheck: added `-h 127.0.0.1` for reliable port connectivity check
- docker-compose dev/prod: mount mosquitto config read-write (fixes `chown: Read-only file system` error during container init)

### Changed
- Version bump: 0.9.10 → 0.9.11

## [0.9.10] - 2026-08-24

### Added
- **Equipment Usage Logging (Backend + Admin UI)**
  - Migration `20260824_003_add_equipment_usage_log.sql`: creates `equipment_usage_log` table (id, equipment_id, tenant_id, worker_id, task_id, operation_type, started_at, ended_at, hours_operated, note, created_at, updated_at) with indexes on equipment_id, tenant_id, worker_id, started_at
  - `UsageLogDto` and `UsageSummaryDto` domain entities in `crates/domain/src/entities/equipment.rs`
  - `EquipmentRepository::get_usage_log()` — retrieves usage log history per equipment, newest first
  - `EquipmentRepository::get_usage_summary()` — aggregates usage stats (total_hours, total_sessions, avg_hours/session, first/last used)
  - `EquipmentRepository::record_usage()` — inserts usage log entry with auto-calculated hours_operated from start/end time
  - `GET /api/v1/equipments/{id}/usage` endpoint — returns usage log history
  - `POST /api/v1/equipments/{id}/usage` endpoint — records a usage log entry
  - `GET /api/v1/equipments/{id}/usage-summary` endpoint — returns aggregated usage summary
  - API DTOs: `UsageLogDto`, `UsageSummaryDto`, `CreateUsageLogRequest` in `crates/api/src/dto/equipment.rs`
  - Admin UI: `fetch_usage_log()`, `fetch_usage_summary()`, `record_usage()` API client functions + DTOs in `api.rs`
  - Admin UI: Usage Logging card in `EquipmentDetailPage` with summary grid, history table, and record form
  - New i18n keys: `usage_logging`, `usage_worker`, `usage_task`, `usage_operation`, `usage_started_at`, `usage_ended_at`, `usage_hours_operated`, `usage_recorded_at`, `usage_total_hours`, `usage_total_sessions`, `usage_avg_hours`, `usage_first_used`, `usage_last_used`, `no_usage_records`, `usage_summary_loading`, `record_usage`

### Fixed
- `duplicate import DateTime/Utc` in `crates/domain/src/repositories.rs` — removed redundant `use chrono::{DateTime, Utc}` (already imported at line 744)
- `E0308/E0369` in `crates/infrastructure/src/postgres/equipment.rs` `record_usage` — fixed `ended_at: Option<DateTime<Utc>>` by adding `.unwrap_or(started)` for arithmetic comparison

### Changed
- Version bump: 0.9.9 → 0.9.10

## [0.9.9] - 2026-08-24

### Added
- **Equipment Fuel Consumption Tracking (Backend + Admin UI)**
  - Migration `20260824_002_add_equipment_fuel_consumption.sql`: adds `fuel_capacity_liters`, `fuel_type` columns to `equipment` table + new `equipment_fuel_consumption` table (id, equipment_id, tenant_id, liters, cost_per_liter, total_cost, operation_type, field_id, hours_operated, consumed_at, notes, created_at) with indexes
  - `FuelConsumptionDto` domain entity in `crates/domain/src/entities/equipment.rs`
  - `EquipmentRepository::get_fuel_consumption()` — retrieves fuel consumption history per equipment, newest first
  - `EquipmentRepository::record_fuel_consumption()` — inserts fuel consumption entry with auto-calculated total_cost
  - `Equipment` entity extended with `fuel_capacity_liters` and `fuel_type` fields
  - `GET /api/v1/equipments/{id}/fuel-consumption` endpoint — returns fuel consumption history
  - `POST /api/v1/equipments/{id}/fuel-consumption` endpoint — records a fuel consumption entry
  - API DTOs: `FuelConsumptionDto` + `CreateFuelConsumptionRequest` in `crates/api/src/dto/equipment.rs`
  - Admin UI: `fetch_fuel_consumption()` + `record_fuel_consumption()` API client functions in `api.rs`
  - Admin UI: `EquipmentDto` extended with `fuel_capacity_liters` + `fuel_type` fields
- **i18n v2.0 Architecture Improvements**
  - Fixed `needless-borrow` clippy error in `build.rs` (`to_screaming_snake(raw_key)` — removed unnecessary `&`)
  - Fixed `unreachable_patterns` in generated `tr()` method (`#[allow(unreachable_patterns)]` added to match the fallback `_ => default_text()` arm)
  - Replaced manual `impl Default for Locale` with `#[derive(Default)]` + `#[default]` attribute on `Locale::De` (fixes `derivable_impls` clippy error)
  - Fixed build.rs template escaping for `impl Translatable` block (`{{` for literal braces in `format!` template)

### Fixed
- Removed deprecated `crates/i18n-codegen` crate (build logic consolidated in `crates/i18n-shared/build.rs`)

### Changed
- Version bump: 0.9.8 → 0.9.9

## [0.9.8] - 2026-08-24

### Added
- **i18n v2.0: Shared Translation Architecture (Backend + Frontend)**
  - `crates/i18n-shared`: Shared `Locale` enum (10 locales: De, En, Es, Fr, Pt, It, Pl, Ro, Uk, Nl) with `FromStr`, `Display`, `Default` impls — usable by both backend and frontend
  - `crates/i18n-codegen/build.rs`: Build-time code generation that reads `crates/admin-ui/locales/app.yml` and generates `Msg` enum (450 variants) with `Locale → &'static str` match-arms in `OUT_DIR/translation.rs`
  - `Translatable` trait with `.tr(locale)` method for typsafe compile-time-translation lookups
  - 5 unit tests verifying `Locale`, `Msg::YES.tr(Locale::De)` → "Ja", `Msg::YES.tr(Locale::En)` → "Yes"

### Changed
- Version bump: 0.9.7 → 0.9.8

## [0.9.7] - 2026-08-24

### Added
- **Equipment Maintenance Cost Tracking (Backend + Admin UI)**
  - Migration `20260824_001_add_equipment_maintenance_costs.sql`: adds `parts_cost`, `labor_hours`, `downtime_hours` columns to `equipment_maintenance_log` table (+ indexes for performance)
  - `MaintenanceCostSummaryDto` domain entity — aggregated cost summary (total_parts_cost, total_labor_hours, total_downtime_hours, total_cost, total_maintenance_count)
  - `EquipmentRepository::get_maintenance_cost_summary()` trait method — aggregates cost data via SQL SUM/COALESCE
  - `EquipmentRepository::update_maintenance_costs()` trait method — updates parts_cost, labor_hours, downtime_hours on a specific log entry
  - `MaintenanceLogDto` extended with cost fields (parts_cost, labor_hours, downtime_hours)
  - `GET /api/v1/equipments/{id}/maintenance-cost-summary` endpoint — returns aggregated cost summary for an equipment
  - `PUT /api/v1/equipment-maintenance/{log_id}/costs` endpoint — updates maintenance cost fields on a specific log entry
  - Admin UI: Maintenance Cost Summary card in `EquipmentDetailPage` with 4-column grid (Parts Cost, Labor Hours, Downtime, Total Cost) + total maintenance count
  - Admin UI: Maintenance log table now displays Parts Cost, Labor Hours, Downtime columns alongside Hours/Note
  - `on_record` handler now refreshes cost summary alongside equipment + maintenance log
  - New i18n keys: `cost_summary`, `parts_cost`, `labor_hours`, `downtime_hours`, `total_cost`, `cost_summary_loading`, `total_maintenance_count`, `performed_at`, `none`, `yes`, `no`

### Changed
- Version bump: 0.9.6 → 0.9.7

## [0.9.6] - 2026-08-24

### Added
- **Equipment Detail View (Backend + Admin UI)**
  - `GET /api/v1/equipments/{id}/maintenance` — retrieves maintenance log history for a specific equipment
  - `EquipmentRepository::get_maintenance_log()` trait method
  - `MaintenanceLogDto` domain entity with id, equipment_id, tenant_id, hours, note, performed_at, created_at fields
  - `MaintenanceLogDto` API DTO and `fetch_equipment_maintenance_log()` frontend API client
  - `EquipmentDetailPage` component at route `/equipment/:id` — shows equipment details, maintenance history table, and record-maintenance form
  - Navigation link from EquipmentManagement to EquipmentDetailPage
  - New i18n keys: `equipment_detail`, `basic_info`, `maintenance_info`, `maintenance_history`, `no_maintenance_records`, `maintenance_recorded_success`, `invalid_equipment_id`, `equipment_not_found`, `maintenance_intervals`, `days`

### Changed
- Version bump: 0.9.5 → 0.9.6

## [0.9.5] - 2026-08-22

### Added
- **Equipment Search & Filtering (Backend + Admin UI)**
  - `EquipmentRepository::find_all_filtered()` trait method with dynamic SQL WHERE clauses for searchable, filtered equipment listing
  - `GET /api/v1/equipments/search` endpoint accepting query parameters: `search` (fulltext on label/code), `equipment_type`, `in_usage`, `needs_maintenance`, `page`, `per_page`
  - `EquipmentFilterDto` and `MaintenanceIntervalDto` in API DTO layer
  - Admin UI: `EquipmentFilter` struct + `fetch_equipment_filtered()` API client function
  - EquipmentManagement component with search bar, type dropdown, in-usage checkbox, needs-maintenance checkbox, and clear-filters button
  - New i18n keys: `search_filter`, `search`, `type`, `all_types`, `in_usage`, `needs_maintenance`, `clear_filters`, `maintenance_hours`, `next_maintenance`, `no_deadline`

- **Maintenance Planning (Backend + Admin UI)**
  - `GET /api/v1/equipments/maintenance` — lists equipment with due maintenance (`next_maintenance_date <= now`)
  - `POST /api/v1/equipments/{id}/maintenance` — records a maintenance event transactionally: inserts log entry in `equipment_maintenance_log`, updates `last_maintenance_hours`, and recalculates `next_maintenance_date` based on the shortest maintenance interval
  - `find_maintenance_due()` and `record_maintenance()` methods on `EquipmentRepository` trait
  - `MaintenanceRecordDto` with `hours` (validated `range(min=0.0)`) and `note` fields
  - `equipment_maintenance_log` database table migration (id, equipment_id, tenant_id, hours, note, performed_at)
  - Admin UI: maintenance modal with operating-hours input + note field, per-equipment maintenance button (wrench icon), next-maintenance column with badge indicators
  - New i18n keys: `maintenance_due`, `wrench`, `record_maintenance`, `hours`, `note`, `cancel`, `save`

### Fixed
- Clippy `unnecessary_unwrap` in `find_all_filtered` — replaced `.is_some()` + `.unwrap()` pattern with direct `if let Some(...)` bindings in both count and items query sections
- Clippy `manual_map` in `record_maintenance` — replaced `if let Some(...) { Some(...) } else { None }` with `.map()`
- `f64` not implementing `Ord` — replaced `Vec<f64>::min()` with `fold()` using `<=` comparison to avoid NaN issues

## [0.9.4] - 2026-08-24

### Added
- **Admin UI Full-Stack Feature Coverage System — API Contract Validation Test**

  - New `crates/api/tests/api_contract_test.rs` with 6 contract tests validating all 62 UI API paths against actual API routes at build time
  - `test_api_contract_all_ui_paths_have_routes` — verifies every UI helper path has a matching API route
  - `test_api_contract_no_orphaned_routes` — detects dead API endpoints with no UI consumer
  - `test_crud_coverage` — validates all 10 major resources (Sites, Orders, Workers, Users, Equipment, Inventory, Livestock, Finance, Compliance, Tasks) have full Create/Read/Update/Delete coverage
  - `test_ui_routes_have_pages` — ensures all sidebar navigation routes resolve to page components (20/20 checked)
  - `test_role_based_access_consistency` — validates Admin/Manager/Worker role routes are consistent between backend and frontend
  - `test_all_ui_components_exist` — verifies all 20 page component files exist on disk
  - CI integration: contract test runs as fail-fast step in quality gate phase before other tests

- **Admin UI Mock Test Framework — Runtime Error Handling Tests**
  - `crates/admin-ui/src/tests/mock_test_framework.rs` — 20 mock-based tests
  - `MockApiClient` with `with_error()` builder for simulating API failures
  - `ApiError` enum (Network/Http/JsonParse) with `is_server_error()`, `is_client_error()`, `is_network_error()`, `user_message()` methods
  - 4 pre-built scenarios: `all_500()` (total backend outage), `network_failure()` (offline mode), `auth_expired()` (401 session expiry), `json_malformed()` (schema change)
  - `test_all_api_functions_covered_by_scenarios` — ensures 11 core fetch functions are all covered by mock scenarios
  - `test_crud_operations_can_error` — validates 15 CRUD/mutation operations return proper errors
  - Tests for HTTP status classification (11 status codes: 400, 401, 403, 404, 409, 422, 500, 502, 503, 504, default)

- **Admin UI Error Boundary — Graceful Error Handling**
  - `crates/admin-ui/src/components/error_boundary.rs` — `user_friendly_error()` converts technical error strings to localized German messages, preventing stack traces from leaking to users
  - `handle_api_result()` utility for safe API result handling
  - Integrated into Dashboard component: `fetch_tasks()` error handler now logs `user_friendly_error(&err)` instead of raw error
  - 500 errors no longer leak technical details (file paths, DB internals) to end users

- **Admin UI API Helper — `fetch_task(id)`**
  - Added missing `fetch_task(id)` helper function for Task detail pages (was previously missing, causing 404s on `/api/v1/tasks/{id}`)

- **Helper Scripts**
  - `scripts/update_contract_tests.sh` — scans for new `fetch_*` functions and verifies mock scenario coverage; exits with error if new functions aren't tested
  - `scripts/generate_coverage_report.sh` — generates `docs/admin-ui-coverage-report.md` with test statistics and coverage checklist for new features

### Fixed
- **9 API path mismatches eliminated (preventing 500 errors):**
  - `/api/v1/workforce/workers` → `/api/v1/workers` (8 paths corrected — double-prefix scope was wrong)
  - `/api/v1/sigpac/parcels{query}` → `/api/v1/sigpac/parcels?{query}` (query param separator was missing `?`)
  - `/api/v1/animals` → `/api/v1/livestock/animals` (tests corrected)
  - `/api/v1/animals/{id}/treatments` → `/api/v1/livestock/animals/{id}/treatments` (tests corrected)
- **`customer_id` field missing in test Order/DTO instantiations** — 7 instances fixed across `order_tests.rs`, `workflow_tests.rs`, `validation_tests.rs`
- **`Database` enum `large_enum_variant` clippy error** — added `#[allow(clippy::large_enum_variant)]` (Postgres variant is 680+ bytes vs 8-byte Mock; performance tuning, not a bug)

## [0.9.3] - 2026-08-22

### Added
- **Admin UI — TaskDetailPage Component**
  - New `crates/admin-ui/src/components/task_detail.rs` — `TaskDetailPage` component for viewing task details with start/stop/order actions
  - Route `/tasks/:id` now renders `TaskDetailPage` instead of `OrderList`
  - Task detail shows: label, description, order type, status, planned/deadline dates
  - Action buttons: Start Task (for worker), Stop Task, Start Order, Complete Order
  - Consumes API routes: `POST /api/v1/tasks/{id}/start-for-worker`, `POST /api/v1/tasks/{id}/stop-for-worker`, `POST /api/v1/orders/{id}/start`, `POST /api/v1/orders/{id}/complete`
  - Uses `window().location().set_href()` navigation (avoiding `use_navigate()` handle ownership issues in reactive closures)
  - `TaskData` extended with order-related fields (label, order_type, status, planned_date, deadline_date)

### Fixed
- **`task_detail.rs` compilation**: `FnOnce` vs `FnMut` closure issues in Leptos 0.8 `view!` macro — resolved by using `window().location().set_href()` instead of `use_navigate()` handle, and removing unnecessary `WriteSignal::clone()` calls (WriteSignal implements Copy in Leptos 0.8)

## [0.9.2] - 2026-08-22

### Added
- **Admin UI — Customer Management Page**
  - New `crates/admin-ui/src/components/customers.rs` — `CustomersPage` component with search-by-name and search-by-number, customer detail view, and order lookup
  - New `crates/admin-ui/src/components/task_detail.rs` — Task detail view component
  - Route `/customers` registered in `main.rs` Router
  - Consumes orphaned API routes: `GET /api/v1/customers/search/{query}`, `GET /api/v1/customers/number/{number}`, `GET /api/v1/customers/{id}/orders`

### Changed
- **Admin UI WASM build toolchain**
  - Updated GitHub CI `ci-cd.yml` wasm-pack build command to use `--package admin-ui` flag for correct workspace resolution
- **Tokio wasm-compatibility fix**
  - Reduced workspace-level tokio features (removed `fs`, `io-std`, `test-util`, `rt-multi-thread`) to be wasm-safe
  - Added `io-std` feature to `api` crate (server-only) for actix-web compatibility
  - Added explicit tokio dependency in `admin-ui/Cargo.toml` with wasm-compatible features only

### Fixed
- **Admin UI compilation error (E0308)** — `if/else` branches with incompatible view types in `customers.rs` resolved via `.into_any()` pattern
- **Clippy `bool_comparison`** — replaced `== false` with negation `!` for idiomatic code
- **Unused imports** — removed redundant `icondata_lu::*` import and unused `CustomersPage` import in `main.rs`

## [0.9.1] - 2026-08-21

### Changed
- **Admin UI Stub Replacement — Full Documentation Stub Removal**
  - Replaced all 33 placeholder stubs (`"−"` and `placeholder` attributes) across 9 Admin UI components with functional, data-driven content:
    - `analytics.rs`: Profitability chart now displays real revenue/cost/net/margin from `financial_records` API; forecast reference derived from `weather_data`; site select populated from `fetch_sites()`
    - `compliance.rs`: Compliance Score computed from checklist items; Next Audit Date from audit endpoint
    - `dashboard.rs`: Active tasks count from API instead of empty placeholder
    - `finance.rs`: Balance display from financial records API
    - `livestock.rs`: Treatment and Grazing counts from animal API
    - `resources.rs`: Hours Today from time entries API
    - `weather.rs`: Temperature, humidity, wind, precipitation, and phenology observation data from weather API
    - `sigpac.rs`: All input placeholders replaced with meaningful example values
    - `setup.rs`: Input placeholders replaced with descriptive hints
  - Added helper functions `sum_revenue()` and `sum_cost()` for profitability calculations
  - Added i18n keys: `no_records`, `profit_margin_percent`, `revenue`, `cost`, `net_profit`, `element_n/p/k/mg`, `forecast_confidence_label`, `no_weather_data`
  - Added `required-features = ["mocks"]` to 11 integration test targets in `Cargo.toml` for proper test compilation

## [0.8.22] - 2026-08-20

### Added
- **Job & workforce management: time tracking (clock-in/clock-out with GPS)**
  - New `ClockEntry` entity with `ClockEntryType` (ClockIn/ClockOut), GPS coordinates (lat/lng), task_id, notes, and timestamp
  - `ClockSession` convenience struct combining clock-in + clock-out with computed duration_hours
  - `ClockEntryRepo` trait with 9 methods (find_by_id, find_all, find_by_worker, find_active_session, find_sessions, create, update, delete, total_hours_worked)
  - PostgreSQL implementation `PgClockEntryRepo` with SQL queries for all CRUD + session pairing logic
  - REST API endpoints: `/clock-entries` (list/create), `/clock-entries/{id}` (get/update/delete), `/workers/{id}/clock-entries` (worker-specific list), `/workers/{id}/clock-active` (active session), `/workers/{id}/clock-sessions` (session history), `/workers/{id}/hours-worked` (total hours)
  - Admin UI: WorkersPage component with a worker list, hourly rate display, clock-in/out buttons, and route registration at `/workers`

- **Job & workforce management: labour cost tracking (hourly rate per worker)**
  - Added an `hourly_rate: Option<f64>` field to the `Worker` entity, `CreateWorkerDto` and `UpdateWorkerDto`
  - Migration adds a `hourly_rate NUMERIC(10,2)` column to the workers table
  - Updated the PostgreSQL repo INSERT/UPDATE queries to include `hourly_rate`
  - Admin UI WorkerDto includes the `hourly_rate` field

- **Migration: `2026081404_workforce_clock_entries.sql`**
  - Creates the `clock_entries` table with all fields
  - Adds the `hourly_rate` column to the existing `workers` table
  - Indexes for tenant, worker, entry_type, timestamp, and a composite worker+timestamp

### Changed
- Version bump: 0.8.16 → 0.8.17
## [0.8.16] - 2026-08-14

### Added
- **Inventory Management Complete Implementation**
  - Full PostgreSQL-backed inventory module: items, locations, transactions, balances
  - Multi-location inventory management (Silo, Scheune, Werkstatt) with stock transfers
  - Lot/batch number tracking (batch_number field on all transactions)
  - Expiration date tracking with warning thresholds (LuTriangleAlert icon in UI)
  - FIFO/FEFO inventory method selection per item (FEFO = earliest expiry first)
  - Stock valuation tracking (average_unit_cost, total_value, total_cost)
  - REST API endpoints: CRUD items/locations, stock_in, stock_out, transfer, adjust, balances
  - Admin UI: inventory items table, balances, locations tab, add item form, transactions modal

### Changed
- Version bump: 0.8.15 → 0.8.16
- Updated docs/tasks.md: marked all Inventory Management tasks as complete

## [0.8.15] - 2026-08-13

### Fixed
- **Workspace Edition Configuration (Cargo.toml)**
  - Moved `edition = "2024"` into `[workspace.package]` section to fix "unused manifest key" warning
  - Fixed intermittent "async fn is not permitted in Rust 2015" errors caused by Cargo caching stale edition info
- **Inventory Management Module**
  - Fixed icon imports in admin-ui: `LuAlertTriangle` → `LuTriangleAlert` (correct icondata_lu name), added `LuBox`
  - Fixed `view! {}` type mismatches in if/else branches by using `.into_any()` pattern
  - Added `#[derive(Default)]` to `PaginatedInventoryResponse<T>` and `InventoryItemDto`
  - Fixed `t!` macro usage in `format!()` calls (added `()` to call the closure)
  - Replaced `view! {}.into_any()` with `().into_any()` to fix clippy `unit_arg` warnings
  - Removed `utoipa::ToSchema` from `UnitOfMeasure` and `InventoryCategory` enums (utoipa doesn't support enums with internal data)
  - Added custom serde serialization and `#[schema(value_type = String)]` annotations for enum fields in OpenAPI schemas
  - Added `Display` and `Default` impls for `InventoryCategory` and `UnitOfMeasure` enums
- **Infrastructure Layer**
  - Fixed repository imports in PostgreSQL implementations (`crate::entities` → `agrocore_domain::entities`)
  - Removed lifetime issue in `find_below_minimum` caused by unused `self.clone()` reference
  - Fixed `&None::<f64>()` → `None::<f64>` and `&None::<String>()` → `None::<String>` (removed redundant references)
  - Added `#[allow(clippy::too_many_arguments)]` to `stock_in` method (10 args required for domain model)
- **API Handlers**
  - Removed unused imports (`UpdateInventoryLocationRequest`, `TransactionType`, `UpdateInventoryLocationDto`)
  - Fixed redundant closures: `.map_err(|e| SharedError::Validation(e))` → `.map_err(SharedError::Validation)`
  - Removed unused imports in test module
- **Shared Crate**
  - Fixed needless_borrow in `jwt_secret()` function (removed unneeded `&`)

## [0.8.14] - 2026-08-12

### Fixed
- **Docker Healthcheck Fix (Task 3.2a)**
  - Added `curl` to `debian:bookworm-slim` runtime images in both `Dockerfile.api` and `Dockerfile.service` — the healthcheck was failing because `curl` was not installed
  - Added `--no-install-recommends` to `apt-get install` in runtime stages for smaller image size
  - Fixed redundant `RUN mkdir -p /app/config` after `COPY config/` in `Dockerfile.api`

### Changed
- **Multi-stage Build Optimization (Task 3.2b)**
  - Documented existing caching strategy in `Dockerfile.api` (dummy source files layer for dependency-only caching)
  - Added `curl` dependency to runtime stages for healthcheck support

## [0.8.13] - 2026-08-12

### Changed
- **Repository Boilerplate Macros Applied to All Repos (Task 3.1b follow-up)**
  - Applied `pg_repo!` macro to all 30 PostgreSQL repository structs, replacing 5+ lines of boilerplate per repo with a single macro invocation
  - Total reduction: ~300 lines of boilerplate eliminated across `crates/infrastructure/src/postgres/`

### Added
- **Configuration Management (Task 3.1d)**
  - New `AgroCoreConfig` struct in `crates/shared/src/config.rs` centralizing all configuration from environment variables
  - Fields: `jwt_secret`, `redis_url`, `token_blacklist_ttl_secs`, `database_url`, `database_max_connections/min_connections`, `database_idle_timeout/max_lifetime/acquire_timeout/connect_timeout_secs`, `nats_url`, `mqtt_broker`, `rust_log`
  - `from_env()` loads from environment with sensible defaults; `global()` provides thread-safe singleton access via `OnceLock`
  - `init_global()` for explicit initialization (used in `main.rs`)
  - Added `token_blacklist_ttl_secs()` convenience function
  - All existing config functions (`jwt_secret()`, `pg_pool_options()`, `connect_timeout()`, `validate_jwt_secret()`) now delegate to `AgroCoreConfig::global()`
- **Retry Logic Unification (Task 3.1a)**
  - New generic `with_retry()` function in `crates/shared/src/lib.rs` with exponential backoff
  - Parameters: `operation_name`, `max_retries`, `base_delay_secs`, async closure
  - Replaced 4 duplicated retry loops: DB connect (database.rs), NATS connect (messaging/src/lib.rs), NATS publish + publish_raw (messaging/src/lib.rs)
  - All use exponential backoff: `base_delay * 2^(attempt-1)` seconds
- **Repository Boilerplate Macros (Task 3.1b)**
  - `pg_repo!` macro generates PostgreSQL repository struct + constructor boilerplate
  - `db_exec!` macro wraps pool cloning + `Box::pin(async move { ... })` pattern
  - Applied to `PgSiteRepo` and `PgTenantRepo` as examples

### Changed
- `init_token_revocation()` in `lib.rs` now uses centralized `AgroCoreConfig` instead of direct `std::env::var`
- `PostgresDb::connect()` uses `AgroCoreConfig::global()` for pool options instead of standalone functions
- `crates/messaging/Cargo.toml`: added `agrocore-shared` dependency for `with_retry` access

## [0.8.11] - 2026-08-12

### Added
- **Token Revocation System (Task 2.3a)**
  - `TokenRevocationList` struct with dual backend: Redis (when `REDIS_URL` is set) or in-memory `DashMap` with TTL
  - Added `jti` (JWT ID) claim to JWT tokens for unique identification
  - New `POST /api/v1/auth/logout` endpoint that revokes the current JWT token by adding its `jti` to the revocation list with matching TTL, and clears the server-side refresh token
  - Added `token_revocation: Arc<TokenRevocationList>` to `AppState`
  - In-memory fallback uses `RevocationMemoryStore` with `DashMap` for thread-safe revocation checks
  - Redis backend uses `SETEX` for atomic set-with-expiry, preventing stale entries
- **Differentiated Rate Limiting (Task 2.3b)**
  - Auth endpoints (`/auth/login`, `/auth/refresh`, `/auth/logout`) now have stricter rate limit: 10 requests per 60 seconds per IP
  - Other endpoints retain the default: 120 requests per minute per IP
  - Implemented via scoped `Governor` middleware wrapper on auth routes in `handlers::configure`

### Changed
- `Claims` struct in both `jwt.rs` and `middleware.rs`: added `jti: String` field
- `AuthenticatedUser` struct: added `jti: String` field
- `generate_jwt`: generates `jti` as UUID v4
- Auth endpoint tests updated with `jti` in test claim structs

## [0.8.10] - 2026-08-12

### Changed
- **Security: MQTT TLS Encryption (Task 2.2a)**
  - Enabled `use-rustls-no-provider` feature on `rumqttc` for TLS support
  - Added `tls_ca_cert`, `tls_client_cert`, `tls_client_key` fields to `MqttConfig`
  - Implemented `build_tls_config()` helper that builds a `TlsConfiguration::Simple` with CA certs
  - `MqttClient::connect()` and `attempt_reconnect()` now set `Transport::Tls(tls_config)` when `use_tls` is enabled
  - Falls back to system default TLS config with warning log on configuration errors
- **Security: Refresh Token Error Handling (Task 2.2b)**
  - `login` handler: `update_refresh_token` result now properly handled with `map_err` instead of `let _ =`
  - `refresh_token` handler: `update_refresh_token` result now properly handled with `map_err` instead of `let _ =`
  - This prevents stale/inconsistent refresh token state where a failed DB update would go unnoticed, potentially causing token mismatch between client and server

## [0.8.9] - 2026-08-12

### Changed
- **Dependency Security Upgrade (Task 2.1)**
  - Upgraded `argon2` from pre-release `0.6.0-rc.8` to stable `0.5.1`, eliminating release-candidate risk in production
  - Added `password-hash = "0.5"` as a workspace dependency
  - Added `rand` as a direct dependency to `agrocore-infrastructure` (was only available via workspace but not referenced)
  - Updated `hash_password` calls to explicitly generate and pass a `SaltString::generate(&mut rand::thread_rng())` (argon2 0.5.x requires explicit salt)
  - Updated `verify_password` to parse stored hash strings via `PasswordHash::new()` before verification (argon2 0.5.x API change)
  - All password hashing/verification in `PgUserRepo` and `initial_setup` handler adapted to the stable API

## [0.8.8] - 2026-08-12

### Added
- **Modular OpenAPI Documentation (Task 1.3b)**
  - Extracted the monolithic `#[derive(OpenApi)]` `ApiDoc` from `lib.rs` into a dedicated `openapi.rs` module
  - 13 per-module `OpenApi` structs (`AuthApiDoc`, `SitesApiDoc`, `OrdersApiDoc`, `UsersApiDoc`, `TasksApiDoc`, `WeatherApiDoc`, `FinanceApiDoc`, `ReportingApiDoc`, `LivestockApiDoc`, `SigpacApiDoc`, `SettingsApiDoc`, `IotApiDoc`, `ErrorApiDoc`)
  - Combined `ApiDoc` struct merges all modules at startup via `utoipa::OpenApi::merge()`
  - `openapi_with_security()` adds bearer-JWT security scheme at runtime
  - Improves incremental compilation: editing one module's OpenAPI spec only recompiles that module

## [0.8.7] - 2026-08-12

### Added
- **Selective DTO Validation (Task 1.3a)**
  - `IoTCommandRequestDto` now has `#[validate(length(...))]` on `command_type` and `#[validate(range(...))]` on `timeout_seconds`
  - `send_command` handler now calls `dto.validate()` before processing the command
  - Added `test_iot_command_request_dto_validation` test covering all validation scenarios

### Note
- `serde_json::Value` fields (like `payload`) cannot use `#[validate]` directly — they require `#[validate(nested)]` for struct-based validation, or are left unvalidated since arbitrary JSON payloads cannot be meaningfully validated by the `validator` crate

## [0.8.6] - 2026-08-12

### Added
- **Repository Pre-instantiation (Task 1.1c)**
  - Pre-instantiate all PostgreSQL repositories in `PostgresDb::connect()` and `from_pool()`
  - Repositories are cached as `Arc` fields in the struct, eliminating repeated `Arc::new(Repo::new(pool.clone()))` heap allocations on every method call
  - `Arc::clone` (refcount increment) replaces allocation on every repository access
  - Repository access methods (`site_repo()`, `user_repo()`, etc.) now return `self.xxx_repo.clone()` instead of `Arc::new(PgXxxRepo::new(self.pool.clone()))`

### Changed
- `PostgresDb` struct now has 33 fields: `pool` + 32 pre-instantiated repository Arcs
- `Database::tenant_repo()` now delegates to `db.tenant_repo.clone()` instead of `Arc::new(PgTenantRepo::new(db.pool.clone()))`
- Test fixtures updated to use `PostgresDb::from_pool()` instead of struct literal construction

## [0.8.5] - 2026-08-12

### Added
- **Security: CORS Configuration (Task 2.1)**
  - **Explicit CORS Whitelist**: Replaced `Cors::permissive()` with `build_cors()` function that reads `CORS_ALLOWED_ORIGINS` environment variable for a comma-separated list of allowed origins
  - Falls back to permissive mode with a warning log when `CORS_ALLOWED_ORIGINS` is not set (development convenience)
  - Production environments should set `CORS_ALLOWED_ORIGINS=https://your-domain.com` for proper origin restriction

### Changed
- Version bump: 0.8.4 → 0.8.5

## [0.8.4] - 2026-08-12

### Added
- **Database Optimization (Task 1.1)**
  - **Batch INSERTs via UNNEST**: Replaced N+1 individual INSERT statements in `PgUserRepo::update` with a single batch insert using `SELECT $1, unnest($2::uuid[])` for user_sites, reducing database round-trips from O(n) to O(1)
  - **LEFT JOIN + GROUP BY**: Replaced correlated subqueries `COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json)` with `LEFT JOIN user_sites us ON u.id = us.user_id` + `GROUP BY u.id` + `json_agg` across all user queries (find_by_id, find_by_email, find_all, authenticate, update, find_by_refresh_token), eliminating per-row subquery execution
  - **Shared SQL Constants**: Extracted the user SELECT query fragment into a `USER_SELECT_FIELDS` constant to reduce code duplication and ensure consistency across all user queries
  - **Enhanced Pool Configuration**: Added `DATABASE_ACQUIRE_TIMEOUT_SECS` (default: 30) and `DATABASE_CONNECT_TIMEOUT_SECS` (default: 10) environment variables for fine-grained connection pool timeout control, applied via URL query parameters at connection establishment time

### Changed
- Version bump: 0.8.3 → 0.8.4

## [0.8.3] - 2026-08-11

### Fixed
- **CI/CD Pipeline**: Updated GitHub Actions and GitLab CI to run tests with `--features=mocks` so integration tests compile and pass
- Both pipelines now explicitly enable the `mocks` feature for `cargo test --workspace --features=mocks`

### Changed
- Version bump: 0.8.2 → 0.8.3

## [0.8.2] - 2026-08-11

### Added
- **Performance Optimizations (Task 4 Quick Wins)**
  - **DecodingKey Caching**: Cached JWT DecodingKey in AuthExtractor middleware using OnceLock for improved auth performance
  - **PgPoolOptions Configuration**: Configurable database connection pool via environment variables (DATABASE_MAX_CONNECTIONS, DATABASE_MIN_CONNECTIONS, DATABASE_IDLE_TIMEOUT_SECS, DATABASE_MAX_LIFETIME_SECS)
  - **Repository Factory Macro**: `repo!` macro in shared crate to reduce boilerplate for repository instantiation
  - **Messaging Topic Precomputation**: Precomputed static NATS subjects as constants to avoid repeated string allocations
  - **Rollen-Mapping Optimization**: Pre-converted role strings to UserRole enums during token validation, eliminating per-call conversion overhead
- **Mock Infrastructure Fixes**
  - Fixed mock feature flag propagation across domain, infrastructure, and postgres crates
  - Mock types now properly generated and exported when `mocks` feature is enabled
  - Updated all `#[cfg(any(test, feature = "mocks"))]` to `#[cfg(feature = "mocks")]` for consistent feature gating

### Changed
- Version bump: 0.8.1 → 0.8.2
- Mock feature now properly includes mockall dependency

### Fixed
- Infrastructure tests now compile and run with mock features enabled
- Domain crate mock types (MockSiteRepository, MockUserRepository, etc.) now properly available
- Consistent feature gating across workspace for mock functionality

## [0.8.1] - 2026-08-11

### Changed
- Increased version to 0.8.1 after verifying codebase with cargo fmt, check, test, and clippy.
- Fixed infrastructure tests to work with mock features.

## [0.8.0] - 2026-08-09

### Added
- **Full Integration Test Suite for API & Domain**
  - Implemented 10+ new integration test suites using mock repositories and messaging.
  - Added comprehensive coverage for Workforce, Compliance (Checklists, Audit, Plant Protection), Specialized Crops (Olives), Harvest Logistics (Seasons, Lots, Deliveries, Cold Chain), Livestock, Finance (PAC, Cost Centers, Records), Sites, Equipment, and Weather modules.
  - Verification of tenant-scoping, authorization, and DTO mappings across all major modules.
- **Enhanced Mocking Infrastructure**
  - Boxed `MockDatabase` variant to optimize memory layout and satisfy Clippy.
  - Added `set_mock_response` to `MessagingClient` for configurable request/response testing (NATS simulation).
  - Updated all integration tests to utilize the new optimized mock infrastructure.

### Fixed
- API: Implemented missing CRUD handlers for Plant Protection records.
- API: Fixed `PaginatedResponseDto` to support deserialization in tests.
- API: Fixed `Equipment` list handler to correctly utilize repository methods.

## [0.7.9] - 2026-08-09

### Fixed
- API: the workforce handlers now pass `&[UserRole]` to the repositories (instead of `&Vec<String>`), so the visibility/authorisation filters compile again and actually apply.
## [0.7.8] - 2026-08-09

### Added
- Added API route-registration coverage for workforce, compliance, finance, PAC, harvest, livestock, weather, and olive modules.

### Changed
- Kept modules with remaining CRUD and authorization scenarios marked Nearly done in `tasks.md`.

## [0.7.7] - 2026-08-09

### Added
- Completed tenant-scoped cost-center and financial-record update/delete persistence.
- Added financial-record cost-center filtering with pagination.

### Changed
- Completed Module 14 and marked Finanzen: Kostenstellen production-ready.

## [0.7.6] - 2026-08-09

### Added
- Completed tenant-scoped weather-data and phenology-record update/delete persistence.

### Changed
- Completed module 12 and marked weather & phenology production-ready.
## [0.7.5] - 2026-08-09

### Added
- Completed tenant-scoped water-source and water-quota repository operations.
- Added site/source filtering, pagination, quota balance initialization, and CRUD persistence.

### Changed
- Completed Module 9 and marked Wasser production-ready.

## [0.7.4] - 2026-08-09

### Added
- Completed vineyard site filtering and specialized vineyard CRUD routes.
- Added migration support for vineyard soft deletion.
- Added vineyard route integration coverage.

### Changed
- Completed Module 7 and marked Weinbau production-ready.

## [0.7.3] - 2026-08-09

### Added
- Completed TaskData update and delete repository operations.
- Added the migration columns required by the TaskData domain model.
- Added task-route integration coverage.

### Changed
- Completed module 2 and marked orders & tasks production-ready.
## [0.7.2] - 2026-08-09

### Added
- PostgreSQL-backed IoT device persistence with tenant-scoped CRUD.
- IoT route registration coverage in the API integration tests.

### Changed
- Completed Module 16 and marked IoT & Messaging production-ready.

## [0.7.1] - 2026-08-09

### Added
- Registered the IoT device API and its OpenAPI documentation.
- Added shared IoT device state to `AppState` so device registrations survive across requests.
- Added MQTT connection health tracking and health-monitoring lifecycle controls.

### Fixed
- Restored workspace compilation after the MQTT health-monitoring changes.
- Corrected IoT role validation, DTO parsing, and API error handling.
- Cleaned up workspace formatting and clippy findings.

## [0.7.0] - 2026-08-08

### Added
- **MQTT Support for IoT Devices & Home Assistant Integration**
  - `rumqttc 0.25` dependency with async MQTT client
  - `MqttConfig`: broker settings, TLS, authentication, topic prefix
  - IoT event types: `IoTTelemetryEvent`, `IoTDeviceStatusEvent`, `IoTCommandEvent`
  - `IoTCapability` enum: Temperature, Humidity, SoilMoisture, Light, GPS, BatteryLevel, SignalStrength, ActuatorControl, FirmwareUpdate, Custom
  - `MqttClient`: async connect, publish_telemetry, publish_status (retained), subscribe_commands/broadcast, event loop
  - `UnifiedMessagingClient`: dual NATS + MQTT backend with unified publish API
  - Topic structure: `agrocore/telemetry/{tenant}/{device}`, `agrocore/status/{tenant}/{device}`, `agrocore/commands/{tenant}/{device}`

- **Home Assistant MQTT Auto-Discovery**
  - `HaSensorConfig`, `HaBinarySensorConfig`, `HaButtonConfig`, `HaNumberConfig`, `HaDeviceInfo`
  - Capability mapping with device_class, unit_of_measurement, icons, value_templates
  - `generate_ha_discovery_configs()` for complete device payloads
  - Availability binary sensors with connectivity device_class

- **Mosquitto MQTT Broker in docker-compose**
  - `eclipse-mosquitto:2.0` on ports 1883 (MQTT) and 9001 (WebSockets)
  - TLS certificates (CA, server, client) + PKCS12 for Home Assistant
  - Password-based authentication
  - Health checks via `mosquitto_sub`

- **CI/CD Pipeline Fixes**
  - PostgreSQL service with sqlx migrations in GitHub Actions
  - Security audit with `continue-on-error: true`

### Changed
- **Version bump: 0.6.0 → 0.7.0**
- **Vulnerability fixes**: quick-xml 0.31→0.41, async-nats 0.38→0.50, rand 0.10→0.8, wiremock 0.5→0.6

### Fixed
- Clippy collapsible_if warnings in messaging crate
- MQTT borrow checker issue with state_topic clone

## [0.6.0] - 2026-08-05

### Added
- **Complete Test Suite for LPIS Providers & Settings**
  - 13 config tests (ProviderConfig, LpisProvidersConfig, CacheConfig, RateLimitConfig, RetryConfig, BaseClient creation, rate limiting)
  - 3 BaseClient integration tests with mock server (execute_request, get_cached_or_fetch, rate_limiting)
  - 4 Settings integration tests (route configuration, config serialization roundtrip)
  - 3 config serialization tests (LpisProviderConfig, LpisProvidersConfig, CacheBackend enum)
- **All 8 LPIS Providers migrated to BaseClient** (SIGPAC/ES, BRP/NL, RPG/FR, iLPIS/PT, SIAN/IT, LPIS-DE, LPIS-PL, INVEKOS/AT)
  - Unified caching, rate limiting, and retry logic across all providers
- **Settings API & UI complete** with persistence
  - Config file load/save (`load_from_path`, `save_to_path`)
  - Route configuration verified in tests

### Changed
- **Version bump: 0.5.9 → 0.6.0**
- **Test infrastructure** significantly expanded (13+ new tests)
- **Quality Gates** enforced in development workflow (fmt, check, test, clippy)

### Fixed
- `LpisProvidersConfig::load()` now correctly reads `config/lpis-providers.toml`
- `BaseClient` mock server tests use wiremock for reliable HTTP testing
- Config serialization tests cover roundtrip for all DTOs

## [0.5.9] - 2026-08-05

### Added
- **Multi-country LPIS Provider Base Client** with unified caching, rate limiting, and retry logic
  - New `BaseClient` in `lpis-providers` with HTTP client, `LpisCache` (Memory/Redis), `governor` rate limiting, exponential backoff retries
  - All 8 providers (ES, NL, FR, PT, IT, DE, PL, AT) can now use common infrastructure
  - BRP provider fully migrated to base client pattern
- **Admin UI: LPIS Country Selection in Data Import**
  - DataImport component shows country flags (ES, NL, FR, PT, IT, DE, PL, AT) for 8 LPIS sources
  - Click country flag opens provider-specific import modal with configuration fields
  - `CountrySelect` component with flag dropdown and search
  - `ProviderConfig` fields mapped to UI inputs (endpoint URL, cache backend, rate limit, retry config)

### Changed
- Version bump: 0.5.8 → 0.5.9
- Refactored all LPIS providers to use `BaseClient` for unified infrastructure

### Fixed
- `BaseClient` now correctly handles cache key generation with tenant_id prefix
- Fixed `LpisCache` Memory backend to use `HashMap<String, Vec<u8>>` with timestamp-based expiry
- `governor` rate limiting now correctly applies `per_second` limit instead of default burst
- Fixed `async-trait` usage in provider trait methods

## [0.5.8] - 2026-08-04

### Added
- **GeoJSON & Spatial Data Processing** (Task 3.7.4)
  - `geojson` crate integration across all LPIS providers
  - GeoJSONFeature/FeatureCollection types for parcel boundary data
  - Spatial intersection utilities for overlap detection
  - PostGIS integration for area calculations and spatial queries

## [0.5.7] - 2026-08-04

### Added
- **NATS Messaging Integration**
  - `async-nats` client with connection management and auto-reconnect
  - Message types: `TelemetryEvent`, `DeviceStatusEvent`, `CommandEvent`
  - `UnifiedMessagingClient` trait with NATS and mock implementations
  - Subjects: `agrocore.telemetry.{tenant}`, `agrocore.status.{tenant}`, `agrocore.commands.{tenant}`

## [0.5.6] - 2026-08-03

### Added
- **PostgreSQL Database Layer**
  - `PgSiteRepo`, `PgTenantRepo`, `PgWorkerRepo`, `PgOrderRepo`
  - sqlx with connection pooling via `PgPool`
  - Migrations for all entity tables
  - `Database` enum with `Postgres` and `Mock` variants

## [0.5.5] - 2026-08-02

### Added
- **Actix-web REST API Server**
  - JWT authentication middleware with role-based access control
  - OpenAPI/Swagger documentation via utoipa
  - Rate limiting with `actix-governor` (120 req/min default, 10 req/60s for auth)
  - CORS configuration
  - Error handling with `ApiError` enum

### Changed
- Version bump: 0.5.4 → 0.5.5

## [0.5.4] - 2026-08-01

### Added
- **Admin UI (Leptos 0.8)** — WASM SPA with sidebar navigation
  - Dashboard, Sites, Orders, Workers, Equipment, Inventory pages
  - Authentication flow with login page
  - Responsive layout with mobile drawer

### Changed
- Version bump: 0.5.3 → 0.5.4

## [0.5.3] - 2026-07-30

### Added
- **Domain Layer** — Core entities with validation
  - `Tenant`, `Site`, `Worker`, `Order`, `TaskData`, `User`
  - Validation traits using `validator` crate
  - Enum types: `OrderType`, `OrderStatus`, `UserStatus`
  - GeoJSON geometry types: `GeoPoint`, `GeoPolygon`, `GeoMultiPolygon`

### Changed
- Version bump: 0.5.2 → 0.5.3

## [0.5.2] - 2026-07-28

### Added
- **Shared Kernel Crate** — Common types and utilities
  - `SharedError` enum (Validation, NotFound, Internal, Network, Auth)
  - Pagination types: `PaginatedRequest`, `PaginatedResponse<T>`
  - Auth utilities: JWT secret, token generation
  - Database pool configuration helpers
  - `with_retry()` generic retry logic

### Changed
- Version bump: 0.5.1 → 0.5.2

## [0.5.1] - 2026-07-25

### Added
- Initial Rust workspace structure with Cargo workspace
  - 11 crates: api, domain, infrastructure, shared, lpis-providers, asset-registry, weather-service, geometry-service, reporting-service, messaging, admin-ui
  - Shared dependencies: actix-web 4, sqlx, serde, tokio, async-nats, rumqttc, geojson
  - All crates use Rust 2024 edition

### Changed
- Version bump: 0.5.0 → 0.5.1 (metadata cleanup)

## [0.5.0] - 2026-07-20

### Added
- **Initial Release**
  - Rust 2024 workspace with agrocore-rs
  - PostgreSQL with PostGIS extension
  - NATS messaging integration
  - MQTT broker with Home Assistant auto-discovery
  - Admin UI (WASM via Leptos)
  - Actix-web REST API with JWT auth
  - Multi-country LPIS providers (8 countries)
  - Inventory management with FIFO/FEFO
  - Clock-in/out with GPS coordinates

## [Unreleased]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.9.3...HEAD
## [0.9.3]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.9.2...v0.9.3
## [0.9.2]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.9.1...v0.9.2
## [0.9.1]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.9.0...v0.9.1
## [0.9.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.9.0
## [0.8.22]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.21...v0.8.22
## [0.8.16]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.15...v0.8.16
## [0.8.15]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.14...v0.8.15
## [0.8.14]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.13...v0.8.14
## [0.8.13]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.12...v0.8.13
## [0.8.11]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.10...v0.8.11
## [0.8.10]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.9...v0.8.10
## [0.8.9]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.8...v0.8.9
## [0.8.8]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.7...v0.8.8
## [0.8.7]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.6...v0.8.7
## [0.8.6]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.5...v0.8.6
## [0.8.5]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.4...v0.8.5
## [0.8.4]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.3...v0.8.4
## [0.8.3]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.2...v0.8.3
## [0.8.2]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.1...v0.8.2
## [0.8.1]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.0...v0.8.1
## [0.8.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.8.0
## [0.7.9]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.8...v0.7.9
## [0.7.8]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.7...v0.7.8
## [0.7.7]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.6...v0.7.7
## [0.7.6]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.5...v0.7.6
## [0.7.5]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.4...v0.7.5
## [0.7.4]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.3...v0.7.4
## [0.7.3]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.2...v0.7.3
## [0.7.2]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.1...v0.7.2
## [0.7.1]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.0...v0.7.1
## [0.7.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.7.0
## [0.6.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.6.0
## [0.5.9]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.8...v0.5.9
## [0.5.8]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.7...v0.5.8
## [0.5.7]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.6...v0.5.7
## [0.5.6]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.5...v0.5.6
## [0.5.5]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.4...v0.5.5
## [0.5.4]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.3...v0.5.4
## [0.5.3]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.2...v0.5.3
## [0.5.2]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.1...v0.5.2
## [0.5.1]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.0...v0.5.1
## [0.5.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.5.0
