//! Livestock management — `/livestock/animals`.
//!
//! The page this replaces collected a label and a count and discarded both:
//!
//! ```text
//! spawn_local(async move { let _ = (label.get(), count.get()); })
//! ```
//!
//! The routes underneath it — list, create, update, delete, treatments, grazing —
//! had no caller at all. It now lists real animals and creates them through the
//! API.
//!
//! `CreateAnimalDto` requires `livestock_type` and `status` with no default on the
//! Rust side, so they are sent explicitly. Leaving them out produced a 422 the
//! page could not have shown.

use crate::api;
use icondata::{LuPlus, LuTrash2};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// The species `AnimalSpecies` accepts, with the wire value the domain uses.
///
/// `AnimalSpecies::as_str` returns the lower-case form for every named variant,
/// and `FromStr` parses that back, so the request sends the lower-case value. The
/// `Other(String)` variant carries caller text and is not offered here.
const SPECIES: [(&str, &str); 9] = [
    ("Goat", "goat"),
    ("Sheep", "sheep"),
    ("Cattle", "cattle"),
    ("Pig", "pig"),
    ("Horse", "horse"),
    ("Chicken", "chicken"),
    ("Duck", "duck"),
    ("Turkey", "turkey"),
    ("Goose", "goose"),
];

/// The statuses `AnimalStatus` accepts. These are capitalised on the wire.
const STATUSES: [&str; 4] = ["Active", "Inactive", "Sold", "Deceased"];

#[component]
pub fn LivestockManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let animals = LocalResource::new(|| async move { api::fetch_animals().await.ok() });
    let (reload, set_reload) = signal(0u32);

    let (identifier, set_identifier) = signal(String::new());
    let (species, set_species) = signal(String::from("Goat"));
    let (breed, set_breed) = signal(String::new());
    let (status, set_status) = signal(String::from("Active"));
    let (error, set_error) = signal(None::<String>);

    let (editing, set_editing) = signal(None::<uuid::Uuid>);
    let (edit_breed, set_edit_breed) = signal(String::new());
    let (edit_status, set_edit_status) = signal(String::from("Active"));

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let ident = identifier.get().trim().to_string();
        if ident.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }

        let breed_text = breed.get().trim().to_string();
        let req = api::CreateAnimalRequest {
            // `as_str` lower-cases the named variants; the API parses the
            // lower-case form back into the enum.
            species: species.get().to_lowercase(),
            breed: if breed_text.is_empty() {
                None
            } else {
                Some(breed_text)
            },
            identifier: ident,
            birth_date: None,
            gender: None,
            livestock_type: "herd".to_string(),
            status: status.get(),
            current_site_id: None,
            mother_id: None,
            father_id: None,
            plot_id: None,
        };

        spawn_local(async move {
            match api::create_animal(req).await {
                Ok(_) => {
                    set_identifier.set(String::new());
                    set_breed.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_update = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let Some(id) = editing.get() else {
            return;
        };
        let text = edit_breed.get().trim().to_string();
        let req = api::UpdateAnimalRequest {
            species: None,
            breed: if text.is_empty() { None } else { Some(text) },
            identifier: None,
            birth_date: None,
            gender: None,
            status: Some(edit_status.get()),
            current_site_id: None,
            group_id: None,
            weight_kg: None,
            livestock_type: None,
            plot_id: None,
        };
        spawn_local(async move {
            match api::update_animal(id, req).await {
                Ok(_) => {
                    set_editing.set(None);
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_start_edit = move |animal: api::AnimalDto| {
        set_editing.set(Some(animal.id));
        set_edit_breed.set(animal.breed.unwrap_or_default());
        set_edit_status.set(animal.status);
    };

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            if let Err(e) = api::delete_animal(id).await {
                set_error.set(Some(format!("{}: {e}", crate::t!(t, "delete_failed")())));
            } else {
                set_reload.update(|n| *n += 1);
            }
        });
    };

    view! {
        <div class="p-6">
            <div class="flex justify-between items-center mb-4">
                <h1 class="text-2xl font-bold">{crate::t!(t, "nav_livestock")}</h1>
            </div>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <form class="card bg-base-100 shadow p-4 mb-6" on:submit=on_create>
                <h2 class="font-semibold mb-3">{crate::t!(t, "btn_add")}</h2>
                <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "animal_identifier")}</span>
                        <input
                            class="input input-bordered"
                            prop:value=move || identifier.get()
                            on:input=move |ev| set_identifier.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "animal_species")}</span>
                        <select
                            class="select select-bordered"
                            prop:value=move || species.get()
                            on:change=move |ev| set_species.set(event_target_value(&ev))
                        >
                            {SPECIES
                                .iter()
                                .map(|(value, label_key)| {
                                    view! {
                                        <option value=*value>{crate::t!(t, *label_key)()}</option>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </label>
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "animal_breed")}</span>
                        <input
                            class="input input-bordered"
                            prop:value=move || breed.get()
                            on:input=move |ev| set_breed.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "animal_status")}</span>
                        <select
                            class="select select-bordered"
                            prop:value=move || status.get()
                            on:change=move |ev| set_status.set(event_target_value(&ev))
                        >
                            {STATUSES
                                .iter()
                                .map(|value| {
                                    let v = *value;
                                    view! { <option value=v>{v}</option> }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </label>
                </div>
                <button type="submit" class="btn btn-primary mt-3">
                    <Icon icon=LuPlus width="16" height="16" />
                    {crate::t!(t, "btn_add")}
                </button>
            </form>

            {move || {
                let current = editing.get();
                if current.is_none() {
                    return ().into_any();
                }
                view! {
                    <form
                        class="card bg-base-100 shadow p-4 mb-6 border-l-4 border-primary"
                        on:submit=on_update
                    >
                        <h2 class="font-semibold mb-3">
                            {crate::t!(t, "edit_label")}
                        </h2>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                            <label class="form-control">
                                <span class="label-text">{crate::t!(t, "animal_breed")}</span>
                                <input
                                    class="input input-bordered"
                                    prop:value=move || edit_breed.get()
                                    on:input=move |ev| set_edit_breed.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="form-control">
                                <span class="label-text">{crate::t!(t, "animal_status")}</span>
                                <select
                                    class="select select-bordered"
                                    prop:value=move || edit_status.get()
                                    on:change=move |ev| set_edit_status.set(event_target_value(&ev))
                                >
                                    {STATUSES
                                        .iter()
                                        .map(|value| {
                                            let v = *value;
                                            view! { <option value=v>{v}</option> }
                                        })
                                        .collect::<Vec<_>>()}
                                </select>
                            </label>
                        </div>
                        <div class="flex gap-2 mt-3">
                            <button type="submit" class="btn btn-primary">
                                {crate::t!(t, "btn_save")}
                            </button>
                            <button
                                type="button"
                                class="btn btn-ghost"
                                on:click=move |_| set_editing.set(None)
                            >
                                {crate::t!(t, "btn_cancel")}
                            </button>
                        </div>
                    </form>
                }
                    .into_any()
            }}

            {move || {
                let _ = reload.get();
                let loaded = animals.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "animal_identifier")}</th>
                                            <th>{crate::t!(t, "animal_species")}</th>
                                            <th>{crate::t!(t, "animal_breed")}</th>
                                            <th>{crate::t!(t, "animal_status")}</th>
                                            <th>{crate::t!(t, "animal_birth_date")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|animal: &api::AnimalDto| animal.id
                                            children=move |animal: api::AnimalDto| {
                                                let id = animal.id;
                                                let animal_for_edit = animal.clone();
                                                view! {
                                                    <tr>
                                                        <td>{animal.identifier.clone()}</td>
                                                        <td>{animal.species.clone()}</td>
                                                        <td>
                                                            {animal
                                                                .breed
                                                                .clone()
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>{animal.status.clone()}</td>
                                                        <td>
                                                            {animal
                                                                .birth_date
                                                                .clone()
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td class="flex gap-1">
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| on_start_edit(animal_for_edit.clone())
                                                            >
                                                                {crate::t!(t, "edit_label")}
                                                            </button>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| on_delete(id)
                                                            >
                                                                <Icon icon=LuTrash2 width="14" height="14" />
                                                                {crate::t!(t, "btn_delete")}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            </div>
                        }
                            .into_any()
                    }
                    Some(_) => {
                        view! {
                            <div class="alert">{crate::t!(t, "no_records")}</div>
                        }
                            .into_any()
                    }
                    None => {
                        view! {
                            <div class="alert">{crate::t!(t, "loading")}</div>
                        }
                            .into_any()
                    }
                }
            }}
        </div>
    }
}
