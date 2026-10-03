//! Applicator licenses — `GET/POST /compliance/applicator-licenses`.
//!
//! Registered with full CRUD handlers and no UI caller. A license without a UI is
//! not a paperwork problem: applying plant protection products is illegal without
//! a valid certificate, and the other compliance pages — checklists, fertiliser
//! records, plant protection records — were already reachable.
//!
//! `LicenseType` derives `strum::EnumString` with `serialize_all = "snake_case"`
//! but has no `serde` rename, so it serialises as the variant name
//! (`"Basic"`, `"Professional"`), not the snake-case form. The select sends those.

use crate::api;
use icondata::{LuPlus, LuTrash2};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// The license types `LicenseType` accepts, with the serde wire value.
///
/// `Custom(String)` carries caller text and is not offered here.
const LICENSE_TYPES: [(&str, &str); 3] = [
    ("Basic", "license_basic"),
    ("Advanced", "license_advanced"),
    ("Professional", "license_professional"),
];

fn text(label: String, value: ReadSignal<String>, set: WriteSignal<String>) -> impl IntoView {
    view! {
        <label class="form-control">
            <span class="label-text">{label}</span>
            <input
                class="input input-bordered"
                prop:value=move || value.get()
                on:input=move |ev| set.set(event_target_value(&ev))
            />
        </label>
    }
}

fn date(label: String, value: ReadSignal<String>, set: WriteSignal<String>) -> impl IntoView {
    view! {
        <label class="form-control">
            <span class="label-text">{label}</span>
            <input
                class="input input-bordered"
                type="date"
                prop:value=move || value.get()
                on:input=move |ev| set.set(event_target_value(&ev))
            />
        </label>
    }
}

#[component]
pub fn ApplicatorLicenses() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let licenses =
        LocalResource::new(|| async move { api::fetch_applicator_licenses().await.ok() });
    let (reload, set_reload) = signal(0u32);

    let (user_id, set_user_id) = signal(String::new());
    let (license_type, set_license_type) = signal(String::from("Basic"));
    let (number, set_number) = signal(String::new());
    let (issuer, set_issuer) = signal(String::new());
    let (valid_from, set_valid_from) = signal(String::new());
    let (valid_until, set_valid_until) = signal(String::new());
    let (error, set_error) = signal(None::<String>);

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let user = match uuid::Uuid::parse_str(user_id.get().trim()) {
            Ok(id) => id,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "invalid_uuid")().to_string()));
                return;
            }
        };
        let num = number.get().trim().to_string();
        let by = issuer.get().trim().to_string();
        if num.is_empty() || by.is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }
        if valid_from.get().trim().is_empty() || valid_until.get().trim().is_empty() {
            set_error.set(Some(crate::t!(t, "field_required")().to_string()));
            return;
        }

        let req = api::CreateApplicatorLicenseRequest {
            user_id: user,
            license_type: license_type.get(),
            license_number: num,
            issued_by: by,
            // `valid_from` and `valid_until` are `DateTime<Utc>`; a bare date is
            // read as midnight UTC.
            valid_from: format!("{}T00:00:00Z", valid_from.get().trim()),
            valid_until: format!("{}T00:00:00Z", valid_until.get().trim()),
        };

        spawn_local(async move {
            match api::create_applicator_license(req).await {
                Ok(_) => {
                    set_number.set(String::new());
                    set_issuer.set(String::new());
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            if let Err(e) = api::delete_applicator_license(id).await {
                set_error.set(Some(format!("{}: {e}", crate::t!(t, "delete_failed")())));
            } else {
                set_reload.update(|n| *n += 1);
            }
        });
    };

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4">{crate::t!(t, "applicator_licenses")}</h1>

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
                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                    {text(crate::t!(t, "user_id")().to_string(), user_id, set_user_id)}
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "license_type")}</span>
                        <select
                            class="select select-bordered"
                            prop:value=move || license_type.get()
                            on:change=move |ev| set_license_type.set(event_target_value(&ev))
                        >
                            {LICENSE_TYPES
                                .iter()
                                .map(|(value, key)| {
                                    view! {
                                        <option value=*value>{crate::t!(t, *key)()}</option>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </label>
                    {text(
                        crate::t!(t, "license_number")().to_string(),
                        number,
                        set_number,
                    )}
                    {text(
                        crate::t!(t, "issued_by")().to_string(),
                        issuer,
                        set_issuer,
                    )}
                    {date(
                        crate::t!(t, "valid_from")().to_string(),
                        valid_from,
                        set_valid_from,
                    )}
                    {date(
                        crate::t!(t, "valid_until")().to_string(),
                        valid_until,
                        set_valid_until,
                    )}
                </div>
                <button type="submit" class="btn btn-primary mt-3">
                    <Icon icon=LuPlus width="16" height="16" />
                    {crate::t!(t, "btn_add")}
                </button>
            </form>

            {move || {
                let _ = reload.get();
                let loaded = licenses.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "user_id")}</th>
                                            <th>{crate::t!(t, "license_type")}</th>
                                            <th>{crate::t!(t, "license_number")}</th>
                                            <th>{crate::t!(t, "issued_by")}</th>
                                            <th>{crate::t!(t, "valid_from")}</th>
                                            <th>{crate::t!(t, "valid_until")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|l: &api::ApplicatorLicenseDto| l.id
                                            children=move |l: api::ApplicatorLicenseDto| {
                                                let id = l.id;
                                                view! {
                                                    <tr>
                                                        <td>{l.user_id.to_string()}</td>
                                                        <td>{l.license_type.clone()}</td>
                                                        <td>{l.license_number.clone()}</td>
                                                        <td>{l.issued_by.clone()}</td>
                                                        <td>
                                                            {l.valid_from.get(..10).unwrap_or(&l.valid_from).to_string()}
                                                        </td>
                                                        <td>
                                                            {l.valid_until.get(..10).unwrap_or(&l.valid_until).to_string()}
                                                        </td>
                                                        <td>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| on_delete(id)
                                                            >
                                                                <Icon
                                                                    icon=LuTrash2
                                                                    width="14"
                                                                    height="14"
                                                                />
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
                        view! { <div class="alert">{crate::t!(t, "no_records")}</div> }.into_any()
                    }
                    None => {
                        view! { <div class="alert">{crate::t!(t, "loading")}</div> }.into_any()
                    }
                }
            }}
        </div>
    }
}
