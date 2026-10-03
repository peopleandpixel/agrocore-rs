//! Water management — sources, usage and quotas.
//!
//! The three `/water/*` collections were registered with full CRUD handlers and
//! had no UI caller.
//!
//! `WaterSourceType` and `IrrigationMethod` derive
//! `strum::EnumString` with `serialize_all = "snake_case"` but carry no `serde`
//! rename, so they serialise as the variant name — `"Well"`, `"Drip"` — not the
//! snake-case form the `strum` attribute would suggest for `FromStr`. The selects
//! send the variant names.

use crate::api;
use icondata::LuPlus;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// The source types `WaterSourceType` accepts. `Custom(String)` carries caller
/// text and is not offered.
const SOURCE_TYPES: [(&str, &str); 7] = [
    ("Well", "water_source_well"),
    ("Reservoir", "water_source_reservoir"),
    ("River", "water_source_river"),
    ("Canal", "water_source_canal"),
    ("ComunidadDeRegantes", "water_source_comunidad"),
    ("RainwaterHarvesting", "water_source_rainwater"),
    ("Desalination", "water_source_desalination"),
];

/// The irrigation methods `IrrigationMethod` accepts.
const IRRIGATION_METHODS: [(&str, &str); 6] = [
    ("Drip", "irrigation_drip"),
    ("Sprinkler", "irrigation_sprinkler"),
    ("Flood", "irrigation_flood"),
    ("Pivot", "irrigation_pivot"),
    ("MicroSprinkler", "irrigation_micro"),
    ("Subsurface", "irrigation_subsurface"),
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

/// Parse a required UUID.
///
/// The message names the field and says what is wrong rather than being
/// translated: it appears next to the field label the operator is looking at, and
/// "not a UUID" is more useful there than a translated sentence.
fn read_uuid(name: &str, raw: String) -> Result<uuid::Uuid, String> {
    uuid::Uuid::parse_str(raw.trim()).map_err(|_| format!("{name}: not a UUID"))
}

#[component]
pub fn WaterManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let sources = LocalResource::new(|| async move { api::fetch_water_sources().await.ok() });
    let usage = LocalResource::new(|| async move { api::fetch_water_usage().await.ok() });
    let quotas = LocalResource::new(|| async move { api::fetch_water_quotas().await.ok() });
    let (reload, set_reload) = signal(0u32);

    // Source form
    let (src_site, set_src_site) = signal(String::new());
    let (src_name, set_src_name) = signal(String::new());
    let (src_type, set_src_type) = signal(String::from("Well"));
    let (src_capacity, set_src_capacity) = signal(String::new());

    // Usage form
    let (use_site, set_use_site) = signal(String::new());
    let (use_source, set_use_source) = signal(String::new());
    let (use_date, set_use_date) = signal(String::new());
    let (use_volume, set_use_volume) = signal(String::new());
    let (use_method, set_use_method) = signal(String::from("Drip"));
    let (use_efficiency, set_use_efficiency) = signal(String::new());

    // Quota form
    let (q_source, set_q_source) = signal(String::new());
    let (q_site, set_q_site) = signal(String::new());
    let (q_year, set_q_year) = signal(String::new());
    let (q_allocated, set_q_allocated) = signal(String::new());

    let (error, set_error) = signal(None::<String>);

    let on_create_source = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let name = src_name.get().trim().to_string();
        if name.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }
        let site = match read_uuid("site_id", src_site.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let capacity = match src_capacity.get().trim() {
            "" => None,
            raw => match read_num("capacity", raw.to_string()) {
                Ok(v) => Some(v),
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
        };
        let req = api::CreateWaterSourceRequest {
            site_id: site,
            name,
            source_type: src_type.get(),
            capacity_m3: capacity,
        };
        spawn_local(async move {
            match api::create_water_source(req).await {
                Ok(_) => {
                    set_src_name.set(String::new());
                    set_src_capacity.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_usage = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let site = match read_uuid("site_id", use_site.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let source = match read_uuid("source_id", use_source.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let volume = match read_num("volume", use_volume.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let efficiency = match use_efficiency.get().trim() {
            "" => None,
            raw => match read_num("efficiency", raw.to_string()) {
                Ok(v) => Some(v),
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
        };
        let req = api::CreateWaterUsageRequest {
            site_id: site,
            source_id: source,
            usage_date: use_date.get().trim().to_string(),
            volume_m3: volume,
            irrigation_method: use_method.get(),
            efficiency_pct: efficiency,
        };
        spawn_local(async move {
            match api::create_water_usage(req).await {
                Ok(_) => {
                    set_use_volume.set(String::new());
                    set_use_efficiency.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_quota = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let source = match read_uuid("source_id", q_source.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let site = match read_uuid("site_id", q_site.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let year = match q_year.get().trim().parse::<i32>() {
            Ok(v) => v,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "field_required")().to_string()));
                return;
            }
        };
        let allocated = match read_num("allocated", q_allocated.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let req = api::CreateWaterQuotaRequest {
            source_id: source,
            site_id: site,
            year,
            allocated_m3: allocated,
            comunidad_id: None,
        };
        spawn_local(async move {
            match api::create_water_quota(req).await {
                Ok(_) => {
                    set_q_allocated.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4">{crate::t!(t, "nav_water")}</h1>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
                <form class="card bg-base-100 shadow p-4" on:submit=on_create_source>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "water_sources")}</h2>
                    <div class="grid grid-cols-1 gap-3">
                        {text(crate::t!(t, "plot_id_field")().to_string(), src_site, set_src_site)}
                        {text(
                            crate::t!(t, "tree_label")().to_string(),
                            src_name,
                            set_src_name,
                        )}
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "water_source_type")}</span>
                            <select
                                class="select select-bordered"
                                prop:value=move || src_type.get()
                                on:change=move |ev| set_src_type.set(event_target_value(&ev))
                            >
                                {SOURCE_TYPES
                                    .iter()
                                    .map(|(value, key)| {
                                        view! {
                                            <option value=*value>{crate::t!(t, *key)()}</option>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </select>
                        </label>
                        {num(
                            crate::t!(t, "water_capacity")().to_string(),
                            src_capacity,
                            set_src_capacity,
                        )}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_usage>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "water_usage")}</h2>
                    <div class="grid grid-cols-1 gap-3">
                        {text(crate::t!(t, "plot_id_field")().to_string(), use_site, set_use_site)}
                        {text(
                            crate::t!(t, "water_source_id")().to_string(),
                            use_source,
                            set_use_source,
                        )}
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "usage_date")}</span>
                            <input
                                class="input input-bordered"
                                type="date"
                                prop:value=move || use_date.get()
                                on:input=move |ev| set_use_date.set(event_target_value(&ev))
                            />
                        </label>
                        {num(
                            crate::t!(t, "water_volume")().to_string(),
                            use_volume,
                            set_use_volume,
                        )}
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "irrigation_method")}</span>
                            <select
                                class="select select-bordered"
                                prop:value=move || use_method.get()
                                on:change=move |ev| set_use_method.set(event_target_value(&ev))
                            >
                                {IRRIGATION_METHODS
                                    .iter()
                                    .map(|(value, key)| {
                                        view! {
                                            <option value=*value>{crate::t!(t, *key)()}</option>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </select>
                        </label>
                        {num(
                            crate::t!(t, "efficiency_pct")().to_string(),
                            use_efficiency,
                            set_use_efficiency,
                        )}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_quota>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "water_quotas")}</h2>
                    <div class="grid grid-cols-1 gap-3">
                        {text(
                            crate::t!(t, "water_source_id")().to_string(),
                            q_source,
                            set_q_source,
                        )}
                        {text(crate::t!(t, "plot_id_field")().to_string(), q_site, set_q_site)}
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "quota_year")}</span>
                            <input
                                class="input input-bordered"
                                type="number"
                                prop:value=move || q_year.get()
                                on:input=move |ev| set_q_year.set(event_target_value(&ev))
                            />
                        </label>
                        {num(
                            crate::t!(t, "quota_allocated")().to_string(),
                            q_allocated,
                            set_q_allocated,
                        )}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>
            </div>

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "water_sources")}</h2>
            {move || {
                let _ = reload.get();
                let loaded = sources.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "tree_label")}</th>
                                            <th>{crate::t!(t, "water_source_type")}</th>
                                            <th>{crate::t!(t, "water_capacity")}</th>
                                            <th>{crate::t!(t, "water_current_usage")}</th>
                                            <th>{crate::t!(t, "plot_id_field")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|s: &api::WaterSourceDto| s.id
                                            children=move |s: api::WaterSourceDto| {
                                                view! {
                                                    <tr>
                                                        <td>{s.name.clone()}</td>
                                                        <td>{s.source_type.clone()}</td>
                                                        <td>
                                                            {s.capacity_m3
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>{s.current_usage_m3}</td>
                                                        <td>{s.site_id.to_string()}</td>
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

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "water_usage")}</h2>
            {move || {
                let loaded = usage.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "usage_date")}</th>
                                            <th>{crate::t!(t, "water_source_id")}</th>
                                            <th>{crate::t!(t, "water_volume")}</th>
                                            <th>{crate::t!(t, "irrigation_method")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|u: &api::WaterUsageDto| u.id
                                            children=move |u: api::WaterUsageDto| {
                                                view! {
                                                    <tr>
                                                        <td>{u.usage_date.clone()}</td>
                                                        <td>{u.source_id.to_string()}</td>
                                                        <td>{u.volume_m3}</td>
                                                        <td>{u.irrigation_method.clone()}</td>
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

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "water_quotas")}</h2>
            {move || {
                let loaded = quotas.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "quota_year")}</th>
                                            <th>{crate::t!(t, "water_source_id")}</th>
                                            <th>{crate::t!(t, "quota_allocated")}</th>
                                            <th>{crate::t!(t, "quota_used")}</th>
                                            <th>{crate::t!(t, "quota_remaining")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|q: &api::WaterQuotaDto| q.id
                                            children=move |q: api::WaterQuotaDto| {
                                                view! {
                                                    <tr>
                                                        <td>{q.year}</td>
                                                        <td>{q.source_id.to_string()}</td>
                                                        <td>{q.allocated_m3}</td>
                                                        <td>{q.used_m3}</td>
                                                        <td>{q.remaining_m3}</td>
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
