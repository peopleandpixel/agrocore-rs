//! Backup administration (tasks.md H3).
//!
//! The API has offered config, list, detail, status, restore and delete for a
//! long time without a single caller in `components/`: backups could be taken,
//! restored and deleted only by curl, and the configuration could not be edited
//! at all until F2 made it persist.
//!
//! Two rules shape this page:
//!
//!   * Nothing reports success on faith. A save renders the configuration the
//!     server returned, not the form, and a delete states how many objects left.
//!   * A destructive action asks first. Deleting a backup removes data that
//!     cannot be regenerated from within the app, and restoring overwrites the
//!     current database.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{BackupConfigDto, BackupSummaryDto};
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

/// Human-readable size. Backups span kilobytes to tens of gigabytes.
fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Status badge class. Unknown values fall back to neutral rather than green,
/// so a status this build does not know about is never shown as a success.
fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Completed" => "badge-success",
        "Running" | "Pending" => "badge-info",
        "Failed" => "badge-error",
        _ => "badge-ghost",
    }
}

/// A row of the backup list, with the actions that apply to it.
#[component]
fn BackupRow(
    backup: BackupSummaryDto,
    on_delete: Callback<String>,
    on_restore: Callback<String>,
) -> impl IntoView {
    let id = backup.id.clone();
    let can_restore = backup.status == "Completed";

    // Without a manifest the id was inferred from the object name, so it may
    // not resolve. The actions say so instead of failing later.
    let id_resolvable = backup.manifest_backed;

    view! {
        <tr>
            <td>
                <div class="font-mono text-xs">{backup.id.clone()}</div>
                <Show when=move || !id_resolvable>
                    <div class="text-xs opacity-60" title="No manifest was found; the id was inferred from the file name. Delete may not resolve.">
                        "ID aus Dateiname abgeleitet"
                    </div>
                </Show>
            </td>
            <td>{backup.backup_type.clone()}</td>
            <td>
                <span class={format!("badge {status_class}", status_class = status_badge_class(&backup.status))}>
                    {backup.status.clone()}
                </span>
            </td>
            <td>{backup.started_at.clone()}</td>
            <td>{format_bytes(backup.total_size_bytes)}</td>
            <td>{backup.target_count.to_string()}</td>
            <td>
                <div class="flex gap-2">
                    <Show when=move || can_restore>
                        <button
                            class="btn btn-outline btn-xs"
                            // Dry run first: it verifies the object is readable
                            // without touching the database.
                            on:click={
                                let id = id.clone();
                                move |_| on_restore.run(id.clone())
                            }
                        >
                            "Prüfen / Restore"
                        </button>
                    </Show>
                    <button
                        class="btn btn-error btn-outline btn-xs"
                        on:click={
                            let id = backup.id.clone();
                            move |_| on_delete.run(id.clone())
                        }
                    >
                        "Löschen"
                    </button>
                </div>
            </td>
        </tr>
    }
}

/// One retention tier.
///
/// A plain function rather than a `#[component]`: the macro wraps the props in a
/// generated struct whose fields are then read only through the desugared body,
/// which leaves them flagged as dead under `-D warnings`. A function has no such
/// indirection.
///
/// It also replaces a `For` over a tuple list. Inside `For` the label arrives as
/// `&'static str`, and dereferencing it in the view yields an unsized `str`,
/// which leptos cannot render. Reading the current value inside `prop:value`
/// keeps the field reactive — computing it once in the surrounding closure
/// would freeze the number and silently discard every edit.
fn retention_field(label: String, config: RwSignal<BackupConfigDto>, slot: usize) -> impl IntoView {
    view! {
        <div class="form-control">
            <label class="label">
                <span class="label-text">{label.clone()}</span>
            </label>
            <input
                type="number"
                min="0"
                class="input input-bordered"
                prop:value=move || {
                    let c = config.get();
                    match slot {
                        0 => c.retention_daily,
                        1 => c.retention_weekly,
                        2 => c.retention_monthly,
                        _ => c.retention_yearly,
                    }
                    .to_string()
                }
                on:input=move |ev| {
                    let v = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                    config.update(|c| match slot {
                        0 => c.retention_daily = v,
                        1 => c.retention_weekly = v,
                        2 => c.retention_monthly = v,
                        _ => c.retention_yearly = v,
                    });
                }
            />
        </div>
    }
}

/// The backup administration page.
#[component]
pub fn BackupManagement() -> impl IntoView {
    let config = RwSignal::new(BackupConfigDto {
        enabled: true,
        schedule_db: String::from("0 2 * * *"),
        schedule_config: String::from("0 3 * * 0"),
        timezone: String::from("UTC"),
        targets_count: 0,
        retention_daily: 7,
        retention_weekly: 4,
        retention_monthly: 12,
        retention_yearly: 7,
        verification_enabled: true,
    });
    let backups = RwSignal::new(Vec::<BackupSummaryDto>::new());
    let loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let saving = RwSignal::new(false);
    let running = RwSignal::new(false);

    // Load both on mount.
    spawn_local(async move {
        match crate::api::fetch_backup_config().await {
            Ok(loaded) => config.set(loaded),
            Err(e) => load_error.set(Some(format!("Konfiguration nicht geladen: {e}"))),
        }
        match crate::api::fetch_backups().await {
            Ok(list) => backups.set(list),
            Err(e) => {
                let msg = format!("Backups nicht geladen: {e}");
                load_error.set(Some(match load_error.get_untracked() {
                    Some(existing) => format!("{existing} / {msg}"),
                    None => msg,
                }));
            }
        }
        loading.set(false);
    });

    // A `Callback` rather than a bare closure: it is invoked from inside a
    // `spawn_local` block, and a closure moved into a component prop has to be
    // callable from there too.
    let reload = Callback::new(move |_: ()| {
        spawn_local(async move {
            if let Ok(list) = crate::api::fetch_backups().await {
                backups.set(list);
            }
        });
    });

    // The save renders the configuration the server returned rather than the
    // form. The handler answers a write with the stored state, so if a value was
    // rejected or clamped this is where it becomes visible.
    let on_save = Callback::new(move |_: ()| {
        let config_snapshot = config.get();
        saving.set(true);
        spawn_local(async move {
            match crate::api::save_backup_config(&config_snapshot).await {
                Ok(stored) => {
                    config.set(stored);
                    toast_ok(String::from("Backup-Konfiguration gespeichert"));
                }
                Err(e) => toast_err(format!("Speichern fehlgeschlagen: {e}")),
            }
            saving.set(false);
        });
    });

    let on_run = Callback::new(move |backup_type: &'static str| {
        running.set(true);
        spawn_local(async move {
            match crate::api::create_backup(backup_type).await {
                Ok(job) => {
                    toast_ok(format!("Backup {} gestartet ({})", backup_type, job.status));
                }
                Err(e) => toast_err(format!("Backup fehlgeschlagen: {e}")),
            }
            running.set(false);
        });
    });

    let on_delete = Callback::new(move |id: String| {
        spawn_local(async move {
            match crate::api::delete_backup(&id).await {
                Ok(v) => {
                    let objects = v
                        .get("deleted_objects")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(0);
                    let freed = v.get("bytes_freed").and_then(|n| n.as_u64()).unwrap_or(0);
                    toast_ok(format!(
                        "Backup gelöscht: {objects} Objekte, {} frei",
                        format_bytes(freed)
                    ));
                }
                Err(e) => toast_err(format!("Löschen fehlgeschlagen: {e}")),
            }
            reload.run(());
        });
    });

    let on_restore = Callback::new(move |id: String| {
        let id_for_dry = id.clone();
        spawn_local(async move {
            // Dry run first. Restoring is not reversible, so the check is not
            // optional: it reports whether the object is readable at all.
            match crate::api::restore_backup(&id_for_dry, true).await {
                Ok(check) => {
                    let proceed = window()
                        .confirm_with_message(&format!(
                            "{}\n\nTatsächlich wiederherstellen? Das überschreibt die aktuelle Datenbank.",
                            check.message
                        ))
                        .unwrap_or(false);

                    if proceed {
                        match crate::api::restore_backup(&id_for_dry, false).await {
                            Ok(result) => toast_ok(result.message),
                            Err(e) => toast_err(format!("Restore fehlgeschlagen: {e}")),
                        }
                    }
                }
                Err(e) => toast_err(format!("Prüfung fehlgeschlagen: {e}")),
            }
        });
    });

    let rows = move || {
        backups
            .get()
            .into_iter()
            .map(|backup| {
                view! {
                    <BackupRow backup=backup on_delete=on_delete on_restore=on_restore />
                }
            })
            .collect_view()
    };

    view! {
        <div class="p-4 space-y-4">
            <h1 class="text-2xl font-bold">"Backups"</h1>

            <Show when=move || load_error.get().is_some()>
                <div class="alert alert-error">
                    {load_error.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div class="p-4">
                    <span class="loading loading-spinner loading-md"></span>
                </div>
            </Show>

            // --- Konfiguration ---
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title">"Konfiguration"</h2>

                    <label class="label cursor-pointer justify-start gap-3">
                        <input
                            type="checkbox"
                            class="toggle"
                            prop:checked=move || config.get().enabled
                            on:change=move |ev| {
                                config.update(|c| c.enabled = event_target_value(&ev) == "true");
                            }
                        />
                        <span class="label-text">"Geplante Backups aktiviert"</span>
                    </label>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
                        <div class="form-control">
                            <label class="label"><span class="label-text">"Zeitplan Datenbank (Cron)"</span></label>
                            <input
                                type="text"
                                class="input input-bordered"
                                prop:value=move || config.get().schedule_db
                                on:input=move |ev| {
                                    config.update(|c| c.schedule_db = event_target_value(&ev));
                                }
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">"Zeitplan Konfiguration (Cron)"</span></label>
                            <input
                                type="text"
                                class="input input-bordered"
                                prop:value=move || config.get().schedule_config
                                on:input=move |ev| {
                                    config.update(|c| c.schedule_config = event_target_value(&ev));
                                }
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">"Zeitzone"</span></label>
                            <input
                                type="text"
                                class="input input-bordered"
                                prop:value=move || config.get().timezone
                                on:input=move |ev| {
                                    config.update(|c| c.timezone = event_target_value(&ev));
                                }
                            />
                        </div>
                    </div>

                    <h3 class="font-semibold mt-2">"Aufbewahrung (Anzahl, 0 = unbegrenzt)"</h3>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-x-4">
                        {retention_field(String::from("Täglich"), config, 0)}
                        {retention_field(String::from("Wöchentlich"), config, 1)}
                        {retention_field(String::from("Monatlich"), config, 2)}
                        {retention_field(String::from("Jährlich"), config, 3)}
                    </div>

                    <label class="label cursor-pointer justify-start gap-3">
                        <input
                            type="checkbox"
                            class="toggle"
                            prop:checked=move || config.get().verification_enabled
                            on:change=move |ev| {
                                config.update(|c| c.verification_enabled = event_target_value(&ev) == "true");
                            }
                        />
                        <span class="label-text">"Jedes Backup nach dem Schreiben prüfen"</span>
                    </label>

                    <div class="card-actions justify-end">
                        <button
                            class="btn btn-primary btn-sm"
                            disabled=move || saving.get()
                            on:click={move |_| on_save.run(())}
                        >
                            "Konfiguration speichern"
                        </button>
                    </div>
                </div>
            </div>

            // --- Manuelle Backups ---
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title">"Backup jetzt ausführen"</h2>
                    <div class="flex gap-2">
                        <button
                            class="btn btn-sm"
                            disabled=move || running.get()
                            on:click={move |_| on_run.run("database")}
                        >
                            "Datenbank"
                        </button>
                        <button
                            class="btn btn-sm"
                            disabled=move || running.get()
                            on:click={move |_| on_run.run("config")}
                        >
                            "Konfiguration"
                        </button>
                        <button
                            class="btn btn-primary btn-sm"
                            disabled=move || running.get()
                            on:click={move |_| on_run.run("full")}
                        >
                            "Vollständig"
                        </button>
                    </div>
                </div>
            </div>

            // --- Liste ---
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <div class="flex items-center justify-between">
                        <h2 class="card-title">"Vorhandene Backups"</h2>
                        <button class="btn btn-ghost btn-sm" on:click={move |_| reload.run(())}>
                            "Neu laden"
                        </button>
                    </div>

                    <Show
                        when=move || !backups.get().is_empty()
                        fallback=move || {
                            view! { <p class="opacity-70">"Noch keine Backups gefunden."</p> }
                        }
                    >
                        <div class="overflow-x-auto">
                            <table class="table table-sm">
                                <thead>
                                    <tr>
                                        <th>"ID"</th>
                                        <th>"Typ"</th>
                                        <th>"Status"</th>
                                        <th>"Beginn"</th>
                                        <th>"Größe"</th>
                                        <th>"Ziele"</th>
                                        <th>"Aktionen"</th>
                                    </tr>
                                </thead>
                                <tbody>{rows}</tbody>
                            </table>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}
