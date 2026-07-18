use crate::api;
use crate::components::form::RequiredLabel;
use crate::components::toast::{ToastContext, ToastType};
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

fn submit_order(
    label: String,
    order_type: String,
    site_id: String,
    set_error: WriteSignal<Option<String>>,
    toast_context: ToastContext,
    i18n: crate::i18n::I18n,
    lang: String,
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
                        toast_context
                            .add_toast
                            .run((i18n.t(&lang, "order_created"), ToastType::Success));
                        let _ = window().location().reload();
                    }
                    Err(e) => {
                        toast_context.add_toast.run((e.clone(), ToastType::Error));
                        set_error.set(Some(e));
                    }
                }
            });
        }
        Err(_) => {
            let msg = String::from("Please choose a valid site ID.");
            toast_context
                .add_toast
                .run((msg.clone(), ToastType::Warning));
            set_error.set(Some(msg));
        }
    }
}

#[component]
pub fn OrderList() -> impl IntoView {
    let t = crate::i18n::use_i18n();
    let i18n = use_context::<crate::i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<crate::i18n::Language>>().expect("lang signal");
    let toast_context = use_context::<ToastContext>().expect("ToastContext not provided");

    let orders = LocalResource::new(|| async move { api::fetch_orders().await.ok() });
    let sites = LocalResource::new(|| async move { api::fetch_sites().await.ok() });
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (label, set_label) = signal(String::new());
    let (order_type, set_order_type) = signal(String::from("plant_protection"));
    let (site_id, set_site_id) = signal(String::new());

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            match api::delete_order(id).await {
                Ok(_) => {
                    toast_context
                        .add_toast
                        .run((t("order_deleted").to_string(), ToastType::Success));
                    let _ = window().location().reload();
                }
                Err(e) => {
                    toast_context.add_toast.run((e, ToastType::Error));
                }
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">{crate::t!(t, "order_management")}</h1>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <Icon icon=LuPlus width="20" height="20" />
                    {crate::t!(t, "new_order_btn")}
                </button>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table table-hover">
                        <thead>
                            <tr>
                                <th>{crate::t!(t, "order_type")}</th>
                                <th>{crate::t!(t, "description")}</th>
                                <th>{crate::t!(t, "sites")}</th>
                                <th>{crate::t!(t, "actions")}</th>
                                <th>{crate::t!(t, "status")}</th>
                                <th></th>
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
                                children=move |order| {
                                    let order_id = order.id;
                                    let order_type_val = order.order_type.clone();
                                    let order_label = order.label.clone();
                                    let site_count = order.site_ids.len();
                                    let worker_count = order.assigned_worker_ids.len();
                                    let status_val = order.status.clone();

                                    view! {
                                        <tr>
                                            <td>{move || t(&format!("order_type_{}", order_type_val))}</td>
                                            <td>{order_label}</td>
                                            <td>{site_count}</td>
                                            <td>{worker_count}</td>
                                            <td><div class="badge badge-outline">{status_val}</div></td>
                                            <td>
                                                <button class="btn btn-sm btn-ghost text-error" on:click=move |_| on_delete(order_id)>
                                                    <Icon icon=LuTrash2 width="16" height="16" />
                                                </button>
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
                        <h3 class="font-bold text-lg">{crate::t!(t, "new_order_form")}</h3>

                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="form-control w-full mt-4">
                            <label class="label">
                                <RequiredLabel required=true>
                                    {crate::t!(t, "description")}
                                </RequiredLabel>
                            </label>
                            <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_label.set(event_target_value(&ev)) />
                        </div>

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label">
                                    <RequiredLabel required=true>
                                        {crate::t!(t, "order_type")}
                                    </RequiredLabel>
                                </label>
                                <select class="select select-bordered w-full" required on:change=move |ev| set_order_type.set(event_target_value(&ev))>
                                    <option value="plant_protection">{crate::t!(t, "order_type_plant_protection")}</option>
                                    <option value="fertilization">{crate::t!(t, "order_type_fertilization")}</option>
                                    <option value="pruning">{crate::t!(t, "order_type_pruning")}</option>
                                    <option value="harvest">{crate::t!(t, "order_type_harvest")}</option>
                                    <option value="soil_work">{crate::t!(t, "order_type_soil_work")}</option>
                                    <option value="irrigation">{crate::t!(t, "order_type_irrigation")}</option>
                                    <option value="monitoring">{crate::t!(t, "order_type_monitoring")}</option>
                                    <option value="livestock_feeding">{crate::t!(t, "order_type_livestock_feeding")}</option>
                                    <option value="livestock_watering">{crate::t!(t, "order_type_livestock_watering")}</option>
                                    <option value="livestock_relocation">{crate::t!(t, "order_type_livestock_relocation")}</option>
                                    <option value="livestock_health_check">{crate::t!(t, "order_type_livestock_health_check")}</option>
                                    <option value="barn_cleaning">{crate::t!(t, "order_type_barn_cleaning")}</option>
                                    <option value="egg_collection">{crate::t!(t, "order_type_egg_collection")}</option>
                                    <option value="shearing">{crate::t!(t, "order_type_shearing")}</option>
                                    <option value="milking">{crate::t!(t, "order_type_milking")}</option>
                                </select>
                            </div>
                            <div class="form-control">
                                <label class="label">
                                    <RequiredLabel required=true>
                                        {crate::t!(t, "site")}
                                    </RequiredLabel>
                                </label>
                                <select class="select select-bordered w-full" required on:change=move |ev| set_site_id.set(event_target_value(&ev))>
                                    <option value="">{crate::t!(t, "choose_site")}</option>
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
                                        children=move |site| {
                                            let site_id_val = site.id.to_string();
                                            let site_label_val = site.label.clone();
                                            view! {
                                                <option value=site_id_val>{site_label_val}</option>
                                            }
                                        }
                                    />
                                </select>
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)>{crate::t!(t, "cancel")}</button>
                            <button class="btn btn-primary" on:click=move |_| {
                                    let label_value = label.get();
                                    let order_type_value = order_type.get();
                                    let site_id_value = site_id.get();
                                    set_error.set(None);

                                    if label_value.trim().is_empty() || site_id_value.trim().is_empty() {
                                        toast_context.add_toast.run((t("validation_required").to_string(), ToastType::Warning));
                                        return;
                                    }

                                    submit_order(label_value, order_type_value, site_id_value, set_error, toast_context, i18n, lang.get().as_str().to_string());
                                }
                            >
                                {crate::t!(t, "save")}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
