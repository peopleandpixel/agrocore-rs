use crate::api;
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn EquipmentManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let equipment = LocalResource::new(|| async move { api::fetch_equipment().await.ok() });

    let (show_add_modal, set_show_add_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (label, set_label) = signal(String::new());
    let (equipment_type, set_equipment_type) = signal(String::from("tractor"));

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            if api::delete_equipment(id).await.is_ok() {
                let _ = window().location().reload();
            }
        });
    };

    let on_create = move |_| {
        let label_val = label.get();
        let equipment_type_val = equipment_type.get();
        set_error.set(None);

        if label_val.trim().is_empty() {
            set_error.set(Some(t("validation_required")));
            return;
        }

        spawn_local(async move {
            match api::create_equipment(api::CreateEquipmentRequest {
                label: label_val,
                code: None,
                equipment_type: equipment_type_val,
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
                <div>
                    <h1 class="text-3xl font-bold">{crate::t!(t, "nav_equipment")}</h1>
                </div>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <Icon icon=LuPlus width="20" height="20" />
                    {crate::t!(t, "add_equipment_btn")}
                </button>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>{crate::t!(t, "name")}</th>
                                <th>{crate::t!(t, "type")}</th>
                                <th>{crate::t!(t, "status")}</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
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
                                key=|e| e.id
                                children=move |e| {
                                    let id = e.id;
                                    let e_label = e.label.clone();
                                    let e_type = e.equipment_type.clone();
                                    let is_active = !e.in_usage; // Fallback since is_active is missing

                                    view! {
                                        <tr>
                                            <td>{e_label}</td>
                                            <td>{move || t(&format!("equipment_type_{}", e_type))}</td>
                                            <td>{if is_active { t("active") } else { t("inactive") }}</td>
                                            <td>
                                                <div class="flex gap-2 justify-end">
                                                    <button class="btn btn-sm btn-ghost text-error" on:click=move |_| on_delete(id)>
                                                        <Icon icon=LuTrash2 width="16" height="16" />
                                                    </button>
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

            <Show when=move || show_add_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">{crate::t!(t, "add_equipment_btn")}</h3>

                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">{crate::t!(t, "name")}</span></label>
                            <input type="text" class="input input-bordered w-full" on:input=move |ev| set_label.set(event_target_value(&ev)) />
                        </div>

                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">{crate::t!(t, "type")}</span></label>
                            <select class="select select-bordered w-full" on:change=move |ev| set_equipment_type.set(event_target_value(&ev))>
                                <option value="tractor">{crate::t!(t, "equipment_type_tractor")}</option>
                                <option value="plow">{crate::t!(t, "equipment_type_plow")}</option>
                                <option value="harvester">{crate::t!(t, "equipment_type_harvester")}</option>
                                <option value="sprayer">{crate::t!(t, "equipment_type_sprayer")}</option>
                                <option value="seeder">{crate::t!(t, "equipment_type_seeder")}</option>
                                <option value="trailer">{crate::t!(t, "equipment_type_trailer")}</option>
                            </select>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)>{crate::t!(t, "cancel")}</button>
                            <button class="btn btn-primary" on:click=on_create>{crate::t!(t, "save")}</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
