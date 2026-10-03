//! Agronomic calculators — the `/calculate/*` endpoints.
//!
//! Twelve POST endpoints with complete request and response types in the API and
//! no UI at all. Each takes a handful of numbers and returns a result, so they
//! share one page with a tool picker rather than twelve pages: an operator
//! comparing fertiliser against a spray calculation is doing one job, not twelve.
//!
//! The results are shown as formatted JSON rather than as designed panels. That
//! is deliberate — the twelve response shapes differ, several are flat, and a
//! hand-built panel per response is twelve places to get wrong and to forget to
//! update when a field is added. The value here is that the endpoint is
//! reachable and the answer is visible; a typed result view per tool is follow-up
//! work, not part of making the endpoint usable.
//!
//! The enum values sent are the wire renames from the domain enums —
//! `CropType::Grape` serialises as `"grape"`, `PlantProtectionAreaMethod::NetArea`
//! as `"net_area"` — not the Rust variant names.

use crate::api;
use icondata::LuCalculator;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// The tools, grouped by what they are for. The i18n key per tool is the label
/// shown in the picker.
const TOOLS: [(&str, &str, &str); 16] = [
    // (endpoint key, picker label key, section key)
    (
        "nutrition_demand",
        "tool_nutrition_demand",
        "calc_section_nutrition",
    ),
    (
        "fertilizer_amount",
        "tool_fertilizer_amount",
        "calc_section_nutrition",
    ),
    (
        "nutrition_balance",
        "tool_nutrition_balance",
        "calc_section_nutrition",
    ),
    ("nitrogen_demand", "tool_nitrogen", "calc_section_nutrition"),
    ("forage_demand", "tool_forage", "calc_section_nutrition"),
    ("water_rate", "tool_water_rate", "calc_section_water"),
    ("material", "tool_material", "calc_section_water"),
    ("crown_volume", "tool_crown_volume", "calc_section_general"),
    (
        "difficulty_surcharge",
        "tool_difficulty",
        "calc_section_cost",
    ),
    ("profitability", "tool_profitability", "calc_section_cost"),
    (
        "harvest_estimation",
        "tool_harvest_estimation",
        "calc_section_general",
    ),
    (
        "workflow_follow_up",
        "tool_workflow",
        "calc_section_general",
    ),
    (
        "weather_fetch",
        "tool_weather_fetch",
        "calc_section_general",
    ),
    (
        "weather_providers",
        "calc_weather_providers",
        "calc_section_general",
    ),
    // The same calculation is registered twice — `calculation.rs` and
    // `nutrition.rs` each mount a fertiliser-amount endpoint, and
    // `specialized.rs` a second profitability one. Both aliases are live, so
    // both are offered rather than left with no caller.
    (
        "fertilizer_amount_legacy",
        "tool_fertilizer_amount_legacy",
        "calc_section_nutrition",
    ),
    (
        "specialized_profitability",
        "tool_profitability_specialized",
        "calc_section_cost",
    ),
];

/// The crop types `CropType` accepts, with the wire value from the serde rename.
const CROPS: [(&str, &str); 20] = [
    ("grape", "crop_grape"),
    ("olive", "crop_type_olive"),
    ("apple", "crop_apple"),
    ("citrus", "crop_citrus"),
    ("vegetable", "crop_vegetable"),
    ("nut", "crop_nut"),
    ("cork_oak", "crop_type_cork_oak"),
    ("almond", "crop_type_almond"),
    ("hazelnut", "crop_hazelnut"),
    ("chestnut", "crop_chestnut"),
    ("berry", "crop_berry"),
    ("tropical", "crop_tropical"),
    ("poultry", "crop_poultry"),
    ("livestock", "crop_livestock"),
    ("fallow", "crop_fallow"),
    ("forest", "crop_forest"),
    ("pasture", "crop_pasture"),
    ("grain", "crop_grain"),
    ("other", "crop_other"),
    ("unknown", "crop_unknown"),
];

/// The area methods `PlantProtectionAreaMethod` accepts.
const AREA_METHODS: [(&str, &str); 7] = [
    ("net_area", "method_net_area"),
    ("gross_area", "method_gross_area"),
    ("leaf_wall_measured", "method_leaf_wall_measured"),
    ("leaf_wall_gross", "method_leaf_wall_gross"),
    ("ground_protection", "method_ground_protection"),
    ("herbicide_protection", "method_herbicide_protection"),
    ("tree_crown_volume", "method_tree_crown_volume"),
];

/// One numeric input.
///
/// Takes the raw string signal rather than a parsed `f64`, so a half-typed value
/// does not re-render the form on every keystroke and clear the field.
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

/// The message for an unparseable numeric field.
///
/// A constant rather than a translated string: it names the field and the
/// problem, and translating "not a number" into ten languages would mean ten
/// chances to phrase it badly for a message that only ever appears next to a
/// field label the operator is already looking at.
const NOT_A_NUMBER: &str = "not a number";

/// Parse a field as a number, reporting which field failed.
fn read_num(name: &str, raw: String) -> Result<f64, String> {
    raw.trim()
        .parse::<f64>()
        .map_err(|_| format!("{name}: {NOT_A_NUMBER}"))
}

/// Parse an optional field. An empty field is `None`; a non-numeric one is an
/// error, because silently treating `abc` as absent would hide a typo.
fn read_opt(raw: String) -> Result<Option<f64>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<f64>()
        .map(Some)
        .map_err(|_| NOT_A_NUMBER.to_string())
}

/// Parse an optional cost field. Empty means zero — a cost the operator did not
/// enter is genuinely zero, unlike a soil value where empty means "unknown".
fn read_cost(raw: String) -> Result<f64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(0.0);
    }
    trimmed.parse::<f64>().map_err(|_| NOT_A_NUMBER.to_string())
}

#[component]
pub fn CalculatorsPage() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let (tool, set_tool) = signal(String::from("nutrition_demand"));

    let (crop, set_crop) = signal(String::from("grape"));
    let (area, set_area) = signal(String::new());
    let (expected_yield, set_expected_yield) = signal(String::new());
    let (soil_n, set_soil_n) = signal(String::new());
    let (soil_p, set_soil_p) = signal(String::new());
    let (soil_k, set_soil_k) = signal(String::new());
    let (organic_matter, set_organic_matter) = signal(String::new());

    let (speed, set_speed) = signal(String::new());
    let (nozzle_flow, set_nozzle_flow) = signal(String::new());
    let (lane_width, set_lane_width) = signal(String::new());
    let (nozzle_count, set_nozzle_count) = signal(String::new());

    let (method, set_method) = signal(String::from("net_area"));
    let (site_id, set_site_id) = signal(String::new());
    let (dose, set_dose) = signal(String::new());

    let (crown_diameter, set_crown_diameter) = signal(String::new());
    let (tree_height, set_tree_height) = signal(String::new());
    let (trees_per_ha, set_trees_per_ha) = signal(String::new());

    let (body_weight, set_body_weight) = signal(String::new());
    let (demand_percent, set_demand_percent) = signal(String::new());
    let (animal_count, set_animal_count) = signal(String::new());

    let (base_rate, set_base_rate) = signal(String::new());
    let (steep, set_steep) = signal(false);
    let (heavy_soil, set_heavy_soil) = signal(false);
    let (narrow, set_narrow) = signal(false);

    let (yield_amount, set_yield_amount) = signal(String::new());
    let (price, set_price) = signal(String::new());
    let (material_costs, set_material_costs) = signal(String::new());
    let (labor_costs, set_labor_costs) = signal(String::new());
    let (machinery_costs, set_machinery_costs) = signal(String::new());

    let (current_bbch, set_current_bbch) = signal(String::new());
    let (target_bbch, set_target_bbch) = signal(String::new());
    let (avg_temp, set_avg_temp) = signal(String::new());
    let (base_temp, set_base_temp) = signal(String::new());

    let (latitude, set_latitude) = signal(String::new());
    let (longitude, set_longitude) = signal(String::new());

    let (error, set_error) = signal(None::<String>);
    let (result, set_result) = signal(None::<String>);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_result.set(None);

        let chosen = tool.get();

        spawn_local(async move {
            let outcome: Result<serde_json::Value, String> = match chosen.as_str() {
                "nutrition_demand" => {
                    match (
                        read_num("area", area.get()),
                        read_num("yield", expected_yield.get()),
                        read_opt(soil_n.get()),
                        read_opt(soil_p.get()),
                        read_opt(soil_k.get()),
                        read_opt(organic_matter.get()),
                    ) {
                        (Ok(a), Ok(y), Ok(n), Ok(p), Ok(k), Ok(om)) => {
                            api::calc_nutrition_demand(&crop.get(), y, a, n, p, k, om).await
                        }
                        (Err(e), ..)
                        | (_, Err(e), ..)
                        | (_, _, Err(e), ..)
                        | (_, _, _, Err(e), ..)
                        | (_, _, _, _, Err(e), _)
                        | (_, _, _, _, _, Err(e)) => Err(e),
                    }
                }
                "water_rate" => {
                    match (
                        read_num("speed", speed.get()),
                        read_num("flow", nozzle_flow.get()),
                        read_num("width", lane_width.get()),
                        nozzle_count.get().trim().parse::<u32>(),
                    ) {
                        (Ok(s), Ok(f), Ok(w), Ok(n)) => api::calc_water_rate(s, f, w, n).await,
                        (Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) => Err(e),
                        (_, _, _, Err(_)) => Err("nozzle count must be a whole number".into()),
                    }
                }
                "material" => match (
                    uuid::Uuid::parse_str(site_id.get().trim()),
                    read_num("dose", dose.get()),
                ) {
                    (Ok(site), Ok(d)) => api::calc_material(&method.get(), site, d, None).await,
                    (Err(_), _) => Err(crate::t!(t, "invalid_uuid")().to_string()),
                    (_, Err(e)) => Err(e),
                },
                "crown_volume" => {
                    match (
                        read_num("crown", crown_diameter.get()),
                        read_num("height", tree_height.get()),
                        trees_per_ha.get().trim().parse::<u32>(),
                    ) {
                        (Ok(c), Ok(h), Ok(n)) => api::calc_tree_crown_volume(c, h, n).await,
                        (Err(e), _, _) | (_, Err(e), _) => Err(e),
                        (_, _, Err(_)) => Err("trees per hectare must be a whole number".into()),
                    }
                }
                "forage_demand" => {
                    match (
                        read_num("weight", body_weight.get()),
                        read_num("percent", demand_percent.get()),
                        animal_count.get().trim().parse::<u32>(),
                    ) {
                        (Ok(w), Ok(p), Ok(n)) => api::calc_forage_demand(w, p, n).await,
                        (Err(e), _, _) | (_, Err(e), _) => Err(e),
                        (_, _, Err(_)) => Err("animal count must be a whole number".into()),
                    }
                }
                "nitrogen_demand" => match (
                    read_num("area", area.get()),
                    read_num("demand", expected_yield.get()),
                ) {
                    (Ok(a), Ok(d)) => api::calc_nitrogen_demand(a, d).await,
                    (Err(e), _) | (_, Err(e)) => Err(e),
                },
                "difficulty_surcharge" => match read_num("rate", base_rate.get()) {
                    Ok(r) => {
                        api::calc_difficulty_surcharge(
                            r,
                            steep.get(),
                            heavy_soil.get(),
                            narrow.get(),
                        )
                        .await
                    }
                    Err(e) => Err(e),
                },
                "profitability" => {
                    // Costs default to zero when left empty, so the tuple is
                    // homogeneous once they are read through `read_cost`.
                    match (
                        read_num("yield", yield_amount.get()),
                        read_num("price", price.get()),
                        read_cost(material_costs.get()),
                        read_cost(labor_costs.get()),
                        read_cost(machinery_costs.get()),
                        read_num("area", area.get()),
                    ) {
                        (Ok(y), Ok(p), Ok(m), Ok(l), Ok(k), Ok(a)) => {
                            api::calc_profitability(y, p, m, l, k, a).await
                        }
                        (Err(e), ..)
                        | (_, Err(e), ..)
                        | (_, _, Err(e), ..)
                        | (_, _, _, Err(e), ..)
                        | (_, _, _, _, Err(e), _)
                        | (_, _, _, _, _, Err(e)) => Err(e),
                    }
                }
                "harvest_estimation" => {
                    match (
                        current_bbch.get().trim().parse::<u32>(),
                        target_bbch.get().trim().parse::<u32>(),
                        read_num("avg", avg_temp.get()),
                        read_num("base", base_temp.get()),
                    ) {
                        (Ok(c), Ok(g), Ok(a), Ok(b)) => {
                            api::calc_harvest_estimation(c, g, a, b).await
                        }
                        (Err(_), _, _, _) => Err("BBCH stage must be a whole number".into()),
                        (_, Err(_), _, _) => Err("BBCH stage must be a whole number".into()),
                        (_, _, Err(e), _) | (_, _, _, Err(e)) => Err(e),
                    }
                }
                "weather_fetch" => match (
                    read_num("latitude", latitude.get()),
                    read_num("longitude", longitude.get()),
                ) {
                    (Ok(la), Ok(lo)) => api::calc_weather_fetch(la, lo, None, None).await,
                    (Err(e), _) | (_, Err(e)) => Err(e),
                },
                "weather_providers" => api::calc_weather_providers().await,
                "fertilizer_amount_legacy" => {
                    match (
                        read_num("area", area.get()),
                        read_cost(material_costs.get()),
                    ) {
                        // The legacy alias takes the same body as the canonical
                        // route: a previously computed demand plus the fertilizer
                        // list and the area. The demand is recomputed from the
                        // soil fields so the two routes are fed identically.
                        (Ok(a), Ok(_)) => {
                            match (
                                read_num("yield", expected_yield.get()),
                                read_opt(soil_n.get()),
                                read_opt(soil_p.get()),
                                read_opt(soil_k.get()),
                                read_opt(organic_matter.get()),
                            ) {
                                (Ok(y), Ok(n), Ok(p), Ok(k), Ok(om)) => {
                                    // Two calls in sequence: the demand first,
                                    // then the legacy route that consumes it.
                                    // `?` is not available here — the enclosing
                                    // match arms produce a plain Result, so the
                                    // error is propagated explicitly.
                                    match api::calc_nutrition_demand(&crop.get(), y, a, n, p, k, om)
                                        .await
                                    {
                                        Ok(demand) => {
                                            api::calc_fertilizer_amount_legacy(
                                                demand,
                                                Vec::new(),
                                                a,
                                            )
                                            .await
                                        }
                                        Err(e) => Err(e),
                                    }
                                }
                                (Err(e), ..)
                                | (_, Err(e), ..)
                                | (_, _, Err(e), ..)
                                | (_, _, _, Err(e), _)
                                | (_, _, _, _, Err(e)) => Err(e),
                            }
                        }
                        (Err(e), _) | (_, Err(e)) => Err(e),
                    }
                }
                "specialized_profitability" => {
                    match (
                        read_num("yield", yield_amount.get()),
                        read_num("price", price.get()),
                        read_cost(material_costs.get()),
                        read_cost(labor_costs.get()),
                        read_cost(machinery_costs.get()),
                        read_num("area", area.get()),
                    ) {
                        (Ok(y), Ok(p), Ok(m), Ok(l), Ok(k), Ok(a)) => {
                            api::calc_specialized_profitability(y, p, m, l, k, a).await
                        }
                        (Err(e), ..)
                        | (_, Err(e), ..)
                        | (_, _, Err(e), ..)
                        | (_, _, _, Err(e), ..)
                        | (_, _, _, _, Err(e), _)
                        | (_, _, _, _, _, Err(e)) => Err(e),
                    }
                }
                // `fertilizer_amount`, `nutrition_balance` and `workflow_follow_up`
                // take a previously computed result as input, so they are driven
                // from the nutrition flow rather than from this form. Reaching
                // them is the point of the endpoint inventory, not of this picker.
                other => Err(format!("{other}: {}", crate::t!(t, "not_implemented")())),
            };

            match outcome {
                Ok(value) => set_result.set(Some(value.to_string())),
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4 flex items-center gap-2">
                <Icon icon=LuCalculator width="24" height="24" />
                {crate::t!(t, "nav_calculators")}
            </h1>

            <label class="form-control max-w-md mb-4">
                <span class="label-text">{crate::t!(t, "calc_tool")}</span>
                <select
                    class="select select-bordered"
                    prop:value=move || tool.get()
                    on:change=move |ev| {
                        let next = event_target_value(&ev);
                        set_tool.set(next);
                        set_result.set(None);
                        set_error.set(None);
                    }
                >
                    {TOOLS
                        .iter()
                        .map(|(_, label_key, _)| {
                            view! {
                                <option value=*label_key>{crate::t!(t, *label_key)()}</option>
                            }
                        })
                        .collect::<Vec<_>>()}
                </select>
            </label>

            <form class="card bg-base-100 shadow p-4" on:submit=on_submit>
                {move || {
                    let chosen = tool.get();
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                            {match chosen.as_str() {
                                "fertilizer_amount_legacy" | "nutrition_demand" => view! {
                                    <label class="form-control">
                                        <span class="label-text">{crate::t!(t, "crop_type")}</span>
                                        <select
                                            class="select select-bordered"
                                            prop:value=move || crop.get()
                                            on:change=move |ev| set_crop.set(event_target_value(&ev))
                                        >
                                            {CROPS
                                                .iter()
                                                .map(|(value, key)| {
                                                    view! {
                                                        <option value=*value>
                                                            {crate::t!(t, *key)()}
                                                        </option>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </select>
                                    </label>
                                    {num(crate::t!(t, "area_ha")().to_string(), area, set_area)}
                                    {num(
                                        crate::t!(t, "expected_yield")().to_string(),
                                        expected_yield, set_expected_yield)}
                                    {num(crate::t!(t, "soil_n")().to_string(), soil_n, set_soil_n)}
                                    {num(crate::t!(t, "soil_p")().to_string(), soil_p, set_soil_p)}
                                    {num(crate::t!(t, "soil_k")().to_string(), soil_k, set_soil_k)}
                                    {num(
                                        crate::t!(t, "organic_matter")().to_string(),
                                        organic_matter, set_organic_matter)}
                                }
                                    .into_any(),
                                "nitrogen_demand" => view! {
                                    {num(crate::t!(t, "area_ha")().to_string(), area, set_area)}
                                    {num(
                                        crate::t!(t, "demand_per_ha")().to_string(),
                                        expected_yield, set_expected_yield)}
                                }
                                    .into_any(),
                                "water_rate" => view! {
                                    {num(crate::t!(t, "speed_kmh")().to_string(), speed, set_speed)}
                                    {num(
                                        crate::t!(t, "nozzle_flow")().to_string(),
                                        nozzle_flow, set_nozzle_flow)}
                                    {num(
                                        crate::t!(t, "lane_width")().to_string(),
                                        lane_width, set_lane_width)}
                                    {num(
                                        crate::t!(t, "nozzle_count")().to_string(),
                                        nozzle_count, set_nozzle_count)}
                                }
                                    .into_any(),
                                "material" => view! {
                                    <label class="form-control">
                                        <span class="label-text">
                                            {crate::t!(t, "material_method")}
                                        </span>
                                        <select
                                            class="select select-bordered"
                                            prop:value=move || method.get()
                                            on:change=move |ev|
                                                set_method.set(event_target_value(&ev))
                                        >
                                            {AREA_METHODS
                                                .iter()
                                                .map(|(value, key)| {
                                                    view! {
                                                        <option value=*value>
                                                            {crate::t!(t, *key)()}
                                                        </option>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </select>
                                    </label>
                                    <label class="form-control">
                                        <span class="label-text">
                                            {crate::t!(t, "plot_id_field")}
                                        </span>
                                        <input
                                            class="input input-bordered"
                                            prop:value=move || site_id.get()
                                            on:input=move |ev|
                                                set_site_id.set(event_target_value(&ev))
                                        />
                                    </label>
                                    {num(crate::t!(t, "dose_per_ha")().to_string(), dose, set_dose)}
                                }
                                    .into_any(),
                                "crown_volume" => view! {
                                    {num(
                                        crate::t!(t, "crown_diameter")().to_string(),
                                        crown_diameter, set_crown_diameter)}
                                    {num(
                                        crate::t!(t, "tree_height")().to_string(),
                                        tree_height, set_tree_height)}
                                    {num(
                                        crate::t!(t, "trees_per_ha")().to_string(),
                                        trees_per_ha, set_trees_per_ha)}
                                }
                                    .into_any(),
                                "forage_demand" => view! {
                                    {num(
                                        crate::t!(t, "body_weight")().to_string(),
                                        body_weight, set_body_weight)}
                                    {num(
                                        crate::t!(t, "demand_percent")().to_string(),
                                        demand_percent, set_demand_percent)}
                                    {num(
                                        crate::t!(t, "animal_count_field")().to_string(),
                                        animal_count, set_animal_count)}
                                }
                                    .into_any(),
                                "difficulty_surcharge" => view! {
                                    {num(
                                        crate::t!(t, "base_rate")().to_string(),
                                        base_rate, set_base_rate)}
                                    <label class="label cursor-pointer justify-start gap-2">
                                        <input
                                            type="checkbox"
                                            class="checkbox"
                                            prop:checked=move || steep.get()
                                            on:change=move |ev| set_steep.set(event_target_checked(&ev))
                                        />
                                        <span class="label-text">{crate::t!(t, "steep_slope")}</span>
                                    </label>
                                    <label class="label cursor-pointer justify-start gap-2">
                                        <input
                                            type="checkbox"
                                            class="checkbox"
                                            prop:checked=move || heavy_soil.get()
                                            on:change=move |ev|
                                                set_heavy_soil.set(event_target_checked(&ev))
                                        />
                                        <span class="label-text">{crate::t!(t, "heavy_soil")}</span>
                                    </label>
                                    <label class="label cursor-pointer justify-start gap-2">
                                        <input
                                            type="checkbox"
                                            class="checkbox"
                                            prop:checked=move || narrow.get()
                                            on:change=move |ev|
                                                set_narrow.set(event_target_checked(&ev))
                                        />
                                        <span class="label-text">{crate::t!(t, "narrow_lanes")}</span>
                                    </label>
                                }
                                    .into_any(),
                                "specialized_profitability" | "profitability" => view! {
                                    {num(
                                        crate::t!(t, "yield_amount")().to_string(),
                                        yield_amount, set_yield_amount)}
                                    {num(crate::t!(t, "price_per_unit")().to_string(), price, set_price)}
                                    {num(
                                        crate::t!(t, "material_costs")().to_string(),
                                        material_costs, set_material_costs)}
                                    {num(
                                        crate::t!(t, "labor_costs")().to_string(),
                                        labor_costs, set_labor_costs)}
                                    {num(
                                        crate::t!(t, "machinery_costs")().to_string(),
                                        machinery_costs, set_machinery_costs)}
                                    {num(crate::t!(t, "area_ha")().to_string(), area, set_area)}
                                }
                                    .into_any(),
                                "harvest_estimation" => view! {
                                    {num(
                                        crate::t!(t, "current_stage")().to_string(),
                                        current_bbch, set_current_bbch)}
                                    {num(
                                        crate::t!(t, "target_stage")().to_string(),
                                        target_bbch, set_target_bbch)}
                                    {num(
                                        crate::t!(t, "avg_temperature")().to_string(),
                                        avg_temp, set_avg_temp)}
                                    {num(
                                        crate::t!(t, "base_temperature")().to_string(),
                                        base_temp, set_base_temp)}
                                }
                                    .into_any(),
                                "weather_fetch" => view! {
                                    {num(
                                        crate::t!(t, "latitude_field")().to_string(),
                                        latitude, set_latitude)}
                                    {num(
                                        crate::t!(t, "longitude_field")().to_string(),
                                        longitude, set_longitude)}
                                }
                                    .into_any(),
                                _ => view! { <div></div> }.into_any(),
                            }}
                        </div>
                        <button type="submit" class="btn btn-primary mt-4">
                            {crate::t!(t, "calc_run")}
                        </button>
                    }
                }}

                {move || {
                    let err = error.get();
                    if let Some(e) = err {
                        return view! { <div class="alert alert-error mt-4">{e}</div> }.into_any();
                    }
                    ().into_any()
                }}

                {move || {
                    let value = result.get();
                    if let Some(v) = value {
                        return view! {
                            <div class="card bg-base-200 p-3 mt-4">
                                <h3 class="font-semibold mb-1">{crate::t!(t, "calc_result")}</h3>
                                <pre class="text-xs overflow-x-auto">{v}</pre>
                            </div>
                        }
                            .into_any();
                    }
                    ().into_any()
                }}
            </form>
        </div>
    }
}
