mod api;
mod components;
mod i18n;

use crate::components::analytics::AnalyticsPage;
use crate::components::audit::AuditLogPage;
use crate::components::compliance::CompliancePage;
use crate::components::dashboard::DashboardView;
use crate::components::equipment::EquipmentManagement;
use crate::components::finance::FinanceManagement;
use crate::components::import::DataImport;
use crate::components::inventory::InventoryManagement;
use crate::components::livestock::LivestockManagement;
use crate::components::login::LoginView;
use crate::components::map::MapView;
use crate::components::orders::OrderList;
use crate::components::resources::ResourcesPage;
use crate::components::settings::SettingsPage;
use crate::components::sites::SiteManagement;
use crate::components::toast::{ToastContainer, provide_toast_context};
use crate::components::users::UserManagement;
use crate::components::weather::WeatherManagement;
use crate::components::wizard::WizardView;
use crate::components::worker_tasks::WorkerTasksPage;
use I::{
    ImMagicWand, LuBeef, LuBox, LuBriefcase, LuChartBar, LuChartNoAxesColumn, LuCircleUser,
    LuClipboardList, LuCloudSun, LuFileDown, LuHistory, LuLayoutDashboard, LuLogOut, LuMap,
    LuMapPin, LuMenu, LuSettings, LuSquareCheck, LuSun, LuTractor, LuUsers, LuWallet,
};
use icondata as I;
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
        .map(|role| match role.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "manager" => UserRole::Manager,
            "worker" => UserRole::Worker,
            "viewer" => UserRole::Viewer,
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
    provide_context(i18n_engine);
    provide_toast_context();

    Effect::new(move |_| {
        use web_sys::window;
        let win = window();
        if let Some(win) = win
            && let Some(doc) = win.document()
            && let Some(root) = doc.document_element()
        {
            let _ = root.set_attribute("data-theme", theme.get().as_str());
        }
    });

    view! {
        <ToastContainer />
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
    let t = i18n::use_i18n();

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

                <div class="w-full px-4 pt-2">
                    {move || {
                        if api::auth_token().map(|t| t.contains("impersonate")).unwrap_or(false) {
                            Some(view! {
                                <div class="alert alert-warning shadow-lg mb-4">
                                    <Icon icon=LuCircleUser width="24" height="24" />
                                    <div>
                                        <h3 class="font-bold">{crate::t!(t, "impersonating_as")}</h3>
                                        <div class="text-xs">"Admin Control Active"</div>
                                    </div>
                                    <button class="btn btn-sm btn-outline btn-ghost" on:click=move |_| {
                                        leptos::task::spawn_local(async move {
                                            if let Ok(resp) = api::stop_impersonation().await {
                                                api::set_auth_token(&resp.token);
                                                api::set_user_role(&resp.roles.first().cloned().unwrap_or_else(|| String::from("Admin")));
                                                let _ = window().location().set_href("/users");
                                            }
                                        });
                                    }>
                                        {crate::t!(t, "stop_impersonation")}
                                    </button>
                                </div>
                            })
                        } else {
                            None
                        }
                    }}
                </div>

                <div class="w-full max-w-5xl animate-fade-in" id="main-content">
                    <Router>
                        <Routes fallback=move || {
                            let t = t;
                            view! { <span>{crate::t!(t, "not_found")}</span> }
                        }>
                            <Route path=path!("/") view=|| view! { <DashboardView /> } />
                            <Route path=path!("/sites") view=|| view! { <SiteManagement /> } />
                            <Route path=path!("/import") view=|| view! { <DataImport /> } />
                            <Route path=path!("/sigpac") view=|| view! { <components::sigpac::SigpacParcels /> } />
                            <Route path=path!("/map") view=|| view! { <MapView /> } />
                            <Route path=path!("/tasks") view=|| view! { <OrderList /> } />
                            <Route path=path!("/livestock") view=|| view! { <LivestockManagement /> } />
                            <Route path=path!("/weather") view=|| view! { <WeatherManagement /> } />
                            <Route path=path!("/finance") view=|| view! { <FinanceManagement /> } />
                            <Route path=path!("/equipment") view=|| view! { <EquipmentManagement /> } />
                            <Route path=path!("/inventory") view=|| view! { <InventoryManagement /> } />
                            <Route path=path!("/analytics") view=|| view! { <AnalyticsPage /> } />
                            <Route path=path!("/audit") view=|| view! { <AuditLogPage /> } />
                            <Route path=path!("/resources") view=|| view! { <ResourcesPage /> } />
                            <Route path=path!("/compliance") view=|| view! { <CompliancePage /> } />
                            <Route path=path!("/users") view=|| view! { <UserManagement /> } />
                            <Route path=path!("/settings") view=|| view! { <SettingsPage /> } />
                            <Route path=path!("/wizard") view=|| view! { <WizardView /> } />
                            <Route path=path!("/worker/tasks") view=|| view! { <WorkerTasksPage /> } />
                        </Routes>
                    </Router>
                </div>
            </div>

            <div class="drawer-side">
                <label for="my-drawer-2" aria-label="close sidebar" class="drawer-overlay"></label>
                <ul class="menu p-4 w-80 min-h-full bg-base-200 text-base-content flex flex-col">
                    <li class="mb-4 text-2xl font-bold p-4">{crate::t!(t, "app_title")}</li>
                    <li><a href="/"><Icon icon=LuLayoutDashboard width="20" height="20" />{crate::t!(t, "nav_dashboard")}</a></li>
                    <li class=move || if user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager { "" } else { "hidden" }>
                        <a href="/users"><Icon icon=LuUsers width="20" height="20" />{crate::t!(t, "nav_users")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Simple { "" } else { "hidden" }>
                        <a href="/wizard" class="bg-primary text-primary-content font-bold"><Icon icon=ImMagicWand width="20" height="20" />{crate::t!(t, "nav_wizard")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/import"><Icon icon=LuFileDown width="20" height="20" />{crate::t!(t, "nav_import")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/sigpac"><Icon icon=LuMap width="20" height="20" />{crate::t!(t, "nav_sigpac")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/sites"><Icon icon=LuMap width="20" height="20" />{crate::t!(t, "nav_sites")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/tasks"><Icon icon=LuSquareCheck width="20" height="20" />{crate::t!(t, "nav_tasks")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/map"><Icon icon=LuMapPin width="20" height="20" />{crate::t!(t, "nav_map")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/livestock"><Icon icon=LuBeef width="20" height="20" />{crate::t!(t, "nav_livestock")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full { "" } else { "hidden" }>
                        <a href="/weather"><Icon icon=LuCloudSun width="20" height="20" />{crate::t!(t, "nav_weather")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/resources"><Icon icon=LuBriefcase width="20" height="20" />{crate::t!(t, "nav_resources")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/equipment"><Icon icon=LuTractor width="20" height="20" />{crate::t!(t, "nav_equipment")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/finance"><Icon icon=LuWallet width="20" height="20" />{crate::t!(t, "nav_finance")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/analytics"><Icon icon=LuChartBar width="20" height="20" />{crate::t!(t, "nav_analytics")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/inventory"><Icon icon=LuBox width="20" height="20" />{crate::t!(t, "nav_inventory")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin) { "" } else { "hidden" }>
                        <a href="/audit"><Icon icon=LuHistory width="20" height="20" />{crate::t!(t, "nav_audit")}</a>
                    </li>
                    <li class=move || if view_mode.get() == ViewMode::Full && (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager) { "" } else { "hidden" }>
                        <a href="/compliance"><Icon icon=LuChartNoAxesColumn width="20" height="20" />{crate::t!(t, "nav_compliance")}</a>
                    </li>
                    <li><a href="/settings"><Icon icon=LuSettings width="20" height="20" />{crate::t!(t, "nav_settings")}</a></li>
                    // Worker-only navigation
                    <li class=move || if user_role.get() == UserRole::Worker { "" } else { "hidden" }>
                        <a href="/worker/tasks"><Icon icon=LuClipboardList width="20" height="20" />{crate::t!(t, "nav_my_tasks")}</a>
                    </li>
                    <li><a href="http://localhost:3001" target="_blank"><Icon icon=LuLayoutDashboard width="20" height="20" />{crate::t!(t, "nav_grafana")}</a></li>
                    <div class="mt-auto">
                        <div class="divider"></div>
                        <li>
                            <div class="flex justify-between items-center p-4">
                                <div class="flex gap-2 text-base-content/70">
                                    <Icon icon=ImMagicWand width="20" height="20" />
                                    {move || if view_mode.get() == ViewMode::Full { t("mode_simple") } else { t("mode_full") }}
                                </div>
                                <input
                                    type="checkbox"
                                    class="toggle toggle-primary toggle-sm"
                                    checked=move || view_mode.get() == ViewMode::Simple
                                    on:change=move |_| {
                                        set_view_mode.set(if view_mode.get() == ViewMode::Full { ViewMode::Simple } else { ViewMode::Full });
                                    }
                                />
                            </div>
                        </li>
                        <li>
                            <a class="flex justify-between items-center" on:click=move |_| {
                                set_theme.set(if theme.get() == Theme::Light { Theme::Dark } else { Theme::Light });
                            }>
                                <div class="flex gap-2 text-base-content/70">
                                    <Icon icon=LuSun width="20" height="20" />
                                    {crate::t!(t, "theme")}
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
                                {crate::t!(t, "logout")}
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
