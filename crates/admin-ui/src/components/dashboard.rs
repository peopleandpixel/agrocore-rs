use crate::ViewMode;
use crate::api;
use crate::components::error_boundary::user_friendly_error;
use icondata::*;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn DashboardView() -> impl IntoView {
    let t = crate::i18n::use_i18n();
    let view_mode = use_context::<ReadSignal<ViewMode>>().expect("view mode signal");
    let user_role = use_context::<ReadSignal<crate::UserRole>>().expect("user role signal");
    let _set_view_mode = use_context::<WriteSignal<ViewMode>>().expect("set view mode signal");
    let company_profile = api::load_company_profile().unwrap_or_default();
    let company_name = company_profile
        .company_name
        .unwrap_or_else(|| String::from("AgroCore"));

    let tasks_resource = LocalResource::new(move || async move {
        api::fetch_tasks().await.unwrap_or_else(|err| {
            // Log user-friendly error for debugging
            web_sys::console::log_1(
                &format!("Dashboard tasks error: {}", user_friendly_error(&err)).into(),
            );
            api::PaginatedTasks {
                data: vec![],
                total: 0,
            }
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

    view! {
        <div class="flex flex-col gap-8">
            <div class="flex justify-between items-center bg-base-100 p-6 rounded-box shadow-sm border border-base-200">
                <div>
                    <h1 class="text-4xl font-black tracking-tight">{crate::t!(t, "dashboard")}</h1>
                    <p class="text-base-content/50 text-sm mt-1">{
                        let company_name = company_name.clone();
                        move || format!("{} {}", t("dashboard_welcome_prefix"), company_name)
                    }</p>
                </div>

                <div class="flex items-center gap-4">
                    <div class="glass bg-success/10 text-success border-success/20 badge gap-2 p-4 hidden md:flex font-medium">
                        <Icon icon=LuActivity width="16" height="16" />
                        {move || {
                            let total = tasks_resource.read().as_ref().map(|t| t.total).unwrap_or(0);
                            format!("{} {}", total, t("tasks"))
                        }}
                    </div>
                </div>
            </div>

            {
                move || if view_mode.get() == ViewMode::Simple {
                view! {
                    <div class="card bg-gradient-to-br from-primary to-primary-focus text-primary-content shadow-2xl overflow-hidden relative border-none">
                        <div class="absolute top-[-20%] right-[-10%] opacity-10 rotate-12">
                            <Icon icon=LuActivity width="300" height="300" />
                        </div>
                        <div class="card-body items-center text-center py-16 relative z-10">
                            <h2 class="card-title text-4xl font-black mb-6 leading-tight">{crate::t!(t, "wizard_tasks")}</h2>
                            <p class="mb-10 max-w-lg text-lg opacity-90">{crate::t!(t, "dashboard_simple_mode_text")}</p>
                            <a href="/wizard" class="btn btn-lg bg-base-100 text-primary border-none hover:scale-105 transition-transform duration-300 gap-3 px-10">
                                <Icon icon=ImMagicWand width="24" height="24" />
                                {crate::t!(t, "start_wizard")}
                            </a>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                        <div class="stat bg-base-100 shadow-sm border border-base-200 rounded-2xl hover:shadow-md transition-shadow">
                            <div class="stat-figure text-primary/40">
                                <Icon icon=LuClock width="32" height="32" />
                            </div>
                            <div class="stat-title font-medium text-base-content/60">{crate::t!(t, "tasks")}</div>
                            <div class="stat-value text-primary font-black tracking-tight">
                                {move || tasks_resource.read().as_ref().map(|t| t.total).unwrap_or(0)}
                            </div>
                            <div class="stat-desc mt-1 font-medium">{crate::t!(t, "dashboard_tasks_desc")}</div>
                        </div>

                        <div class="stat bg-base-100 shadow-sm border border-base-200 rounded-2xl hover:shadow-md transition-shadow">
                            <div class="stat-figure text-secondary/40">
                                <Icon icon=LuMap width="32" height="32" />
                            </div>
                            <div class="stat-title font-medium text-base-content/60">{crate::t!(t, "sites")}</div>
                            <div class="stat-value text-secondary font-black tracking-tight">
                                {move || sites_resource.read().as_ref().copied().unwrap_or(0)}
                            </div>
                            <div class="stat-desc mt-1 font-medium">{crate::t!(t, "dashboard_sites_desc")}</div>
                        </div>

                        <div class="stat bg-base-100 shadow-sm border border-base-200 rounded-2xl hover:shadow-md transition-shadow">
                            <div class="stat-figure text-warning/40">
                                <Icon icon=LuUsers width="32" height="32" />
                            </div>
                            <div class="stat-title font-medium text-base-content/60">{crate::t!(t, "users")}</div>
                            <div class="stat-value text-warning font-black tracking-tight">
                                {move || users_resource.read().as_ref().copied().unwrap_or(0)}
                            </div>
                            <div class="stat-desc mt-1 font-medium">{crate::t!(t, "dashboard_users_desc")}</div>
                        </div>

                        <div class="stat bg-base-100 shadow-sm border border-base-200 rounded-2xl hover:shadow-md transition-shadow">
                            <div class="stat-figure text-info/40">
                                <Icon icon=LuTractor width="32" height="32" />
                            </div>
                            <div class="stat-title font-medium text-base-content/60">{crate::t!(t, "nav_equipment")}</div>
                            <div class="stat-value text-info font-black tracking-tight">
                                {move || equipment_resource.read().as_ref().copied().unwrap_or(0)}
                            </div>
                            <div class="stat-desc mt-1 font-medium">{crate::t!(t, "dashboard_equipment_desc")}</div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 mt-4">
                        <div class="lg:col-span-2 flex flex-col gap-6">
                            <div class="card bg-base-100 shadow-sm border border-base-200 rounded-2xl">
                                <div class="card-body">
                                    <div class="flex justify-between items-center mb-6">
                                        <h2 class="card-title text-xl font-bold">{crate::t!(t, "active_tasks")}</h2>
                                        <span class="badge badge-primary badge-outline">{move || tasks_resource.read().as_ref().map(|t| t.total).unwrap_or(0)}</span>
                                    </div>
                                    <div class="space-y-4">
                                        <div class="flex flex-col items-center justify-center py-12 opacity-30 gap-4">
                                            <Icon icon=LuClipboardList width="64" height="64" />
                                            <p class="text-sm font-medium italic">{crate::t!(t, "no_active_tasks")}</p>
                                        </div>
                                    </div>
                                    <div class="card-actions justify-end mt-4 border-t border-base-200 pt-4">
                                        <a href="/tasks" class="btn btn-ghost btn-sm gap-2">
                                            {crate::t!(t, "view_all")}
                                            <Icon icon=LuChevronRight width="16" height="16" />
                                        </a>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="flex flex-col gap-6">
                            <div class="card bg-gradient-to-br from-info/20 to-info/5 border border-info/10 text-info-content shadow-sm rounded-2xl overflow-hidden backdrop-blur-xl">
                                <div class="card-body">
                                    <h2 class="card-title text-xl font-bold text-info-content mb-4">{crate::t!(t, "weather_forecast")}</h2>
                            {move || match weather_resource.read().as_ref().and_then(|w| w.as_ref()) {
                                Some(snapshot) => {
                                    let humidity_l = crate::t!(t, "humidity");
                                    let wind_l = crate::t!(t, "wind");
                                    let precip_l = crate::t!(t, "precipitation");
                                    let no_data_l = crate::t!(t, "no_weather_data");
                                    let location = snapshot.location_label.clone();
                                    let temp = snapshot.temperature_c.map(|value| format!("{:.1}°C", value)).unwrap_or_else(|| String::from("—"));
                                    let humidity_val = snapshot.humidity_percent;
                                    let wind_val = snapshot.wind_kmh;
                                    let precip_val = snapshot.precipitation_mm;

                                    view! {
                                        <div class="flex justify-between items-end mb-4">
                                            <div class="text-6xl font-black tracking-tighter text-info">
                                                {temp}
                                            </div>
                                            <div class="text-right">
                                                <div class="font-bold text-lg">{location}</div>
                                                <div class="text-xs opacity-60 font-medium">
                                                    {move || humidity_val.map(|value| format!("{}: {:.0}%", humidity_l(), value)).unwrap_or_else(&no_data_l)}
                                                </div>
                                            </div>
                                        </div>
                                        <div class="divider opacity-10"></div>
                                        <p class="text-sm font-medium">
                                            {move || wind_val.map(|value| format!("{} {:.0} km/h", wind_l(), value)).unwrap_or_else(&no_data_l)}
                                            {move || precip_val.map(|value| format!(" · {} {:.1} mm", precip_l(), value)).unwrap_or_default()}
                                        </p>
                                    }.into_any()
                                },
                                None => view! {
                                        <div>
                                            <div class="flex justify-between items-center opacity-60">
                                                <div class="text-lg font-bold">{crate::t!(t, "no_weather_data")}</div>
                                                <Icon icon=LuCloudSun width="32" height="32" />
                                            </div>
                                            <div class="divider opacity-10"></div>
                                            <p class="text-sm font-medium opacity-50">{crate::t!(t, "no_weather_stations")}</p>
                                        </div>
                                    }.into_any(),
                            }}
                                </div>
                            </div>

                            <div class="card bg-base-100 shadow-sm border border-base-200 rounded-2xl">
                                <div class="card-body">
                                    <h2 class="card-title text-xl font-bold mb-4">{crate::t!(t, "quick_actions")}</h2>
                                    <div class="grid grid-cols-1 gap-3 mt-2">
                                        <a href="/tasks" class="btn btn-primary btn-sm justify-start gap-3 rounded-xl h-10 shadow-sm border-none">
                                            <Icon icon=LuPlus width="18" height="18" /> {crate::t!(t, "new_order_btn")}
                                        </a>
                                        <a href="/resources" class="btn btn-outline btn-sm justify-start gap-3 rounded-xl h-10">
                                            <Icon icon=LuActivity width="18" height="18" /> {crate::t!(t, "issue_report")}
                                        </a>
                                        <Show when=move || user_role.get() == crate::UserRole::Admin || user_role.get() == crate::UserRole::Manager>
                                            <a href="/users" class="btn btn-outline btn-info btn-sm justify-start gap-3 rounded-xl h-10">
                                                <Icon icon=LuUsers width="18" height="18" /> {crate::t!(t, "nav_users")}
                                            </a>
                                        </Show>
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
