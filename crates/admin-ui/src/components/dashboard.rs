use crate::ViewMode;
use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::api;

#[component]
pub fn DashboardView() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = |key: &str| i18n.t(lang.get().as_str(), key);
    let dashboard_label = t("dashboard");
    let welcome_prefix = t("dashboard_welcome_prefix");
    let simple_mode_text = t("dashboard_simple_mode_text");
    let mode_full_label = t("mode_full");
    let mode_simple_label = t("mode_simple");
    let tasks_label = t("tasks");
    let tasks_desc = t("dashboard_tasks_desc");
    let sites_label = t("sites");
    let sites_desc = t("dashboard_sites_desc");
    let users_label = t("users");
    let users_desc = t("dashboard_users_desc");
    let equipment_label = t("nav_equipment");
    let equipment_desc = t("dashboard_equipment_desc");
    let active_tasks_label = t("active_tasks");
    let no_active_tasks_label = t("no_active_tasks");
    let view_all_label = t("view_all");
    let weather_forecast_label = t("weather_forecast");
    let no_weather_data_label = t("no_weather_data");
    let no_sensor_weather_label = t("no_sensor_weather");
    let weather_setup_address_label = t("dashboard_weather_setup_address");
    let no_weather_stations_label = t("no_weather_stations");
    let quick_actions_label = t("quick_actions");
    let new_order_label = t("new_order_btn");
    let issue_report_label = t("issue_report");
    let humidity_label = t("humidity");
    let wind_label = t("wind");
    let precipitation_label = t("precipitation");
    let view_mode = use_context::<ReadSignal<ViewMode>>().expect("view mode signal");
    let set_view_mode = use_context::<WriteSignal<ViewMode>>().expect("set view mode signal");
    let company_profile = api::load_company_profile().unwrap_or_default();
    let company_name = company_profile
        .company_name
        .unwrap_or_else(|| String::from("AgroCore"));

    let tasks_resource = LocalResource::new(|| async move {
        api::fetch_tasks()
            .await
            .unwrap_or_else(|_| api::PaginatedTasks {
                data: vec![],
                total: 0,
            })
    });
    let sites_resource = LocalResource::new(|| async move {
        api::fetch_sites()
            .await
            .ok()
            .map(|page| page.total)
            .unwrap_or(0)
    });
    let users_resource = LocalResource::new(|| async move {
        api::fetch_users()
            .await
            .ok()
            .map(|page| page.total)
            .unwrap_or(0)
    });
    let equipment_resource = LocalResource::new(|| async move {
        api::fetch_equipment()
            .await
            .ok()
            .map(|page| page.total)
            .unwrap_or(0)
    });
    let weather_resource = LocalResource::new(|| async move {
        api::fetch_weather_for_company_profile()
            .await
            .ok()
            .flatten()
    });
    let weather_snapshot = weather_resource
        .read()
        .as_ref()
        .and_then(|snapshot| snapshot.clone());

    view! {
        <div class="flex flex-col gap-8">
            <div class="flex justify-between items-center bg-base-100 p-6 rounded-box shadow-sm border border-base-200">
                <div>
                    <h1 class="text-3xl font-bold">{dashboard_label.clone()}</h1>
                    <p class="text-base-content/60">{format!("{} {}", welcome_prefix, company_name)}</p>
                </div>

                <div class="flex items-center gap-4">
                    <div class="join border border-base-300">
                        <button
                            class=move || format!("join-item btn btn-sm {}", if view_mode.get() == ViewMode::Full { "btn-primary" } else { "" })
                            on:click=move |_| set_view_mode.set(ViewMode::Full)
                        >
                            {mode_full_label.clone()}
                        </button>
                        <button
                            class=move || format!("join-item btn btn-sm {}", if view_mode.get() == ViewMode::Simple { "btn-primary" } else { "" })
                            on:click=move |_| set_view_mode.set(ViewMode::Simple)
                        >
                            {mode_simple_label.clone()}
                        </button>
                    </div>

                    <div class="badge badge-outline gap-2 p-4 hidden md:flex">
                        <Icon icon=LuActivity width="16" height="16" attr:class="text-success" />
                        {format!("{} {}", tasks_resource.read().as_ref().map(|t| t.total).unwrap_or(0), tasks_label)}
                    </div>
                </div>
            </div>

            {
                if view_mode.get() == ViewMode::Simple {
                view! {
                    <div class="card bg-primary text-primary-content shadow-xl">
                        <div class="card-body items-center text-center py-10">
                            <h2 class="card-title text-3xl mb-4">{t("wizard_tasks")}</h2>
                            <p class="mb-6 max-w-md">{simple_mode_text.clone()}</p>
                            <a href="/wizard" class="btn btn-lg btn-secondary gap-2">
                                <Icon icon=ImMagicWand width="24" height="24" />
                                {t("start_wizard")}
                            </a>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                        <div class="stat bg-base-100 shadow rounded-box">
                            <div class="stat-figure text-primary">
                                <Icon icon=LuClock width="32" height="32" />
                            </div>
                            <div class="stat-title">{tasks_label.clone()}</div>
                            <div class="stat-value text-primary">
                                {move || tasks_resource.read().as_ref().map(|t| t.total).unwrap_or(0)}
                            </div>
                            <div class="stat-desc">{tasks_desc.clone()}</div>
                        </div>

                        <div class="stat bg-base-100 shadow rounded-box">
                            <div class="stat-figure text-secondary">
                                <Icon icon=LuMap width="32" height="32" />
                            </div>
                            <div class="stat-title">{sites_label.clone()}</div>
                            <div class="stat-value text-secondary">
                                {move || sites_resource.read().as_ref().copied().unwrap_or(0)}
                            </div>
                            <div class="stat-desc">{sites_desc.clone()}</div>
                        </div>

                        <div class="stat bg-base-100 shadow rounded-box">
                            <div class="stat-figure text-warning">
                                <Icon icon=LuUsers width="32" height="32" />
                            </div>
                            <div class="stat-title">{users_label.clone()}</div>
                            <div class="stat-value text-warning">
                                {move || users_resource.read().as_ref().copied().unwrap_or(0)}
                            </div>
                            <div class="stat-desc">{users_desc.clone()}</div>
                        </div>

                        <div class="stat bg-base-100 shadow rounded-box">
                            <div class="stat-figure text-info">
                                <Icon icon=LuTractor width="32" height="32" />
                            </div>
                            <div class="stat-title">{equipment_label.clone()}</div>
                            <div class="stat-value text-info">
                                {move || equipment_resource.read().as_ref().copied().unwrap_or(0)}
                            </div>
                            <div class="stat-desc">{equipment_desc.clone()}</div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                        <div class="lg:col-span-2 flex flex-col gap-6">
                            <div class="card bg-base-100 shadow">
                                <div class="card-body">
                                    <h2 class="card-title mb-4">{active_tasks_label.clone()}</h2>
                                    <div class="space-y-4">
                                        <p class="text-sm opacity-50 italic">{no_active_tasks_label.clone()}</p>
                                    </div>
                                    <div class="card-actions justify-end mt-4">
                                        <a href="/tasks" class="btn btn-ghost btn-sm">{view_all_label.clone()}</a>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="flex flex-col gap-6">
                            <div class="card bg-primary text-primary-content shadow">
                                <div class="card-body">
                                    <h2 class="card-title">{weather_forecast_label.clone()}</h2>
                            {match weather_snapshot {
                                Some(snapshot) => view! {
                                <div class="flex justify-between items-center">
                                    <div class="text-4xl font-bold">
                                        {snapshot.temperature_c.map(|value| format!("{:.1}°C", value)).unwrap_or_else(|| String::from("—"))}
                                    </div>
                                            <div class="text-right">
                                                <div class="font-bold">{snapshot.location_label}</div>
                                                <div class="text-xs opacity-80">
                                                    {snapshot.humidity_percent.map(|value| format!("{}: {:.0}%", humidity_label.as_str(), value)).unwrap_or_else(|| no_weather_data_label.clone())}
                                                </div>
                                            </div>
                                        </div>
                                        <div class="divider divider-neutral"></div>
                                        <p class="text-sm">
                                            {snapshot.wind_kmh.map(|value| format!("{} {:.0} km/h", wind_label.as_str(), value)).unwrap_or_else(|| no_weather_data_label.clone())}
                                            {snapshot.precipitation_mm.map(|value| format!(" · {} {:.1} mm", precipitation_label.as_str(), value)).unwrap_or_default()}
                                        </p>
                                    }.into_any(),
                                None => view! {
                                        <div>
                                            <div class="flex justify-between items-center">
                                                <div class="text-lg font-bold">{no_weather_data_label.clone()}</div>
                                                <div class="text-right">
                                                    <div class="font-bold">{no_sensor_weather_label.clone()}</div>
                                                    <div class="text-xs opacity-80">{weather_setup_address_label.clone()}</div>
                                                </div>
                                            </div>
                                            <div class="divider divider-neutral"></div>
                                            <p class="text-sm">{no_weather_stations_label.clone()}</p>
                                        </div>
                                    }.into_any(),
                            }}
                                </div>
                            </div>

                            <div class="card bg-base-100 shadow">
                                <div class="card-body">
                                    <h2 class="card-title">{quick_actions_label.clone()}</h2>
                                    <div class="grid grid-cols-1 gap-2 mt-2">
                                        <a href="/tasks" class="btn btn-outline btn-sm justify-start">
                                            <Icon icon=LuPlus width="16" height="16" /> {new_order_label.clone()}
                                        </a>
                                        <a href="/resources" class="btn btn-outline btn-sm justify-start">
                                            <Icon icon=LuActivity width="16" height="16" /> {issue_report_label.clone()}
                                        </a>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
