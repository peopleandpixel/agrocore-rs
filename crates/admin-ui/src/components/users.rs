use crate::api;
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;
use uuid::Uuid;
use crate::components::form::{RequiredLabel, is_valid_email};

#[component]
pub fn UserManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

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
    let (show_permissions_modal, set_show_permissions_modal) = signal(false);

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
            set_add_error.set(Some(t("validation_required")));
            return;
        }

        if !is_valid_email(&email) {
            set_add_error.set(Some(t("validation_invalid_email")));
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
        let firstname = if edit_firstname.get().trim().is_empty() {
            None
        } else {
            Some(edit_firstname.get())
        };
        let lastname = if edit_lastname.get().trim().is_empty() {
            None
        } else {
            Some(edit_lastname.get())
        };
        let email = if edit_email.get().trim().is_empty() {
            None
        } else {
            Some(edit_email.get())
        };
        let roles = Some(vec![edit_role.get()]);
        let is_active = Some(edit_is_active.get());

        set_edit_error.set(None);

        spawn_local(async move {
            match api::update_user(
                user_id,
                api::UpdateUserRequest {
                    firstname,
                    lastname,
                    email,
                    roles,
                    is_active,
                },
            )
            .await
            {
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

    let on_impersonate = move |id: Uuid| {
        spawn_local(async move {
            match api::impersonate_user(id).await {
                Ok(resp) => {
                    api::set_auth_token(&resp.token);
                    api::set_user_role(&resp.roles.first().cloned().unwrap_or_else(|| String::from("Viewer")));
                    let _ = window().location().set_href("/");
                }
                Err(_) => {}
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">{crate::t!(t, "users")}</h1>
                    <p class="text-base-content/60">{crate::t!(t, "users_desc")}</p>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-outline" on:click=move |_| set_show_permissions_modal.set(true)>
                        <Icon icon=LuShield width="20" height="20" />
                        {crate::t!(t, "role_permissions")}
                    </button>
                    <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                        <Icon icon=LuUserPlus width="20" height="20" />
                        {crate::t!(t, "invite_user")}
                    </button>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>{crate::t!(t, "name")}</th>
                                <th>{crate::t!(t, "email")}</th>
                                <th>{crate::t!(t, "role")}</th>
                                <th>{crate::t!(t, "status")}</th>
                                <th>{crate::t!(t, "last_login")}</th>
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
                                    let user_id = user.id;
                                    let email_for_view = user.email.clone();
                                    let email_for_edit = email_for_view.clone();
                                    let firstname = user.firstname.clone();
                                    let lastname = user.lastname.clone();
                                    let name = format!("{} {}", firstname, lastname);
                                    let role = user.roles.first().cloned().unwrap_or_else(|| String::from("Worker"));
                                    let roles_for_edit = user.roles.clone();
                                    let last_login = user.last_login.clone().unwrap_or_else(|| String::from("-"));
                                    let is_active = user.is_active;

                                    let edit_click = move |_| {
                                        set_edit_user_id.set(Some(user_id));
                                        set_edit_firstname.set(firstname.clone());
                                        set_edit_lastname.set(lastname.clone());
                                        set_edit_email.set(email_for_edit.clone());
                                        set_edit_role.set(roles_for_edit.first().cloned().unwrap_or_default());
                                        set_edit_is_active.set(is_active);
                                        set_show_edit_modal.set(true);
                                    };

                                    let delete_click = move |_| {
                                        on_delete(user_id);
                                    };

                                    let impersonate_click = move |_| {
                                        on_impersonate(user_id);
                                    };

                                    view! {
                                        <tr>
                                            <td>{name}</td>
                                            <td>{email_for_view}</td>
                                            <td><div class="badge badge-primary">{role}</div></td>
                                            <td>{if is_active { t("active") } else { t("inactive") }}</td>
                                            <td>{last_login}</td>
                                            <td>
                                                <div class="flex gap-2 justify-end">
                                                    <button class="btn btn-sm btn-ghost" on:click=impersonate_click title=move || t("impersonate")>
                                                        <Icon icon=LuCircleUser width="16" height="16" />
                                                    </button>
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
                        <h3 class="font-bold text-lg">{crate::t!(t, "invite_user")}</h3>

                        {move || add_error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{crate::t!(t, "first_name")}</RequiredLabel></label>
                                <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_add_firstname.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{crate::t!(t, "last_name")}</RequiredLabel></label>
                                <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_add_lastname.set(event_target_value(&ev)) />
                            </div>
                        </div>

                        <div class="form-control mt-4">
                            <label class="label"><RequiredLabel required=true>{crate::t!(t, "email")}</RequiredLabel></label>
                            <input type="email" class="input input-bordered w-full" required on:input=move |ev| set_add_email.set(event_target_value(&ev)) />
                        </div>

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{crate::t!(t, "password")}</RequiredLabel></label>
                                <input type="password" class="input input-bordered w-full" required minlength="8" on:input=move |ev| set_add_password.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><RequiredLabel required=true>{crate::t!(t, "role")}</RequiredLabel></label>
                                <select class="select select-bordered w-full" required on:change=move |ev| set_add_role.set(event_target_value(&ev))>
                                    <option value="Admin">{crate::t!(t, "role_admin")}</option>
                                    <option value="Manager">{crate::t!(t, "role_manager")}</option>
                                    <option value="Worker" selected>{crate::t!(t, "role_worker")}</option>
                                    <option value="Viewer">{crate::t!(t, "role_viewer")}</option>
                                </select>
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_add_modal.set(false)> {crate::t!(t, "cancel")} </button>
                            <button class="btn btn-primary" on:click=on_create> {crate::t!(t, "save")} </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Edit User Modal
            <Show when=move || show_edit_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box max-w-2xl">
                        <h3 class="font-bold text-lg">{crate::t!(t, "edit_user")}</h3>

                        {move || edit_error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><span class="label-text">{crate::t!(t, "first_name")}</span></label>
                                <input type="text" class="input input-bordered w-full" prop:value=move || edit_firstname.get() on:input=move |ev| set_edit_firstname.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{crate::t!(t, "last_name")}</span></label>
                                <input type="text" class="input input-bordered w-full" prop:value=move || edit_lastname.get() on:input=move |ev| set_edit_lastname.set(event_target_value(&ev)) />
                            </div>
                        </div>

                        <div class="form-control mt-4">
                            <label class="label"><span class="label-text">{crate::t!(t, "email")}</span></label>
                            <input type="email" class="input input-bordered w-full" prop:value=move || edit_email.get() on:input=move |ev| set_edit_email.set(event_target_value(&ev)) />
                        </div>

                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><span class="label-text">{crate::t!(t, "role")}</span></label>
                                <select class="select select-bordered w-full" on:change=move |ev| set_edit_role.set(event_target_value(&ev))>
                                    <option value="Admin" selected=move || edit_role.get() == "Admin">{crate::t!(t, "role_admin")}</option>
                                    <option value="Manager" selected=move || edit_role.get() == "Manager">{crate::t!(t, "role_manager")}</option>
                                    <option value="Worker" selected=move || edit_role.get() == "Worker">{crate::t!(t, "role_worker")}</option>
                                    <option value="Viewer" selected=move || edit_role.get() == "Viewer">{crate::t!(t, "role_viewer")}</option>
                                </select>
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{crate::t!(t, "status")}</span></label>
                                <select class="select select-bordered w-full" on:change=move |ev| set_edit_is_active.set(event_target_value(&ev) == "true")>
                                    <option value="true" selected=move || edit_is_active.get()> {crate::t!(t, "active")}</option>
                                    <option value="false" selected=move || !edit_is_active.get()> {crate::t!(t, "inactive")}</option>
                                </select>
                            </div>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_edit_modal.set(false)> {crate::t!(t, "cancel")} </button>
                            <button class="btn btn-primary" on:click=on_update> {crate::t!(t, "save")} </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Permissions Overview Modal
            <Show when=move || show_permissions_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box w-11/12 max-w-4xl">
                        <h3 class="font-bold text-lg mb-4">{crate::t!(t, "role_permissions_overview")}</h3>
                        <p class="text-base-content/60 mb-4">
                            {crate::t!(t, "role_permissions_desc")}
                        </p>

                        <div class="overflow-x-auto">
                            <table class="table table-sm">
                                <thead>
                                    <tr>
                                        <th>{crate::t!(t, "role")}</th>
                                        <th>{crate::t!(t, "description")}</th>
                                        <th>{crate::t!(t, "sites")}</th>
                                        <th>{crate::t!(t, "equipment")}</th>
                                        <th>{crate::t!(t, "orders")}</th>
                                        <th>{crate::t!(t, "users")}</th>
                                        <th>{crate::t!(t, "finance")}</th>
                                        <th>{crate::t!(t, "analytics")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr>
                                        <td><strong>{crate::t!(t, "role_admin")}</strong></td>
                                        <td class="text-xs">{crate::t!(t, "role_admin_desc")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                    </tr>
                                    <tr>
                                        <td><strong>{crate::t!(t, "role_manager")}</strong></td>
                                        <td class="text-xs">{crate::t!(t, "role_manager_desc")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_all")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read_update")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read_create")}</td>
                                    </tr>
                                    <tr>
                                        <td><strong>{crate::t!(t, "role_worker")}</strong></td>
                                        <td class="text-xs">{crate::t!(t, "role_worker_desc")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_none")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_none")}</td>
                                    </tr>
                                    <tr>
                                        <td><strong>{crate::t!(t, "role_viewer")}</strong></td>
                                        <td class="text-xs">{crate::t!(t, "role_viewer_desc")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_none")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                        <td class="text-xs">{crate::t!(t, "permissions_read")}</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>

                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_permissions_modal.set(false)> {crate::t!(t, "close")} </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
