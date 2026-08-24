use crate::api;
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn EquipmentManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    // Search & filter signals
    let (search_query, set_search_query) = signal(String::new());
    let (selected_type, set_selected_type) = signal(String::new());
    let (in_usage_only, set_in_usage_only) = signal(false);
    let (needs_maintenance, set_needs_maintenance) = signal(false);

    // Derived filter values — re-fetch when any filter changes
    let equipment = LocalResource::new(move || {
        let filter = api::EquipmentFilter {
            search: if search_query.get().is_empty() {
                None
            } else {
                Some(search_query.get())
            },
            equipment_type: if selected_type.get().is_empty() {
                None
            } else {
                Some(selected_type.get())
            },
            in_usage: if in_usage_only.get() {
                Some(true)
            } else {
                None
            },
            needs_maintenance: if needs_maintenance.get() {
                Some(true)
            } else {
                None
            },
        };
        async move { api::fetch_equipment_filtered(&filter).await.ok() }
    });

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

    let clear_filters = move |_| {
        set_search_query.set(String::new());
        set_selected_type.set(String::new());
        set_in_usage_only.set(false);
        set_needs_maintenance.set(false);
    };

    // Maintenance modal state
    let (show_maint_modal, set_show_maint_modal) = signal(false);
    let (maint_equip_id, set_maint_equip_id) = signal(uuid::Uuid::nil());
    let (maint_equip_label, set_maint_equip_label) = signal(String::new());
    let (maint_hours, set_maint_hours) = signal(String::new());
    let (maint_note, set_maint_note) = signal(String::new());

    let on_record_maintenance = move |id: uuid::Uuid, label: String| {
        set_maint_equip_id.set(id);
        set_maint_equip_label.set(label);
        set_maint_hours.set(String::new());
        set_maint_note.set(String::new());
        set_show_maint_modal.set(true);
    };

    let submit_maintenance = move |_| {
        let id = maint_equip_id.get();
        let hours_str = maint_hours.get();
        let note = maint_note.get();
        set_show_maint_modal.set(false);
        spawn_local(async move {
            let hours: f64 = hours_str.trim().parse().unwrap_or(0.0);
            let note_opt = if note.trim().is_empty() {
                None
            } else {
                Some(note.as_str())
            };
            match api::record_maintenance(id, hours, note_opt).await {
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

            {/* Search & Filter Bar */}
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title text-lg">{crate::t!(t, "search_filter")}</h2>
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">{crate::t!(t, "search")}</span>
                            </label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                prop:value=move || search_query.get()
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
                                placeholder="Name oder Code..."
                            />
                        </div>

                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">{crate::t!(t, "type")}</span>
                            </label>
                            <select
                                class="select select-bordered w-full"
                                prop:value=move || selected_type.get()
                                on:change=move |ev| set_selected_type.set(event_target_value(&ev))
                            >
                                <option value="">{crate::t!(t, "all_types")}</option>
                                <option value="tractor">{crate::t!(t, "equipment_type_tractor")}</option>
                                <option value="plow">{crate::t!(t, "equipment_type_plow")}</option>
                                <option value="harvester">{crate::t!(t, "equipment_type_harvester")}</option>
                                <option value="sprayer">{crate::t!(t, "equipment_type_sprayer")}</option>
                                <option value="seeder">{crate::t!(t, "equipment_type_seeder")}</option>
                                <option value="trailer">{crate::t!(t, "equipment_type_trailer")}</option>
                            </select>
                        </div>

                        <div class="form-control justify-end">
                            <label class="label cursor-pointer justify-start">
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-sm"
                                    checked=move || in_usage_only.get()
                                    on:change=move |ev| set_in_usage_only.set(event_target_checked(&ev))
                                />
                                <span class="label-text ml-2">{crate::t!(t, "in_usage")}</span>
                            </label>
                        </div>

                        <div class="form-control justify-end">
                            <label class="label cursor-pointer justify-start">
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-sm"
                                    checked=move || needs_maintenance.get()
                                    on:change=move |ev| set_needs_maintenance.set(event_target_checked(&ev))
                                />
                                <span class="label-text ml-2">{crate::t!(t, "needs_maintenance")}</span>
                            </label>
                        </div>
                    </div>

                    <div class="card-actions justify-end mt-4">
                        <button class="btn btn-sm" on:click=clear_filters>
                            <Icon icon=LuRefreshCw width="16" height="16" />
                            {crate::t!(t, "clear_filters")}
                        </button>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>{crate::t!(t, "name")}</th>
                                <th>{crate::t!(t, "type")}</th>
                                <th>{crate::t!(t, "status")}</th>
                                <th>{crate::t!(t, "maintenance_hours")}</th>
                                <th>{crate::t!(t, "next_maintenance")}</th>
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
                                    let usage_hours = e.last_maintenance_hours.unwrap_or(0.0);
                                    let next_maint = e.next_maintenance_date.clone().unwrap_or_default();
                                    let has_intervals = !e.maintenance_intervals.clone().unwrap_or_default().is_empty();

                                    view! {
                                        <tr>
                                            <td>{e_label.clone()}</td>
                                            <td>{move || t(&format!("equipment_type_{}", e_type))}</td>
                                            <td>{if is_active { t("active") } else { t("inactive") }}</td>
                                            <td>{format!("{:.1}", usage_hours)}</td>
                                            <td>
                                                {move || {
                                                    if next_maint.is_empty() {
                                                        t("no_deadline")
                                                    } else {
                                                        next_maint.clone()
                                                    }
                                                }}
                                                {if has_intervals {
                                                    view! { <span class="badge badge-warning badge-sm">!</span> }.into_any()
                                                } else {
                                                    view! { <span class="badge badge-ghost badge-sm">-</span> }.into_any()
                                                }}
                                            </td>
                                            <td>
                                                <div class="flex gap-2 justify-end">
                                                    <button class="btn btn-sm btn-ghost" on:click=move |_| on_record_maintenance(id, e_label.clone())>
                                                        <Icon icon=LuWrench width="16" height="16" />
                                                        <span class="hidden sm:inline">Wartung</span>
                                                    </button>
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

            <Show when=move || show_maint_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">Wartung für: {move || maint_equip_label.get()}</h3>

                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">Betriebsstunden</span></label>
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                prop:value=move || maint_hours.get()
                                on:input=move |ev| set_maint_hours.set(event_target_value(&ev))
                                placeholder="Aktuelle Betriebsstunden..."
                            />
                        </div>

                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">Notiz</span></label>
                            <textarea
                                class="textarea textarea-bordered w-full"
                                prop:value=move || maint_note.get()
                                on:input=move |ev| set_maint_note.set(event_target_value(&ev))
                                rows=3
                                placeholder="Wartungsnotiz (optional)..."
                            ></textarea>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_maint_modal.set(false)>{crate::t!(t, "cancel")}</button>
                            <button class="btn btn-primary" on:click=submit_maintenance>Wartung speichern</button>
                        </div>
                    </div>
                </div>
            </Show>

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
