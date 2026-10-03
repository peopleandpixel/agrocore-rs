//! Frost warnings and pest risks — the two weather endpoints with no UI caller.
//!
//! `GET /weather/frost-warnings`, `GET /weather/frost-warnings/active` and
//! `GET /weather/pest-risks` were registered with full handlers and had no
//! caller.
//!
//! `RiskLevel` serialises lower-case (`"low"`, `"medium"`, `"high"`,
//! `"critical"`), which is what the select sends — not the Rust variant names.

use crate::api;
use icondata::{LuPlus, LuTrash2};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// The risk levels `RiskLevel` accepts, with the serde wire value.
const RISK_LEVELS: [(&str, &str); 4] = [
    ("low", "risk_low"),
    ("medium", "risk_medium"),
    ("high", "risk_high"),
    ("critical", "risk_critical"),
];

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
pub fn WeatherWarnings() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let warnings = LocalResource::new(|| async move { api::fetch_frost_warnings().await.ok() });
    let active =
        LocalResource::new(|| async move { api::fetch_active_frost_warnings().await.ok() });
    let risks = LocalResource::new(|| async move { api::fetch_pest_risks().await.ok() });

    let (station_id, set_station_id) = signal(String::new());
    let (threshold, set_threshold) = signal(String::new());
    let (notify_email, set_notify_email) = signal(true);
    let (notify_sms, set_notify_sms) = signal(false);

    let (site_id, set_site_id) = signal(String::new());
    let (assessment_date, set_assessment_date) = signal(String::new());
    let (risk_level, set_risk_level) = signal(String::from("low"));
    let (pest_type, set_pest_type) = signal(String::new());
    let (confidence, set_confidence) = signal(String::new());
    let (action, set_action) = signal(String::new());
    let (model_version, set_model_version) = signal(String::from("v1"));

    let (error, set_error) = signal(None::<String>);
    let (reload, set_reload) = signal(0u32);

    let on_create_warning = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let station = match uuid::Uuid::parse_str(station_id.get().trim()) {
            Ok(id) => id,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "invalid_uuid")().to_string()));
                return;
            }
        };
        let req = api::CreateFrostWarningRequest {
            station_id: station,
            threshold_temp_c: match read_num("threshold", threshold.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            is_active: true,
            notify_email: notify_email.get(),
            notify_sms: notify_sms.get(),
        };

        spawn_local(async move {
            match api::create_frost_warning(req).await {
                Ok(_) => {
                    set_threshold.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_risk = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let site = match uuid::Uuid::parse_str(site_id.get().trim()) {
            Ok(id) => id,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "invalid_uuid")().to_string()));
                return;
            }
        };
        let pest = pest_type.get().trim().to_string();
        let recommended = action.get().trim().to_string();
        if pest.is_empty() || recommended.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }

        let req = api::CreatePestRiskRequest {
            site_id: site,
            assessment_date: assessment_date.get().trim().to_string(),
            risk_level: risk_level.get(),
            pest_type: pest,
            // `confidence` is a `f64` with no Option in the DTO, so an empty
            // field is a zero rather than an absent value.
            confidence: confidence.get().trim().parse().unwrap_or(0.0),
            recommended_action: recommended,
            model_version: model_version.get().trim().to_string(),
        };

        spawn_local(async move {
            match api::create_pest_risk(req).await {
                Ok(_) => {
                    set_pest_type.set(String::new());
                    set_action.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            if let Err(e) = api::delete_frost_warning(id).await {
                set_error.set(Some(format!("{}: {e}", crate::t!(t, "delete_failed")())));
            } else {
                set_reload.update(|n| *n += 1);
            }
        });
    };

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4">{crate::t!(t, "weather_warnings")}</h1>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                <form class="card bg-base-100 shadow p-4" on:submit=on_create_warning>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "frost_warning")}</h2>
                    <div class="grid grid-cols-1 gap-3">
                        {text(
                            crate::t!(t, "station_id")().to_string(),
                            station_id,
                            set_station_id,
                        )}
                        {num(
                            crate::t!(t, "threshold_temp")().to_string(),
                            threshold,
                            set_threshold,
                        )}
                        <label class="label cursor-pointer justify-start gap-2">
                            <input
                                type="checkbox"
                                class="checkbox"
                                prop:checked=move || notify_email.get()
                                on:change=move |ev| set_notify_email.set(event_target_checked(&ev))
                            />
                            <span class="label-text">{crate::t!(t, "notify_email")}</span>
                        </label>
                        <label class="label cursor-pointer justify-start gap-2">
                            <input
                                type="checkbox"
                                class="checkbox"
                                prop:checked=move || notify_sms.get()
                                on:change=move |ev| set_notify_sms.set(event_target_checked(&ev))
                            />
                            <span class="label-text">{crate::t!(t, "notify_sms")}</span>
                        </label>
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_risk>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "pest_risk")}</h2>
                    <div class="grid grid-cols-1 gap-3">
                        {text(crate::t!(t, "plot_id_field")().to_string(), site_id, set_site_id)}
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "assessment_date")}</span>
                            <input
                                class="input input-bordered"
                                type="date"
                                prop:value=move || assessment_date.get()
                                on:input=move |ev| set_assessment_date.set(event_target_value(&ev))
                            />
                        </label>
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "risk_level")}</span>
                            <select
                                class="select select-bordered"
                                prop:value=move || risk_level.get()
                                on:change=move |ev| set_risk_level.set(event_target_value(&ev))
                            >
                                {RISK_LEVELS
                                    .iter()
                                    .map(|(value, key)| {
                                        view! {
                                            <option value=*value>{crate::t!(t, *key)()}</option>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </select>
                        </label>
                        {text(
                            crate::t!(t, "pest_type")().to_string(),
                            pest_type,
                            set_pest_type,
                        )}
                        {num(
                            crate::t!(t, "confidence_2")().to_string(),
                            confidence,
                            set_confidence,
                        )}
                        {text(
                            crate::t!(t, "recommended_action")().to_string(),
                            action,
                            set_action,
                        )}
                        {text(
                            crate::t!(t, "model_version")().to_string(),
                            model_version,
                            set_model_version,
                        )}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>
            </div>

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "frost_warnings")}</h2>
            {move || {
                let _ = reload.get();
                let loaded = warnings.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "station_id")}</th>
                                            <th>{crate::t!(t, "threshold_temp")}</th>
                                            <th>{crate::t!(t, "notify_email")}</th>
                                            <th>{crate::t!(t, "notify_sms")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|w: &api::FrostWarningDto| w.id
                                            children=move |w: api::FrostWarningDto| {
                                                let id = w.id;
                                                view! {
                                                    <tr>
                                                        <td>{w.station_id.to_string()}</td>
                                                        <td>{w.threshold_temp_c}</td>
                                                        <td>{if w.notify_email { "✓" } else { "" }}</td>
                                                        <td>{if w.notify_sms { "✓" } else { "" }}</td>
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
                        view! { <div class="alert mb-6">{crate::t!(t, "no_records")}</div> }.into_any()
                    }
                    None => {
                        view! { <div class="alert mb-6">{crate::t!(t, "loading")}</div> }.into_any()
                    }
                }
            }}

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "active_frost_warnings")}</h2>
            {move || {
                let loaded = active.read();
                let list = loaded.as_ref().and_then(|l| l.clone());
                match list {
                    Some(list) if !list.is_empty() => {
                        view! {
                            <div class="alert alert-warning mb-6">
                                <ul>
                                    {list
                                        .into_iter()
                                        .map(|w| {
                                            let station = w.station_id.to_string();
                                            let threshold = w.threshold_temp_c;
                                            view! {
                                                <li>
                                                    {station}
                                                    " — "
                                                    {threshold}
                                                    " °C"
                                                </li>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </ul>
                            </div>
                        }
                            .into_any()
                    }
                    _ => {
                        view! {
                            <div class="alert alert-success mb-6">
                                {crate::t!(t, "no_active_warnings")}
                            </div>
                        }
                            .into_any()
                    }
                }
            }}

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "pest_risks")}</h2>
            {move || {
                let loaded = risks.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "plot_id_field")}</th>
                                            <th>{crate::t!(t, "assessment_date")}</th>
                                            <th>{crate::t!(t, "pest_type")}</th>
                                            <th>{crate::t!(t, "risk_level")}</th>
                                            <th>{crate::t!(t, "recommended_action")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|r: &api::PestRiskDto| r.id
                                            children=move |r: api::PestRiskDto| {
                                                view! {
                                                    <tr>
                                                        <td>{r.site_id.to_string()}</td>
                                                        <td>{r.assessment_date.clone()}</td>
                                                        <td>{r.pest_type.clone()}</td>
                                                        <td>{r.risk_level.clone()}</td>
                                                        <td>{r.recommended_action.clone()}</td>
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
