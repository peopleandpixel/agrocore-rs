use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

#[component]
pub fn ResourcesPage() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">"Ressourcenverwaltung"</h1>
                    <p class="text-base-content/60">"Überblick über Wasser, Energie, Betriebsmittel und Personal."</p>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-outline">
                        <Icon icon=LuHistory width="20" height="20" />
                        "Verlauf"
                    </button>
                    <button class="btn btn-primary">
                        <Icon icon=LuPlus width="20" height="20" />
                        "Inventar hinzufügen"
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Wasser & Energie
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuDroplets attr:class="text-info" width="24" height="24" /> "Wasser & Bewässerung"</h2>
                        <div class="stats bg-base-200 w-full mt-2">
                            <div class="stat">
                                <div class="stat-title">"Zisterne Hauptbetrieb"</div>
                                <div class="stat-value text-info">"70%"</div>
                                <div class="stat-desc">"~140.000 Liter verbleibend"</div>
                            </div>
                            <div class="stat">
                                <div class="stat-title">"Verbrauch heute"</div>
                                <div class="stat-value text-sm">"12.400 L"</div>
                                <div class="stat-desc">"Pumpe A2 aktiv"</div>
                            </div>
                        </div>
                        <div class="mt-4">
                            <p class="text-sm font-bold mb-2">"Status Bewässerungskreise"</p>
                            <div class="space-y-2">
                                <div class="flex justify-between items-center text-xs">
                                    <span>"Sektor Nord (Wein)"</span>
                                    <span class="badge badge-success badge-sm">"In Betrieb"</span>
                                </div>
                                <div class="flex justify-between items-center text-xs">
                                    <span>"Sektor Süd (Oliven)"</span>
                                    <span class="badge badge-ghost badge-sm">"Standby"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // Personal
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuUsers attr:class="text-success" width="24" height="24" /> "Arbeitskräfte"</h2>
                        <div class="stats bg-base-200 w-full mt-2">
                            <div class="stat">
                                <div class="stat-title">"Aktuell im Einsatz"</div>
                                <div class="stat-value text-success">"8"</div>
                                <div class="stat-desc">"6 Festangestellt, 2 Saisonkr."</div>
                            </div>
                            <div class="stat">
                                <div class="stat-title">"Stunden heute"</div>
                                <div class="stat-value text-sm">"42,5 h"</div>
                                <div class="stat-desc">"Schnitt 5,3 h/Pers."</div>
                            </div>
                        </div>
                        <div class="mt-4">
                            <p class="text-sm font-bold mb-2">"Zuweisungen"</p>
                            <div class="flex flex-wrap gap-2">
                                <div class="badge badge-outline">"Rebschnitt: 4"</div>
                                <div class="badge badge-outline">"Wartung: 2"</div>
                                <div class="badge badge-outline">"Logistik: 2"</div>
                            </div>
                        </div>
                    </div>
                </div>

                // Betriebsmittel (Lager)
                <div class="card bg-base-100 shadow lg:col-span-2">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuPackage attr:class="text-warning" width="24" height="24" /> "Lagerbestand Betriebsmittel"</h2>
                        <div class="overflow-x-auto mt-4">
                            <table class="table table-sm">
                                <thead>
                                    <tr>
                                        <th>"Produkt"</th>
                                        <th>"Kategorie"</th>
                                        <th>"Bestand"</th>
                                        <th>"Min. Bestand"</th>
                                        <th>"Status"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr>
                                        <td>"Vitisan"</td>
                                        <td>"Pflanzenschutz"</td>
                                        <td>"250 kg"</td>
                                        <td>"50 kg"</td>
                                        <td><div class="badge badge-success">"OK"</div></td>
                                    </tr>
                                    <tr>
                                        <td>"Diesel (B7)"</td>
                                        <td>"Treibstoff"</td>
                                        <td>"1.200 L"</td>
                                        <td>"2.000 L"</td>
                                        <td><div class="badge badge-warning">"Nachbestellen"</div></td>
                                    </tr>
                                    <tr>
                                        <td>"Düngemittel NPK"</td>
                                        <td>"Dünger"</td>
                                        <td>"15 kg"</td>
                                        <td>"100 kg"</td>
                                        <td><div class="badge badge-error">"Kritisch"</div></td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
