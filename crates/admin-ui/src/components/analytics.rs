use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;
use crate::i18n::I18n;
use crate::i18n::Language;

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    view! {
        <div class="flex flex-col gap-6">
            <h1 class="text-3xl font-bold">{move || t("analytics_and_predictions")}</h1>
            
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuTrendingUp attr:class="text-primary" /> "Erntevorhersage"</h2>
                        <p class="text-sm">"Basierend auf Wetterdaten und BBCH-Stadium."</p>
                        <div class="mt-4 p-4 bg-base-200 rounded-box">
                            <div class="flex justify-between items-center mb-2">
                                <span class="font-bold">"Merlot Block A"</span>
                                <span class="text-primary">"~ 12.09.2026"</span>
                            </div>
                            <progress class="progress progress-primary w-full" value="65" max="100"></progress>
                            <div class="text-xs mt-1 text-right">"Konfidenz: 85%"</div>
                        </div>
                        <div class="card-actions justify-end mt-4">
                            <button class="btn btn-sm btn-primary">"Simulation starten"</button>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title"><Icon icon=LuDollarSign attr:class="text-success" /> "Wirtschaftlichkeit"</h2>
                        <p class="text-sm">"Analyse der Deckungsbeiträge pro Schlag."</p>
                        <div class="mt-4 h-32 bg-base-200 rounded-box flex items-center justify-center italic text-base-content/50">
                            "[Graph: Profitabilität pro Standort]"
                        </div>
                        <div class="card-actions justify-end mt-4">
                            <button class="btn btn-sm btn-outline">"Detailbericht"</button>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-100 shadow lg:col-span-2">
                    <div class="card-body">
                        <h2 class="card-title">"Materialkalkulation"</h2>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-2">
                            <div class="form-control">
                                <label class="label"><span class="label-text">"Standort"</span></label>
                                <select class="select select-bordered select-sm">
                                    <option>"Weinberg Nord"</option>
                                    <option>"Olivenhain B"</option>
                                </select>
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">"Maßnahme"</span></label>
                                <select class="select select-bordered select-sm">
                                    <option>"Düngung"</option>
                                    <option>"Pflanzenschutz"</option>
                                </select>
                            </div>
                            <div class="form-control flex items-end pb-1">
                                <button class="btn btn-sm btn-secondary w-full">"Bedarf berechnen"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
