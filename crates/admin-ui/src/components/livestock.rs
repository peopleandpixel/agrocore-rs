use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;
use crate::i18n::I18n;
use crate::i18n::Language;

#[component]
pub fn LivestockManagement() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">{move || t("livestock_management")}</h1>
                <button class="btn btn-primary">
                    <Icon icon=LuPlus width="20" height="20" />
                    "Tier hinzufügen"
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">"Gesamtbestand"</div>
                        <div class="stat-value">"142"</div>
                        <div class="stat-desc">"Tiere im System"</div>
                    </div>
                </div>
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">"In Behandlung"</div>
                        <div class="stat-value text-warning">"5"</div>
                        <div class="stat-desc">"Wartezeiten beachten"</div>
                    </div>
                </div>
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">"Auf Weide"</div>
                        <div class="stat-value text-success">"120"</div>
                        <div class="stat-desc">"Aktive Weiderotation"</div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title mb-4">"Tierliste"</h2>
                    <div class="overflow-x-auto">
                        <table class="table w-full">
                            <thead>
                                <tr>
                                    <th>"ID / Name"</th>
                                    <th>"Art"</th>
                                    <th>"Status"</th>
                                    <th>"Letzte Aktion"</th>
                                    <th>"Aktionen"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td>
                                        <div class="font-bold">"DE 01 234 5678"</div>
                                        <div class="text-sm opacity-50">"Bella"</div>
                                    </td>
                                    <td>"Rind"</td>
                                    <td><div class="badge badge-success">"Gesund"</div></td>
                                    <td>"Weidegang (Südweide)"</td>
                                    <td>
                                        <div class="flex gap-2">
                                            <button class="btn btn-ghost btn-xs">"Details"</button>
                                            <button class="btn btn-ghost btn-xs text-info">"Behandlung"</button>
                                        </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td>
                                        <div class="font-bold">"DE 01 234 5679"</div>
                                        <div class="text-sm opacity-50">"Luna"</div>
                                    </td>
                                    <td>"Rind"</td>
                                    <td><div class="badge badge-warning">"Behandlung"</div></td>
                                    <td>"Impfung (24.06.)"</td>
                                    <td>
                                        <div class="flex gap-2">
                                            <button class="btn btn-ghost btn-xs">"Details"</button>
                                            <button class="btn btn-ghost btn-xs text-info">"Behandlung"</button>
                                        </div>
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
