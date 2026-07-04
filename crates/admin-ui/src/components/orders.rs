use crate::api;
use crate::components::form::RequiredLabel;
use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

fn submit_order(
    label: String,
    order_type: String,
    site_id: String,
    set_error: WriteSignal<Option<String>>,
) {
    match uuid::Uuid::parse_str(&site_id) {
        Ok(site_uuid) => {
            spawn_local(async move {
                match api::create_order(api::CreateOrderRequest {
                    label,
                    order_type,
                    site_ids: vec![site_uuid],
                    assigned_worker_ids: None,
                    planned_date: None,
                    deadline_date: None,
                    recurrence: None,
                })
                .await
                {
                    Ok(_) => {
                        let _ = window().location().reload();
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
        Err(_) => set_error.set(Some(String::from("Bitte eine gültige Flächen-ID wählen."))),
    }
}

#[component]
pub fn OrderList() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let order_management = i18n.t(lang.get().as_str(), "order_management");
    let new_order_btn = i18n.t(lang.get().as_str(), "new_order_btn");
    let order_type_label = i18n.t(lang.get().as_str(), "order_type");
    let description_label = i18n.t(lang.get().as_str(), "description");
    let sites_label = i18n.t(lang.get().as_str(), "sites");
    let actions_label = i18n.t(lang.get().as_str(), "actions");
    let status_label = i18n.t(lang.get().as_str(), "status");
    let _new_order_form = i18n.t(lang.get().as_str(), "new_order_form");
    let _cancel_label = i18n.t(lang.get().as_str(), "cancel");
    let _save_label = i18n.t(lang.get().as_str(), "save");
    let _site_label = i18n.t(lang.get().as_str(), "site");
    let _choose_site_label = i18n.t(lang.get().as_str(), "choose_site");
    let _task_protection = i18n.t(lang.get().as_str(), "task_protection");
    let _task_harvest = i18n.t(lang.get().as_str(), "task_harvest");
    let _required_error = i18n.t(lang.get().as_str(), "validation_required");
    let order_type_plant_protection: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_plant_protection")
            .into_boxed_str(),
    );
    let order_type_fertilization: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_fertilization")
            .into_boxed_str(),
    );
    let order_type_pruning: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_pruning")
            .into_boxed_str(),
    );
    let order_type_harvest: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_harvest")
            .into_boxed_str(),
    );
    let order_type_soil_work: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_soil_work")
            .into_boxed_str(),
    );
    let order_type_irrigation: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_irrigation")
            .into_boxed_str(),
    );
    let order_type_monitoring: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_monitoring")
            .into_boxed_str(),
    );
    let order_type_livestock_feeding: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_livestock_feeding")
            .into_boxed_str(),
    );
    let order_type_livestock_watering: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_livestock_watering")
            .into_boxed_str(),
    );
    let order_type_livestock_relocation: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_livestock_relocation")
            .into_boxed_str(),
    );
    let order_type_livestock_health_check: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_livestock_health_check")
            .into_boxed_str(),
    );
    let order_type_barn_cleaning: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_barn_cleaning")
            .into_boxed_str(),
    );
    let order_type_egg_collection: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_egg_collection")
            .into_boxed_str(),
    );
    let order_type_shearing: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_shearing")
            .into_boxed_str(),
    );
    let order_type_milking: &'static str = Box::leak(
        i18n.t(lang.get().as_str(), "order_type_milking")
            .into_boxed_str(),
    );

    let orders = LocalResource::new(|| async move { api::fetch_orders().await.ok() });
    let sites = LocalResource::new(|| async move { api::fetch_sites().await.ok() });
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (label, set_label) = signal(String::new());
    let (order_type, set_order_type) = signal(String::from("plant_protection"));
    let (site_id, set_site_id) = signal(String::new());
    let order_type_text = move |value: &str| match value {
        "plant_protection" => i18n.t(lang.get().as_str(), "order_type_plant_protection"),
        "fertilization" => i18n.t(lang.get().as_str(), "order_type_fertilization"),
        "pruning" => i18n.t(lang.get().as_str(), "order_type_pruning"),
        "harvest" => i18n.t(lang.get().as_str(), "order_type_harvest"),
        "soil_work" => i18n.t(lang.get().as_str(), "order_type_soil_work"),
        "irrigation" => i18n.t(lang.get().as_str(), "order_type_irrigation"),
        "monitoring" => i18n.t(lang.get().as_str(), "order_type_monitoring"),
        "livestock_feeding" => i18n.t(lang.get().as_str(), "order_type_livestock_feeding"),
        "livestock_watering" => i18n.t(lang.get().as_str(), "order_type_livestock_watering"),
        "livestock_relocation" => i18n.t(lang.get().as_str(), "order_type_livestock_relocation"),
        "livestock_health_check" => {
            i18n.t(lang.get().as_str(), "order_type_livestock_health_check")
        }
        "barn_cleaning" => i18n.t(lang.get().as_str(), "order_type_barn_cleaning"),
        "egg_collection" => i18n.t(lang.get().as_str(), "order_type_egg_collection"),
        "shearing" => i18n.t(lang.get().as_str(), "order_type_shearing"),
        "milking" => i18n.t(lang.get().as_str(), "order_type_milking"),
        _ => value.to_string(),
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">{order_management}</h1>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <Icon icon=LuPlus width="20" height="20" />
                    {new_order_btn}
                </button>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table table-hover">
                        <thead>
                            <tr>
                                <th>{order_type_label}</th>
                                <th>{description_label}</th>
                                <th>{sites_label}</th>
                                <th>{actions_label}</th>
                                <th>{status_label}</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || {
                                    orders
                                        .read()
                                        .as_ref()
                                        .map(|o| {
                                            o.as_ref()
                                                .map(|page| page.data.clone())
                                                .unwrap_or_default()
                                        })
                                        .unwrap_or_default()
                                }
                                key=|order| order.id
                                children=move |order| view! {
                                    <tr>
                                        <td>{order_type_text(order.order_type.as_str())}</td>
                                        <td>{order.label}</td>
                                        <td>{order.site_ids.len()}</td>
                                        <td>{order.assigned_worker_ids.len()}</td>
                                        <td><div class="badge badge-outline">{order.status}</div></td>
                                    </tr>
                                }
                            />
                        </tbody>
                    </table>
                </div>
            </div>

            <Show when=move || show_add_modal.get()>
                {move || {
                    view! {
                        <div class="modal modal-open">
                            <div class="modal-box">
                                <h3 class="font-bold text-lg">"Neuer Auftrag"</h3>

                                {move || error.get().map(|err| view! {
                                    <div class="alert alert-error mt-4">
                                        <span>{err}</span>
                                    </div>
                                })}

                                <div class="form-control w-full mt-4">
                                    <label class="label">
                                        <RequiredLabel required=true>
                                            "Beschreibung"
                                        </RequiredLabel>
                                    </label>
                                    <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_label.set(event_target_value(&ev)) />
                                </div>

                                <div class="grid grid-cols-2 gap-4 mt-4">
                                    <div class="form-control">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                "Auftragstyp"
                                            </RequiredLabel>
                                        </label>
                                        <select class="select select-bordered w-full" required on:change=move |ev| set_order_type.set(event_target_value(&ev))>
                                            <option value="plant_protection">{order_type_plant_protection}</option>
                                            <option value="fertilization">{order_type_fertilization}</option>
                                            <option value="pruning">{order_type_pruning}</option>
                                            <option value="harvest">{order_type_harvest}</option>
                                            <option value="soil_work">{order_type_soil_work}</option>
                                            <option value="irrigation">{order_type_irrigation}</option>
                                            <option value="monitoring">{order_type_monitoring}</option>
                                            <option value="livestock_feeding">{order_type_livestock_feeding}</option>
                                            <option value="livestock_watering">{order_type_livestock_watering}</option>
                                            <option value="livestock_relocation">{order_type_livestock_relocation}</option>
                                            <option value="livestock_health_check">{order_type_livestock_health_check}</option>
                                            <option value="barn_cleaning">{order_type_barn_cleaning}</option>
                                            <option value="egg_collection">{order_type_egg_collection}</option>
                                            <option value="shearing">{order_type_shearing}</option>
                                            <option value="milking">{order_type_milking}</option>
                                        </select>
                                    </div>
                                    <div class="form-control">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                "Fläche"
                                            </RequiredLabel>
                                        </label>
                                        <select class="select select-bordered w-full" required on:change=move |ev| set_site_id.set(event_target_value(&ev))>
                                            <option value="">"Bitte wählen"</option>
                                            <For
                                                each=move || {
                                                    sites
                                                        .read()
                                                        .as_ref()
                                                        .map(|s| {
                                                            s.as_ref()
                                                                .map(|page| page.data.clone())
                                                                .unwrap_or_default()
                                                        })
                                                        .unwrap_or_default()
                                                }
                                                key=|site| site.id
                                                children=move |site| view! {
                                                    <option value=site.id.to_string()>{site.label}</option>
                                                }
                                            />
                                        </select>
                                    </div>
                                </div>

                                <div class="modal-action">
                                    <button class="btn" on:click=move |_| set_show_add_modal.set(false)>"Abbrechen"</button>
                                    <button class="btn btn-primary" on:click=move |_| {
                                        let label_value = label.get();
                                        let order_type_value = order_type.get();
                                        let site_id_value = site_id.get();
                                        set_error.set(None);

                                        if label_value.trim().is_empty() || site_id_value.trim().is_empty() {
                                            set_error.set(Some(String::from("Bitte alle Pflichtfelder ausfüllen.")));
                                            return;
                                        }

                                        submit_order(label_value, order_type_value, site_id_value, set_error);
                                    }>"Speichern"</button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }}
            </Show>
        </div>
    }
}
