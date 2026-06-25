use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

#[component]
pub fn SiteManagement() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">"Standortverwaltung"</h1>
                    <p class="text-base-content/60">"Verwalten Sie Ihre Weinberge, Olivenhaine und andere Betriebsflächen."</p>
                </div>
                <button class="btn btn-primary">
                    <Icon icon=LuPlus width="20" height="20" />
                    "Fläche hinzufügen"
                </button>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
                // Sidebar für Filter/Kategorien
                <div class="lg:col-span-1">
                    <div class="card bg-base-100 shadow h-full">
                        <div class="card-body p-4">
                            <h2 class="font-bold mb-4">"Filter"</h2>
                            <div class="form-control mb-4">
                                <div class="join">
                                    <input class="input input-bordered input-sm join-item w-full" placeholder="Suchen..."/>
                                    <button class="btn btn-sm join-item"><Icon icon=LuSearch width="14" height="14" /></button>
                                </div>
                            </div>
                            
                            <div class="flex flex-col gap-2">
                                <label class="label cursor-pointer justify-start gap-4 p-2 hover:bg-base-200 rounded-lg">
                                    <input type="checkbox" class="checkbox checkbox-sm checkbox-primary" checked="checked" />
                                    <span class="label-text">"Weinberge"</span>
                                    <span class="badge badge-sm ml-auto">"15"</span>
                                </label>
                                <label class="label cursor-pointer justify-start gap-4 p-2 hover:bg-base-200 rounded-lg">
                                    <input type="checkbox" class="checkbox checkbox-sm checkbox-secondary" checked="checked" />
                                    <span class="label-text">"Olivenhaine"</span>
                                    <span class="badge badge-sm ml-auto">"8"</span>
                                </label>
                                <label class="label cursor-pointer justify-start gap-4 p-2 hover:bg-base-200 rounded-lg">
                                    <input type="checkbox" class="checkbox checkbox-sm" />
                                    <span class="label-text">"Brachland"</span>
                                    <span class="badge badge-sm ml-auto">"2"</span>
                                </label>
                            </div>

                            <div class="divider"></div>

                            <h2 class="font-bold mb-4">"Layer"</h2>
                            <div class="flex flex-col gap-2 text-sm">
                                <div class="flex items-center gap-2">
                                    <Icon icon=LuLayers width="14" height="14" />
                                    "Bodenanalyse"
                                </div>
                                <div class="flex items-center gap-2">
                                    <Icon icon=LuLayers width="14" height="14" />
                                    "Ertragsprognose"
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // Hauptinhalt: Karten/Liste der Flächen
                <div class="lg:col-span-3 grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="card bg-base-100 shadow hover:shadow-lg transition-shadow">
                        <figure class="h-32 bg-primary/10">
                            <Icon icon=LuMap width="48" height="48" attr:class="text-primary opacity-20" />
                        </figure>
                        <div class="card-body p-4">
                            <div class="flex justify-between items-start">
                                <div>
                                    <h2 class="card-title text-sm">"Parzelle A1 - Schlossberg"</h2>
                                    <p class="text-xs text-base-content/60">"Riesling | 2.4 ha"</p>
                                </div>
                                <div class="badge badge-primary">"Aktiv"</div>
                            </div>
                            <div class="divider my-2"></div>
                            <div class="flex justify-between text-xs">
                                <span>"Letzte Maßnahme:"</span>
                                <span class="font-bold">"Düngung (vor 2 Tagen)"</span>
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <button class="btn btn-ghost btn-xs">"Details"</button>
                                <button class="btn btn-primary btn-xs">"Karte"</button>
                            </div>
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow hover:shadow-lg transition-shadow">
                        <figure class="h-32 bg-secondary/10">
                            <Icon icon=LuMap width="48" height="48" attr:class="text-secondary opacity-20" />
                        </figure>
                        <div class="card-body p-4">
                            <div class="flex justify-between items-start">
                                <div>
                                    <h2 class="card-title text-sm">"Olivenhain West"</h2>
                                    <p class="text-xs text-base-content/60">"Arbequina | 5.0 ha"</p>
                                </div>
                                <div class="badge badge-secondary">"Aktiv"</div>
                            </div>
                            <div class="divider my-2"></div>
                            <div class="flex justify-between text-xs">
                                <span>"Status:"</span>
                                <span class="font-bold">"Bewässerung OK"</span>
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <button class="btn btn-ghost btn-xs">"Details"</button>
                                <button class="btn btn-secondary btn-xs">"Karte"</button>
                            </div>
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow hover:shadow-lg transition-shadow">
                        <figure class="h-32 bg-primary/10">
                            <Icon icon=LuMap width="48" height="48" attr:class="text-primary opacity-20" />
                        </figure>
                        <div class="card-body p-4">
                            <div class="flex justify-between items-start">
                                <div>
                                    <h2 class="card-title text-sm">"Parzelle B3 - Sonnenhang"</h2>
                                    <p class="text-xs text-base-content/60">"Spätburgunder | 1.8 ha"</p>
                                </div>
                                <div class="badge badge-warning">"Wartung"</div>
                            </div>
                            <div class="divider my-2"></div>
                            <div class="flex justify-between text-xs">
                                <span>"Status:"</span>
                                <span class="font-bold">"Bodenanalyse fällig"</span>
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <button class="btn btn-ghost btn-xs">"Details"</button>
                                <button class="btn btn-primary btn-xs">"Karte"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
