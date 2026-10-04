//! A failed fetch must not render as an empty list.
//!
//! This is the first page converted from the `.await.ok()` pattern that turned a
//! failed fetch into `None`, and `None` was indistinguishable from "this tenant has
//! nothing". An order list that could not be fetched rendered as an empty table.
//!
//! These tests cannot drive a browser, so they assert the source property that
//! makes the three states distinguishable: the page must branch on a load error
//! *before* it branches on an empty list, and must offer a retry. A page that checks
//! emptiness first cannot distinguish a failed fetch from an empty tenant no matter
//! how the data is stored.
//!
//! The behavioural proof is the ordering assertion below — `error_before_empty` is
//! the property that matters, and it is asserted on the render closure rather than
//! on a comment about it.

use std::path::PathBuf;

fn orders_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/components/orders.rs");
    std::fs::read_to_string(&path).expect("read orders.rs")
}

/// The page must not discard a fetch error.
///
/// A test on the source rather than on behaviour, and that is the honest limit of
/// what is checkable without a browser harness: the assertion is that the pattern
/// which caused the defect is absent, not that the rendering works. The ordering
/// test below covers the part that can actually go wrong silently.
#[test]
fn the_page_does_not_discard_fetch_errors() {
    let src = orders_source();
    let code: String = src
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !code.contains(".await.ok()"),
        "orders.rs still contains `.await.ok()`, which turns a failed fetch into None"
    );
}

/// The load error is threaded through to the view rather than dropped at the fetch.
#[test]
fn the_page_tracks_a_load_error_separately_from_the_data() {
    let src = orders_source();

    assert!(
        src.contains("set_load_error"),
        "the page must store the fetch error, not only the data"
    );
    assert!(
        src.contains("Err(e) =>") && src.contains("set_load_error.set(Some(e))"),
        "the fetch's Err arm must record the error"
    );
}

/// The error branch is reached before the empty branch.
///
/// This is the property the defect defeated. With the empty check first, a failed
/// fetch and a tenant with no orders both render the empty state, and no amount of
/// separate error storage helps — the error simply never gets shown.
#[test]
fn error_before_empty() {
    let src = orders_source();

    let error_branch = src.find("else if let Some(err) = load_error.get()");
    let empty_branch = src.find("is_empty()");

    assert!(
        error_branch.is_some(),
        "the page must branch on the load error"
    );
    assert!(
        empty_branch.is_some(),
        "the page must still have an empty state"
    );
    assert!(
        error_branch < empty_branch,
        "the error branch (char {error_branch:?}) must precede the empty branch \
         (char {empty_branch:?}); otherwise a failed fetch renders as an empty list"
    );
}

/// A failed fetch must not be able to look like loaded-but-empty data.
///
/// `orders` is `None` until a fetch succeeds and `None` again on failure, so the
/// empty branch can only be reached with a value that actually came from the server.
/// If the page had left the previous data in place on failure, the two states would
/// look identical here too.
#[test]
fn a_failed_fetch_clears_the_data_so_empty_stays_honest() {
    let src = orders_source();

    // Anchor on the load-error signal, not on the first `Err(e)` in the file: the
    // delete handler has one too, and matching that would test the wrong thing.
    let at = src
        .find("set_load_error.set(Some(e))")
        .expect("the load error must be recorded");
    // The clear sits immediately before the record, so look backwards from the
    // record rather than forwards. Searching forwards found the next handler's code
    // and reported a defect that was not there.
    let from = at.saturating_sub(120);
    let segment = &src[from..at + 120];

    assert!(
        segment.contains("set_orders.set(None)"),
        "the Err arm must clear the data as well as recording the error, or a stale \
         list would be shown as if it were current"
    );
}

/// A worker in the field needs to be able to try again without navigating away.
#[test]
fn there_is_a_retry() {
    let src = orders_source();

    assert!(
        src.contains("on_retry"),
        "the page must have a retry handler"
    );
    assert!(
        src.contains("on:click=on_retry"),
        "the retry handler must be reachable from the error branch"
    );
    assert!(
        src.contains("orders_load_failed"),
        "the error branch needs a message; a bare error string is not an explanation"
    );
}

/// The three new i18n keys exist in every supported language, so the page does not
/// render raw key names in nine of ten locales.
#[test]
fn the_new_keys_are_translated() {
    let locale = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("locales/app.yml");
    let text = std::fs::read_to_string(&locale).expect("read app.yml");

    for key in ["orders_load_failed", "retry", "no_orders_yet"] {
        let at = text
            .find(&format!("\n{key}:\n"))
            .unwrap_or_else(|| panic!("missing key {key}"));
        // Walk the lines: a key's block is every following line that is indented.
        // Slicing by offsets found the first translation rather than the end of the
        // block, which made every key look untranslated.
        let block: String = text[at..]
            .lines()
            // `at` points at the newline before the key, so `lines()` yields an
            // empty first element and the key itself. Both must be skipped before
            // the translations start.
            .skip(2)
            .take_while(|l| l.starts_with("  ") || l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        // `join` drops the newline that separated the key from its first
        // translation, and the assertion below looks for one. Prefix it back rather
        // than weakening the pattern to `"{lang}: "`, which would also match
        // `de:` inside a longer key name.
        let block = format!("\n{block}");

        for lang in ["en", "de", "es", "fr", "pt", "it", "pl", "ro", "uk", "nl"] {
            assert!(
                block.contains(&format!("\n  {lang}: ")),
                "{key} is missing the {lang} translation"
            );
        }
    }
}
