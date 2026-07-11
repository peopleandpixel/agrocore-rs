use crate::api;
use crate::components::form::{is_valid_email, RequiredLabel};
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;
use uuid::Uuid;

#[component]
pub fn UserManagement() -> impl IntoView {
    let i18n = use_context::<crate::i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<crate::i18n::Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    let users = LocalResource::new(|| async move { api::fetch_users().await.ok() });
    
    // Add modal state
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (add_error, set_add_error) = signal(None::<String>);
    let (add_firstname, set_add_firstname) = signal(String::new());
    let (add_lastname, set_add_lastname) = signal(String::new());
    let (add_email, set_add_email) = signal(String::new());
    let (add_password, set_add_password) = signal(String::new());
    let (add_role, set_add_role) = signal(String::from("Worker"));
    
    // Edit modal state
    let (show_edit_modal, set_show_edit_modal) = signal(false);
    let (edit_user_id, set_edit_user_id) = signal(None::<Uuid>);
    let (edit_error, set_edit_error) = signal(None::<String>);
    let (edit_firstname, set_edit_firstname) = signal(String::new());
    let (edit_lastname, set_edit_lastname) = signal(String::new());
    let (edit_email, set_edit_email) = signal(String::new());
    let (edit_role, set_edit_role) = signal(String::from("Worker"));
    let (edit_is_active, set_edit_is_active) = signal(true);
    
    let on_create = move |_| {
        let firstname = add_firstname.get();
        let lastname = add_lastname.get();
        let email = add_email.get();
        let password = add_password.get();
        let role = add_role.get();
        set_add_error.set(None);

        if firstname.trim().is_empty()
            || lastname.trim().is_empty()
            || email.trim().is_empty()
            || password.trim().is_empty()
        {
            set_add_error.set(Some(String::from("Bitte alle Pflichtfelder ausfüllen.")));
            return;
        }

        if !is_valid_email(&email) {
            set_add_error.set(Some(String::from(
                "Bitte eine gültige E-Mail-Adresse eingeben.",
            )));
            return;
        }

        spawn_local(async move {
            match api::create_user(api::CreateUserRequest {
                firstname,
                lastname,
                email,
                password,
                roles: Some(vec![role]),
            })
            .await
            {
                Ok(_) => {
                    let _ = window().location().reload();
                }
                Err(e) => set_add_error.set(Some(e)),
            }
        });
    };

    let on_update = move |_| {
        let user_id = match edit_user_id.get() {
            Some(id) => id,
            None => return,
        };
        let firstname = if edit_firstname.get().trim().is_empty() { None } else { Some(edit_firstname.get()) };
        let lastname = if edit_lastname.get().trim().is_empty() { None } else { Some(edit_lastname.get()) };
        let email = if edit_email.get().trim().is_empty() { None } else { Some(edit_email.get()) };
        let roles = Some(vec![edit_role.get()]);
        let is_active = Some(edit_is_active.get());

        set_edit_error.set(None);

        spawn_local(async move {
            match api::update_user(user_id, api::UpdateUserRequest {
                firstname,
                lastname,
                email,
                roles,
                is_active,
            }).await {
                Ok(_) => {
                    let _ = window().location().reload();
                }
                Err(e) => set_edit_error.set(Some(e)),
            }
        });
    };

    let on_delete = move |id: Uuid| {
        spawn_local(async move {
            if api::delete_user(id).await.is_ok() {
                let _ = window().location().reload();
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">{move || t("users")}</h1>
                    <p class="text-base-content/60">"Verwalten Sie Teammitglieder und deren Zugriffsberechtigungen."</p>
                </div>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <Icon icon=LuUserPlus width="20" height="20" />
                    "Benutzer einladen"
                </button>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"E-Mail"</th>
                                <th>"Rolle"</th>
                                <th>"Status"</th>
                                <th>"Letzter Login"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || {
                                    users
                                        .read()
                                        .as_ref()
                                        .map(|u| {
                                            u.as_ref()
                                                .map(|page| page.data.clone())
                                                .unwrap_or_default()
                                        })
                                        .unwrap_or_default()
                                }
                                key=|user| user.id
                                children=move |user| {
                                    let name = format!("{} {}", user.firstname, user.lastname);
                                    let role = user.roles.first().cloned().unwrap_or_else(|| String::from("Worker"));
                                    
                                    let edit_click = move |_| {
                                        set_edit_user_id.set(Some(user.id));
                                        set_edit_firstname.set(user.firstname.clone());
                                        set_edit_lastname.set(user.lastname.clone());
                                        set_edit_email.set(user.email.clone());
                                        set_edit_role.set(user.roles.first().cloned().unwrap_or_default());
                                        set_edit_is_active.set(user.is_active);
                                        set_show_edit_modal.set(true);
                                    };
                                    
                                    let delete_click = move |_| {
                                        if window().confirm(Some("Benutzer wirklich löschen?")).unwrap_or(false) {
                                            on_delete(user.id);
                                        }
                                    };
                                    
                                    view! {
                                        <tr>
                                            <td>{name}</td>
                                            <td>{user.email}</td>
                                            <td><div class="badge badge-primary">{role}</div></td>
                                            <td>{if user.is_active { "Aktiv" } else { "Inaktiv" }}</td>
                                            <td>{user.last_login.clone().unwrap_or_else(|| String::from("-"))}</td>
                                            <td>
                                                <div class="flex gap-2 justify-end">
                                                    <button class="btn btn-sm btn-ghost" on:click=edit_click>
                                                        <Icon icon=LuPencil width="16" height="16" />
                                                    </button>
                                                    <button class="btn btn-sm btn-ghost text-error" on:click=delete_click>
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

            // Add User Modal
            <Show when=move || show_add_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box max-w-2xl">
                        <h3 class="font-bold text-lg">"Benutzer einladen"</h3>

                        {move || add_error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Vorname"}</RequiredLabel></label>
                                <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_add_firstname.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Nachname"}</RequiredLabel></label>
                                <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_add_lastname.set(event_target_value(&ev)) />
                            </div>
                        </div>

                        <div class="form-control mt-4">
                            <label class="label"><RequiredLabel required=true>{"E-Mail"}</RequiredLabel></label>
                            <input type="email" class="input input-bordered w-full" required on:input=move |ev| set_add_email.set(event_target_value(&ev)) />
                        </div>

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Passwort"}</RequiredLabel></label>
                                <input type="password" class="input input-bordered w-full" required minlength="8" on:input=move |ev| set_add_password.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Rolle"}</RequiredLabel></label>
                                <select class="select select-bordered w-full" required on:change=move |ev| set_add_role.set(event_target_value(&ev))>
                                    <option value="Admin">"Admin"</option>
                                    <option value="Manager">"Manager"</option>
                                    <option value="Worker" selected>"Worker"</option>
                                    <option value="Viewer">"Viewer"</option>
                                </select>
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)> "Abbrechen" </button>
                            <button class="btn btn-primary" on:click=on_create> "Speichern" </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Edit User Modal
            <Show when=move || show_edit_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box max-w-2xl">
                        <h3 class="font-bold text-lg">"Benutzer bearbeiten"</h3>

                        {move || edit_error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><span class="label-text">{"Vorname"}</span></label>
                                <input type="text" class="input input-bordered w-full" prop:value=move || edit_firstname.get() on:input=move |ev| set_edit_firstname.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{"Nachname"}</span></label>
                                <input type="text" class="input input-bordered w-full" prop:value=move || edit_lastname.get() on:input=move |ev| set_edit_lastname.set(event_target_value(&ev)) />
                            </div>
                        </div>

                        <div class="form-control mt-4">
                            <label class="label"><span class="label-text">{"E-Mail"}</span></label>
                            <input type="email" class="input input-bordered w-full" prop:value=move || edit_email.get() on:input=move |ev| set_edit_email.set(event_target_value(&ev)) />
                        </div>

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><span class="label-text">{"Rolle"}</span></label>
                                <select class="select select-bordered w-full" on:change=move |ev| set_edit_role.set(event_target_value(&ev))>
                                    <option value="Admin">"Admin"</option>
                                    <option value="Manager">"Manager"</option>
                                    <option value="Worker">"Worker"</option>
                                    <option value="Viewer">"Viewer"</option>
                                </select>
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{"Status"}</span></label>
                                <select class="select select-bordered w-full" on:change=move |ev| set_edit_is_active.set(event_target_value(&ev) == "true")>
                                    <option value="true" selected=move || edit_is_active.get()> "Aktiv"</option>
                                    <option value="false" selected=move || !edit_is_active.get()> "Inaktiv"</option>
                                </select>
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_edit_modal.set(false)> "Abbrechen" </button>
                            <button class="btn btn-primary" on:click=on_update> "Speichern" </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}