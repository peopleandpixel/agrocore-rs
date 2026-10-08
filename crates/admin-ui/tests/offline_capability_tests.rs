//! The Admin UI must start without a network, and the pieces that make that true
//! must keep existing.
//!
//! What was wrong, in order of severity:
//!
//! 1. Nothing registered the service worker. `public/sw.js` was 6,924 bytes of
//!    complete strategy — cache-first for static files, a network-first fallback for
//!    API calls, background sync for task updates — and `serviceWorker.register`
//!    appeared nowhere in the repository. None of it ran.
//! 2. Every stylesheet and script came from a public CDN. The document arrived, the
//!    styles and the map library did not, and Chrome reported
//!    `net::ERR_INTERNET_DISCONNECTED` for the page itself.
//! 3. The worker would have failed on its first registration anyway:
//!    - `STATIC_ASSETS` named `/styles/tailwind.css` and `/styles/daisyui.css`, which
//!      do not exist — and `cache.addAll` rejects the whole install if any URL fails;
//!    - it tried to cache two API endpoints during install, where an unauthenticated
//!      request answers 401 and `addAll` rejects non-2xx as well;
//!    - `manifest.json` declared `/icons/icon-*.png`, and that directory did not exist.
//! 4. Both offline pages loaded Tailwind from the CDN — the pages that exist only for
//!    the no-network case could not render in it.
//!
//! The tests below are source and filesystem assertions. They cannot drive a browser,
//! which is the honest limit here: the actual proof is loading the page with the
//! network off. What they do catch is regression of the things that make that
//! possible — a vendored file that goes missing, a CDN reference creeping back in, or
//! the registration call being removed.

use std::path::{Path, PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn public_dir() -> PathBuf {
    ui_dir().join("public")
}

/// Every `src`/`href` in the three served documents must resolve.
///
/// Skips `data-trunk` attributes, which Trunk resolves at build time rather than
/// serving as written.
#[test]
fn every_referenced_asset_exists_and_is_local() {
    let pages = [
        ("index.html", ui_dir().join("index.html")),
        ("offline.html", public_dir().join("offline.html")),
        (
            "worker-tasks-offline.html",
            public_dir().join("worker-tasks-offline.html"),
        ),
    ];

    for (name, path) in pages {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {name}: {e}"));
        let dir = path.parent().expect("page has a parent").to_path_buf();

        for attr in ["src", "href"] {
            let needle = format!("{attr}=\"");
            let mut rest = text.as_str();
            while let Some(at) = rest.find(&needle) {
                let after = &rest[at + needle.len()..];
                let Some(end) = after.find('"') else {
                    break;
                };
                let value = &after[..end];
                rest = &after[end + 1..];

                if value.starts_with("http://") || value.starts_with("https://") {
                    panic!(
                        "{name} references {value} over the network; the UI cannot \\
                         start offline with a CDN reference"
                    );
                }
                if value.starts_with("data-trunk") || value.is_empty() {
                    continue;
                }

                let target = if let Some(rest) = value.strip_prefix('/') {
                    public_dir().join(rest)
                } else {
                    dir.join(value)
                };

                assert!(
                    target.exists(),
                    "{name} references {value}, which does not exist at {}",
                    target.display()
                );
            }
        }
    }
}

/// The service worker has to be registered by something. Its absence was the single
/// reason a complete offline implementation never took effect.
#[test]
fn the_service_worker_is_registered() {
    let main_rs = std::fs::read_to_string(ui_dir().join("src/main.rs")).expect("read main.rs");

    assert!(
        main_rs.contains("service_worker()") && main_rs.contains("register("),
        "src/main.rs must register the service worker; without it sw.js never runs"
    );
    // Registration must not be able to stop the application from starting. Matched
    // inside `fn main` only: the doc comment mentions `mount_to_body` earlier in the
    // file, so a whole-file search finds the wrong occurrence and compares two
    // positions that mean nothing.
    let main_body = main_rs
        .split("fn main()")
        .nth(1)
        .expect("main.rs has a main function");
    let mount = main_body
        .find("mount_to_body")
        .expect("main() mounts the app");
    let register = main_body
        .find("register_service_worker()")
        .expect("main() registers the worker");
    assert!(
        mount < register,
        "the app must be mounted before the worker is registered, so a registration \\
         failure cannot prevent the UI from starting"
    );
}

/// The worker's precache list must name files that exist, and must not try to cache
/// API endpoints during install.
///
/// `cache.addAll` rejects the entire install when any URL is missing or answers
/// non-2xx, so one wrong entry here means no service worker at all — which is what
/// the previous list did.
#[test]
fn the_precache_list_is_servable() {
    let sw = std::fs::read_to_string(public_dir().join("sw.js")).expect("read sw.js");

    let block = sw
        .split("const SHELL_ASSETS = [")
        .nth(1)
        .and_then(|rest| rest.split("];").next())
        .expect("SHELL_ASSETS must exist");

    let mut checked = 0;
    let mut rest = block;
    while let Some(at) = rest.find('\'') {
        let after = &rest[at + 1..];
        let Some(end) = after.find('\'') else { break };
        let url = &after[..end];
        rest = &after[end + 1..];

        assert!(
            !url.starts_with("/api/"),
            "{url} is an API endpoint in the precache list; an unauthenticated \\
             request answers 401, `addAll` rejects it, and the worker never installs"
        );
        if url == "/" || url == "/index.html" {
            // Emitted by Trunk from the crate root, not copied from `public/`.
            continue;
        }
        let target = public_dir().join(url.trim_start_matches('/'));
        assert!(
            target.exists(),
            "SHELL_ASSETS names {url}, which does not exist; `precacheAll` skips it \\
             but the shell is incomplete offline"
        );
        checked += 1;
    }
    assert!(
        checked >= 6,
        "only {checked} precache entries were verifiable"
    );
}

/// The vendored files are the reason the UI starts offline. Their absence, or a
/// reference back to a CDN, and the page is unstyled.
#[test]
fn the_vendored_assets_are_present() {
    for name in [
        "daisyui.css",
        "tailwind.js",
        "leaflet.css",
        "leaflet-draw.css",
        "leaflet.js",
        "leaflet-draw.js",
        "maplibre-gl.js",
        "maplibre-gl.css",
    ] {
        let path = public_dir().join("vendor").join(name);
        assert!(path.exists(), "vendor/{name} is missing");
        assert!(
            path.metadata().expect("stat").len() > 1000,
            "vendor/{name} is implausibly small — the download probably failed"
        );
    }

    // Leaflet resolves its marker icons relative to the script, so they travel with it.
    for icon in [
        "marker-icon.png",
        "marker-icon-2x.png",
        "marker-shadow.png",
        "marker-icon-red.png",
        "layers.png",
        "layers-2x.png",
    ] {
        assert!(
            public_dir().join("vendor/images").join(icon).exists(),
            "vendor/images/{icon} is missing; Leaflet requests it by a bare filename"
        );
    }
}

/// The map code must not reach for a remote icon.
///
/// `leaflet.js` referenced the red location marker from `raw.githubusercontent.com`
/// and its shadow from `cdnjs.cloudflare.com` — not reachable without a network.
#[test]
fn the_map_javascript_has_no_remote_assets() {
    let js = std::fs::read_to_string(ui_dir().join("src/leaflet.js")).expect("read leaflet.js");

    for forbidden in ["marker-icon-red.png", "marker-shadow.png"] {
        for line in js.lines() {
            if line.contains(forbidden) && line.contains("http") {
                panic!(
                    "src/leaflet.js loads {forbidden} over the network: {}",
                    line.trim()
                );
            }
        }
    }
    // The tile server is a product decision, not a startup dependency, so it stays.
    assert!(
        js.contains("openstreetmap.org"),
        "the tile server reference should still be there; only the icons were vendored"
    );
}

/// `manifest.json` is fetched by the install step, so an icon it names must exist.
#[test]
fn the_manifest_only_names_icons_that_exist() {
    let raw = std::fs::read_to_string(public_dir().join("manifest.json")).expect("read manifest");

    for line in raw.lines() {
        let Some(rest) = line.split("\"src\": \"").nth(1) else {
            continue;
        };
        let Some(end) = rest.find('"') else { continue };
        let src = &rest[..end];
        assert!(
            public_dir().join(src.trim_start_matches('/')).exists(),
            "manifest.json names {src}, which does not exist"
        );
    }
}

/// The worker must not cache first for navigations.
///
/// A cache-first shell is served from cache until the cache is cleared by hand, which
/// means a deployed fix does not reach a client that has the application open. The
/// previous version did exactly this for every non-API request.
#[test]
fn navigations_are_network_first() {
    let sw = std::fs::read_to_string(public_dir().join("sw.js")).expect("read sw.js");

    assert!(
        sw.contains("request.mode === 'navigate'"),
        "sw.js must handle navigations explicitly"
    );
    let nav = sw.find("navigationHandler").expect("a navigation handler");
    let handler = &sw[nav..];
    assert!(
        handler.contains("fetch(request)"),
        "the navigation handler must try the network first"
    );
    assert!(
        handler.contains("index.html"),
        "the navigation handler must fall back to the shell so a deep link resolves"
    );
}

/// Cache names must be versioned, and activation must delete the old ones.
///
/// Without this, a client that was offline during a deploy keeps the previous build
/// indefinitely — which is the failure the cache-first strategy had.
#[test]
fn caches_are_versioned_and_old_ones_removed() {
    let sw = std::fs::read_to_string(public_dir().join("sw.js")).expect("read sw.js");

    assert!(
        sw.contains("const VERSION"),
        "sw.js must carry a cache version"
    );
    assert!(
        sw.contains("caches.delete"),
        "activation must delete caches that are not in the current version set"
    );
    assert!(
        sw.contains("KEEP"),
        "the caches to keep must be named, or deletion would remove the live ones"
    );
}

#[test]
fn the_service_worker_is_valid_javascript() {
    let sw = public_dir().join("sw.js");
    assert!(sw.exists(), "sw.js must exist");

    // Cheap structural checks, not a parse: `node --check` is not available in every
    // environment this runs in, but an unbalanced brace count catches the truncation
    // that actually happened while editing this file.
    let text = std::fs::read_to_string(&sw).expect("read sw.js");
    let opens = text.matches('{').count();
    let closes = text.matches('}').count();
    assert_eq!(
        opens, closes,
        "sw.js has unbalanced braces: {opens} open, {closes} close"
    );
    assert!(
        text.contains("addEventListener('install'")
            && text.contains("addEventListener('activate'")
            && text.contains("addEventListener('fetch'"),
        "sw.js must handle install, activate and fetch"
    );
    let _ = Path::new("/");
}

/// The optimisation step must stay switched off, or the Docker build fails.
///
/// Not an offline concern directly — it is here because the same HTML file carries
/// the attribute and the failure mode is identical to the CDN one: a silent
/// regression that only shows up in a Docker build ten minutes in.
///
/// `data-wasm-opt="0"` is the control; Trunk has no `build.wasm_opt` setting at all,
/// and a `wasm_opt = false` in Trunk.toml is accepted and ignored. Binaryen
/// `version_116` exits 1 on the module this toolchain produces. wasm-opt is a size
/// optimisation, not a correctness one, so skipping it is preferable to shipping
/// nothing. Recorded as O3 in docs/tasks.md.
#[test]
fn the_wasm_optimisation_step_is_switched_off() {
    let html = std::fs::read_to_string(ui_dir().join("index.html")).expect("read index.html");

    let rust_link = html
        .lines()
        .find(|l| l.contains("data-trunk rel=\"rust\""))
        .expect("index.html has a <link data-trunk rel=\"rust\">");

    assert!(
        rust_link.contains("data-wasm-opt=\"0\""),
        "the rust link must carry data-wasm-opt=\"0\"; without it a release build \
         runs wasm-opt, Binaryen version_116 exits 1, and the Docker build fails"
    );

    // The TOML trap: a key that looks right and does nothing.
    let trunk_toml = ui_dir().join("Trunk.toml");
    if trunk_toml.exists() {
        let toml = std::fs::read_to_string(&trunk_toml).expect("read Trunk.toml");
        assert!(
            !toml.contains("wasm_opt"),
            "Trunk.toml sets wasm_opt, which Trunk ignores in [build] and reads as a \
             version in [tools] — the control is the data-wasm-opt attribute"
        );
    }
}

/// Everything under `public/` must be reachable from Trunk's output.
///
/// Trunk copies nothing from `public/` on its own. A directory or file only lands in
/// `dist/` when the HTML declares it with `rel="copy-dir"` or `rel="copy-file"`.
///
/// This was found by building the image and looking inside it: the WASM bundle was
/// there, and `sw.js`, `manifest.json`, the icons and the whole `vendor/` tree were
/// all in the repository and all absent from the artefact. Nine tests and 12 vendored
/// files passed, the build was green, and the offline capability was not in the image
/// at all. Every assertion in this file would have passed anyway — they read the
/// repository, not the artefact.
#[test]
fn public_assets_are_declared_for_copying() {
    let html = std::fs::read_to_string(ui_dir().join("index.html")).expect("read index.html");

    let copied: Vec<&str> = html
        .lines()
        .filter(|l| {
            l.contains("data-trunk rel=\"copy-dir\"") || l.contains("data-trunk rel=\"copy-file\"")
        })
        .filter_map(|l| l.split("href=\"").nth(1))
        .filter_map(|l| l.split('"').next())
        .collect();

    // The two directories that hold the bulk of it.
    for dir in ["public/vendor", "public/icons"] {
        assert!(
            copied.contains(&dir),
            "index.html must declare {dir} for copying; Trunk copies nothing from \
             public/ on its own, so without this the directory is missing from the \
             built image"
        );
        assert!(
            ui_dir().join(dir).is_dir(),
            "{dir} is declared but does not exist"
        );
    }

    // The root-level files the service worker precaches or that the manifest needs.
    for file in [
        "public/sw.js",
        "public/manifest.json",
        "public/offline.html",
    ] {
        assert!(
            copied.contains(&file),
            "index.html must declare {file} for copying; the service worker cannot \
             install without it and the shell cannot start offline"
        );
        assert!(
            ui_dir().join(file).exists(),
            "{file} is declared but does not exist"
        );
    }
}
