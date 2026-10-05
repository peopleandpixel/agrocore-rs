use admin_ui::App;

/// Registers the service worker.
///
/// This was never called, which is why `public/sw.js` was dead code: a service
/// worker has to be registered explicitly and nothing in the application did it. The
/// file was 6,924 bytes of complete strategy — cache-first for static files, a
/// network-first fallback for API calls, background sync for task updates — and none
/// of it ran.
///
/// Three details that are easy to get wrong and were:
///
/// * Registration happens *after* the mount. If it ran before, a failure during
///   registration — a browser without service worker support, an insecure context,
///   a 404 — could stop the application from starting at all. The offline capability
///   is an enhancement; the application must work without it.
/// * The result is ignored on purpose. A rejected promise here means "no offline
///   support on this browser", which is not an application error and has no business
///   reaching the user as one.
/// * `sw.js` is served from the root of the scope it should control. Registering
///   `/sw.js` means the worker controls `/`, which is what the navigation handler in
///   `sw.js` assumes when it falls back to `/index.html`.
fn register_service_worker() {
    let Some(win) = web_sys::window() else {
        return;
    };

    // `register` returns a promise directly rather than a `Result`, so the only
    // failure mode to handle is the promise itself rejecting.
    let promise = win.navigator().service_worker().register("/sw.js");

    leptos::task::spawn_local(async move {
        // A rejected registration means "no offline support here" — an older browser,
        // an insecure context, or a 404. It is logged and swallowed: the offline
        // capability is an enhancement, and surfacing it as an error would make a
        // working online session look broken.
        let outcome = wasm_bindgen_futures::JsFuture::from(promise).await;
        if let Err(e) = outcome {
            web_sys::console::warn_1(
                &format!("AgroCore: service worker registration failed ({e:?})").into(),
            );
        }
    });
}

fn main() {
    leptos::mount::mount_to_body(App);
    register_service_worker();
}
