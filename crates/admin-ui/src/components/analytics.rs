use crate::api;
use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = |key: &str| i18n.t(lang.get().as_str(), key);
    let title = t("analytics_and_predictions");
    let harvest_prediction = t("harvest_prediction");
    let based_on_weather = t("based_on_weather");
    let forecast_reference = t("forecast_reference");
    let confidence = t("confidence");
    let start_simulation = t("start_simulation");
    let profitability = t("profitability");
    let profitability_analysis = t("profitability_analysis");
    let profitability_chart_placeholder = t("profitability_chart_placeholder");
    let detail_report = t("detail_report");
    let material_calculation = t("material_calculation");
    let site_label = t("site_label");
    let measure_label = t("measure");
    let calculate_demand = t("calculate_demand");
    let sites = LocalResource::new(|| async move { api::fetch_sites().await.ok() });
    let (error, set_error) = signal(None::<String>);
    let (prediction, set_prediction) = signal(None::<String>);
    let (demand, set_demand) = signal(None::<String>);

    let on_predict = move |_| {
        let site_id = sites
            .read()
            .clone()
            .flatten()
            .and_then(|page| page.data.first().map(|site| site.id));
        set_error.set(None);

        spawn_local(async move {
            let Some(site_id) = site_id else {
                set_error.set(Some(String::from("analytics_no_site_for_prediction")));
                return;
            };

            match api::predict_harvest(site_id).await {
                Ok(value) => set_prediction.set(Some(value.to_string())),
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_export = move |_| {
        spawn_local(async move {
            if let Ok(bytes) = api::export_orders_excel().await {
                let _ = api::download_bytes(
                    "orders.xlsx",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    &bytes,
                );
            }
        });
    };

    let on_calculate = move |_| {
        let site_id = sites
            .read()
            .clone()
            .flatten()
            .and_then(|page| page.data.first().map(|site| site.id));
        set_error.set(None);

        spawn_local(async move {
            let Some(site_id) = site_id else {
                set_error.set(Some(String::from("analytics_no_site_for_calculation")));
                return;
            };

            let payload = serde_json::json!({
                "site_id": site_id,
                "target_yield_t_ha": 8.0,
                "demand_per_t": {
                    "n": 2.5,
                    "p": 0.8,
                    "k": 3.0,
                    "mg": 0.5
                }
            });

            match api::calculate_nutrition_demand(payload).await {
                Ok(value) => set_demand.set(Some(value.to_string())),
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <h1 class="text-3xl font-bold">{title}</h1>

            {move || error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span>{err}</span>
                </div>
            })}

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuTrendingUp attr:class="text-primary" /> {harvest_prediction.clone()}</h2>
                        <p class="text-sm">{based_on_weather.clone()}</p>
                        <div class="mt-4 p-4 bg-base-200 rounded-box">
                            <div class="flex justify-between items-center mb-2">
                                <span class="font-bold">{forecast_reference.clone()}</span>
                                <span class="text-primary">"—"</span>
                            </div>
                            <progress class="progress progress-primary w-full" value="65" max="100"></progress>
                            <div class="text-xs mt-1 text-right">{format!("{} 85%", confidence)}</div>
                        </div>
                        {move || prediction.get().map(|value| view! {
                            <div class="alert alert-info mt-4">
                                <span>{value}</span>
                            </div>
                        })}
                        <div class="card-actions justify-end mt-4">
                            <button class="btn btn-sm btn-primary" on:click=on_predict>{start_simulation}</button>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuDollarSign attr:class="text-success" /> {profitability.clone()}</h2>
                        <p class="text-sm">{profitability_analysis.clone()}</p>
                        <div class="mt-4 h-32 bg-base-200 rounded-box flex items-center justify-center italic text-base-content/50">
                            {profitability_chart_placeholder.clone()}
                        </div>
                        <div class="card-actions justify-end mt-4">
                            <button class="btn btn-sm btn-outline" on:click=on_export>{detail_report}</button>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-100 shadow lg:col-span-2">
                    <div class="card-body">
                        <h2 class="card-title">{material_calculation}</h2>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-2">
                            <div class="form-control">
                                <label class="label"><span class="label-text">{site_label.clone()}</span></label>
                                <select class="select select-bordered select-sm">
                                    <option>"—"</option>
                                    <option>"—"</option>
                                </select>
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{measure_label.clone()}</span></label>
                                <select class="select select-bordered select-sm">
                                    <option>"—"</option>
                                    <option>"—"</option>
                                </select>
                            </div>
                            <div class="form-control flex items-end pb-1">
                                <button class="btn btn-sm btn-secondary w-full" on:click=on_calculate>{calculate_demand}</button>
                            </div>
                        </div>
                        {move || demand.get().map(|value| view! {
                            <div class="alert alert-success mt-4">
                                <span>{value}</span>
                            </div>
                        })}
                    </div>
                </div>
            </div>
        </div>
    }
}
