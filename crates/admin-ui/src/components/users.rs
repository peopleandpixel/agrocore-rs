use crate::api;
use crate::api::UserDto;
use crate::components::form::{RequiredLabel, is_valid_email};
use crate::components::list::{ColumnDef, DataList, FilterOption};
use chrono::{DateTime, Utc};
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;
use std::sync::Arc;
use uuid::Uuid;

/// Format a datetime string as a localized relative "time ago" display.
/// Uses the provided translation function for i18n support.
/// Returns (humanized_display, original_datetime_for_tooltip).
fn format_time_ago(datetime_str: &str, t: impl Fn(&str) -> String) -> (String, String) {
    let dt = match DateTime::parse_from_rfc3339(datetime_str)
        .ok()
        .and_then(|d| d.with_timezone(&Utc).into())
    {
        Some(dt) => dt,
        None => return (String::from("-"), datetime_str.to_string()),
    };

    let now = Utc::now();
    let duration = now.signed_duration_since(dt);
    let seconds = duration.num_seconds();

    if seconds < 0 {
        return (String::from("-"), datetime_str.to_string());
    }

    let humanized = if seconds < 60 {
        t("just_now")
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        if minutes == 1 {
            t("one_minute_ago")
        } else {
            format!("{} {}", minutes, t("minutes_ago"))
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        if hours == 1 {
            t("one_hour_ago")
        } else {
            format!("{} {}", hours, t("hours_ago"))
        }
    } else if seconds < 604800 {
        let days = seconds / 86400;
        if days == 1 {
            t("one_day_ago")
        } else {
            format!("{} {}", days, t("days_ago"))
        }
    } else if seconds < 2592000 {
        let weeks = seconds / 604800;
        if weeks == 1 {
            t("one_week_ago")
        } else {
            format!("{} {}", weeks, t("weeks_ago"))
        }
    } else if seconds < 31536000 {
        let months = seconds / 2592000;
        if months == 1 {
            t("one_month_ago")
        } else {
            format!("{} {}", months, t("months_ago"))
        }
    } else {
        let years = seconds / 31536000;
        if years == 1 {
            t("one_year_ago")
        } else {
            format!("{} {}", years, t("years_ago"))
        }
    };

    (humanized, datetime_str.to_string())
}

#[component]
pub fn UserManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    // Fetch users and store in a signal for the DataList component
    let (users, set_users) = signal(Vec::<UserDto>::new());
    {
        let set_users = set_users;
        spawn_local(async move {
            if let Ok(resp) = api::fetch_users().await {
                set_users.set(resp.data);
            }
        });
    }

    // Current user ID — used to prevent self-deletion in the UI
    let current_user_id = api::user_id();

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

    // Delete confirmation modal state
    let (show_delete_confirm, set_show_delete_confirm) = signal(false);
    let (delete_user_id, set_delete_user_id) = signal(None::<Uuid>);
    let (delete_user_name, set_delete_user_name) = signal(String::new());

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

        let set_users = set_users;
        let set_show_add_modal = set_show_add_modal;
        let set_add_firstname = set_add_firstname;
        let set_add_lastname = set_add_lastname;
        let set_add_email = set_add_email;
        let set_add_password = set_add_password;
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
                    // Reload users list reactively instead of full page reload
                    if let Ok(resp) = api::fetch_users().await {
                        set_users.set(resp.data);
                    }
                    set_show_add_modal.set(false);
                    set_add_firstname.set(String::new());
                    set_add_lastname.set(String::new());
                    set_add_email.set(String::new());
                    set_add_password.set(String::new());
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

        let set_users = set_users;
        let set_show_edit_modal = set_show_edit_modal;
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
                    // Reload users list reactively instead of full page reload
                    if let Ok(resp) = api::fetch_users().await {
                        set_users.set(resp.data);
                    }
                    set_show_edit_modal.set(false);
                }
                Err(e) => set_edit_error.set(Some(e)),
            }
        });
    };

    let on_delete_confirm = move |_| {
        let user_id = match delete_user_id.get() {
            Some(id) => id,
            None => return,
        };
        let set_users = set_users;
        let set_show_delete_confirm = set_show_delete_confirm;
        spawn_local(async move {
            if api::delete_user(user_id).await.is_ok() {
                // Reload users list reactively instead of full page reload
                if let Ok(resp) = api::fetch_users().await {
                    set_users.set(resp.data);
                }
            }
            set_show_delete_confirm.set(false);
        });
    };

    let on_delete_cancel = move |_| {
        set_show_delete_confirm.set(false);
    };

    let on_delete = move |id: Uuid| {
        let name = users
            .get()
            .iter()
            .find(|u| u.id == id)
            .map(|u| format!("{} {}", u.firstname, u.lastname))
            .unwrap_or_else(|| String::from("User"));
        set_delete_user_id.set(Some(id));
        set_delete_user_name.set(name);
        set_show_delete_confirm.set(true);
    };

    // Check if the current session is impersonating — read once at component mount
    let impersonator_id = api::impersonator_id();
    let is_impersonating = impersonator_id.is_some();

    // Check if current user can impersonate others
    // SuperAdmin and Admin can impersonate, others need explicit permissions
    let current_user_role = api::user_role();
    let can_impersonate = current_user_role
        .as_deref()
        .map(|r| {
            let lower = r.to_lowercase();
            lower == "superadmin" || lower == "admin"
        })
        .unwrap_or(false);

    // Admin/SuperAdmin can manage user status (activate/deactivate)
    let can_manage_users = can_impersonate;

    let on_impersonate = move |id: Uuid| {
        spawn_local(async move {
            if let Ok(resp) = api::impersonate_user(id).await {
                // Save the impersonated user's name for the banner
                let name = format!("{} {}", resp.firstname, resp.lastname);
                api::set_impersonated_user_name(&name);
                api::set_auth_token(&resp.token);
                api::set_user_id(&resp.user_id);
                if let Some(imp_id) = resp.impersonator_id {
                    api::set_impersonator_id(&imp_id);
                }
                api::set_user_role(
                    &resp
                        .roles
                        .first()
                        .cloned()
                        .unwrap_or_else(|| String::from("Viewer")),
                );
                let _ = window().location().set_href("/");
            }
        });
    };

    let on_stop_impersonation = move |_| {
        spawn_local(async move {
            if let Ok(resp) = api::stop_impersonation().await {
                api::set_auth_token(&resp.token);
                api::set_user_id(&resp.user_id);
                api::clear_impersonator_id();
                api::set_user_role(
                    &resp
                        .roles
                        .first()
                        .cloned()
                        .unwrap_or_else(|| String::from("Viewer")),
                );
                let _ = window().location().set_href("/");
            }
        });
    };

    let on_toggle_status = move |user_id: Uuid, new_status: bool| {
        let set_users = set_users;
        spawn_local(async move {
            match api::update_user(
                user_id,
                api::UpdateUserRequest {
                    firstname: None,
                    lastname: None,
                    email: None,
                    roles: None,
                    is_active: Some(new_status),
                },
            )
            .await
            {
                Ok(_) => {
                    // Reload users list reactively instead of full page reload
                    if let Ok(resp) = api::fetch_users().await {
                        set_users.set(resp.data);
                    }
                }
                Err(_) => {
                    // Silently fail — the backend returns 403 for non-admin,
                    // which the user already can't see the toggle for.
                }
            }
        });
    };

    // Column definitions for the DataList
    let columns = vec![
        ColumnDef {
            key: "name",
            label: t("name"),
            sortable: true,
        },
        ColumnDef {
            key: "email",
            label: t("email"),
            sortable: true,
        },
        ColumnDef {
            key: "role",
            label: t("role"),
            sortable: true,
        },
        ColumnDef {
            key: "status",
            label: t("status"),
            sortable: false,
        },
        ColumnDef {
            key: "last_login",
            label: t("last_login"),
            sortable: true,
        },
    ];

    // Filter options
    let filters = vec![
        FilterOption {
            key: "active",
            label: t("active"),
        },
        FilterOption {
            key: "inactive",
            label: t("inactive"),
        },
    ];

    // Filter function
    let filter_fn = |items: &[UserDto], filter: Option<&str>| -> Vec<UserDto> {
        match filter {
            Some("active") => items.iter().filter(|u| u.is_active).cloned().collect(),
            Some("inactive") => items.iter().filter(|u| !u.is_active).cloned().collect(),
            _ => items.to_vec(),
        }
    };

    // Search config
    let search_config =
        |u: &UserDto| -> String { format!("{} {} {}", u.firstname, u.lastname, u.email) };

    // Sort extractor - extracts the sort key for a given column
    let sort_extractor = |u: &UserDto, key: &str| -> String {
        match key {
            "name" => format!("{} {}", u.firstname, u.lastname),
            "email" => u.email.clone(),
            "role" => u.roles.first().cloned().unwrap_or_default(),
            "status" => {
                if u.is_active {
                    "active".to_string()
                } else {
                    "inactive".to_string()
                }
            }
            "last_login" => u.last_login.clone().unwrap_or_default(),
            _ => String::new(),
        }
    };

    // Header buttons
    let header_buttons: Option<Arc<dyn Fn() -> AnyView + Send + Sync>> = {
        let is_impersonating = is_impersonating;
        let on_stop_impersonation = on_stop_impersonation;
        let set_show_permissions_modal = set_show_permissions_modal;
        let t = t;

        if is_impersonating {
            Some(Arc::new(move || {
                let on_stop = on_stop_impersonation;
                let t = t;
                view! {
                    <button class="btn btn-warning" on:click=on_stop>
                        <Icon icon=LuLogOut width="20" height="20" />
                        {crate::t!(t, "stop_impersonation")}
                    </button>
                }
                .into_any()
            }))
        } else {
            Some(Arc::new(move || {
                let set_show = set_show_permissions_modal;
                let t = t;
                view! {
                    <button class="btn btn-outline" on:click=move |_| set_show.set(true)>
                        <Icon icon=LuShield width="20" height="20" />
                        {crate::t!(t, "role_permissions")}
                    </button>
                }
                .into_any()
            }))
        }
    };

    // Add button
    let add_button: Option<(String, Arc<dyn Fn() + Send + Sync>)> =
        Some((t("user"), Arc::new(move || set_show_add_modal.set(true))));

    view! {
        <div class="flex flex-col gap-6">
            <DataList
            item_name_plural=t("users")
            description=t("users_desc")
            items=users
            key_fn=|u: &UserDto| u.id.to_string()
            columns=columns
            filters=filters
            filter_fn=filter_fn
            search_config=search_config
            sort_extractor=sort_extractor
            show_search=true
            header_buttons=header_buttons
            add_button=add_button
            row=move |user: &UserDto| {
                let user_id = user.id;
                let email_for_view = user.email.clone();
                let email_for_edit = email_for_view.clone();
                let firstname = user.firstname.clone();
                let lastname = user.lastname.clone();
                let name = format!("{} {}", firstname, lastname);
                let role = user.roles.first().cloned().unwrap_or_else(|| String::from("Worker"));
                let roles_for_edit = user.roles.clone();
                let last_login_raw = user.last_login.clone();
                let (last_login_human, last_login_title) = last_login_raw.as_deref()
                    .map(|s| format_time_ago(s, t))
                    .unwrap_or_else(|| (String::from("-"), String::from("-")));
                let is_active = user.is_active;
                let is_admin = user.roles.iter().any(|r| r == "Admin");

                // Determine if this is the current user
                let is_current_user = current_user_id.as_ref().map(|uid| *uid == user_id).unwrap_or(false);

                // Count admins to determine if this is the last admin
                let all_users = users.get();
                let admin_count = all_users.iter().filter(|u| u.roles.iter().any(|r| r == "Admin")).count();
                let is_last_admin = is_admin && admin_count <= 1;

                // Hide delete if it's the current user or the last admin
                let can_delete = !is_current_user && !is_last_admin;

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

                let can_toggle_status = can_manage_users && !is_current_user;

                view! {
                    <tr>
                        <td>{name}</td>
                        <td>{email_for_view}</td>
                        <td><div class="badge badge-primary">{role}</div></td>
                        <td>
                            <Show when=move || can_toggle_status>
                                <label class="swap swap-flip">
                                    <input type="checkbox" checked=is_active on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        on_toggle_status(user_id, checked);
                                    } />
                                    <div class="swap-on badge badge-success gap-2">
                                        <span class="inline-block w-2 h-2 rounded-full bg-green-400"></span>
                                        {t("active")}
                                    </div>
                                    <div class="swap-off badge badge-error gap-2">
                                        <span class="inline-block w-2 h-2 rounded-full bg-red-400"></span>
                                        {t("inactive")}
                                    </div>
                                </label>
                            </Show>
                            <Show when=move || !can_toggle_status>
                                <span class={if is_active { "badge badge-success gap-2" } else { "badge badge-warning gap-2" }}>
                                    <span class={if is_active { "inline-block w-2 h-2 rounded-full bg-green-400" } else { "inline-block w-2 h-2 rounded-full bg-orange-400" }}></span>
                                    {if is_active { t("active") } else { t("inactive") }}
                                </span>
                            </Show>
                        </td>
                        <td title={last_login_title}>{last_login_human}</td>
                        <td>
                            <div class="flex gap-2 justify-end">
                                <Show when=move || can_impersonate && !is_current_user && !is_impersonating>
                                    <button class="btn btn-sm btn-ghost" on:click=impersonate_click title=move || t("impersonate")>
                                        <Icon icon=LuCircleUser width="16" height="16" />
                                    </button>
                                </Show>
                                <Show when=move || !(can_impersonate && !is_current_user && !is_impersonating)>
                                    <span class="btn btn-sm btn-ghost text-base-content/30 cursor-not-allowed pointer-events-none" title=move || t("impersonate")>
                                        <Icon icon=LuCircleUser width="16" height="16" />
                                    </span>
                                </Show>
                                <button class="btn btn-sm btn-ghost" on:click=edit_click>
                                    <Icon icon=LuPencil width="16" height="16" />
                                </button>
                                <Show when=move || can_delete>
                                    <button class="btn btn-sm btn-ghost text-error" on:click=delete_click>
                                        <Icon icon=LuTrash2 width="16" height="16" />
                                    </button>
                                </Show>
                                <Show when=move || !can_delete>
                                    <span class="btn btn-sm btn-ghost text-base-content/30 cursor-not-allowed pointer-events-none">
                                        <Icon icon=LuTrash2 width="16" height="16" />
                                    </span>
                                </Show>
                            </div>
                        </td>
                    </tr>
                }.into_any()
            }
        />

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
                                    <option value="SuperAdmin">{crate::t!(t, "role_superadmin")}</option>
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
                                    <option value="SuperAdmin" selected=move || edit_role.get() == "SuperAdmin">{crate::t!(t, "role_superadmin")}</option>
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

            // Delete Confirmation Modal
            <Show when=move || show_delete_confirm.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg text-error">{crate::t!(t, "delete_user")}</h3>
                        <p class="py-4">
                            {crate::t!(t, "delete_user_confirm")}
                        </p>
                        <p class="font-semibold">
                            {move || delete_user_name.get()}
                        </p>
                        <p class="text-base-content/60 text-sm">
                            {crate::t!(t, "delete_user_warning")}
                        </p>
                        <div class="modal-action">
                            <button class="btn" on:click=on_delete_cancel>
                                {crate::t!(t, "cancel")}
                            </button>
                            <button class="btn btn-error" on:click=on_delete_confirm>
                                {crate::t!(t, "delete")}
                            </button>
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
