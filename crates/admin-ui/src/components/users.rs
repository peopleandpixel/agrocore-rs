use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

#[component]
pub fn UserManagement() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">"Benutzerverwaltung"</h1>
                    <p class="text-base-content/60">"Verwalten Sie Teammitglieder und deren Zugriffsberechtigungen."</p>
                </div>
                <button class="btn btn-primary">
                    <Icon icon=LuUserPlus width="20" height="20" />
                    "Benutzer einladen"
                </button>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"Rolle"</th>
                                <th>"Status"</th>
                                <th>"Letzter Login"</th>
                                <th class="text-right">"Aktionen"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>
                                    <div class="flex items-center gap-3">
                                        <div class="avatar placeholder">
                                            <div class="bg-neutral text-neutral-content rounded-full w-10">
                                                <span>"JR"</span>
                                            </div>
                                        </div>
                                        <div>
                                            <div class="font-bold">"Jens Reinemuth"</div>
                                            <div class="text-sm opacity-50">"jens@example.com"</div>
                                        </div>
                                    </div>
                                </td>
                                <td>
                                    <div class="badge badge-primary gap-1">
                                        <Icon icon=LuShield width="12" height="12" />
                                        "Administrator"
                                    </div>
                                </td>
                                <td><span class="badge badge-success badge-outline">"Aktiv"</span></td>
                                <td>"Vor 10 Minuten"</td>
                                <th class="text-right">
                                    <button class="btn btn-ghost btn-xs"><Icon icon=LuPencil width="14" height="14" /></button>
                                    <button class="btn btn-ghost btn-xs text-error"><Icon icon=LuMail width="14" height="14" /></button>
                                </th>
                            </tr>
                            <tr>
                                <td>
                                    <div class="flex items-center gap-3">
                                        <div class="avatar placeholder">
                                            <div class="bg-neutral text-neutral-content rounded-full w-10">
                                                <span>"MS"</span>
                                            </div>
                                        </div>
                                        <div>
                                            <div class="font-bold">"Maria Santos"</div>
                                            <div class="text-sm opacity-50">"maria@example.com"</div>
                                        </div>
                                    </div>
                                </td>
                                <td>
                                    <div class="badge badge-secondary gap-1">
                                        <Icon icon=LuShield width="12" height="12" />
                                        "Manager"
                                    </div>
                                </td>
                                <td><span class="badge badge-success badge-outline">"Aktiv"</span></td>
                                <td>"Vor 2 Stunden"</td>
                                <th class="text-right">
                                    <button class="btn btn-ghost btn-xs"><Icon icon=LuPencil width="14" height="14" /></button>
                                    <button class="btn btn-ghost btn-xs text-error"><Icon icon=LuMail width="14" height="14" /></button>
                                </th>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
