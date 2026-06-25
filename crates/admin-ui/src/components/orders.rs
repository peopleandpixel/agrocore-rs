use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

#[component]
pub fn OrderList() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">"Aufträge"</h1>
                <button class="btn btn-primary">
                    <Icon icon=LuPlus width="20" height="20" />
                    "Neuer Auftrag"
                </button>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body p-0">
                    <div class="flex justify-between items-center p-4 border-b border-base-200">
                        <div class="flex gap-2">
                            <div class="join">
                                <input class="input input-bordered join-item" placeholder="Suchen..."/>
                                <button class="btn join-item">
                                    <Icon icon=LuListFilter width="18" height="18" />
                                </button>
                            </div>
                        </div>
                        <div class="flex gap-2">
                            <select class="select select-bordered select-sm">
                                <option disabled selected>"Status"</option>
                                <option>"Alle"</option>
                                <option>"In Arbeit"</option>
                                <option>"Abgeschlossen"</option>
                            </select>
                        </div>
                    </div>

                    <div class="overflow-x-auto">
                        <table class="table table-hover">
                            <thead>
                                <tr>
                                    <th>"Typ"</th>
                                    <th>"Bezeichnung"</th>
                                    <th>"Standorte"</th>
                                    <th>"Zuweisung"</th>
                                    <th>"Fälligkeit"</th>
                                    <th>"Fortschritt"</th>
                                    <th class="text-center">"Aktionen"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr class="hover">
                                    <td>
                                        <div class="badge badge-outline">"Bodenbearbeitung"</div>
                                    </td>
                                    <td class="font-medium">"Grunddüngung Frühjahr 2024"</td>
                                    <td>
                                        <div class="flex flex-wrap gap-1">
                                            <div class="badge badge-sm badge-ghost">"Parzelle A1"</div>
                                            <div class="badge badge-sm badge-ghost">"Parzelle A2"</div>
                                        </div>
                                    </td>
                                    <td>
                                        <div class="flex items-center gap-2">
                                            <div class="avatar placeholder">
                                                <div class="bg-neutral text-neutral-content rounded-full w-6">
                                                    <span class="text-xs">"JR"</span>
                                                </div>
                                            </div>
                                            <span>"Jens Reinemuth"</span>
                                        </div>
                                    </td>
                                    <td>"28.06.2024"</td>
                                    <td>
                                        <div class="flex flex-col gap-1">
                                            <progress class="progress progress-primary w-24" value="45" max="100"></progress>
                                            <span class="text-xs">"45%"</span>
                                        </div>
                                    </td>
                                    <td class="text-center">
                                        <button class="btn btn-ghost btn-xs">"Bearbeiten"</button>
                                    </td>
                                </tr>
                                <tr class="hover">
                                    <td>
                                        <div class="badge badge-outline text-info">"Pflanzenschutz"</div>
                                    </td>
                                    <td class="font-medium">"Mehltau-Prophylaxe"</td>
                                    <td>
                                        <div class="badge badge-sm badge-ghost">"Weinberg Süd"</div>
                                    </td>
                                    <td>
                                        <div class="flex items-center gap-2">
                                            <div class="avatar placeholder">
                                                <div class="bg-neutral text-neutral-content rounded-full w-6">
                                                    <span class="text-xs">"MS"</span>
                                                </div>
                                            </div>
                                            <span>"Maria Santos"</span>
                                        </div>
                                    </td>
                                    <td>"30.06.2024"</td>
                                    <td>
                                        <div class="flex flex-col gap-1">
                                            <progress class="progress progress-info w-24" value="0" max="100"></progress>
                                            <span class="text-xs">"0%"</span>
                                        </div>
                                    </td>
                                    <td class="text-center">
                                        <button class="btn btn-ghost btn-xs">"Bearbeiten"</button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}
