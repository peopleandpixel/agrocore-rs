//! Server-backed settings UI (tasks.md H1).
//!
//! Separate module rather than more of `settings.rs`: the existing page is a
//! static display plus a company-profile form that only ever wrote to the
//! browser's localStorage, so its settings were per-device and the backend never
//! learned what was configured. Everything here persists through
//! `/api/v1/settings` and is tenant-scoped by row-level security.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::SettingDto;
use crate::components::toast::{ToastContext, ToastType};

fn toast_ok(msg: String) {
    use_context::<ToastContext>()
        .expect("ToastContext not provided")
        .add_toast
        .run((msg, ToastType::Success));
}

fn toast_err(msg: String) {
    use_context::<ToastContext>()
        .expect("ToastContext not provided")
        .add_toast
        .run((msg, ToastType::Error));
}

/// One setting, rendered as the input its declared type calls for.
///
/// `value_type` drives the widget rather than a hand-written field per key, so
/// a setting added to the database appears with a usable editor without any UI
/// change.
#[component]
fn SettingField(
    key: String,
    value: serde_json::Value,
    value_type: String,
    label: String,
    description: Option<String>,
    is_default: bool,
) -> impl IntoView {
    let as_text = match &value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    };

    let input_value = RwSignal::new(as_text);
    let checked = RwSignal::new(matches!(value, serde_json::Value::Bool(true)));

    // Handlers are named closures outside the view: an inline `move |_|` inside
    // `view!` is `FnOnce`, which the view rejects.
    let save_key = key.clone();
    let on_save = Callback::new(move |val: serde_json::Value| {
        let key = save_key.clone();
        spawn_local(async move {
            match crate::api::save_setting(&key, val).await {
                Ok(_) => toast_ok(format!("{} gespeichert", key)),
                Err(e) => toast_err(format!("{}: {}", key, e)),
            }
        });
    });

    let on_text = Callback::new(move |ev: leptos::ev::Event| {
        input_value.set(event_target_value(&ev));
    });

    let on_change = {
        let value_type = value_type.clone();
        Callback::new(move |_: ()| {
            let raw = input_value.get();
            let parsed = match value_type.as_str() {
                "number" => raw
                    .trim()
                    .parse::<f64>()
                    .map(serde_json::Value::from)
                    .unwrap_or(serde_json::Value::Null),
                _ => serde_json::Value::String(raw),
            };
            on_save.run(parsed);
        })
    };

    let on_toggle = Callback::new(move |_: ()| {
        let next = !checked.get();
        checked.set(next);
        on_save.run(serde_json::Value::Bool(next));
    });

    let on_reset = {
        let key = key.clone();
        Callback::new(move |_: ()| {
            let key = key.clone();
            spawn_local(async move {
                match crate::api::reset_setting(&key).await {
                    Ok(_) => toast_ok(format!("{} zurückgesetzt", key)),
                    Err(e) => toast_err(format!("{}: {}", key, e)),
                }
            });
        })
    };

    let has_description = description.is_some();
    let description_text = description;
    let inherited = is_default;
    let field_id = format!("setting-{}", key);

    // Built once outside `view!`: a `fallback=` closure that builds the node
    // inline captures and moves its bindings, which makes it `FnOnce` and the
    // view rejects it.
    let text_input = {
        let input_type = if value_type == "number" {
            "number"
        } else {
            "text"
        };
        view! {
            <input
                type=input_type
                step="any"
                class="input input-bordered w-full"
                id=field_id.clone()
                prop:value=move || input_value.get()
                on:input={move |ev| on_text.run(ev)}
                on:change={move |_| on_change.run(())}
            />
        }
    };

    view! {
        <div class="form-control mb-4">
            <label class="label" for=field_id.clone()>
                <span class="label-text">
                    {label.clone()}
                    <Show when=move || inherited>
                        <span class="badge badge-ghost badge-sm ml-2">
                            "Standard (geerbt)"
                        </span>
                    </Show>
                </span>
            </label>
            <div class="flex items-center gap-2">
                <Show
                    when=move || value_type == "boolean"
                    fallback=move || text_input.clone()
                >
                    <input
                        type="checkbox"
                        class="toggle"
                        id=field_id.clone()
                        prop:checked=move || checked.get()
                        on:change={move |_| on_toggle.run(())}
                    />
                </Show>
                <Show when=move || inherited>
                    <button class="btn btn-ghost btn-sm" on:click={move |_| on_reset.run(())}>
                        "Zurücksetzen"
                    </button>
                </Show>
            </div>
            <Show when=move || has_description>
                <div class="label">
                    <span class="label-text-alt">{description_text.clone().unwrap_or_default()}</span>
                </div>
            </Show>
        </div>
    }
}

/// The editable settings page.
///
/// Loads once on mount and refetches after each write, so what is displayed is
/// what the server stored rather than what was typed.
#[component]
pub fn ServerSettingsPanel() -> impl IntoView {
    let settings = RwSignal::new(Vec::<SettingDto>::new());
    let loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    spawn_local(async move {
        match crate::api::fetch_settings().await {
            Ok(resp) => settings.set(resp.settings),
            Err(e) => load_error.set(Some(e)),
        }
        loading.set(false);
    });

    let reload = move || {
        spawn_local(async move {
            if let Ok(resp) = crate::api::fetch_settings().await {
                settings.set(resp.settings);
            }
        });
    };

    let on_restore = move || {
        spawn_local(async move {
            match crate::api::restore_default_settings().await {
                Ok(_) => toast_ok(String::from("Standardwerte wiederhergestellt")),
                Err(e) => toast_err(e),
            }
        });
    };

    // Grouped by namespace so the page mirrors the key layout instead of
    // rendering one flat list.
    let sections = move || {
        let mut groups: std::collections::BTreeMap<String, Vec<SettingDto>> = Default::default();
        for s in settings.get() {
            let ns = s.key.split('.').next().unwrap_or("all").to_string();
            groups.entry(ns).or_default().push(s);
        }
        groups
    };

    let body = move || {
        sections()
            .into_iter()
            .map(|(namespace, items)| {
                view! {
                    <div>
                        <div class="divider">{namespace.clone()}</div>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-6">
                            {items.iter().map(|item| {
                                view! {
                                    <SettingField
                                        key=item.key.clone()
                                        value=item.value.clone()
                                        value_type=item.value_type.clone()
                                        label=item.key.clone()
                                        description=item.description.clone()
                                        is_default=item.is_default
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }
            })
            .collect_view()
    };

    view! {
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">"Einstellungen"</h2>
                <p class="text-sm opacity-70">
                    "Diese Werte werden serverseitig gespeichert und gelten für den gesamten Mandanten."
                </p>

                <Show when=move || loading.get()>
                    <div class="p-4">
                        <span class="loading loading-spinner loading-md"></span>
                    </div>
                </Show>
                <Show when=move || load_error.get().is_some()>
                    <div class="alert alert-error">
                        {format!(
                            "Einstellungen konnten nicht geladen werden: {}",
                            load_error.get().unwrap_or_default()
                        )}
                    </div>
                </Show>
                <Show when=move || !loading.get() && load_error.get().is_none()>
                    {body}
                    <div class="card-actions justify-end mt-4">
                        <button class="btn btn-outline btn-sm" on:click=move |_| reload()>
                            "Neu laden"
                        </button>
                        <button
                            class="btn btn-warning btn-outline btn-sm"
                            on:click=move |_| on_restore()
                        >
                            "Alle auf Standard zurücksetzen"
                        </button>
                    </div>
                </Show>
            </div>
        </div>
    }
}
