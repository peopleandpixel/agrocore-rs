use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

#[component]
pub fn SiteManagement() -> impl IntoView {
    let i18n = use_context::<crate::i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<crate::i18n::Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);
    let (show_add_modal, set_show_add_modal) = signal(false);

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">{move || t("site_management")}</h1>
                    <p class="text-base-content/60">"Verwalten Sie Ihre Weinberge, Olivenhaine und andere Betriebsflächen."</p>
                </div>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
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
                    <div class="col-span-full p-12 text-center bg-base-100 rounded-box shadow">
                         <Icon icon=LuMap attr:class="mx-auto opacity-10 mb-4" width="64" height="64" />
                         <p class="text-base-content/50 italic">"Noch keine Flächen angelegt."</p>
                    </div>
                </div>
            </div>

            <Show when=move || show_add_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">"Neue Fläche anlegen"</h3>
                        
                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">"Name der Fläche"</span></label>
                            <input type="text" placeholder="Parzelle A1" class="input input-bordered w-full" />
                        </div>

                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">"Typ"</span></label>
                            <select class="select select-bordered">
                                <option>"Weinberg"</option>
                                <option>"Olivenhain"</option>
                                <option>"Ackerland"</option>
                                <option>"Wald"</option>
                            </select>
                        </div>

                        <div class="divider">"Details"</div>
                        <p class="text-xs text-base-content/60">"Je nach Typ können später unterschiedliche Aufgaben (z.B. Pflanzenschutz im Weinbau) definiert werden."</p>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)>"Abbrechen"</button>
                            <button class="btn btn-primary">"Speichern"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
