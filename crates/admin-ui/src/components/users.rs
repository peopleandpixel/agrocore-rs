use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;
use crate::UserRole;

#[component]
pub fn UserManagement() -> impl IntoView {
    let i18n = use_context::<crate::i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<crate::i18n::Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);
    let (show_role_modal, set_show_role_modal) = signal(false);

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">{move || t("users")}</h1>
                    <p class="text-base-content/60">"Verwalten Sie Teammitglieder und deren Zugriffsberechtigungen."</p>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-outline btn-primary" on:click=move |_| set_show_role_modal.set(true)>
                        <Icon icon=LuSettings width="20" height="20" />
                        "Rollen verwalten"
                    </button>
                    <button class="btn btn-primary">
                        <Icon icon=LuUserPlus width="20" height="20" />
                        "Benutzer einladen"
                    </button>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"Rolle"</th>
                                <th>"Status"</th>
                                <th>"Letzter Login"</th>
                                <th class="text-right">"Aktionen"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>"Jens Reinemuth"</td>
                                <td><div class="badge badge-primary">"Admin"</div></td>
                                <td>"Aktiv"</td>
                                <td>"Heute"</td>
                                <td class="text-right"><button class="btn btn-ghost btn-xs">"Edit"</button></td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <Show when=move || show_role_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box max-w-2xl">
                        <h3 class="font-bold text-lg">"Rollen & Berechtigungen"</h3>
                        <p class="py-4">"Definieren Sie hier feingranulare Rollen für Ihren Tenant."</p>
                        
                        <div class="space-y-4">
                            <div class="collapse collapse-plus bg-base-200">
                                <input type="radio" name="my-accordion-3" checked="checked" /> 
                                <div class="collapse-title text-xl font-medium">
                                    "Neue Rolle erstellen"
                                </div>
                                <div class="collapse-content"> 
                                    <div class="form-control w-full">
                                        <label class="label"><span class="label-text">"Rollenname"</span></label>
                                        <input type="text" placeholder="z.B. Erntehelfer Spezial" class="input input-bordered w-full" />
                                    </div>
                                    
                                    <div class="divider">"Berechtigungen"</div>
                                    <div class="grid grid-cols-2 gap-4">
                                        <div class="form-control">
                                            <label class="label cursor-pointer">
                                                <span class="label-text font-bold">"Flächen (Sites)"</span>
                                                <select class="select select-sm select-bordered">
                                                    <option>"Kein Zugriff"</option>
                                                    <option selected>"Nur Lesen"</option>
                                                    <option>"Lesen & Schreiben"</option>
                                                    <option>"Vollzugriff"</option>
                                                </select>
                                            </label>
                                        </div>
                                        <div class="form-control">
                                            <label class="label cursor-pointer">
                                                <span class="label-text font-bold">"Equipment"</span>
                                                <select class="select select-sm select-bordered">
                                                    <option>"Kein Zugriff"</option>
                                                    <option>"Nur Lesen"</option>
                                                    <option>"Lesen & Schreiben"</option>
                                                    <option>"Vollzugriff"</option>
                                                </select>
                                            </label>
                                        </div>
                                        <div class="form-control">
                                            <label class="label cursor-pointer">
                                                <span class="label-text font-bold">"Finanzen"</span>
                                                <select class="select select-sm select-bordered">
                                                    <option selected>"Kein Zugriff"</option>
                                                    <option>"Nur Lesen"</option>
                                                    <option>"Vollzugriff"</option>
                                                </select>
                                            </label>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_role_modal.set(false)>"Schließen"</button>
                            <button class="btn btn-primary">"Rolle speichern"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
