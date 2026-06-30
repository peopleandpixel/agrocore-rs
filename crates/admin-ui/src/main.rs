mod components;
mod i18n;
mod api;

use crate::components::dashboard::DashboardView;
use crate::components::map::MapView;
use crate::components::wizard::WizardView;
use crate::components::livestock::LivestockManagement;
use crate::components::weather::WeatherManagement;
use crate::components::finance::FinanceManagement;
use crate::components::equipment::EquipmentManagement;
use crate::components::analytics::AnalyticsPage;
use crate::components::resources::ResourcesPage;
use crate::components::compliance::CompliancePage;
use crate::components::settings::SettingsPage;
use crate::components::users::UserManagement;
use crate::components::sites::SiteManagement;
use crate::components::orders::OrderList;
use leptos::prelude::*;
use leptos_router::{components::*, path};
use leptos_icons::Icon;
use icondata::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum UserRole {
    Admin,
    Manager,
    Worker,
    Viewer,
    Custom(uuid::Uuid),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewMode {
    Full,
    Simple,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (user_role, set_user_role) = signal(UserRole::Admin);
    let (view_mode, set_view_mode) = signal(ViewMode::Full);
    let (theme, set_theme) = signal(Theme::Light);
    let (lang, set_lang) = signal(i18n::Language::DE);
    let i18n_engine = i18n::I18n::new();

    let system_status = LocalResource::new(|| async move {
        api::fetch_system_status().await.unwrap_or_default()
    });

    let initialized = move || system_status.read().as_ref().map(|s| s.initialized).unwrap_or(true);

    provide_context(user_role);
    provide_context(set_user_role);
    provide_context(view_mode);
    provide_context(set_view_mode);
    provide_context(theme);
    provide_context(set_theme);
    provide_context(lang);
    provide_context(set_lang);
    provide_context(i18n_engine.clone());

    // Effect to check system status
    Effect::new(move || {
        // In a real app, this would be an async fetch
        // For now we just keep it in a signal to demonstrate the logic
    });

    Effect::new(move |_| {
        use web_sys::window;
        if let Some(win) = window() {
            if let Some(doc) = win.document() {
                if let Some(root) = doc.document_element() {
                    let _ = root.set_attribute("data-theme", theme.get().as_str());
                }
            }
        }
    });

    view! {
        <Show 
            when=move || initialized() 
            fallback=|| view! { <crate::components::setup::SetupAssistant /> }
        >
            {
                let i18n_engine = i18n_engine.clone();
                move || {
                    let i18n_engine = i18n_engine.clone();
                    view! {
                        <div class="drawer lg:drawer-open">
                            <input id="my-drawer-2" type="checkbox" class="drawer-toggle" />
                            <div class="drawer-content flex flex-col items-center justify-start p-4">
                                // Navbar for mobile
                                <div class="navbar bg-base-100 lg:hidden w-full">
                                    <div class="flex-none">
                                        <label for="my-drawer-2" class="btn btn-square btn-ghost">
                                            <Icon icon=LuMenu width="24" height="24" />
                                        </label>
                                    </div>
                                    <div class="flex-1">
                                        <a class="btn btn-ghost text-xl">"AgroCore"</a>
                                    </div>
                                </div>

                                // Role Switcher (for demo purposes)
                                <div class="flex gap-2 mb-8 p-4 bg-base-200 rounded-box">
                                    <span class="self-center font-bold">"Rolle simulieren:"</span>
                                    <button class="btn btn-sm" on:click=move |_| set_user_role.set(UserRole::Admin)>"Admin"</button>
                                    <button class="btn btn-sm" on:click=move |_| set_user_role.set(UserRole::Manager)>"Manager"</button>
                                    <button class="btn btn-sm" on:click=move |_| set_user_role.set(UserRole::Worker)>"Worker"</button>
                                </div>

                                <div class="w-full max-w-5xl">
                                    <Router>
                                        <Routes fallback=|| view! { "404 Not Found" }>
                                            <Route path=path!("/") view=|| view! { <DashboardView /> } />
                                            <Route path=path!("/sites") view=|| view! { <SiteManagement /> } />
                                            <Route path=path!("/map") view=|| view! { <MapView /> } />
                                            <Route path=path!("/tasks") view=|| view! { <OrderList /> } />
                                            <Route path=path!("/livestock") view=|| view! { <LivestockManagement /> } />
                                            <Route path=path!("/weather") view=|| view! { <WeatherManagement /> } />
                                            <Route path=path!("/finance") view=|| view! { <FinanceManagement /> } />
                                            <Route path=path!("/equipment") view=|| view! { <EquipmentManagement /> } />
                                            <Route path=path!("/analytics") view=|| view! { <AnalyticsPage /> } />
                                            <Route path=path!("/resources") view=|| view! { <ResourcesPage /> } />
                                            <Route path=path!("/compliance") view=|| view! { <CompliancePage /> } />
                                            <Route path=path!("/users") view=|| view! { <UserManagement /> } />
                                            <Route path=path!("/settings") view=|| view! { <SettingsPage /> } />
                                            <Route path=path!("/wizard") view=|| view! { <WizardView /> } />
                                        </Routes>
                                    </Router>
                                </div>
                            </div>

                            <div class="drawer-side">
                                <label for="my-drawer-2" aria-label="close sidebar" class="drawer-overlay"></label>
                                <ul class="menu p-4 w-80 min-h-full bg-base-200 text-base-content flex flex-col">
                                    <li class="mb-4 text-2xl font-bold p-4">"AgroCore Admin"</li>
                                    
                                    <li>
                                        <a href="/">
                                            <Icon icon=LuLayoutDashboard width="20" height="20" />
                                            {
                                                let i18n_engine = i18n_engine.clone();
                                                move || i18n_engine.t(lang.get().as_str(), "dashboard")
                                            }
                                        </a>
                                    </li>

                                    {
                                        let i18n_engine = i18n_engine.clone();
                                        move || {
                                            if view_mode.get() == ViewMode::Simple {
                                                let i18n_engine = i18n_engine.clone();
                                                view! {
                                                    <li>
                                                        <a href="/wizard" class="bg-primary text-primary-content font-bold">
                                                            <Icon icon=ImMagicWand width="20" height="20" />
                                                            {
                                                                let i18n_engine = i18n_engine.clone();
                                                                move || i18n_engine.t(lang.get().as_str(), "start_wizard")
                                                            }
                                                        </a>
                                                    </li>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }
                                    }

                                    {
                                        let i18n_engine = i18n_engine.clone();
                                        move || {
                                            if view_mode.get() == ViewMode::Full {
                                                let i18n_engine = i18n_engine.clone();
                                                view! {
                                                    <>
                                    <li>
                                        <a href="/sites">
                                            <Icon icon=LuMap width="20" height="20" />
                                            {
                                                let i18n_engine = i18n_engine.clone();
                                                move || i18n_engine.t(lang.get().as_str(), "sites")
                                            }
                                        </a>
                                    </li>

                                    <li>
                                        <a href="/tasks">
                                            <Icon icon=LuSquareCheck width="20" height="20" />
                                            {
                                                let i18n_engine = i18n_engine.clone();
                                                move || i18n_engine.t(lang.get().as_str(), "tasks")
                                            }
                                        </a>
                                    </li>

                                    <li>
                                        <a href="/map">
                                            <Icon icon=LuMapPin width="20" height="20" />
                                            "Karte"
                                        </a>
                                    </li>

                                                        <li>
                                                            <a href="/livestock">
                                                                <Icon icon=LuBeef width="20" height="20" />
                                                                {
                                                                    let i18n_engine = i18n_engine.clone();
                                                                    move || i18n_engine.t(lang.get().as_str(), "livestock_management")
                                                                }
                                                            </a>
                                                        </li>

                                                        <li>
                                                            <a href="/weather">
                                                                <Icon icon=LuCloudSun width="20" height="20" />
                                                                {
                                                                    let i18n_engine = i18n_engine.clone();
                                                                    move || i18n_engine.t(lang.get().as_str(), "weather_and_phenology")
                                                                }
                                                            </a>
                                                        </li>

                                                        {
                                                            let i18n_engine = i18n_engine.clone();
                                                            move || {
                                                                if user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager {
                                                                    let i18n_engine = i18n_engine.clone();
                                                                    view! {
                                                                        <>
                                                                            <li>
                                                                                <a href="/resources">
                                                                                    <Icon icon=LuBriefcase width="20" height="20" />
                                                                                    "Ressourcen"
                                                                                </a>
                                                                            </li>
                                                                            <li>
                                                                                <a href="/equipment">
                                                                                    <Icon icon=LuTractor width="20" height="20" />
                                                                                    {
                                                                                        let i18n_engine = i18n_engine.clone();
                                                                                        move || i18n_engine.t(lang.get().as_str(), "equipment_management")
                                                                                    }
                                                                                </a>
                                                                            </li>
                                                                            <li>
                                                                                <a href="/finance">
                                                                                    <Icon icon=LuWallet width="20" height="20" />
                                                                                    {
                                                                                        let i18n_engine = i18n_engine.clone();
                                                                                        move || i18n_engine.t(lang.get().as_str(), "finance_and_pac")
                                                                                    }
                                                                                </a>
                                                                            </li>
                                                                            <li>
                                                                                <a href="/analytics">
                                                                                    <Icon icon=LuChartBar width="20" height="20" />
                                                                                    {
                                                                                        let i18n_engine = i18n_engine.clone();
                                                                                        move || i18n_engine.t(lang.get().as_str(), "analytics_and_predictions")
                                                                                    }
                                                                                </a>
                                                                            </li>
                                                                            <li>
                                                                                <a href="/compliance">
                                                                                    <Icon icon=LuChartNoAxesColumn width="20" height="20" />
                                                                                    "Compliance"
                                                                                </a>
                                                                            </li>
                                                                            <li>
                                                                                <a href="/users">
                                                                                    <Icon icon=LuUsers width="20" height="20" />
                                                                                    {
                                                                                        let i18n_engine = i18n_engine.clone();
                                                                                        move || i18n_engine.t(lang.get().as_str(), "users")
                                                                                    }
                                                                                </a>
                                                                            </li>
                                                                        </>
                                                                    }.into_any()
                                                                } else {
                                                                    ().into_any()
                                                                }
                                                            }
                                                        }
                                                    </>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }
                                    }

                                    <li>
                                        <a href="/settings">
                                            <Icon icon=LuSettings width="20" height="20" />
                                            {
                                                let i18n_engine = i18n_engine.clone();
                                                move || i18n_engine.t(lang.get().as_str(), "settings")
                                            }
                                        </a>
                                    </li>

                                    <li>
                                        <a href="http://localhost:3001" target="_blank">
                                            <Icon icon=LuLayoutDashboard width="20" height="20" />
                                            "Grafana Dashboards"
                                        </a>
                                    </li>

                                    <div class="mt-auto">
                                        <div class="divider"></div>
                                        <li>
                                            <a class="flex justify-between items-center" on:click=move |_| {
                                                set_theme.set(if theme.get() == Theme::Light { Theme::Dark } else { Theme::Light });
                                            }>
                                                <div class="flex gap-2 text-base-content/70">
                                                    <Icon icon=LuSun width="20" height="20" />
                                                    {
                                                        let i18n_engine = i18n_engine.clone();
                                                        move || i18n_engine.t(lang.get().as_str(), "theme_toggle")
                                                    }
                                                </div>
                                                <input type="checkbox" class="toggle toggle-sm" checked=move || theme.get() == Theme::Dark />
                                            </a>
                                        </li>
                                        <li>
                                            <a class="text-error">
                                                <Icon icon=LuLogOut width="20" height="20" />
                                                "Abmelden"
                                            </a>
                                        </li>
                                    </div>
                                </ul>
                            </div>
                        </div>
                    }
                }
            }
        </Show>
    }
}

fn main() {
    leptos::mount::mount_to_body(App);
}
