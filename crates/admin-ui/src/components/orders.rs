use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

#[component]
pub fn OrderList() -> impl IntoView {
    let i18n = use_context::<crate::i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<crate::i18n::Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">{move || t("order_management")}</h1>
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
                                // Data will be fetched from API
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}
