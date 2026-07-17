use crate::api;
use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn ResourcesPage() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let title = move || i18n.t(lang.get().as_str(), "resource_management");
    let overview = move || i18n.t(lang.get().as_str(), "resource_overview");
    let history = move || i18n.t(lang.get().as_str(), "history");
    let add_inventory = move || i18n.t(lang.get().as_str(), "add_inventory");
    let water_irrigation = move || i18n.t(lang.get().as_str(), "water_irrigation");
    let no_water_sensor = move || i18n.t(lang.get().as_str(), "no_water_sensor");
    let workers = move || i18n.t(lang.get().as_str(), "workers");
    let in_use = move || i18n.t(lang.get().as_str(), "in_use");
    let workers_in_system = move || i18n.t(lang.get().as_str(), "workers_in_system");
    let hours_today = move || i18n.t(lang.get().as_str(), "hours_today");
    let time_tracking_later = move || i18n.t(lang.get().as_str(), "time_tracking_later");
    let resource_assignments_desc = move || i18n.t(lang.get().as_str(), "resource_assignments_desc");
    let inventory_stock = move || i18n.t(lang.get().as_str(), "inventory_stock");
    let inventory_available = move || i18n.t(lang.get().as_str(), "inventory_available");
    let users = LocalResource::new(|| async move { api::fetch_users().await.ok() });
    let equipment = LocalResource::new(|| async move { api::fetch_equipment().await.ok() });

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

    let on_add_inventory = move |_| {
        let _ = leptos::prelude::window().location().set_href("/equipment");
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">{move || title()}</h1>
                    <p class="text-base-content/60">{move || overview()}</p>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-outline" on:click=on_export>
                        <Icon icon=LuHistory width="20" height="20" />
                        {move || history()}
                    </button>
                    <button class="btn btn-primary" on:click=on_add_inventory>
                        <Icon icon=LuPlus width="20" height="20" />
                        {move || add_inventory()}
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Wasser & Energie
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuDroplets attr:class="text-info" width="24" height="24" /> {move || water_irrigation()}</h2>
                        <div class="alert alert-info mt-2">
                            <span>{move || no_water_sensor()}</span>
                        </div>
                    </div>
                </div>

                // Personal
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuUsers attr:class="text-success" width="24" height="24" /> {move || workers()}</h2>
                        <div class="stats bg-base-200 w-full mt-2">
                            <div class="stat">
                                <div class="stat-title">{move || in_use()}</div>
                                <div class="stat-value text-success">
                                    {move || users.read().as_ref().map(|page| page.as_ref().map(|p| p.total).unwrap_or(0)).unwrap_or(0)}
                                </div>
                                <div class="stat-desc">{move || workers_in_system()}</div>
                            </div>
                            <div class="stat">
                                <div class="stat-title">{move || hours_today()}</div>
                                <div class="stat-value text-sm">"—"</div>
                                <div class="stat-desc">{move || time_tracking_later()}</div>
                            </div>
                        </div>
                        <div class="alert alert-ghost mt-4">
                            <span>{move || resource_assignments_desc()}</span>
                        </div>
                    </div>
                </div>

                // Betriebsmittel (Lager)
                <div class="card bg-base-100 shadow lg:col-span-2">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuPackage attr:class="text-warning" width="24" height="24" /> {move || inventory_stock()}</h2>
                        <div class="alert alert-info mt-4">
                            <span>{move || format!("{} {}", equipment.read().as_ref().map(|page| page.as_ref().map(|p| p.total).unwrap_or(0)).unwrap_or(0), inventory_available())}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
