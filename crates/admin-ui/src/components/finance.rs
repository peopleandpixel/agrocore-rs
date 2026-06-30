use leptos::prelude::*;
use crate::i18n::I18n;
use crate::i18n::Language;

#[component]
pub fn FinanceManagement() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    view! {
        <div class="flex flex-col gap-6">
            <h1 class="text-3xl font-bold">{move || t("finance_and_pac")}</h1>
            
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title">"PAC-Anträge"</h2>
                        <div class="space-y-4 mt-4">
                            <div class="p-4 border rounded-box border-primary bg-primary/5">
                                <div class="flex justify-between">
                                    <span class="font-bold">"Antrag 2026"</span>
                                    <span class="badge badge-info">"In Prüfung"</span>
                                </div>
                                <div class="text-xs mt-1">"Eingereicht am 15.05.2026"</div>
                            </div>
                            <button class="btn btn-outline btn-sm w-full">"Neuer Antrag"</button>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title">"Kostenstellen"</h2>
                        <div class="overflow-x-auto mt-4">
                            <table class="table table-xs">
                                <thead>
                                    <tr>
                                        <th>"Name"</th>
                                        <th>"Typ"</th>
                                        <th>"Saldo"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr>
                                        <td>"Maschinenpark"</td>
                                        <td>"Equipment"</td>
                                        <td class="text-error">"-1.240 €"</td>
                                    </tr>
                                    <tr>
                                        <td>"Weinberg Nord"</td>
                                        <td>"Fläche"</td>
                                        <td class="text-success">"+4.500 €"</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-100 shadow border-t-4 border-accent">
                    <div class="card-body">
                        <h2 class="card-title">"Finanz-Quicklink"</h2>
                        <p class="text-sm">"Erfassen Sie schnell neue Einnahmen oder Ausgaben."</p>
                        <div class="card-actions justify-end mt-4">
                            <button class="btn btn-accent btn-sm">"Buchung erstellen"</button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title mb-4">"Letzte Finanzaufzeichnungen"</h2>
                    <div class="overflow-x-auto">
                        <table class="table w-full">
                            <thead>
                                <tr>
                                    <th>"Datum"</th>
                                    <th>"Beschreibung"</th>
                                    <th>"Kategorie"</th>
                                    <th>"Betrag"</th>
                                    <th>"Status"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td>"28.06.2026"</td>
                                    <td>"Dieselkauf"</td>
                                    <td>"Betriebsmittel"</td>
                                    <td class="text-error">"-450,00 €"</td>
                                    <td><div class="badge badge-success">"Verbucht"</div></td>
                                </tr>
                                <tr>
                                    <td>"25.06.2026"</td>
                                    <td>"Verkauf Olivenöl Charge B"</td>
                                    <td>"Erlöse"</td>
                                    <td class="text-success">"+1.200,00 €"</td>
                                    <td><div class="badge badge-success">"Verbucht"</div></td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}
