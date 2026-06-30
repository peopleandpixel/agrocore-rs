use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;
use crate::i18n::{I18n, Language};
use crate::ViewMode;

use crate::api;

#[component]
pub fn DashboardView() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let view_mode = use_context::<ReadSignal<ViewMode>>().expect("view mode signal");
    let set_view_mode = use_context::<WriteSignal<ViewMode>>().expect("set view mode signal");

    let tasks_resource = LocalResource::new(|| async move {
        api::fetch_tasks().await.unwrap_or_else(|_| api::PaginatedTasks { data: vec![], total: 0 })
    });

    view! {
        <div class="flex flex-col gap-8">
            <div class="flex justify-between items-center bg-base-100 p-6 rounded-box shadow-sm border border-base-200">
                <div>
                    <h1 class="text-3xl font-bold">{
                        let i18n = i18n.clone();
                        move || i18n.t(lang.get().as_str(), "dashboard")
                    }</h1>
                    <p class="text-base-content/60">"Willkommen bei AgroCore! Hier ist die Übersicht für heute."</p>
                </div>
                
                <div class="flex items-center gap-4">
                    <div class="join border border-base-300">
                        <button 
                            class=move || format!("join-item btn btn-sm {}", if view_mode.get() == ViewMode::Full { "btn-primary" } else { "" })
                            on:click=move |_| set_view_mode.set(ViewMode::Full)
                        >
                            {
                                let i18n = i18n.clone();
                                move || i18n.t(lang.get().as_str(), "mode_full")
                            }
                        </button>
                        <button 
                            class=move || format!("join-item btn btn-sm {}", if view_mode.get() == ViewMode::Simple { "btn-primary" } else { "" })
                            on:click=move |_| set_view_mode.set(ViewMode::Simple)
                        >
                            {
                                let i18n = i18n.clone();
                                move || i18n.t(lang.get().as_str(), "mode_simple")
                            }
                        </button>
                    </div>

                    <div class="badge badge-outline gap-2 p-4 hidden md:flex">
                        <Icon icon=LuActivity width="16" height="16" attr:class="text-success" />
                        "System Status: OK"
                    </div>
                </div>
            </div>

            {
                let i18n = i18n.clone();
                move || if view_mode.get() == ViewMode::Simple {
                let i18n = i18n.clone();
                view! {
                    <div class="card bg-primary text-primary-content shadow-xl">
                        <div class="card-body items-center text-center py-10">
                            <h2 class="card-title text-3xl mb-4">{
                                let i18n = i18n.clone();
                                move || i18n.t(lang.get().as_str(), "wizard_tasks")
                            }</h2>
                            <p class="mb-6 max-w-md">"Nutzen Sie unsere Assistenten, um komplexe Aufgaben einfach und schnell zu erledigen."</p>
                            <a href="/wizard" class="btn btn-lg btn-secondary gap-2">
                                <Icon icon=ImMagicWand width="24" height="24" />
                                {
                                    let i18n = i18n.clone();
                                    move || i18n.t(lang.get().as_str(), "start_wizard")
                                }
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
                            <div class="stat-title">"Anstehende Aufgaben"</div>
                            <div class="stat-value text-primary">
                                {move || tasks_resource.read().as_ref().map(|t| t.total).unwrap_or(0)}
                            </div>
                            <div class="stat-desc">"Real-time data"</div>
                        </div>

                        <div class="stat bg-base-100 shadow rounded-box">
                            <div class="stat-figure text-secondary">
                                <Icon icon=LuCircleCheck width="32" height="32" />
                            </div>
                            <div class="stat-title">"Abgeschlossen"</div>
                            <div class="stat-value text-secondary">"85%"</div>
                            <div class="stat-desc">"Diese Woche"</div>
                        </div>

                        <div class="stat bg-base-100 shadow rounded-box">
                            <div class="stat-figure text-warning">
                                <Icon icon=LuTriangleAlert width="32" height="32" />
                            </div>
                            <div class="stat-title">"Warnungen"</div>
                            <div class="stat-value text-warning">"3"</div>
                            <div class="stat-desc">"Wartung erforderlich"</div>
                        </div>

                        <div class="stat bg-base-100 shadow rounded-box">
                            <div class="stat-figure text-info">
                                <Icon icon=LuTrendingUp width="32" height="32" />
                            </div>
                            <div class="stat-title">"Effizienz"</div>
                            <div class="stat-value text-info">"+12%"</div>
                            <div class="stat-desc">"vs. Vormonat"</div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                        <div class="lg:col-span-2 flex flex-col gap-6">
                            <div class="card bg-base-100 shadow">
                                <div class="card-body">
                                    <h2 class="card-title mb-4">"Aktive Aufträge"</h2>
                                    <div class="space-y-4">
                                        <p class="text-sm opacity-50 italic">"Keine aktiven Aufträge."</p>
                                    </div>
                                    <div class="card-actions justify-end mt-4">
                                        <a href="/tasks" class="btn btn-ghost btn-sm">"Alle ansehen"</a>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="flex flex-col gap-6">
                            <div class="card bg-primary text-primary-content shadow">
                                <div class="card-body">
                                    <h2 class="card-title">"Wettervorhersage"</h2>
                                    <div class="flex justify-between items-center">
                                        <div class="text-4xl font-bold">"24°C"</div>
                                        <div class="text-right">
                                            <div class="font-bold">"Sonnig"</div>
                                            <div class="text-xs opacity-80">"Niederschlag: 5%"</div>
                                        </div>
                                    </div>
                                    <div class="divider divider-neutral"></div>
                                    <p class="text-sm">"Ideale Bedingungen für Pflanzenschutz heute Nachmittag."</p>
                                </div>
                            </div>

                            <div class="card bg-base-100 shadow">
                                <div class="card-body">
                                    <h2 class="card-title">"Quick Actions"</h2>
                                    <div class="grid grid-cols-1 gap-2 mt-2">
                                        <button class="btn btn-outline btn-sm justify-start">
                                            <Icon icon=LuPlus width="16" height="16" /> "Neuer Auftrag"
                                        </button>
                                        <button class="btn btn-outline btn-sm justify-start">
                                            <Icon icon=LuActivity width="16" height="16" /> "Störfall melden"
                                        </button>
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
