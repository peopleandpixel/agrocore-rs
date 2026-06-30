use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

#[component]
pub fn CompliancePage() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">"Compliance & Zertifizierung"</h1>
                    <p class="text-base-content/60">"Verwalten Sie Ihre gesetzlichen Anforderungen und Zertifizierungen."</p>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-outline">
                        <Icon icon=LuDownload width="20" height="20" />
                        "Bericht exportieren"
                    </button>
                    <button class="btn btn-primary">
                        <Icon icon=LuPlus width="20" height="20" />
                        "Neue Prüfung"
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">"Compliance Score"</div>
                        <div class="stat-value text-success">"94%"</div>
                        <div class="stat-desc">"↑ 2% seit letztem Monat"</div>
                    </div>
                </div>
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">"Offene Maßnahmen"</div>
                        <div class="stat-value text-warning">"3"</div>
                        <div class="stat-desc">"Nächste Frist in 5 Tagen"</div>
                    </div>
                </div>
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">"Nächstes Audit"</div>
                        <div class="stat-value">"15.09."</div>
                        <div class="stat-desc">"GlobalGAP Re-Zertifizierung"</div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title mb-4">"Zertifizierungs-Status"</h2>
                    <div class="space-y-4">
                        <div class="flex items-center justify-between p-4 bg-base-200 rounded-lg">
                            <div class="flex items-center gap-4">
                                <div class="avatar placeholder">
                                    <div class="bg-success text-success-content rounded-full w-12">
                                        <span>"GAP"</span>
                                    </div>
                                </div>
                                <div>
                                    <h3 class="font-bold">"GlobalGAP"</h3>
                                    <p class="text-xs">"Gültig bis 31.12.2026"</p>
                                </div>
                            </div>
                            <div class="badge badge-success">"Zertifiziert"</div>
                        </div>

                        <div class="flex items-center justify-between p-4 bg-base-200 rounded-lg">
                            <div class="flex items-center gap-4">
                                <div class="avatar placeholder">
                                    <div class="bg-info text-info-content rounded-full w-12">
                                        <span>"BIO"</span>
                                    </div>
                                </div>
                                <div>
                                    <h3 class="font-bold">"EU-Bio-Siegel"</h3>
                                    <p class="text-xs">"In Prüfung - Letztes Audit am 10.06."</p>
                                </div>
                            </div>
                            <div class="badge badge-info">"In Prüfung"</div>
                        </div>

                        <div class="flex items-center justify-between p-4 bg-base-200 rounded-lg">
                            <div class="flex items-center gap-4">
                                <div class="avatar placeholder">
                                    <div class="bg-warning text-warning-content rounded-full w-12">
                                        <span>"H2O"</span>
                                    </div>
                                </div>
                                <div>
                                    <h3 class="font-bold">"Wasserentnahmerecht"</h3>
                                    <p class="text-xs">"Ablauf der Genehmigung am 30.08."</p>
                                </div>
                            </div>
                            <div class="badge badge-warning">"Aktion erforderlich"</div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title mb-4">"Anwendungshistorie (Pflanzenschutz)"</h2>
                    <div class="overflow-x-auto">
                        <table class="table table-zebra w-full">
                            <thead>
                                <tr>
                                    <th>"Datum"</th>
                                    <th>"Fläche"</th>
                                    <th>"Mittel"</th>
                                    <th>"Menge"</th>
                                    <th>"Anwender"</th>
                                    <th>"Wartefrist"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td>"20.06.2026"</td>
                                    <td>"Weinberg Nord"</td>
                                    <td>"Cuprozin Progress"</td>
                                    <td>"2,0 kg/ha"</td>
                                    <td>"J. Reinemuth"</td>
                                    <td><span class="badge badge-success">"Abgelaufen"</span></td>
                                </tr>
                                <tr>
                                    <td>"25.06.2026"</td>
                                    <td>"Parzelle B3"</td>
                                    <td>"Vitisan"</td>
                                    <td>"5,0 kg/ha"</td>
                                    <td>"M. Santos"</td>
                                    <td><span class="badge badge-warning">"Aktiv (3 Tage)"</span></td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}
