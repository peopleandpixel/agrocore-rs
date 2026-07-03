use crate::api;
use crate::components::form::RequiredLabel;
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn EquipmentManagement() -> impl IntoView {
    let i18n = use_context::<crate::i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<crate::i18n::Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    let equipment = LocalResource::new(|| async move { api::fetch_equipment().await.ok() });
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (label, set_label) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (equipment_type, set_equipment_type) = signal(String::from("Tractor"));

    let on_create = move |_| {
        let label = label.get();
        let code = {
            let value = code.get();
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        };
        let equipment_type = equipment_type.get();
        set_error.set(None);

        if label.trim().is_empty() {
            set_error.set(Some(String::from("Bitte eine Bezeichnung angeben.")));
            return;
        }

        spawn_local(async move {
            match api::create_equipment(api::CreateEquipmentRequest {
                label,
                code,
                equipment_type,
            })
            .await
            {
                Ok(_) => {
                    let _ = window().location().reload();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">{move || t("equipment_management")}</h1>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <Icon icon=LuPlus width="20" height="20" />
                    "Gerät hinzufügen"
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <For
                    each=move || {
                        equipment
                            .read()
                            .as_ref()
                            .map(|e| {
                                e.as_ref()
                                    .map(|page| page.data.clone())
                                    .unwrap_or_default()
                            })
                            .unwrap_or_default()
                    }
                    key=|item| item.id
                    children=move |item| {
                        let status_class = if item.in_usage { "badge-info" } else { "badge-success" };
                        view! {
                            <div class="card bg-base-100 shadow">
                                <div class="card-body">
                                    <div class="flex justify-between items-start">
                                        <h2 class="card-title">{item.label}</h2>
                                        <div class=format!("badge {}", status_class)>{if item.in_usage { "In Benutzung" } else { "Verfügbar" }}</div>
                                    </div>
                                    <div class="mt-4 space-y-2">
                                        <div class="flex justify-between text-sm">
                                            <span class="opacity-60">"Typ:"</span>
                                            <span class="font-bold">{item.equipment_type}</span>
                                        </div>
                                        <div class="flex justify-between text-sm">
                                            <span class="opacity-60">"Code:"</span>
                                            <span class="font-bold">{item.code.unwrap_or_else(|| String::from("-"))}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            <Show when=move || show_add_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">"Gerät hinzufügen"</h3>

                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="form-control w-full mt-4">
                            <label class="label"><RequiredLabel required=true>{"Bezeichnung"}</RequiredLabel></label>
                            <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_label.set(event_target_value(&ev)) />
                        </div>
                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><span class="label-text">"Code"</span></label>
                                <input type="text" class="input input-bordered w-full" on:input=move |ev| set_code.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Typ"}</RequiredLabel></label>
                                <select class="select select-bordered w-full" required on:change=move |ev| set_equipment_type.set(event_target_value(&ev))>
                                    <option value="Tractor">"Traktor"</option>
                                    <option value="Sprayer">"Spritze"</option>
                                    <option value="Harvester">"Erntemaschine"</option>
                                    <option value="Mulcher">"Mulcher"</option>
                                    <option value="Plow">"Pflug"</option>
                                    <option value="Trailer">"Anhänger"</option>
                                    <option value="Tool">"Werkzeug"</option>
                                </select>
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)>"Abbrechen"</button>
                            <button class="btn btn-primary" on:click=on_create>"Speichern"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
