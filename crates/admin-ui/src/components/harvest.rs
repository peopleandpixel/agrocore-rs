//! Harvest: seasons, lots, deliveries and the cold chain.
//!
//! The four `/harvest/*` collections were registered with full CRUD handlers and
//! had no UI caller.
//!
//! `LotStatus` carries lowercase serde renames (`"collecting"`, `"processed"`,
//! `"shipped"`, `"stored"`), which is what the select sends.
//!
//! `site_ids` on a lot is a `Vec<Uuid>`. The field here takes comma-separated
//! UUIDs rather than offering a site picker: a lot legitimately spans several
//! sites, and a picker that only allows one would be a quiet way to lose the
//! others.

use crate::api;
use icondata::{LuPlus, LuTrash2};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

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

fn date(label: String, value: ReadSignal<String>, set: WriteSignal<String>) -> impl IntoView {
    view! {
        <label class="form-control">
            <span class="label-text">{label}</span>
            <input
                class="input input-bordered"
                type="date"
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

fn read_uuid(name: &str, raw: String) -> Result<uuid::Uuid, String> {
    uuid::Uuid::parse_str(raw.trim()).map_err(|_| format!("{name}: not a UUID"))
}

/// Comma-separated UUIDs into the `Vec<Uuid>` a lot carries.
///
/// Empty is `Ok(vec![])` — a lot with no sites recorded is a valid state, and
/// rejecting it would block creating the lot before its sites are assigned.
fn read_uuid_list(raw: String) -> Result<Vec<uuid::Uuid>, String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| uuid::Uuid::parse_str(s).map_err(|_| format!("site_ids: {s} is not a UUID")))
        .collect()
}

/// An empty field means absent, not zero.
fn read_opt_num(name: &str, raw: String) -> Result<Option<f64>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    read_num(name, trimmed.to_string()).map(Some)
}

fn opt_text(raw: String) -> Option<String> {
    let trimmed = raw.trim().to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

#[component]
pub fn HarvestManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let seasons = LocalResource::new(|| async move { api::fetch_harvest_seasons().await.ok() });
    let lots = LocalResource::new(|| async move { api::fetch_harvest_lots().await.ok() });
    let deliveries =
        LocalResource::new(|| async move { api::fetch_harvest_deliveries().await.ok() });
    let cold_chain = LocalResource::new(|| async move { api::fetch_cold_chain_logs().await.ok() });
    let (reload, set_reload) = signal(0u32);

    // Season
    let (s_year, set_s_year) = signal(String::new());
    let (s_label, set_s_label) = signal(String::new());
    let (s_start, set_s_start) = signal(String::new());
    let (s_end, set_s_end) = signal(String::new());

    // Lot
    let (l_season, set_l_season) = signal(String::new());
    let (l_number, set_l_number) = signal(String::new());
    let (l_sites, set_l_sites) = signal(String::new());
    let (l_crop, set_l_crop) = signal(String::new());
    let (l_variety, set_l_variety) = signal(String::new());
    let (l_quality, set_l_quality) = signal(String::new());

    // Delivery
    let (d_lot, set_d_lot) = signal(String::new());
    let (d_date, set_d_date) = signal(String::new());
    let (d_gross, set_d_gross) = signal(String::new());
    let (d_tare, set_d_tare) = signal(String::new());
    let (d_carrier, set_d_carrier) = signal(String::new());
    let (d_vehicle, set_d_vehicle) = signal(String::new());
    let (d_notes, set_d_notes) = signal(String::new());
    let (d_temp, set_d_temp) = signal(String::new());

    // Cold chain
    let (c_lot, set_c_lot) = signal(String::new());
    let (c_sensor, set_c_sensor) = signal(String::new());
    let (c_recorded, set_c_recorded) = signal(String::new());
    let (c_temp, set_c_temp) = signal(String::new());
    let (c_humidity, set_c_humidity) = signal(String::new());
    let (c_location, set_c_location) = signal(String::new());

    let (error, set_error) = signal(None::<String>);

    let on_create_season = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let label = s_label.get().trim().to_string();
        if label.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }
        let year = match s_year.get().trim().parse::<i32>() {
            Ok(v) => v,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "field_required")().to_string()));
                return;
            }
        };
        let req = api::CreateHarvestSeasonRequest {
            year,
            label,
            start_date: s_start.get().trim().to_string(),
            end_date: opt_text(s_end.get()),
        };
        spawn_local(async move {
            match api::create_harvest_season(req).await {
                Ok(_) => {
                    set_s_label.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_lot = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let season = match read_uuid("season_id", l_season.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let number = l_number.get().trim().to_string();
        let crop = l_crop.get().trim().to_string();
        if number.is_empty() || crop.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }
        let sites = match read_uuid_list(l_sites.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let req = api::CreateHarvestLotRequest {
            season_id: season,
            lot_number: number,
            site_ids: sites,
            crop_type: crop,
            variety: opt_text(l_variety.get()),
            quality_target: opt_text(l_quality.get()),
        };
        spawn_local(async move {
            match api::create_harvest_lot(req).await {
                Ok(_) => {
                    set_l_number.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_delivery = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let lot = match read_uuid("lot_id", d_lot.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let gross = match read_num("gross", d_gross.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let tare = match read_num("tare", d_tare.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let req = api::CreateHarvestDeliveryRequest {
            lot_id: lot,
            delivery_date: d_date.get().trim().to_string(),
            gross_weight_kg: gross,
            tare_weight_kg: tare,
            carrier_name: opt_text(d_carrier.get()),
            vehicle_id: opt_text(d_vehicle.get()),
            quality_notes: opt_text(d_notes.get()),
            temperature_at_delivery: match read_opt_num("temperature", d_temp.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
        };
        spawn_local(async move {
            match api::create_harvest_delivery(req).await {
                Ok(_) => {
                    set_d_gross.set(String::new());
                    set_d_tare.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_cold = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let lot = match read_uuid("lot_id", c_lot.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let sensor = c_sensor.get().trim().to_string();
        if sensor.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }
        let temp = match read_num("temperature", c_temp.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let humidity = match read_opt_num("humidity", c_humidity.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let req = api::CreateColdChainLogRequest {
            lot_id: lot,
            sensor_id: sensor,
            recorded_at: c_recorded.get().trim().to_string(),
            temperature_c: temp,
            humidity_pct: humidity,
            location: opt_text(c_location.get()),
        };
        spawn_local(async move {
            match api::create_cold_chain_log(req).await {
                Ok(_) => {
                    set_c_temp.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    /// One delete handler per collection, since each has its own endpoint.
    macro_rules! deleter {
        ($fn_name:ident, $api_fn:path) => {
            move |id: uuid::Uuid| {
                spawn_local(async move {
                    if let Err(e) = $api_fn(id).await {
                        set_error.set(Some(format!("{}: {e}", crate::t!(t, "delete_failed")())));
                    } else {
                        set_reload.update(|n| *n += 1);
                    }
                })
            }
        };
    }
    let del_season = deleter!(del_season, api::delete_harvest_season);
    let del_lot = deleter!(del_lot, api::delete_harvest_lot);
    let del_delivery = deleter!(del_delivery, api::delete_harvest_delivery);
    let del_cold = deleter!(del_cold, api::delete_cold_chain_log);

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4">{crate::t!(t, "nav_harvest")}</h1>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-6 mb-8">
                <form class="card bg-base-100 shadow p-4" on:submit=on_create_season>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "harvest_seasons")}</h2>
                    <div class="grid grid-cols-2 gap-3">
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "quota_year")}</span>
                            <input
                                class="input input-bordered"
                                type="number"
                                prop:value=move || s_year.get()
                                on:input=move |ev| set_s_year.set(event_target_value(&ev))
                            />
                        </label>
                        {text(
                            crate::t!(t, "tree_label")().to_string(),
                            s_label,
                            set_s_label,
                        )}
                        {date(
                            crate::t!(t, "season_start")().to_string(),
                            s_start,
                            set_s_start,
                        )}
                        {date(crate::t!(t, "season_end")().to_string(), s_end, set_s_end)}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_lot>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "harvest_lots")}</h2>
                    <div class="grid grid-cols-2 gap-3">
                        {text(
                            crate::t!(t, "season_id")().to_string(),
                            l_season,
                            set_l_season,
                        )}
                        {text(
                            crate::t!(t, "lot_number")().to_string(),
                            l_number,
                            set_l_number,
                        )}
                        {text(
                            crate::t!(t, "site_ids")().to_string(),
                            l_sites,
                            set_l_sites,
                        )}
                        {text(
                            crate::t!(t, "crop_type")().to_string(),
                            l_crop,
                            set_l_crop,
                        )}
                        {text(
                            crate::t!(t, "lot_variety")().to_string(),
                            l_variety,
                            set_l_variety,
                        )}
                        {text(
                            crate::t!(t, "quality_target")().to_string(),
                            l_quality,
                            set_l_quality,
                        )}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_delivery>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "harvest_deliveries")}</h2>
                    <div class="grid grid-cols-2 gap-3">
                        {text(crate::t!(t, "lot_id")().to_string(), d_lot, set_d_lot)}
                        {date(
                            crate::t!(t, "delivery_date")().to_string(),
                            d_date,
                            set_d_date,
                        )}
                        {num(
                            crate::t!(t, "gross_weight")().to_string(),
                            d_gross,
                            set_d_gross,
                        )}
                        {num(
                            crate::t!(t, "tare_weight")().to_string(),
                            d_tare,
                            set_d_tare,
                        )}
                        {text(
                            crate::t!(t, "carrier_name")().to_string(),
                            d_carrier,
                            set_d_carrier,
                        )}
                        {text(
                            crate::t!(t, "vehicle_id")().to_string(),
                            d_vehicle,
                            set_d_vehicle,
                        )}
                        {text(
                            crate::t!(t, "quality_target")().to_string(),
                            d_notes,
                            set_d_notes,
                        )}
                        {num(
                            crate::t!(t, "delivery_temperature")().to_string(),
                            d_temp,
                            set_d_temp,
                        )}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_cold>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "cold_chain")}</h2>
                    <div class="grid grid-cols-2 gap-3">
                        {text(crate::t!(t, "lot_id")().to_string(), c_lot, set_c_lot)}
                        {text(
                            crate::t!(t, "sensor_id")().to_string(),
                            c_sensor,
                            set_c_sensor,
                        )}
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "recorded_at")}</span>
                            <input
                                class="input input-bordered"
                                type="datetime-local"
                                prop:value=move || c_recorded.get()
                                on:input=move |ev| set_c_recorded.set(event_target_value(&ev))
                            />
                        </label>
                        {num(
                            crate::t!(t, "temperature_field")().to_string(),
                            c_temp,
                            set_c_temp,
                        )}
                        {num(
                            crate::t!(t, "humidity_pct")().to_string(),
                            c_humidity,
                            set_c_humidity,
                        )}
                        {text(
                            crate::t!(t, "location_field")().to_string(),
                            c_location,
                            set_c_location,
                        )}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>
            </div>

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "harvest_seasons")}</h2>
            {move || {
                let _ = reload.get();
                let loaded = seasons.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "quota_year")}</th>
                                            <th>{crate::t!(t, "tree_label")}</th>
                                            <th>{crate::t!(t, "season_start")}</th>
                                            <th>{crate::t!(t, "season_end")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|s: &api::HarvestSeasonDto| s.id
                                            children=move |s: api::HarvestSeasonDto| {
                                                let id = s.id;
                                                view! {
                                                    <tr>
                                                        <td>{s.year}</td>
                                                        <td>{s.label.clone()}</td>
                                                        <td>{s.start_date.clone()}</td>
                                                        <td>
                                                            {s.end_date
                                                                .clone()
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| (del_season)(id)
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

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "harvest_lots")}</h2>
            {move || {
                let loaded = lots.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "lot_number")}</th>
                                            <th>{crate::t!(t, "crop_type")}</th>
                                            <th>{crate::t!(t, "lot_variety")}</th>
                                            <th>{crate::t!(t, "total_weight")}</th>
                                            <th>{crate::t!(t, "lot_status")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|l: &api::HarvestLotDto| l.id
                                            children=move |l: api::HarvestLotDto| {
                                                let id = l.id;
                                                view! {
                                                    <tr>
                                                        <td>{l.lot_number.clone()}</td>
                                                        <td>{l.crop_type.clone()}</td>
                                                        <td>
                                                            {l.variety
                                                                .clone()
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>{l.total_weight_kg}</td>
                                                        <td>{l.status.clone()}</td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| (del_lot)(id)
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

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "harvest_deliveries")}</h2>
            {move || {
                let loaded = deliveries.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "delivery_date")}</th>
                                            <th>{crate::t!(t, "lot_id")}</th>
                                            <th>{crate::t!(t, "gross_weight")}</th>
                                            <th>{crate::t!(t, "tare_weight")}</th>
                                            <th>{crate::t!(t, "net_weight")}</th>
                                            <th>{crate::t!(t, "carrier_name")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|d: &api::HarvestDeliveryDto| d.id
                                            children=move |d: api::HarvestDeliveryDto| {
                                                let id = d.id;
                                                view! {
                                                    <tr>
                                                        <td>{d.delivery_date.clone()}</td>
                                                        <td>{d.lot_id.to_string()}</td>
                                                        <td>{d.gross_weight_kg}</td>
                                                        <td>{d.tare_weight_kg}</td>
                                                        <td>{d.net_weight_kg}</td>
                                                        <td>
                                                            {d.carrier_name
                                                                .clone()
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| (del_delivery)(id)
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

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "cold_chain")}</h2>
            {move || {
                let loaded = cold_chain.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "sensor_id")}</th>
                                            <th>{crate::t!(t, "lot_id")}</th>
                                            <th>{crate::t!(t, "recorded_at")}</th>
                                            <th>{crate::t!(t, "temperature_field")}</th>
                                            <th>{crate::t!(t, "humidity_pct")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|c: &api::ColdChainLogDto| c.id
                                            children=move |c: api::ColdChainLogDto| {
                                                let id = c.id;
                                                view! {
                                                    <tr>
                                                        <td>{c.sensor_id.clone()}</td>
                                                        <td>{c.lot_id.to_string()}</td>
                                                        <td>{c.recorded_at.clone()}</td>
                                                        <td>{c.temperature_c}</td>
                                                        <td>
                                                            {c.humidity_pct
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| (del_cold)(id)
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
