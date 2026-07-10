use crate::api;
use crate::components::form::RequiredLabel;
use crate::components::map::FieldPolygonEditor;
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn SiteManagement() -> impl IntoView {
    let i18n = use_context::<crate::i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<crate::i18n::Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    let sites = LocalResource::new(|| async move { api::fetch_sites().await.ok() });
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (label, set_label) = signal(String::new());
    let (site_type, set_site_type) = signal(String::from("vineyard"));
    let (crop_type, set_crop_type) = signal(String::from("grape"));
    let (variety, set_variety) = signal(String::new());
    let (boundary, set_boundary) = signal(None::<Vec<api::GeoPoint>>);
    let (area, set_area) = signal(None::<f64>);
    let (center, set_center) = signal(None::<api::GeoPoint>);

    let on_create = move |_| {
        let label = label.get();
        let site_type = site_type.get();
        let crop_type = crop_type.get();
        let area = area.get().unwrap_or(0.0);
        let boundary = boundary.get();
        let center = center.get();
        let variety = {
            let value = variety.get();
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        };
        set_error.set(None);

        if area <= 0.0
            || boundary
                .as_ref()
                .map(|points| points.len() < 3)
                .unwrap_or(true)
        {
            set_error.set(Some(String::from(
                "Bitte zuerst eine Polygonfläche zeichnen.",
            )));
            return;
        }

        if label.trim().is_empty() {
            set_error.set(Some(String::from("Bitte eine Flächenbezeichnung angeben.")));
            return;
        }

        spawn_local(async move {
            match api::create_site(api::CreateSiteRequest {
                label,
                site_type,
                crop_type,
                variety,
                area,
                gross_area: None,
                center,
                boundary,
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
                    <h1 class="text-3xl font-bold">{move || t("site_management")}</h1>
                    <p class="text-base-content/60">"Verwalten Sie Ihre Flächen."</p>
                </div>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <Icon icon=LuPlus width="20" height="20" />
                    "Fläche hinzufügen"
                </button>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"Typ"</th>
                                <th>"Kultur"</th>
                                <th>"Fläche"</th>
                                <th>"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
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
                                    <tr>
                                        <td>{site.label}</td>
                                        <td>{site.site_type}</td>
                                        <td>{site.crop_type}</td>
                                        <td>{format!("{:.2} ha", site.area)}</td>
                                        <td>{if site.is_active { "Aktiv" } else { "Inaktiv" }}</td>
                                    </tr>
                                }
                            />
                        </tbody>
                    </table>
                </div>
            </div>

            <Show when=move || show_add_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box w-11/12 max-w-5xl h-[700px] max-h-[90vh]">
                        <h3 class="font-bold text-lg mb-4">"Neue Fläche anlegen"</h3>

                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="form-control w-full mt-4">
                            <label class="label"><RequiredLabel required=true>{"Name der Fläche"}</RequiredLabel></label>
                            <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_label.set(event_target_value(&ev)) />
                        </div>

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Typ"}</RequiredLabel></label>
                                <select class="select select-bordered w-full" required on:change=move |ev| set_site_type.set(event_target_value(&ev))>
                                    <option value="vineyard">"Weinberg"</option>
                                    <option value="field">"Ackerland"</option>
                                    <option value="cork_oak_montado">"Korkeichen-Montado"</option>
                                    <option value="holm_oak_montado">"Steineichen-Montado"</option>
                                    <option value="olive_grove">"Olivenhain"</option>
                                    <option value="orchard">"Obstgarten"</option>
                                    <option value="almond_orchard">"Mandelhain"</option>
                                    <option value="citrus_grove">"Zitrushain"</option>
                                    <option value="pasture">"Weide"</option>
                                    <option value="greenhouse">"Gewächshaus"</option>
                                    <option value="other">"Sonstiges"</option>
                                </select>
                            </div>
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Kultur"}</RequiredLabel></label>
                                <select class="select select-bordered w-full" required on:change=move |ev| set_crop_type.set(event_target_value(&ev))>
                                    <option value="grape">"Traube"</option>
                                    <option value="olive">"Olive"</option>
                                    <option value="grain">"Getreide"</option>
                                    <option value="other">"Andere"</option>
                                </select>
                            </div>
                        </div>

                        <div class="grid grid-cols-1 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><span class="label-text">"Sorte"</span></label>
                                <input type="text" class="input input-bordered w-full" on:input=move |ev| set_variety.set(event_target_value(&ev)) />
                            </div>
                        </div>

                        <div class="mt-4">
                            <div class="flex items-center justify-between mb-2">
                                <span class="label-text font-medium">"Fläche zeichnen"</span>
                                <span class="text-sm opacity-70">
                                    {move || area.get().map(|value| format!("{:.2} ha", value)).unwrap_or_else(|| String::from("0.00 ha"))}
                                </span>
                            </div>
                            <FieldPolygonEditor
                                on_change=move |new_boundary, new_area, new_center| {
                                    set_boundary.set(new_boundary);
                                    set_area.set(new_area);
                                    set_center.set(new_center);
                                }
                            />
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)> "Abbrechen" </button>
                            <button class="btn btn-primary" on:click=on_create> "Speichern" </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}