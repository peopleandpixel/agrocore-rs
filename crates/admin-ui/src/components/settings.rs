use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;
use crate::i18n::{I18n, Language};

#[component]
pub fn SettingsPage() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n engine");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let set_lang = use_context::<WriteSignal<Language>>().expect("set lang signal");

    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    view! {
        <div class="flex flex-col gap-6">
            <h1 class="text-3xl font-bold">{move || t("settings")}</h1>
            
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Company Profile
                <div class="lg:col-span-2 space-y-6">
                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title mb-4"><Icon icon=LuBriefcase width="20" height="20" /> "Unternehmensprofil"</h2>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">"Firmenname"</span></label>
                                    <input type="text" value="AgroCore Weinbau GmbH" class="input input-bordered w-full" />
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">"Steuernummer / VAT"</span></label>
                                    <input type="text" value="DE 123 456 789" class="input input-bordered w-full" />
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">"E-Mail (Zentrale)"</span></label>
                                    <input type="email" value="office@agrocore.example" class="input input-bordered w-full" />
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">"Telefon"</span></label>
                                    <input type="tel" value="+49 123 4567890" class="input input-bordered w-full" />
                                </div>
                                <div class="form-control w-full md:col-span-2">
                                    <label class="label"><span class="label-text">"Adresse"</span></label>
                                    <input type="text" value="Hauptstraße 1, 12345 Weinstadt" class="input input-bordered w-full" />
                                </div>
                                <div class="form-control w-full md:col-span-2">
                                    <label class="label"><span class="label-text">"Webseite"</span></label>
                                    <input type="url" value="https://www.agrocore.example" class="input input-bordered w-full" />
                                </div>
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <button class="btn btn-primary">"Profil aktualisieren"</button>
                            </div>
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title mb-4"><Icon icon=LuGlobe width="20" height="20" /> "Anzeige & Sprache"</h2>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">"System-Sprache"</span></label>
                                    <select 
                                        class="select select-bordered"
                                        on:change=move |ev| {
                                            let val = event_target_value(&ev);
                                            set_lang.set(Language::from_str(&val));
                                        }
                                    >
                                        <option value="de" selected=move || lang.get() == Language::DE>"Deutsch"</option>
                                        <option value="en" selected=move || lang.get() == Language::EN>"English"</option>
                                        <option value="es" selected=move || lang.get() == Language::ES>"Español"</option>
                                        <option value="fr" selected=move || lang.get() == Language::FR>"Français"</option>
                                        <option value="pt" selected=move || lang.get() == Language::PT>"Português"</option>
                                    </select>
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">"Zeitzone"</span></label>
                                    <select class="select select-bordered">
                                        <option selected>"Europa/Berlin (UTC+2)"</option>
                                        <option>"UTC"</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // Side Cards
                <div class="space-y-6">
                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title"><Icon icon=LuShieldCheck width="20" height="20" /> "System-Status"</h2>
                            <ul class="space-y-2 mt-2">
                                <li class="flex justify-between items-center text-sm">
                                    <span>"API Verbindung"</span>
                                    <span class="badge badge-success badge-sm">"Online"</span>
                                </li>
                                <li class="flex justify-between items-center text-sm">
                                    <span>"Datenbank"</span>
                                    <span class="badge badge-success badge-sm">"Verbunden"</span>
                                </li>
                                <li class="flex justify-between items-center text-sm">
                                    <span>"Version"</span>
                                    <span class="font-mono text-xs">"v0.4.2-stable"</span>
                                </li>
                            </ul>
                            <div class="divider"></div>
                            <button class="btn btn-outline btn-sm w-full">"System-Log exportieren"</button>
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow border-2 border-error/20">
                        <div class="card-body">
                            <h2 class="card-title text-error"><Icon icon=LuTrash2 width="20" height="20" /> "Gefahrenzone"</h2>
                            <p class="text-xs opacity-70">"Diese Aktionen können nicht rückgängig gemacht werden."</p>
                            <div class="space-y-2 mt-4">
                                <button class="btn btn-outline btn-error btn-sm w-full">"Alle Daten exportieren"</button>
                                <button class="btn btn-error btn-sm w-full">"Tenant löschen"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
