//! Workforce work logs and worker locations.
//!
//! `GET /workforce/logs` and `GET /workforce/locations` had no UI caller at all.
//!
//! The locations half is not a CRUD collection and is deliberately not presented
//! as one. `POST /workforce/locations` is what a worker's own device calls to
//! report where it is, and the handler fills the worker id in from the
//! authenticated user rather than reading it from the body — so a client cannot
//! report a position on someone else's behalf. An edit-and-delete table for
//! positions would imply a capability the endpoint does not have; what the page
//! offers instead is the current positions and a way to post the caller's own.

use crate::api;
use icondata::{LuPlus, LuTrash2};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// A numeric input that takes the raw string, so a half-typed value does not
/// re-render the form and clear the field.
fn num(label: String, value: ReadSignal<String>, set: WriteSignal<String>) -> impl IntoView {
    view! {
        <label class="form-control">
            <span class="label-text">{label}</span>
            <input
                class="input input-bordered"
                type="number"
                step="any"
                prop:value=move || value.get()
                on:input=move |ev| set.set(event_target_value(&ev))
            />
        </label>
    }
}

/// A text input bound to a signal.
fn text(label: String, value: ReadSignal<String>, set: WriteSignal<String>) -> impl IntoView {
    view! {
        <label class="form-control">
            <span class="label-text">{label}</span>
            <input
                class="input input-bordered"
                prop:value=move || value.get()
                on:input=move |ev| set.set(event_target_value(&ev))
            />
        </label>
    }
}

fn read_num(name: &str, raw: String) -> Result<f64, String> {
    raw.trim()
        .parse::<f64>()
        .map_err(|_| format!("{name}: not a number"))
}

#[component]
pub fn WorkforceLogs() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let logs = LocalResource::new(|| async move { api::fetch_work_logs().await.ok() });
    let (reload, set_reload) = signal(0u32);

    let (worker_id, set_worker_id) = signal(String::new());
    let (date, set_date) = signal(String::new());
    let (hours, set_hours) = signal(String::new());
    let (overtime, set_overtime) = signal(String::new());
    let (rest, set_rest) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (site_id, set_site_id) = signal(String::new());
    let (breaks, set_breaks) = signal(String::new());
    let (night_shift, set_night_shift) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let worker = match uuid::Uuid::parse_str(worker_id.get().trim()) {
            Ok(id) => id,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "invalid_uuid")().to_string()));
                return;
            }
        };
        let task = description.get().trim().to_string();
        if task.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }

        // `site_id` is optional: an empty field means "not site-specific", and
        // `null` is the wire value for that.
        let site = match site_id.get().trim() {
            "" => None,
            raw => match uuid::Uuid::parse_str(raw) {
                Ok(id) => Some(id),
                Err(_) => {
                    set_error.set(Some(crate::t!(t, "invalid_uuid")().to_string()));
                    return;
                }
            },
        };

        let req = api::CreateWorkLogRequest {
            worker_id: worker,
            // The API expects a full timestamp; a bare date is read as midnight
            // UTC, which is what a person entering "today" means by it.
            date: format!("{}T00:00:00Z", date.get().trim()),
            hours_worked: match read_num("hours", hours.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            overtime_hours: overtime.get().trim().parse().unwrap_or(0.0),
            rest_period_hours: rest.get().trim().parse().unwrap_or(0.0),
            task_description: task,
            site_id: site,
            is_night_shift: night_shift.get(),
            breaks_taken: breaks.get().trim().parse().unwrap_or(0),
        };

        spawn_local(async move {
            match api::create_work_log(req).await {
                Ok(_) => {
                    set_description.set(String::new());
                    set_hours.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            if let Err(e) = api::delete_work_log(id).await {
                set_error.set(Some(format!("{}: {e}", crate::t!(t, "delete_failed")())));
            } else {
                set_reload.update(|n| *n += 1);
            }
        });
    };

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4">{crate::t!(t, "nav_work_logs")}</h1>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <form class="card bg-base-100 shadow p-4 mb-6" on:submit=on_create>
                <h2 class="font-semibold mb-3">{crate::t!(t, "btn_add")}</h2>
                <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
                    {text(
                        crate::t!(t, "worker_id")().to_string(),
                        worker_id,
                        set_worker_id,
                    )}
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "log_date")}</span>
                        <input
                            class="input input-bordered"
                            type="date"
                            prop:value=move || date.get()
                            on:input=move |ev| set_date.set(event_target_value(&ev))
                        />
                    </label>
                    {num(
                        crate::t!(t, "hours_worked")().to_string(),
                        hours,
                        set_hours,
                    )}
                    {num(
                        crate::t!(t, "overtime_hours")().to_string(),
                        overtime,
                        set_overtime,
                    )}
                    {num(
                        crate::t!(t, "rest_hours")().to_string(),
                        rest,
                        set_rest,
                    )}
                    {num(
                        crate::t!(t, "breaks_taken")().to_string(),
                        breaks,
                        set_breaks,
                    )}
                    {text(
                        crate::t!(t, "task_description")().to_string(),
                        description,
                        set_description,
                    )}
                    {text(crate::t!(t, "plot_id_field")().to_string(), site_id, set_site_id)}
                    <label class="label cursor-pointer justify-start gap-2">
                        <input
                            type="checkbox"
                            class="checkbox"
                            prop:checked=move || night_shift.get()
                            on:change=move |ev| set_night_shift.set(event_target_checked(&ev))
                        />
                        <span class="label-text">{crate::t!(t, "night_shift")}</span>
                    </label>
                </div>
                <button type="submit" class="btn btn-primary mt-3">
                    <Icon icon=LuPlus width="16" height="16" />
                    {crate::t!(t, "btn_add")}
                </button>
            </form>

            {move || {
                let _ = reload.get();
                let loaded = logs.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "log_date")}</th>
                                            <th>{crate::t!(t, "worker_id")}</th>
                                            <th>{crate::t!(t, "task_description")}</th>
                                            <th>{crate::t!(t, "hours_worked")}</th>
                                            <th>{crate::t!(t, "overtime_hours")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|log: &api::WorkLogDto| log.id
                                            children=move |log: api::WorkLogDto| {
                                                let id = log.id;
                                                view! {
                                                    <tr>
                                                        <td>
                                                            {log
                                                                .date
                                                                .get(..10)
                                                                .unwrap_or(&log.date)
                                                                .to_string()}
                                                        </td>
                                                        <td>{log.worker_id.to_string()}</td>
                                                        <td>{log.task_description.clone()}</td>
                                                        <td>{log.hours_worked}</td>
                                                        <td>{log.overtime_hours}</td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| on_delete(id)
                                                            >
                                                                <Icon
                                                                    icon=LuTrash2
                                                                    width="14"
                                                                    height="14"
                                                                />
                                                                {crate::t!(t, "btn_delete")}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            </div>
                        }
                            .into_any()
                    }
                    Some(_) => {
                        view! { <div class="alert">{crate::t!(t, "no_records")}</div> }.into_any()
                    }
                    None => {
                        view! { <div class="alert">{crate::t!(t, "loading")}</div> }.into_any()
                    }
                }
            }}
        </div>
    }
}

/// The latest reported position of every worker, plus a way to post the
/// caller's own.
///
/// No edit or delete per row: the endpoint has no such route, and offering a
/// control that cannot work is worse than not offering it.
#[component]
pub fn WorkforceLocations() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let locations = LocalResource::new(|| async move { api::fetch_worker_locations().await.ok() });
    let (lat, set_lat) = signal(String::new());
    let (lng, set_lng) = signal(String::new());
    let (error, set_error) = signal(None::<String>);

    let on_report = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let req = api::ReportLocationRequest {
            lat: match read_num("lat", lat.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            lng: match read_num("lng", lng.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            current_task_id: None,
        };
        spawn_local(async move {
            if let Err(e) = api::report_own_location(req).await {
                set_error.set(Some(e));
            }
        });
    };

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4">{crate::t!(t, "nav_worker_locations")}</h1>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <form class="card bg-base-100 shadow p-4 mb-6" on:submit=on_report>
                <h2 class="font-semibold mb-3">{crate::t!(t, "report_own_position")}</h2>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    {num(crate::t!(t, "latitude_field")().to_string(), lat, set_lat)}
                    {num(crate::t!(t, "longitude_field")().to_string(), lng, set_lng)}
                </div>
                <button type="submit" class="btn btn-primary mt-3">
                    {crate::t!(t, "btn_save")}
                </button>
            </form>

            {move || {
                let loaded = locations.read();
                let list = loaded.as_ref().and_then(|l| l.clone());
                match list {
                    Some(list) if !list.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "worker_id")}</th>
                                            <th>{crate::t!(t, "latitude_field")}</th>
                                            <th>{crate::t!(t, "longitude_field")}</th>
                                            <th>{crate::t!(t, "timestamp")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {list
                                            .into_iter()
                                            .map(|loc| {
                                                let stamp = loc.timestamp.clone();
                                                view! {
                                                    <tr>
                                                        <td>{loc.worker_id.to_string()}</td>
                                                        <td>{loc.lat}</td>
                                                        <td>{loc.lng}</td>
                                                        <td>{stamp}</td>
                                                    </tr>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>
                        }
                            .into_any()
                    }
                    Some(_) => {
                        view! { <div class="alert">{crate::t!(t, "no_records")}</div> }.into_any()
                    }
                    None => {
                        view! { <div class="alert">{crate::t!(t, "loading")}</div> }.into_any()
                    }
                }
            }}
        </div>
    }
}
