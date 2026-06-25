use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

#[component]
pub fn DashboardView() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-8">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">"Dashboard"</h1>
                    <p class="text-base-content/60">"Willkommen zurück, Jens! Hier ist die Übersicht für heute."</p>
                </div>
                <div class="badge badge-outline gap-2 p-4">
                    <Icon icon=LuActivity width="16" height="16" attr:class="text-success" />
                    "System Status: OK"
                </div>
            </div>

            // Statistik-Karten
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div class="stat bg-base-100 shadow rounded-box">
                    <div class="stat-figure text-primary">
                        <Icon icon=LuClock width="32" height="32" />
                    </div>
                    <div class="stat-title">"Anstehende Aufgaben"</div>
                    <div class="stat-value text-primary">"12"</div>
                    <div class="stat-desc">"2 überfällig"</div>
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
                // Linke Spalte: Aktuelle Aktivitäten
                <div class="lg:col-span-2 flex flex-col gap-6">
                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title mb-4">"Aktive Aufträge"</h2>
                            <div class="space-y-4">
                                <div class="flex items-center gap-4 p-3 bg-base-200 rounded-lg">
                                    <div class="avatar placeholder">
                                        <div class="bg-primary text-primary-content rounded-full w-10">
                                            <span>"JR"</span>
                                        </div>
                                    </div>
                                    <div class="flex-1">
                                        <h3 class="font-bold">"Grunddüngung Frühjahr"</h3>
                                        <p class="text-xs text-base-content/60">"Parzelle A1 - In Arbeit"</p>
                                    </div>
                                    <progress class="progress progress-primary w-20" value="65" max="100"></progress>
                                </div>

                                <div class="flex items-center gap-4 p-3 bg-base-200 rounded-lg">
                                    <div class="avatar placeholder">
                                        <div class="bg-secondary text-secondary-content rounded-full w-10">
                                            <span>"MS"</span>
                                        </div>
                                    </div>
                                    <div class="flex-1">
                                        <h3 class="font-bold">"Laubarbeiten"</h3>
                                        <p class="text-xs text-base-content/60">"Weinberg Süd - Geplant"</p>
                                    </div>
                                    <span class="badge badge-ghost">"0%"</span>
                                </div>
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <a href="/tasks" class="btn btn-ghost btn-sm">"Alle ansehen"</a>
                            </div>
                        </div>
                    </div>
                </div>

                // Rechte Spalte: Wetter / Quick Actions
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
        </div>
    }
}
