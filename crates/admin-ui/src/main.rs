mod api;
mod components;
mod i18n;

use crate::components::analytics::AnalyticsPage;
use crate::components::compliance::CompliancePage;
use crate::components::dashboard::DashboardView;
use crate::components::equipment::EquipmentManagement;
use crate::components::finance::FinanceManagement;
use crate::components::livestock::LivestockManagement;
use crate::components::login::LoginView;
use crate::components::map::MapView;
use crate::components::orders::OrderList;
use crate::components::resources::ResourcesPage;
use crate::components::settings::SettingsPage;
use crate::components::sites::SiteManagement;
use crate::components::users::UserManagement;
use crate::components::weather::WeatherManagement;
use crate::components::wizard::WizardView;
use icondata::*;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::{components::*, path};

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
    let initial_role = api::user_role()
        .as_deref()
        .map(|role| match role {
            "Admin" => UserRole::Admin,
            "Manager" => UserRole::Manager,
            "Worker" => UserRole::Worker,
            "Viewer" => UserRole::Viewer,
            _ => UserRole::Viewer,
        })
        .unwrap_or(UserRole::Viewer);
    let (user_role, _set_user_role) = signal(initial_role);
    let (view_mode, set_view_mode) = signal(ViewMode::Full);
    let (theme, set_theme) = signal(Theme::Light);
    let initial_lang = web_sys::window()
        .and_then(|win| win.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("agrocore.lang").ok().flatten())
        .map(|value| i18n::Language::from_str(&value))
        .unwrap_or(i18n::Language::DE);
    let (lang, set_lang) = signal(initial_lang);
    let i18n_engine = i18n::I18n::new();

    let system_status = LocalResource::new(|| async move { api::fetch_system_status().await.ok() });

    let initialized = move || {
        system_status
            .read()
            .as_ref()
            .map(|s| s.as_ref().map(|status| status.initialized).unwrap_or(false))
            .unwrap_or(false)
    };
    let has_token = move || api::auth_token().is_some();

    provide_context(user_role);
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
        if let Some(win) = window()
            && let Some(doc) = win.document()
                && let Some(root) = doc.document_element() {
                    let _ = root.set_attribute("data-theme", theme.get().as_str());
                }
    });

    view! {
        <Show when=move || initialized() fallback=|| view! { <crate::components::setup::SetupAssistant /> }>
            <Show when=move || has_token() fallback=|| view! { <LoginView /> }>
                <AuthenticatedShell
                    user_role=user_role
                    view_mode=view_mode
                    set_view_mode=set_view_mode
                    theme=theme
                    set_theme=set_theme
                />
            </Show>
        </Show>
    }
}

#[component]
fn AuthenticatedShell(
    user_role: ReadSignal<UserRole>,
    view_mode: ReadSignal<ViewMode>,
    set_view_mode: WriteSignal<ViewMode>,
    theme: ReadSignal<Theme>,
    set_theme: WriteSignal<Theme>,
) -> impl IntoView {
    let i18n = use_context::<i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<i18n::Language>>().expect("lang signal");
    let simple_label: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "mode_simple").into_boxed_str());
    let full_label: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "mode_full").into_boxed_str());
    let not_found_label: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "not_found").into_boxed_str());
    let nav_dashboard: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "nav_dashboard")
            .into_boxed_str(),
    );
    let nav_wizard: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "nav_wizard").into_boxed_str());
    let nav_sites: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "nav_sites").into_boxed_str());
    let nav_tasks: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "nav_tasks").into_boxed_str());
    let nav_map: &'static str = Box::leak(i18n.t(lang.get().as_str(), "nav_map").into_boxed_str());
    let nav_livestock: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "nav_livestock")
            .into_boxed_str(),
    );
    let nav_weather: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "nav_weather").into_boxed_str());
    let nav_resources: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "nav_resources")
            .into_boxed_str(),
    );
    let nav_equipment: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "nav_equipment")
            .into_boxed_str(),
    );
    let nav_finance: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "nav_finance").into_boxed_str());
    let nav_analytics: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "nav_analytics")
            .into_boxed_str(),
    );
    let nav_compliance: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "nav_compliance")
            .into_boxed_str(),
    );
    let nav_users: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "nav_users").into_boxed_str());
    let nav_settings: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "nav_settings").into_boxed_str());
    let nav_grafana: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "nav_grafana").into_boxed_str());
    let theme_label: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "theme").into_boxed_str());
    let logout_label: &'static str =
        Box::leak(i18n.t(lang.get().as_str(), "logout").into_boxed_str());
    view! {
        <div class="drawer lg:drawer-open">
            <input id="my-drawer-2" type="checkbox" class="drawer-toggle" />
            <div class="drawer-content flex flex-col items-center justify-start p-4">
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

                <div class="flex gap-2 mb-8 p-4 bg-base-200 rounded-box">
                    <button class="btn btn-sm btn-outline" on:click=move |_| {
                        set_view_mode.set(if view_mode.get() == ViewMode::Full { ViewMode::Simple } else { ViewMode::Full });
                    }>
                        {move || if view_mode.get() == ViewMode::Full { simple_label.to_string() } else { full_label.to_string() }}
                    </button>
                </div>

                <div class="w-full max-w-5xl">
                    <Router>
                        <Routes fallback=|| view! {{
                            view! { <span>{not_found_label.to_string()}</span> }
                        }}>
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
                    <li><a href="/"><Icon icon=LuLayoutDashboard width="20" height="20" />{nav_dashboard}</a></li>
                    <li class=move || if view_mode.get() == ViewMode::Simple { "" } else { "hidden" }>
                        <a href="/wizard" class="bg-primary text-primary-content font-bold"><Icon icon=ImMagicWand width="20" height="20" />{nav_wizard}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/sites"><Icon icon=LuMap width="20" height="20" />{nav_sites}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/tasks"><Icon icon=LuSquareCheck width="20" height="20" />{nav_tasks}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/map"><Icon icon=LuMapPin width="20" height="20" />{nav_map}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/livestock"><Icon icon=LuBeef width="20" height="20" />{nav_livestock}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/weather"><Icon icon=LuCloudSun width="20" height="20" />{nav_weather}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/resources"><Icon icon=LuBriefcase width="20" height="20" />{nav_resources}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/equipment"><Icon icon=LuTractor width="20" height="20" />{nav_equipment}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/finance"><Icon icon=LuWallet width="20" height="20" />{nav_finance}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/analytics"><Icon icon=LuChartBar width="20" height="20" />{nav_analytics}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/compliance"><Icon icon=LuChartNoAxesColumn width="20" height="20" />{nav_compliance}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/users"><Icon icon=LuUsers width="20" height="20" />{nav_users}</a>
                    </li>
                    <li><a href="/settings"><Icon icon=LuSettings width="20" height="20" />{nav_settings}</a></li>
                    <li><a href="http://localhost:3001" target="_blank"><Icon icon=LuLayoutDashboard width="20" height="20" />{nav_grafana}</a></li>
                    <div class="mt-auto">
                        <div class="divider"></div>
                        <li>
                            <a class="flex justify-between items-center" on:click=move |_| {
                                set_theme.set(if theme.get() == Theme::Light { Theme::Dark } else { Theme::Light });
                            }>
                                <div class="flex gap-2 text-base-content/70">
                                    <Icon icon=LuSun width="20" height="20" />
                                    {theme_label}
                                </div>
                                <input type="checkbox" class="toggle toggle-sm" checked=move || theme.get() == Theme::Dark />
                            </a>
                        </li>
                        <li>
                            <a class="text-error" on:click=move |_| {
                            api::clear_auth_token();
                            api::clear_user_role();
                            let _ = window().location().reload();
                        }>
                                <Icon icon=LuLogOut width="20" height="20" />
                                {logout_label}
                            </a>
                        </li>
                    </div>
                </ul>
            </div>
        </div>
    }
}

fn main() {
    leptos::mount::mount_to_body(App);
}
