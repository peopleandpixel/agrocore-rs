use crate::api;
use crate::components::map::FieldPolygonEditor;
use crate::components::toast::{ToastContext, ToastType};
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn SiteManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();
    let toast_context = use_context::<ToastContext>().expect("ToastContext not provided");

    let sites = LocalResource::new(|| async move {
        api::fetch_sites().await.ok() 
    });

    let (show_add_modal, set_show_add_modal) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (label, set_label) = signal(String::new());
    let (site_type, set_site_type) = signal(String::from("field"));
    let (boundary, set_boundary) = signal(Option::<Vec<api::GeoPoint>>::None);
    let (area, set_area) = signal(0.0f64);
    let (center, set_center) = signal(Option::<api::GeoPoint>::None);

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            match api::delete_site(id).await {
                Ok(_) => {
                    toast_context.add_toast.run((t("site_deleted").to_string(), ToastType::Success));
                    let _ = window().location().reload();
                }
                Err(e) => {
                    toast_context.add_toast.run((e, ToastType::Error));
                }
            }
        });
    };

    let on_create = move |_| {
        let label_val = label.get();
        let site_type_val = site_type.get();
        let boundary_val = boundary.get();
        let area_val = area.get();
        let center_val = center.get();
        set_error.set(None);

        if label_val.trim().is_empty() {
            toast_context.add_toast.run((t("validation_site_label").to_string(), ToastType::Warning));
            return;
        }

        if boundary_val.is_none() {
            toast_context.add_toast.run((t("validation_draw_polygon").to_string(), ToastType::Warning));
            return;
        }

        spawn_local(async move {
            match api::create_site(api::CreateSiteRequest {
                label: label_val,
                site_type: site_type_val,
                crop_type: String::from("grape"),
                variety: None,
                area: area_val * 10000.0, // Convert ha to m2
                gross_area: None,
                center: center_val,
                boundary: boundary_val,
            })
                .await
            {
                Ok(_) => {
                    toast_context.add_toast.run((t("site_created").to_string(), ToastType::Success));
                    let _ = window().location().reload();
                }
                Err(e) => {
                    toast_context.add_toast.run((e.clone(), ToastType::Error));
                    set_error.set(Some(e));
                }
            }
        });
    };

    let on_map_change = move |b: Option<Vec<api::GeoPoint>>, a: Option<f64>, c: Option<api::GeoPoint>| {
        set_boundary.set(b);
        set_area.set(a.unwrap_or(0.0));
        set_center.set(c);
    };

    let sites_view = move || {
        sites
            .read()
            .as_ref()
            .and_then(|s| s.as_ref().map(|page| page.data.clone()))
            .unwrap_or_default()
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">{move || t("nav_sites").to_string()}</h1>
                    <p class="text-base-content/60">{move || t("manage_sites_desc").to_string()}</p>
                </div>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <Icon icon=LuPlus width="20" height="20" />
                    {move || t("add_site_btn").to_string()}
                </button>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>{move || t("name").to_string()}</th>
                                <th>{move || t("type").to_string()}</th>
                                <th>{move || t("area").to_string()}</th>
                                <th>{move || t("status").to_string()}</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                        <For
                            each=sites_view
                            key=|site| site.id
                            children=move |site| {
                                let site_id = site.id;
                                let site_label = site.label.clone();
                                let site_type_val = site.site_type.clone();
                                let area_val = format!("{:.2} ha", site.area / 10000.0);
                                let is_active = site.is_active;

                                view! {
                                    <tr>
                                        <td>{site_label}</td>
                                        <td>
                                            {
                                                let st = site_type_val.clone();
                                                move || t(&format!("site_type_{}", st)).to_string()
                                            }
                                        </td>
                                        <td>{area_val}</td>
                                        <td>
                                            {
                                                if is_active {
                                                    t("active").to_string()
                                                } else {
                                                    t("inactive").to_string()
                                                }
                                            }
                                        </td>
                                        <td>
                                            <button
                                                class="btn btn-ghost btn-xs text-error"
                                                on:click=move |_| on_delete(site_id)
                                            >
                                                <Icon icon=LuTrash width="16" height="16" />
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

            <Show
                when=move || show_add_modal.get()
                fallback=|| ()
            >
                <div class="modal modal-open">
                    <div class="modal-box max-w-4xl">
                        <h3 class="font-bold text-lg">{move || t("add_new_site").to_string()}</h3>

                        <Show
                            when=move || error.get().is_some()
                            fallback=|| ()
                        >
                            <div class="alert alert-error mt-4">
                                <span>{move || error.get().unwrap_or_default()}</span>
                            </div>
                        </Show>

                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-4">
                            <div class="space-y-4">
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">{move || t("name").to_string()}</span></label>
                                    <input
                                        type="text"
                                        class="input input-bordered w-full"
                                        on:input=move |ev| set_label.set(event_target_value(&ev))
                                    />
                                </div>

                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">{move || t("type").to_string()}</span></label>
                                    <select
                                        class="select select-bordered w-full"
                                        on:change=move |ev| set_site_type.set(event_target_value(&ev))
                                    >
                                        <option value="field">{move || t("site_type_field").to_string()}</option>
                                        <option value="vineyard">{move || t("site_type_vineyard").to_string()}</option>
                                        <option value="olive_grove">{move || t("site_type_olive_grove").to_string()}</option>
                                        <option value="orchard">{move || t("site_type_orchard").to_string()}</option>
                                        <option value="pasture">{move || t("site_type_pasture").to_string()}</option>
                                        <option value="greenhouse">{move || t("site_type_greenhouse").to_string()}</option>
                                    </select>
                                </div>

                                <div class="stats shadow w-full">
                                    <div class="stat">
                                        <div class="stat-title">{move || t("area").to_string()}</div>
                                        <div class="stat-value text-primary">{move || format!("{:.2} ha", area.get())}</div>
                                    </div>
                                </div>
                            </div>

                            <div class="w-full h-full min-h-[400px]">
                                <label class="label"><span class="label-text">{move || t("draw_area").to_string()}</span></label>
                                {FieldPolygonEditor(on_map_change.clone())}
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)>{move || t("cancel").to_string()}</button>
                            <button class="btn btn-primary" on:click=on_create>{move || t("save").to_string()}</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
