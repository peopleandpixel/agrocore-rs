use crate::api;
use crate::i18n::{I18n, Language};
use chrono::Utc;
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn LivestockManagement() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = |key: &str| i18n.t(lang.get().as_str(), key);
    let title = t("livestock_management");
    let add_animal_btn = t("add_animal_btn");
    let livestock_total = t("livestock_total");
    let animals_in_system = t("animals_in_system");
    let under_treatment = t("under_treatment");
    let treatment_from_data = t("treatment_from_data");
    let grazing = t("grazing");
    let grazing_from_data = t("grazing_from_data");
    let animal_list = t("animal_list");
    let id_name = t("id_name");
    let species_label = t("species");
    let status_label = t("status");
    let last_action = t("last_action");
    let animal_actions = t("animal_actions");
    let add_treatment = t("add_treatment");
    let add_animal = t("add_animal");
    let identifier_label = t("identifier");
    let breed_label = t("breed");
    let species_cattle = t("species_cattle");
    let species_sheep = t("species_sheep");
    let species_goat = t("species_goat");
    let species_pig = t("species_pig");
    let species_poultry = t("species_poultry");
    let species_horse = t("species_horse");
    let cancel_label = t("cancel");
    let save_label = t("save");
    let record_treatment = t("record_treatment");
    let measure_label = t("measure");
    let note_label = t("observation_note");
    let animals = LocalResource::new(|| async move { api::fetch_animals().await.ok() });
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (show_treatment_modal, set_show_treatment_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (identifier, set_identifier) = signal(String::new());
    let (breed, set_breed) = signal(String::new());
    let (species, set_species) = signal(String::from("cattle"));
    let (treatment_type, set_treatment_type) = signal(String::from("Impfung"));
    let (treatment_notes, set_treatment_notes) = signal(String::new());
    let (selected_animal_id, set_selected_animal_id) = signal(None::<uuid::Uuid>);
    let add_cancel_label = cancel_label.clone();
    let add_save_label = save_label.clone();
    let treatment_cancel_label = cancel_label.clone();
    let treatment_save_label = save_label.clone();

    let on_create_animal = move |_| {
        let identifier = identifier.get();
        let breed = {
            let value = breed.get();
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        };
        let species = species.get();
        set_error.set(None);

        spawn_local(async move {
            match api::create_animal(api::CreateAnimalRequest {
                species,
                breed,
                identifier,
                birth_date: None,
                gender: Some(String::from("female")),
                current_site_id: None,
            })
            .await
            {
                Ok(_) => {
                    let _ = leptos::prelude::window().location().reload();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_treatment = move |_| {
        let animal_id = selected_animal_id.get();
        let treatment_type = treatment_type.get();
        let treatment_notes = treatment_notes.get();
        set_error.set(None);

        spawn_local(async move {
            let Some(animal_id) = animal_id else {
                set_error.set(Some(String::from("no_animal_selected")));
                return;
            };

            match api::add_treatment(
                animal_id,
                api::CreateTreatmentRequest {
                    id: uuid::Uuid::new_v4(),
                    date: Utc::now().to_rfc3339(),
                    treatment_type,
                    medication: Some(String::from("Standardimpfstoff")),
                    dosage: None,
                    veterinarian: Some(String::from("AdminUI")),
                    withdrawal_days: Some(0),
                    notes: if treatment_notes.is_empty() {
                        None
                    } else {
                        Some(treatment_notes)
                    },
                },
            )
            .await
            {
                Ok(_) => {
                    let _ = leptos::prelude::window().location().reload();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">{title}</h1>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <Icon icon=LuPlus width="20" height="20" />
                    {add_animal_btn}
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">{livestock_total.clone()}</div>
                        <div class="stat-value">
                            {move || animals.read().as_ref().map(|result| result.as_ref().map(|page| page.total).unwrap_or(0)).unwrap_or(0)}
                        </div>
                        <div class="stat-desc">{animals_in_system}</div>
                    </div>
                </div>
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">{under_treatment}</div>
                        <div class="stat-value text-warning">"—"</div>
                        <div class="stat-desc">{treatment_from_data}</div>
                    </div>
                </div>
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">{grazing}</div>
                        <div class="stat-value text-success">"—"</div>
                        <div class="stat-desc">{grazing_from_data}</div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title mb-4">{animal_list}</h2>
                    <div class="overflow-x-auto">
                        <table class="table w-full">
                            <thead>
                                <tr>
                                    <th>{id_name}</th>
                                    <th>{species_label.clone()}</th>
                                    <th>{status_label}</th>
                                    <th>{last_action}</th>
                                    <th>{animal_actions}</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || {
                                        animals
                                            .read()
                                            .as_ref()
                                            .map(|result| {
                                                result
                                                    .as_ref()
                                                    .map(|page| page.data.clone())
                                                    .unwrap_or_default()
                                            })
                                            .unwrap_or_default()
                                    }
                                    key=|animal| {
                                        animal
                                            .get("id")
                                            .and_then(|value| value.as_str())
                                            .and_then(|value| uuid::Uuid::parse_str(value).ok())
                                            .unwrap_or_else(|| uuid::Uuid::nil())
                                    }
                                    children=move |animal| {
                                        let animal_id = animal
                                            .get("id")
                                            .and_then(|value| value.as_str())
                                            .and_then(|value| uuid::Uuid::parse_str(value).ok());
                                        let species_label = animal
                                            .get("species")
                                            .and_then(|value| value.as_str())
                                            .map(|value| value.to_string())
                                            .unwrap_or_else(|| String::from("-"));
                                        let status_label = animal
                                            .get("status")
                                            .and_then(|value| value.as_str())
                                            .map(|value| value.to_string())
                                            .unwrap_or_else(|| String::from("-"));
                                        let last_action = animal
                                            .get("treatments")
                                            .and_then(|value| value.as_array())
                                            .and_then(|items| items.last())
                                            .and_then(|value| value.get("treatment_type"))
                                            .and_then(|value| value.as_str())
                                            .map(|value| value.to_string())
                                            .or_else(|| {
                                                animal
                                                    .get("grazing_history")
                                                    .and_then(|value| value.as_array())
                                                    .and_then(|items| items.last())
                                                    .and_then(|value| value.get("notes"))
                                                    .and_then(|value| value.as_str())
                                                    .map(|value| value.to_string())
                                            })
                                            .unwrap_or_else(|| String::from("-"));
                                        let animal_identifier = animal
                                            .get("identifier")
                                            .and_then(|value| value.as_str())
                                            .map(|value| value.to_string())
                                            .unwrap_or_else(|| String::from("-"));
                                        let animal_breed = animal
                                            .get("breed")
                                            .and_then(|value| value.as_str())
                                            .map(|value| value.to_string())
                                            .unwrap_or_else(|| String::from("-"));
                                        view! {
                                            <tr>
                                                <td>
                                                    <div class="font-bold">{animal_identifier}</div>
                                                    <div class="text-sm opacity-50">{animal_breed}</div>
                                                </td>
                                                <td>{species_label}</td>
                                                <td><div class="badge badge-success">{status_label}</div></td>
                                                <td>{last_action}</td>
                                                <td>
                                                    <div class="flex gap-2">
                                                        <button class="btn btn-ghost btn-xs text-info" on:click=move |_| {
                                                            set_selected_animal_id.set(animal_id);
                                                            set_show_treatment_modal.set(true);
                                                        }>{add_treatment.clone()}</button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <Show when=move || show_add_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">{add_animal.clone()}</h3>
                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}
                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><span class="label-text">{identifier_label.clone()}</span></label>
                                <input type="text" class="input input-bordered w-full" prop:value=move || identifier.get() on:input=move |ev| set_identifier.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{breed_label.clone()}</span></label>
                                <input type="text" class="input input-bordered w-full" prop:value=move || breed.get() on:input=move |ev| set_breed.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{species_label.clone()}</span></label>
                                <select class="select select-bordered w-full" prop:value=move || species.get() on:change=move |ev| set_species.set(event_target_value(&ev))>
                                    <option value="cattle">{species_cattle.clone()}</option>
                                    <option value="sheep">{species_sheep.clone()}</option>
                                    <option value="goat">{species_goat.clone()}</option>
                                    <option value="pig">{species_pig.clone()}</option>
                                    <option value="poultry">{species_poultry.clone()}</option>
                                    <option value="horse">{species_horse.clone()}</option>
                                </select>
                            </div>
                        </div>
                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)>{add_cancel_label.clone()}</button>
                            <button class="btn btn-primary" on:click=on_create_animal>{add_save_label.clone()}</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_treatment_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">{record_treatment.clone()}</h3>
                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}
                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">{measure_label.clone()}</span></label>
                            <input type="text" class="input input-bordered w-full" prop:value=move || treatment_type.get() on:input=move |ev| set_treatment_type.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">{note_label.clone()}</span></label>
                            <textarea class="textarea textarea-bordered w-full" prop:value=move || treatment_notes.get() on:input=move |ev| set_treatment_notes.set(event_target_value(&ev))></textarea>
                        </div>
                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_treatment_modal.set(false)>{treatment_cancel_label.clone()}</button>
                            <button class="btn btn-primary" on:click=on_create_treatment>{treatment_save_label.clone()}</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
