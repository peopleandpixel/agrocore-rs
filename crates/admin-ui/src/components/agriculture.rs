//! Specialized crops: olive groves, olive oil records, vineyards and kelter
//! deliveries.
//!
//! The four `/specialized/*` collections in `agriculture.rs` were registered with
//! full CRUD handlers and had no UI caller.
//!
//! `QualityGrade` — which the DTO aliases to `VineyardQualityGrade` — carries no
//! `serde` rename, so it serialises as the variant name (`"Reserva"`,
//! `"GrandeReserva"`), not snake-case. The select sends the variant names.

use crate::api;
use icondata::{LuPlus, LuTrash2};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// The grades `QualityGrade` accepts. `Custom(String)` carries caller text.
const QUALITY_GRADES: [(&str, &str); 6] = [
    ("Reserva", "grade_reserva"),
    ("GrandeReserva", "grade_grande_reserva"),
    ("Garrafeira", "grade_garrafeira"),
    ("Superior", "grade_superior"),
    ("Classic", "grade_classic"),
    ("LateHarvest", "grade_late_harvest"),
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

fn read_opt_num(name: &str, raw: String) -> Result<Option<f64>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    read_num(name, trimmed.to_string()).map(Some)
}

fn read_opt_int(name: &str, raw: String) -> Result<Option<i32>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<i32>()
        .map(Some)
        .map_err(|_| format!("{name}: not a whole number"))
}

fn opt_text(raw: String) -> Option<String> {
    let trimmed = raw.trim().to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

#[component]
pub fn AgricultureManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let groves = LocalResource::new(|| async move { api::fetch_olive_groves().await.ok() });
    let oils = LocalResource::new(|| async move { api::fetch_olive_oil_records().await.ok() });
    let vineyards = LocalResource::new(|| async move { api::fetch_vineyards().await.ok() });
    let deliveries =
        LocalResource::new(|| async move { api::fetch_kelter_deliveries().await.ok() });
    let (reload, set_reload) = signal(0u32);

    // Olive grove
    let (g_site, set_g_site) = signal(String::new());
    let (g_label, set_g_label) = signal(String::new());
    let (g_variety, set_g_variety) = signal(String::new());
    let (g_year, set_g_year) = signal(String::new());
    let (g_area, set_g_area) = signal(String::new());
    let (g_trees, set_g_trees) = signal(String::new());
    let (g_spacing, set_g_spacing) = signal(String::new());
    let (g_irrigation, set_g_irrigation) = signal(String::new());
    let (g_organic, set_g_organic) = signal(false);
    let (g_cert_body, set_g_cert_body) = signal(String::new());
    let (g_cert_number, set_g_cert_number) = signal(String::new());

    // Olive oil record
    let (o_grove, set_o_grove) = signal(String::new());
    let (o_date, set_o_date) = signal(String::new());
    let (o_quantity, set_o_quantity) = signal(String::new());
    let (o_yield_kg, set_o_yield_kg) = signal(String::new());
    let (o_acidity, set_o_acidity) = signal(String::new());
    let (o_peroxide, set_o_peroxide) = signal(String::new());
    let (o_k232, set_o_k232) = signal(String::new());
    let (o_k270, set_o_k270) = signal(String::new());
    let (o_grade, set_o_grade) = signal(String::new());
    let (o_notes, set_o_notes) = signal(String::new());

    // Vineyard
    let (v_site, set_v_site) = signal(String::new());
    let (v_doc_area, set_v_doc_area) = signal(String::new());
    let (v_vintage, set_v_vintage) = signal(String::new());
    let (v_variety, set_v_variety) = signal(String::new());
    let (v_brix, set_v_brix) = signal(String::new());
    let (v_ph, set_v_ph) = signal(String::new());
    let (v_acidity, set_v_acidity) = signal(String::new());
    let (v_yield, set_v_yield) = signal(String::new());
    let (v_grade, set_v_grade) = signal(String::from("Classic"));
    let (v_slope, set_v_slope) = signal(String::new());
    let (v_altitude, set_v_altitude) = signal(String::new());
    let (v_organic, set_v_organic) = signal(false);
    let (v_cert_body, set_v_cert_body) = signal(String::new());
    let (v_cert_number, set_v_cert_number) = signal(String::new());

    // Kelter delivery
    let (k_vineyard, set_k_vineyard) = signal(String::new());
    let (k_date, set_k_date) = signal(String::new());
    let (k_gross, set_k_gross) = signal(String::new());
    let (k_net, set_k_net) = signal(String::new());
    let (k_lot, set_k_lot) = signal(String::new());
    let (k_kelter, set_k_kelter) = signal(String::new());
    let (k_transport, set_k_transport) = signal(String::new());
    let (k_temp, set_k_temp) = signal(String::new());
    let (k_notes, set_k_notes) = signal(String::new());

    let (error, set_error) = signal(None::<String>);

    let on_create_grove = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let site = match read_uuid("site_id", g_site.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let label = g_label.get().trim().to_string();
        let variety = g_variety.get().trim().to_string();
        if label.is_empty() || variety.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }
        let area = match read_num("area", g_area.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let req = api::CreateOliveGroveRequest {
            site_id: site,
            label,
            variety,
            planting_year: match read_opt_int("planting_year", g_year.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            area_ha: area,
            tree_count: match read_opt_int("tree_count", g_trees.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            spacing_m: match read_opt_num("spacing", g_spacing.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            irrigation_type: opt_text(g_irrigation.get()),
            is_organic: Some(g_organic.get()),
            certification_body: opt_text(g_cert_body.get()),
            certification_number: opt_text(g_cert_number.get()),
        };
        spawn_local(async move {
            match api::create_olive_grove(req).await {
                Ok(_) => {
                    set_g_label.set(String::new());
                    set_g_variety.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_oil = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let grove = match read_uuid("olive_grove_id", o_grove.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let quantity = match read_num("quantity", o_quantity.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let oil = match read_num("oil_yield", o_yield_kg.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        // `oil_yield_percent` is `f64` with no Option and is capped at 100 by the
        // DTO, so it is required and computed from the two weights above.
        let percent = if quantity > 0.0 {
            oil / quantity * 100.0
        } else {
            0.0
        };
        let req = api::CreateOliveOilRecordRequest {
            olive_grove_id: grove,
            harvest_date: o_date.get().trim().to_string(),
            quantity_kg: quantity,
            oil_yield_kg: oil,
            oil_yield_percent: percent,
            acidity_percent: match read_opt_num("acidity", o_acidity.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            peroxide_value: match read_opt_num("peroxide", o_peroxide.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            k232: match read_opt_num("k232", o_k232.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            k270: match read_opt_num("k270", o_k270.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            quality_grade: opt_text(o_grade.get()),
            notes: opt_text(o_notes.get()),
        };
        spawn_local(async move {
            match api::create_olive_oil_record(req).await {
                Ok(_) => {
                    set_o_quantity.set(String::new());
                    set_o_yield_kg.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_vineyard = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let site = match read_uuid("site_id", v_site.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let req = api::CreateVineyardRequest {
            site_id: site,
            doc_area: opt_text(v_doc_area.get()),
            vintage: match read_opt_int("vintage", v_vintage.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            grape_variety: opt_text(v_variety.get()),
            brix_at_harvest: match read_opt_num("brix", v_brix.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            ph_at_harvest: match read_opt_num("ph", v_ph.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            acidity: match read_opt_num("acidity", v_acidity.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            yield_tons: match read_opt_num("yield", v_yield.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            quality_grade: opt_text(v_grade.get()),
            slope_percent: match read_opt_num("slope", v_slope.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            altitude_m: match read_opt_num("altitude", v_altitude.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            is_organic: v_organic.get(),
            certification_body: opt_text(v_cert_body.get()),
            certification_number: opt_text(v_cert_number.get()),
        };
        spawn_local(async move {
            match api::create_vineyard(req).await {
                Ok(_) => {
                    set_v_variety.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_delivery = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let vineyard = match read_uuid("vineyard_id", k_vineyard.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let lot = k_lot.get().trim().to_string();
        let kelter = k_kelter.get().trim().to_string();
        if lot.is_empty() || kelter.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }
        let gross = match read_num("gross", k_gross.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let net = match read_num("net", k_net.get()) {
            Ok(v) => v,
            Err(e) => {
                set_error.set(Some(e));
                return;
            }
        };
        let req = api::CreateKelterDeliveryRequest {
            vineyard_id: vineyard,
            delivery_date: k_date.get().trim().to_string(),
            gross_weight_kg: gross,
            net_weight_kg: net,
            lot_number: lot,
            kelter_name: kelter,
            transport_company: opt_text(k_transport.get()),
            temperature_c: match read_opt_num("temperature", k_temp.get()) {
                Ok(v) => v,
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            },
            notes: opt_text(k_notes.get()),
        };
        spawn_local(async move {
            match api::create_kelter_delivery(req).await {
                Ok(_) => {
                    set_k_lot.set(String::new());
                    set_k_gross.set(String::new());
                    set_k_net.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    macro_rules! deleter {
        ($api_fn:path) => {
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
    let del_grove = deleter!(api::delete_olive_grove);
    let del_oil = deleter!(api::delete_olive_oil_record);
    let del_vineyard = deleter!(api::delete_vineyard);
    let del_delivery = deleter!(api::delete_kelter_delivery);

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4">{crate::t!(t, "nav_agriculture")}</h1>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-6 mb-8">
                <form class="card bg-base-100 shadow p-4" on:submit=on_create_grove>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "olive_groves")}</h2>
                    <div class="grid grid-cols-2 gap-3">
                        {text(crate::t!(t, "plot_id_field")().to_string(), g_site, set_g_site)}
                        {text(crate::t!(t, "tree_label")().to_string(), g_label, set_g_label)}
                        {text(crate::t!(t, "lot_variety")().to_string(), g_variety, set_g_variety)}
                        {num(crate::t!(t, "planting_year")().to_string(), g_year, set_g_year)}
                        {num(crate::t!(t, "area_ha")().to_string(), g_area, set_g_area)}
                        {num(crate::t!(t, "tree_count")().to_string(), g_trees, set_g_trees)}
                        {num(crate::t!(t, "spacing_m")().to_string(), g_spacing, set_g_spacing)}
                        {text(
                            crate::t!(t, "irrigation_type")().to_string(),
                            g_irrigation,
                            set_g_irrigation,
                        )}
                        {text(
                            crate::t!(t, "certification_body")().to_string(),
                            g_cert_body,
                            set_g_cert_body,
                        )}
                        {text(
                            crate::t!(t, "certification_number")().to_string(),
                            g_cert_number,
                            set_g_cert_number,
                        )}
                        <label class="label cursor-pointer justify-start gap-2">
                            <input
                                type="checkbox"
                                class="checkbox"
                                prop:checked=move || g_organic.get()
                                on:change=move |ev| set_g_organic.set(event_target_checked(&ev))
                            />
                            <span class="label-text">{crate::t!(t, "is_organic")}</span>
                        </label>
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_oil>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "olive_oil_records")}</h2>
                    <div class="grid grid-cols-2 gap-3">
                        {text(
                            crate::t!(t, "olive_grove_id")().to_string(),
                            o_grove,
                            set_o_grove,
                        )}
                        {date(
                            crate::t!(t, "harvest_date")().to_string(),
                            o_date,
                            set_o_date,
                        )}
                        {num(
                            crate::t!(t, "olive_quantity")().to_string(),
                            o_quantity,
                            set_o_quantity,
                        )}
                        {num(
                            crate::t!(t, "olive_yield")().to_string(),
                            o_yield_kg,
                            set_o_yield_kg,
                        )}
                        {num(
                            crate::t!(t, "acidity_pct")().to_string(),
                            o_acidity,
                            set_o_acidity,
                        )}
                        {num(
                            crate::t!(t, "peroxide_value")().to_string(),
                            o_peroxide,
                            set_o_peroxide,
                        )}
                        {num(crate::t!(t, "k232")().to_string(), o_k232, set_o_k232)}
                        {num(crate::t!(t, "k270")().to_string(), o_k270, set_o_k270)}
                        {text(
                            crate::t!(t, "quality_target")().to_string(),
                            o_grade,
                            set_o_grade,
                        )}
                        {text(crate::t!(t, "notes_field")().to_string(), o_notes, set_o_notes)}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_vineyard>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "vineyards")}</h2>
                    <div class="grid grid-cols-2 gap-3">
                        {text(crate::t!(t, "plot_id_field")().to_string(), v_site, set_v_site)}
                        {text(
                            crate::t!(t, "doc_area")().to_string(),
                            v_doc_area,
                            set_v_doc_area,
                        )}
                        {num(crate::t!(t, "vintage")().to_string(), v_vintage, set_v_vintage)}
                        {text(
                            crate::t!(t, "grape_variety")().to_string(),
                            v_variety,
                            set_v_variety,
                        )}
                        {num(crate::t!(t, "brix")().to_string(), v_brix, set_v_brix)}
                        {num(crate::t!(t, "ph")().to_string(), v_ph, set_v_ph)}
                        {num(crate::t!(t, "acidity")().to_string(), v_acidity, set_v_acidity)}
                        {num(crate::t!(t, "yield_tons")().to_string(), v_yield, set_v_yield)}
                        <label class="form-control">
                            <span class="label-text">{crate::t!(t, "quality_grade")}</span>
                            <select
                                class="select select-bordered"
                                prop:value=move || v_grade.get()
                                on:change=move |ev| set_v_grade.set(event_target_value(&ev))
                            >
                                {QUALITY_GRADES
                                    .iter()
                                    .map(|(value, key)| {
                                        view! {
                                            <option value=*value>{crate::t!(t, *key)()}</option>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </select>
                        </label>
                        {num(crate::t!(t, "slope_percent")().to_string(), v_slope, set_v_slope)}
                        {num(
                            crate::t!(t, "altitude_m")().to_string(),
                            v_altitude,
                            set_v_altitude,
                        )}
                        {text(
                            crate::t!(t, "certification_body")().to_string(),
                            v_cert_body,
                            set_v_cert_body,
                        )}
                        {text(
                            crate::t!(t, "certification_number")().to_string(),
                            v_cert_number,
                            set_v_cert_number,
                        )}
                        <label class="label cursor-pointer justify-start gap-2">
                            <input
                                type="checkbox"
                                class="checkbox"
                                prop:checked=move || v_organic.get()
                                on:change=move |ev| set_v_organic.set(event_target_checked(&ev))
                            />
                            <span class="label-text">{crate::t!(t, "is_organic")}</span>
                        </label>
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>

                <form class="card bg-base-100 shadow p-4" on:submit=on_create_delivery>
                    <h2 class="font-semibold mb-3">{crate::t!(t, "kelter_deliveries")}</h2>
                    <div class="grid grid-cols-2 gap-3">
                        {text(
                            crate::t!(t, "vineyard_id")().to_string(),
                            k_vineyard,
                            set_k_vineyard,
                        )}
                        {date(
                            crate::t!(t, "delivery_date")().to_string(),
                            k_date,
                            set_k_date,
                        )}
                        {num(
                            crate::t!(t, "gross_weight")().to_string(),
                            k_gross,
                            set_k_gross,
                        )}
                        {num(crate::t!(t, "net_weight")().to_string(), k_net, set_k_net)}
                        {text(crate::t!(t, "lot_number")().to_string(), k_lot, set_k_lot)}
                        {text(
                            crate::t!(t, "kelter_name")().to_string(),
                            k_kelter,
                            set_k_kelter,
                        )}
                        {text(
                            crate::t!(t, "transport_company")().to_string(),
                            k_transport,
                            set_k_transport,
                        )}
                        {num(
                            crate::t!(t, "temperature_field")().to_string(),
                            k_temp,
                            set_k_temp,
                        )}
                        {text(crate::t!(t, "notes_field")().to_string(), k_notes, set_k_notes)}
                    </div>
                    <button type="submit" class="btn btn-primary mt-3">
                        <Icon icon=LuPlus width="16" height="16" />
                        {crate::t!(t, "btn_add")}
                    </button>
                </form>
            </div>

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "olive_groves")}</h2>
            {move || {
                let _ = reload.get();
                let loaded = groves.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "tree_label")}</th>
                                            <th>{crate::t!(t, "lot_variety")}</th>
                                            <th>{crate::t!(t, "planting_year")}</th>
                                            <th>{crate::t!(t, "area_ha")}</th>
                                            <th>{crate::t!(t, "tree_count")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|g: &api::OliveGroveDto| g.id
                                            children=move |g: api::OliveGroveDto| {
                                                let id = g.id;
                                                view! {
                                                    <tr>
                                                        <td>{g.label.clone()}</td>
                                                        <td>{g.variety.clone()}</td>
                                                        <td>
                                                            {g.planting_year
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>{g.area_ha}</td>
                                                        <td>
                                                            {g.tree_count
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| (del_grove)(id)
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

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "olive_oil_records")}</h2>
            {move || {
                let loaded = oils.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "harvest_date")}</th>
                                            <th>{crate::t!(t, "olive_grove_id")}</th>
                                            <th>{crate::t!(t, "olive_quantity")}</th>
                                            <th>{crate::t!(t, "olive_yield")}</th>
                                            <th>{crate::t!(t, "quality_grade")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|o: &api::OliveOilRecordDto| o.id
                                            children=move |o: api::OliveOilRecordDto| {
                                                let id = o.id;
                                                view! {
                                                    <tr>
                                                        <td>{o.harvest_date.clone()}</td>
                                                        <td>{o.olive_grove_id.to_string()}</td>
                                                        <td>{o.quantity_kg}</td>
                                                        <td>{o.oil_yield_kg}</td>
                                                        <td>
                                                            {o.quality_grade
                                                                .clone()
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| (del_oil)(id)
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

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "vineyards")}</h2>
            {move || {
                let loaded = vineyards.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto mb-6">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "vintage")}</th>
                                            <th>{crate::t!(t, "grape_variety")}</th>
                                            <th>{crate::t!(t, "brix")}</th>
                                            <th>{crate::t!(t, "ph")}</th>
                                            <th>{crate::t!(t, "yield_tons")}</th>
                                            <th>{crate::t!(t, "quality_grade")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|v: &api::VineyardDto| v.id
                                            children=move |v: api::VineyardDto| {
                                                let id = v.id;
                                                view! {
                                                    <tr>
                                                        <td>
                                                            {v.vintage
                                                                .map(|y| y.to_string())
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            {v.grape_variety
                                                                .clone()
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            {v.brix_at_harvest
                                                                .map(|x| x.to_string())
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            {v.ph_at_harvest
                                                                .map(|x| x.to_string())
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            {v.yield_tons
                                                                .map(|x| x.to_string())
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            {v.quality_grade
                                                                .clone()
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| (del_vineyard)(id)
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

            <h2 class="text-lg font-semibold mb-2">{crate::t!(t, "kelter_deliveries")}</h2>
            {move || {
                let loaded = deliveries.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "delivery_date")}</th>
                                            <th>{crate::t!(t, "lot_number")}</th>
                                            <th>{crate::t!(t, "kelter_name")}</th>
                                            <th>{crate::t!(t, "gross_weight")}</th>
                                            <th>{crate::t!(t, "net_weight")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|k: &api::KelterDeliveryDto| k.id
                                            children=move |k: api::KelterDeliveryDto| {
                                                let id = k.id;
                                                view! {
                                                    <tr>
                                                        <td>{k.delivery_date.clone()}</td>
                                                        <td>{k.lot_number.clone()}</td>
                                                        <td>{k.kelter_name.clone()}</td>
                                                        <td>{k.gross_weight_kg}</td>
                                                        <td>{k.net_weight_kg}</td>
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
