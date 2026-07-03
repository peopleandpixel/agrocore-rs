use crate::api;
use crate::components::form::{is_valid_email, RequiredLabel};
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn UserManagement() -> impl IntoView {
    let i18n = use_context::<crate::i18n::I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<crate::i18n::Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    let users = LocalResource::new(|| async move { api::fetch_users().await.ok() });
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (firstname, set_firstname) = signal(String::new());
    let (lastname, set_lastname) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (role, set_role) = signal(String::from("Worker"));

    let on_create = move |_| {
        let firstname = firstname.get();
        let lastname = lastname.get();
        let email = email.get();
        let password = password.get();
        let role = role.get();
        set_error.set(None);

        if firstname.trim().is_empty()
            || lastname.trim().is_empty()
            || email.trim().is_empty()
            || password.trim().is_empty()
        {
            set_error.set(Some(String::from("Bitte alle Pflichtfelder ausfüllen.")));
            return;
        }

        if !is_valid_email(&email) {
            set_error.set(Some(String::from(
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
                Err(e) => set_error.set(Some(e)),
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
                                <th>"Rolle"</th>
                                <th>"Status"</th>
                                <th>"Letzter Login"</th>
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
                                    view! {
                                        <tr>
                                            <td>{name}</td>
                                            <td><div class="badge badge-primary">{role}</div></td>
                                            <td>{if user.is_active { "Aktiv" } else { "Inaktiv" }}</td>
                                            <td>{user.last_login.unwrap_or_else(|| String::from("-"))}</td>
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
                    <div class="modal-box max-w-2xl">
                        <h3 class="font-bold text-lg">"Benutzer einladen"</h3>

                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Vorname"}</RequiredLabel></label>
                                <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_firstname.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Nachname"}</RequiredLabel></label>
                                <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_lastname.set(event_target_value(&ev)) />
                            </div>
                        </div>

                        <div class="form-control mt-4">
                            <label class="label"><RequiredLabel required=true>{"E-Mail"}</RequiredLabel></label>
                            <input type="email" class="input input-bordered w-full" required on:input=move |ev| set_email.set(event_target_value(&ev)) />
                        </div>

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Passwort"}</RequiredLabel></label>
                                <input type="password" class="input input-bordered w-full" required minlength="8" on:input=move |ev| set_password.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{"Rolle"}</RequiredLabel></label>
                                <select class="select select-bordered w-full" required on:change=move |ev| set_role.set(event_target_value(&ev))>
                                    <option value="Admin">"Admin"</option>
                                    <option value="Manager">"Manager"</option>
                                    <option value="Worker" selected>"Worker"</option>
                                    <option value="Viewer">"Viewer"</option>
                                </select>
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)>"Abbrechen"</button>
                            <button class="btn btn-primary" on:click=on_create>"Speichern"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
