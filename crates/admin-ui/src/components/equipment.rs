use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;
use crate::i18n::I18n;
use crate::i18n::Language;

#[component]
pub fn EquipmentManagement() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">{move || t("equipment_management")}</h1>
                <button class="btn btn-primary">
                    <Icon icon=LuPlus width="20" height="20" />
                    "Gerät hinzufügen"
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <EquipmentCard 
                    name="Traktor Fendt 724" 
                    status="Verfügbar" 
                    status_class="badge-success"
                    next_maintenance="15.08.2026"
                    hours="1.240 h"
                />
                <EquipmentCard 
                    name="Pflanzenschutzspritze" 
                    status="In Benutzung" 
                    status_class="badge-info"
                    next_maintenance="01.07.2026"
                    hours="450 h"
                />
                <EquipmentCard 
                    name="Erntemaschine" 
                    status="Wartung" 
                    status_class="badge-warning"
                    next_maintenance="SOFORT"
                    hours="890 h"
                />
            </div>

            <div class="card bg-base-100 shadow mt-6">
                <div class="card-body">
                    <h2 class="card-title mb-4">"Wartungshistorie"</h2>
                    <div class="overflow-x-auto">
                        <table class="table w-full">
                            <thead>
                                <tr>
                                    <th>"Datum"</th>
                                    <th>"Gerät"</th>
                                    <th>"Typ"</th>
                                    <th>"Techniker"</th>
                                    <th>"Kosten"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td>"10.05.2026"</td>
                                    <td>"Traktor Fendt 724"</td>
                                    <td>"Ölwechsel"</td>
                                    <td>"Werkstatt Meyer"</td>
                                    <td>"350 €"</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn EquipmentCard(
    name: &'static str, 
    status: &'static str, 
    status_class: &'static str,
    next_maintenance: &'static str,
    hours: &'static str
) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <div class="flex justify-between items-start">
                    <h2 class="card-title">{name}</h2>
                    <div class=format!("badge {}", status_class)>{status}</div>
                </div>
                <div class="mt-4 space-y-2">
                    <div class="flex justify-between text-sm">
                        <span class="opacity-60">"Betriebsstunden:"</span>
                        <span class="font-bold">{hours}</span>
                    </div>
                    <div class="flex justify-between text-sm">
                        <span class="opacity-60">"Nächste Wartung:"</span>
                        <span class="font-bold">{next_maintenance}</span>
                    </div>
                </div>
                <div class="card-actions justify-end mt-4">
                    <button class="btn btn-ghost btn-xs">"Logbuch"</button>
                    <button class="btn btn-primary btn-xs">"Details"</button>
                </div>
            </div>
        </div>
    }
}
