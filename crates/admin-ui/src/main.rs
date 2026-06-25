mod components;

use crate::components::dashboard::DashboardView;
use crate::components::orders::OrderList;
use crate::components::sites::SiteManagement;
use crate::components::users::UserManagement;
use leptos::prelude::*;
use leptos_router::{components::*, path};
use leptos_icons::Icon;
use icondata::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UserRole {
    Admin,
    Manager,
    Worker,
}

#[component]
pub fn App() -> impl IntoView {
    let (user_role, set_user_role) = signal(UserRole::Admin);

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
                            <Route path=path!("/tasks") view=|| view! { <OrderList /> } />
                            <Route path=path!("/resources") view=|| view! { <ResourcesPage /> } />
                            <Route path=path!("/compliance") view=|| view! { <CompliancePage /> } />
                            <Route path=path!("/users") view=|| view! { <UserManagement /> } />
                            <Route path=path!("/settings") view=|| view! { <SettingsPage /> } />
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
                            "Dashboard"
                        </a>
                    </li>

                    <li>
                        <a href="/sites">
                            <Icon icon=LuMap width="20" height="20" />
                            "Standorte"
                        </a>
                    </li>

                    <li>
                        <a href="/tasks">
                            <Icon icon=LuSquareCheck width="20" height="20" />
                            "Aufgaben"
                        </a>
                    </li>

                    {move || (user_role.get() == UserRole::Admin || user_role.get() == UserRole::Manager).then(|| {
                        view! {
                            <>
                                <li>
                                    <a href="/resources">
                                        <Icon icon=LuBriefcase width="20" height="20" />
                                        "Ressourcen"
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
                                        "Benutzerverwaltung"
                                    </a>
                                </li>
                            </>
                        }
                    })}

                    <li>
                        <a href="/settings">
                            <Icon icon=LuSettings width="20" height="20" />
                            "Einstellungen"
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

#[component]
fn Dashboard(role: ReadSignal<UserRole>) -> impl IntoView {
    view! {
        <h1 class="text-3xl font-bold mb-6">"Dashboard"</h1>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div class="stats shadow bg-primary text-primary-content">
                <div class="stat">
                    <div class="stat-title text-primary-content opacity-70">"Aktive Aufträge"</div>
                    <div class="stat-value">"12"</div>
                    <div class="stat-desc text-primary-content opacity-70">"2 heute fällig"</div>
                </div>
            </div>

            <div class="stats shadow bg-secondary text-secondary-content">
                <div class="stat">
                    <div class="stat-title text-secondary-content opacity-70">"Fläche"</div>
                    <div class="stat-value">"450 ha"</div>
                    <div class="stat-desc text-secondary-content opacity-70">"8 Standorte"</div>
                </div>
            </div>

            <div class="stats shadow">
                <div class="stat">
                    <div class="stat-title">"System Status"</div>
                    <div class="stat-value text-success">"OK"</div>
                    <div class="stat-desc">"Alle Dienste online"</div>
                </div>
            </div>
        </div>

        <div class="mt-8 card bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">"Willkommen zurück!"</h2>
                <p>"Aktuelle Rolle: " <b>{move || format!("{:?}", role.get())}</b></p>
                
                {move || match role.get() {
                    UserRole::Admin => view! {
                        <div class="alert alert-info mt-4">
                            <span>"Admin-Konsole: Voller Zugriff auf alle Systemfunktionen."</span>
                        </div>
                    }.into_any(),
                    UserRole::Manager => view! {
                        <div class="alert alert-warning mt-4">
                            <span>"Manager-Ansicht: Zugriff auf Berichte und Zuweisungen."</span>
                        </div>
                    }.into_any(),
                    UserRole::Worker => view! {
                        <div class="alert alert-success mt-4">
                            <span>"Worker-Ansicht: Fokus auf heute anstehende Aufgaben."</span>
                        </div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}

#[component]
fn UsersPage() -> impl IntoView {
    view! {
        <h1 class="text-3xl font-bold mb-6">"Benutzerverwaltung"</h1>
        <div class="overflow-x-auto">
            <table class="table bg-base-100 shadow">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Rolle"</th>
                        <th>"Status"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>"Jens Reinemuth"</td>
                        <td><div class="badge badge-primary">"Admin"</div></td>
                        <td>"Aktiv"</td>
                        <td><button class="btn btn-ghost btn-xs">"Edit"</button></td>
                    </tr>
                    <tr>
                        <td>"Maria Santos"</td>
                        <td><div class="badge badge-secondary">"Manager"</div></td>
                        <td>"Aktiv"</td>
                        <td><button class="btn btn-ghost btn-xs">"Edit"</button></td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn SettingsPage() -> impl IntoView {
    view! {
        <h1 class="text-3xl font-bold mb-6">"Einstellungen"</h1>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title">"System"</h2>
                    <div class="form-control">
                        <label class="label cursor-pointer">
                            <span class="label-text">"Benachrichtigungen aktivieren"</span>
                            <input type="checkbox" class="toggle toggle-primary" checked="checked" />
                        </label>
                    </div>
                    <div class="form-control mt-4">
                        <label class="label">
                            <span class="label-text">"Sprache"</span>
                        </label>
                        <select class="select select-bordered">
                            <option>"Deutsch"</option>
                            <option>"English"</option>
                            <option>"Português"</option>
                        </select>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title">"Erstkonfiguration"</h2>
                    <p class="text-sm text-base-content/70">"Stellen Sie hier die Verbindung zum API-Backend her."</p>
                    <div class="form-control mt-4">
                        <label class="label">
                            <span class="label-text">"API Base URL"</span>
                        </label>
                        <input type="text" class="input input-bordered" placeholder="http://localhost:3000/api/v1" />
                    </div>
                    <div class="form-control mt-4">
                        <label class="label">
                            <span class="label-text">"Datenbank Name"</span>
                        </label>
                        <input type="text" class="input input-bordered" placeholder="agrocore" />
                    </div>
                    <div class="card-actions justify-end mt-4">
                        <button class="btn btn-primary">"Speichern & Testen"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SitesPage() -> impl IntoView {
    view! {
        <h1 class="text-3xl font-bold mb-6">"Standortverwaltung"</h1>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="card bg-base-100 shadow-xl border-t-4 border-primary">
                <div class="card-body">
                    <h2 class="card-title"><Icon icon=LuMap width="24" height="24" attr:class="text-primary"/> "Weinberge"</h2>
                    <p>"Verwaltung von Rebsorten, Parzellen und Terroir-Daten."</p>
                    <div class="stats bg-base-200 mt-2">
                        <div class="stat">
                            <div class="stat-title">"Fläche"</div>
                            <div class="stat-value text-sm">"240 ha"</div>
                        </div>
                        <div class="stat">
                            <div class="stat-title">"Parzellen"</div>
                            <div class="stat-value text-sm">"15"</div>
                        </div>
                    </div>
                    <div class="card-actions justify-end mt-4">
                        <button class="btn btn-primary btn-sm">"Öffnen"</button>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow-xl border-t-4 border-secondary">
                <div class="card-body">
                    <h2 class="card-title"><Icon icon=LuMap width="24" height="24" attr:class="text-secondary"/> "Olivenhaine"</h2>
                    <p>"Überwachung von Baumbestand, Bewässerung und Ernteprognosen."</p>
                    <div class="stats bg-base-200 mt-2">
                        <div class="stat">
                            <div class="stat-title">"Fläche"</div>
                            <div class="stat-value text-sm">"210 ha"</div>
                        </div>
                        <div class="stat">
                            <div class="stat-title">"Bäume"</div>
                            <div class="stat-value text-sm">"~8.500"</div>
                        </div>
                    </div>
                    <div class="card-actions justify-end mt-4">
                        <button class="btn btn-secondary btn-sm">"Öffnen"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TasksPage() -> impl IntoView {
    view! {
        <h1 class="text-3xl font-bold mb-6">"Aufgabenmanagement"</h1>
        <div class="tabs tabs-boxed mb-4">
            <a class="tab tab-active">"Alle"</a>
            <a class="tab">"Heute"</a>
            <a class="tab">"Geplant"</a>
            <a class="tab">"Abgeschlossen"</a>
        </div>
        <div class="space-y-4">
            <div class="alert bg-base-100 shadow-md border-l-4 border-info">
                <Icon icon=LuSquareCheck width="20" height="20" />
                <div class="flex-1">
                    <h3 class="font-bold">"Bodenanalyse Parzelle B3"</h3>
                    <div class="text-xs">"Zugeordnet: Maria Santos | Fällig: Morgen"</div>
                </div>
                <button class="btn btn-sm">"Details"</button>
            </div>
            <div class="alert bg-base-100 shadow-md border-l-4 border-warning">
                <Icon icon=LuSquareCheck width="20" height="20" />
                <div class="flex-1">
                    <h3 class="font-bold">"Bewässerungssystem Wartung"</h3>
                    <div class="text-xs">"Zugeordnet: Unzugewiesen | Status: Dringend"</div>
                </div>
                <button class="btn btn-sm btn-warning">"Zuweisen"</button>
            </div>
        </div>
    }
}

#[component]
fn ResourcesPage() -> impl IntoView {
    view! {
        <h1 class="text-3xl font-bold mb-6">"Ressourcenverwaltung"</h1>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title"><Icon icon=LuDroplets attr:class="text-info" width="24" height="24" /> "Wasser-Management"</h2>
                    <progress class="progress progress-info w-full" value="70" max="100"></progress>
                    <p class="text-sm">"Aktueller Füllstand Zisterne: 70%"</p>
                </div>
            </div>
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title"><Icon icon=LuBriefcase attr:class="text-success" width="24" height="24" /> "Personal"</h2>
                    <div class="flex items-center gap-4">
                        <div class="avatar-group -space-x-6 rtl:space-x-reverse">
                            <div class="avatar placeholder">
                                <div class="w-12 bg-neutral text-neutral-content"><span>"JR"</span></div>
                            </div>
                            <div class="avatar placeholder">
                                <div class="w-12 bg-neutral text-neutral-content"><span>"MS"</span></div>
                            </div>
                            <div class="avatar">
                                <div class="w-12 bg-neutral text-neutral-content"><span>"+5"</span></div>
                            </div>
                        </div>
                        <p class="text-sm">"7 Mitarbeiter aktuell im Dienst"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn CompliancePage() -> impl IntoView {
    view! {
        <h1 class="text-3xl font-bold mb-6">"Compliance & Berichte"</h1>
        <div class="bg-base-100 p-6 rounded-box shadow">
            <ul class="steps steps-vertical lg:steps-horizontal w-full mb-8">
                <li class="step step-primary">"Datenprüfung"</li>
                <li class="step step-primary">"Validierung"</li>
                <li class="step">"Zertifizierung"</li>
                <li class="step">"Bericht generiert"</li>
            </ul>
            <div class="flex flex-wrap gap-4">
                <button class="btn btn-outline"><Icon icon=LuChartNoAxesColumn width="18" height="18"/> "Export PDF"</button>
                <button class="btn btn-outline"><Icon icon=LuChartNoAxesColumn width="18" height="18"/> "Export XLSX"</button>
            </div>
        </div>
    }
}

fn main() {
    leptos::mount::mount_to_body(App);
}
