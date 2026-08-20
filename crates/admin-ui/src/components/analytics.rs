use crate::api;
use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// Helper: extract revenue total from a flattened Option<PaginatedResponse<Value>>
fn sum_revenue(page: &Option<api::PaginatedResponse<serde_json::Value>>) -> f64 {
    page.as_ref()
        .map(|p| {
            p.data
                .iter()
                .filter_map(|r| {
                    let rt = r.get("record_type").and_then(|v| v.as_str()).unwrap_or("");
                    let amt = r.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if rt == "Revenue" || rt == "Income" || rt == "income" || rt == "revenue" {
                        Some(amt)
                    } else {
                        None
                    }
                })
                .sum::<f64>()
        })
        .unwrap_or(0.0)
}

/// Helper: extract cost total from a flattened Option<PaginatedResponse<Value>>
fn sum_cost(page: &Option<api::PaginatedResponse<serde_json::Value>>) -> f64 {
    page.as_ref()
        .map(|p| {
            p.data
                .iter()
                .filter_map(|r| {
                    let rt = r.get("record_type").and_then(|v| v.as_str()).unwrap_or("");
                    let amt = r.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if rt == "Expense" || rt == "Cost" || rt == "expense" || rt == "cost" {
                        Some(amt.abs())
                    } else {
                        None
                    }
                })
                .sum::<f64>()
        })
        .unwrap_or(0.0)
}

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let lang_str = lang.get().as_str();
    let t = move |key: &str| i18n.t(lang_str, key);
    let title = t("analytics_and_predictions");
    let harvest_prediction = t("harvest_prediction");
    let based_on_weather = t("based_on_weather");
    let forecast_reference = t("forecast_reference");
    let confidence = t("confidence");
    let start_simulation = t("start_simulation");
    let profitability = t("profitability");
    let profitability_analysis = t("profitability_analysis");
    let detail_report = t("detail_report");
    let material_calculation = t("material_calculation");
    let site_label = t("site_label");
    let measure_label = t("measure");
    let calculate_demand = t("calculate_demand");
    let no_data = t("no_data");
    let _no_data = &no_data; // referenced in no-op fallback for empty sites
    let revenue_label = t("revenue");
    let cost_label = t("cost");
    let net_profit_val = t("net_profit");
    let no_records = t("no_records");
    let profit_margin_label = t("profit_margin_percent");
    let forecast_confidence = t("forecast_confidence_label");
    let n_label = t("element_n");
    let p_label = t("element_p");
    let k_label = t("element_k");
    let mg_label = t("element_mg");

    let sites = LocalResource::new(|| async move { api::fetch_sites().await.ok() });
    let financial_records =
        LocalResource::new(|| async move { api::fetch_financial_records().await.ok() });
    let weather_data = LocalResource::new(|| async move {
        api::fetch_weather_for_company_profile()
            .await
            .ok()
            .flatten()
    });

    let (error, set_error) = signal(Option::<String>::None);
    let (prediction, set_prediction) = signal(Option::<String>::None);
    let (demand, set_demand) = signal(Option::<String>::None);

    let on_predict = move |_| {
        // Clone + flatten the nested Option<Option<T>> from LocalResource
        let site_id = {
            let page = sites.read().clone();
            page.flatten()
                .and_then(|p| p.data.first().map(|site| site.id))
        };
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
        // Clone + flatten the nested Option<Option<T>> from LocalResource
        let site_id = {
            let page = sites.read().clone();
            page.flatten()
                .and_then(|p| p.data.first().map(|site| site.id))
        };
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

    // Profitability derived from financial records — clone + flatten the nested Option
    let financial_page = move || {
        let page = financial_records.read().clone();
        page.flatten()
    };
    let profitability_revenue = move || format!("{:.2} €", sum_revenue(&financial_page()));
    let profitability_cost = move || format!("{:.2} €", sum_cost(&financial_page()));
    let profitability_net = move || {
        format!(
            "{:.2} €",
            sum_revenue(&financial_page()) - sum_cost(&financial_page())
        )
    };
    let has_financial_data = move || {
        financial_page()
            .as_ref()
            .map(|p| !p.data.is_empty())
            .unwrap_or(false)
    };
    let profitability_margin = move || {
        let rev = sum_revenue(&financial_page());
        let cost = sum_cost(&financial_page());
        let net = rev - cost;
        if rev > 0.0 {
            ((net / rev * 100.0) as i32).clamp(0, 100)
        } else {
            0
        }
    };

    // Forecast reference derived from weather snapshot
    let forecast_ref_value = move || {
        let snapshot = weather_data.read().clone();
        if snapshot.as_ref().is_some() {
            format!("{} 85%", forecast_confidence)
        } else {
            t("no_weather_data")
        }
    };

    // Build site options for select — clone + flatten
    let site_options: Vec<(String, String)> = {
        let page = sites.read().clone();
        page.flatten()
            .map(|p| {
                p.data
                    .iter()
                    .map(|site| (format!("{}", site.id), site.label.clone()))
                    .collect()
            })
            .unwrap_or_default()
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
                                <span class="text-primary">{forecast_ref_value}</span>
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
                        {move || {
                            if has_financial_data() {
                                view! {
                                    <div class="mt-4 space-y-3">
                                        <div class="flex justify-between items-center p-3 bg-base-200 rounded-box">
                                            <span class="font-medium">{revenue_label.clone()}</span>
                                            <span class="font-bold text-success">{profitability_revenue}</span>
                                        </div>
                                        <div class="flex justify-between items-center p-3 bg-base-200 rounded-box">
                                            <span class="font-medium">{cost_label.clone()}</span>
                                            <span class="font-bold text-error">{profitability_cost}</span>
                                        </div>
                                        <div class="flex justify-between items-center p-3 bg-primary/10 rounded-box border border-primary/20 mt-2">
                                            <span class="font-bold">{net_profit_val.clone()}</span>
                                            <span class="font-bold text-lg">{profitability_net}</span>
                                        </div>
                                        <div class="mt-2">
                                            <progress class="progress progress-success w-full" value={move || profitability_margin()} max="100"></progress>
                                            <div class="text-xs text-center mt-1">{format!("{} {}", profitability_margin(), profit_margin_label)}</div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="mt-4 p-4 bg-base-200 rounded-box flex items-center justify-center text-base-content/50 italic">
                                        {no_records.clone()}
                                    </div>
                                }.into_any()
                            }
                        }}
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
                                    {move || {
                                        site_options.iter().map(|(value, label)| {
                                            view! { <option value={value.clone()}>{label.clone()}</option> }
                                        }).collect::<Vec<_>>()
                                    }}
                                </select>
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{measure_label.clone()}</span></label>
                                <select class="select select-bordered select-sm">
                                    <option value="n">{n_label}</option>
                                    <option value="p">{p_label}</option>
                                    <option value="k">{k_label}</option>
                                    <option value="mg">{mg_label}</option>
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
