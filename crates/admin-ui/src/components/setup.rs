use crate::api;
use crate::components::form::{
    country_flag, is_valid_email, language_flag, normalize_phone, RequiredLabel, PHONE_PREFIXES,
};
use crate::i18n::{I18n, Language};
use leptos::prelude::{window, *};
use leptos::task::spawn_local;

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum SetupStep {
    Admin,
    Tenant,
    Company,
    Resources,
}

fn setup_t(i18n: &I18n, lang: Language, key: &str) -> String {
    i18n.t(lang.as_str(), key)
}

fn slug_is_valid(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

#[allow(clippy::too_many_arguments)]
fn submit_setup(
    i18n: I18n,
    lang: Language,
    admin_firstname: String,
    admin_lastname: String,
    admin_email: String,
    admin_password: String,
    tenant_name: String,
    tenant_slug: String,
    company_name: String,
    company_address: String,
    company_country: String,
    company_email: String,
    company_phone_prefix: String,
    company_phone_local: String,
    set_setup_status: WriteSignal<Option<Result<(), String>>>,
    set_setup_error: WriteSignal<Option<String>>,
) {
    let required = setup_t(&i18n, lang, "validation_required");
    let invalid_email = setup_t(&i18n, lang, "validation_invalid_email");
    let invalid_phone = setup_t(&i18n, lang, "validation_invalid_phone");
    let invalid_slug = setup_t(&i18n, lang, "validation_invalid_slug");

    if admin_firstname.trim().is_empty()
        || admin_lastname.trim().is_empty()
        || admin_email.trim().is_empty()
        || admin_password.trim().is_empty()
        || tenant_name.trim().is_empty()
        || tenant_slug.trim().is_empty()
        || company_name.trim().is_empty()
        || company_address.trim().is_empty()
        || company_country.trim().is_empty()
    {
        set_setup_error.set(Some(required));
        return;
    }

    if !is_valid_email(&admin_email)
        || (!company_email.trim().is_empty() && !is_valid_email(&company_email))
    {
        set_setup_error.set(Some(invalid_email));
        return;
    }

    if !slug_is_valid(&tenant_slug) {
        set_setup_error.set(Some(invalid_slug));
        return;
    }

    let company_phone = if company_phone_local.trim().is_empty() {
        None
    } else {
        match normalize_phone(&company_phone_prefix, &company_phone_local) {
            Some(phone) if crate::components::form::is_valid_phone(&phone) => Some(phone),
            _ => {
                set_setup_error.set(Some(invalid_phone));
                return;
            }
        }
    };

    let admin_email = admin_email.trim().to_string();
    let admin_firstname = admin_firstname.trim().to_string();
    let admin_lastname = admin_lastname.trim().to_string();
    let tenant_name = tenant_name.trim().to_string();
    let tenant_slug = tenant_slug.trim().to_string();
    let company_name = company_name.trim().to_string();
    let company_address = company_address.trim().to_string();
    let company_country = company_country.trim().to_string();
    let company_email = company_email.trim().to_string();
    let selected_language = lang.as_str().to_string();
    let starting_setup = setup_t(&i18n, lang, "starting_setup");

    let company_profile = api::CompanyProfile {
        company_name: Some(company_name.clone()),
        tax_id: None,
        office_email: if company_email.is_empty() {
            None
        } else {
            Some(company_email.clone())
        },
        phone: company_phone.clone(),
        address: Some(company_address.clone()),
        website: None,
        country: Some(company_country.clone()),
    };

    set_setup_error.set(None);
    set_setup_status.set(Some(Err(starting_setup)));

    spawn_local(async move {
        let admin = serde_json::json!({
            "firstname": admin_firstname,
            "lastname": admin_lastname,
            "email": admin_email,
            "password": admin_password,
        });
        let tenant = serde_json::json!({
            "name": tenant_name,
            "slug": tenant_slug,
            "config": {
                "default_language": selected_language,
                "supported_languages": ["de", "en", "es", "fr", "pt"],
                "timezone": "Europe/Lisbon",
                "enabled_modules": ["field_management", "PlantProtection", "Fertilization", "Harvest", "WorkLog", "CostTracking", "Maps", "Reports"],
                "custom_field_schemas": {
                    "company_profile": {
                        "name": company_name,
                        "address": company_address,
                        "country": company_country,
                        "email": company_email,
                        "phone": company_phone
                    }
                }
            }
        });

        let req = api::InitialSetupRequest { admin, tenant };

        match api::initial_setup(req).await {
            Ok(_) => match api::login(api::LoginRequest {
                email: admin_email,
                password: admin_password,
            })
            .await
            {
                Ok(auth) => {
                    api::set_auth_token(&auth.token);
                    api::save_company_profile(&company_profile);
                    if let Some(storage) = window().local_storage().ok().flatten() {
                        let _ = storage.set_item("agrocore.lang", &selected_language);
                    }
                    set_setup_status.set(Some(Ok(())));
                    let _ = window().location().reload();
                }
                Err(e) => {
                    set_setup_status.set(Some(Err(e)));
                }
            },
            Err(e) => {
                set_setup_status.set(Some(Err(e)));
            }
        }
    });
}

#[component]
pub fn SetupAssistant() -> impl IntoView {
    let i18n = I18n::new();
    let (lang, set_lang) = signal(
        window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|storage| storage.get_item("agrocore.lang").ok().flatten())
            .map(|value| Language::from_str(&value))
            .unwrap_or(Language::DE),
    );
    let (step, set_step) = signal(SetupStep::Admin);
    let (setup_status, set_setup_status) = signal(None::<Result<(), String>>);
    let (setup_error, set_setup_error) = signal(None::<String>);

    let (admin_firstname, set_admin_firstname) = signal(String::new());
    let (admin_lastname, set_admin_lastname) = signal(String::new());
    let (admin_email, set_admin_email) = signal(String::new());
    let (admin_password, set_admin_password) = signal(String::new());

    let (tenant_name, set_tenant_name) = signal(String::new());
    let (tenant_slug, set_tenant_slug) = signal(String::new());

    let (company_name, set_company_name) = signal(String::new());
    let (company_address, set_company_address) = signal(String::new());
    let (company_country, set_company_country) = signal(String::from("Portugal"));
    let (company_email, set_company_email) = signal(String::new());
    let (company_phone_prefix, set_company_phone_prefix) = signal(String::from("+351"));
    let (company_phone_local, set_company_phone_local) = signal(String::new());

    let (_resource_type, set_resource_type) = signal("field_management".to_string());
    let setup_title = setup_t(&i18n, lang.get(), "setup_title");
    let wizard_language = setup_t(&i18n, lang.get(), "wizard_language");
    let setup_step_admin = setup_t(&i18n, lang.get(), "setup_step_admin");
    let setup_step_tenant = setup_t(&i18n, lang.get(), "setup_step_tenant");
    let setup_step_company = setup_t(&i18n, lang.get(), "setup_step_company");
    let setup_step_resources = setup_t(&i18n, lang.get(), "setup_step_resources");
    let finish_label = setup_t(&i18n, lang.get(), "finish_setup");

    view! {
        <div class="min-h-screen bg-base-200 flex flex-col items-center justify-center p-4">
            <div class="mb-8 flex flex-col items-center">
                <img src="/docs/agrocore_RS.png" alt="AgroCore Logo" class="w-24 h-24 mb-3" />
                <h1 class="text-5xl font-black mb-2 text-primary">"AgroCore"</h1>
                <p class="text-base-content/50 uppercase tracking-widest font-bold">
                    {setup_title}
                </p>
            </div>

            <div class="w-full max-w-lg mb-4">
                <div class="form-control">
                    <label class="label"><span class="label-text">{wizard_language}</span></label>
                    <select
                        class="select select-bordered w-full"
                        prop:value=move || lang.get().as_str().to_string()
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_lang.set(Language::from_str(&value));
                            if let Some(storage) = window().local_storage().ok().flatten() {
                                let _ = storage.set_item("agrocore.lang", &value);
                            }
                        }
                    >
                        <option value="de">{format!("{} Deutsch", language_flag("de"))}</option>
                        <option value="en">{format!("{} English", language_flag("en"))}</option>
                        <option value="es">{format!("{} Español", language_flag("es"))}</option>
                        <option value="fr">{format!("{} Français", language_flag("fr"))}</option>
                        <option value="pt">{format!("{} Português", language_flag("pt"))}</option>
                    </select>
                </div>
            </div>

            <ul class="steps mb-8 w-full max-w-2xl">
                <li class=move || format!("step {}", if step.get() >= SetupStep::Admin { "step-primary" } else { "" })>{setup_step_admin}</li>
                <li class=move || format!("step {}", if step.get() >= SetupStep::Tenant { "step-primary" } else { "" })>{setup_step_tenant}</li>
                <li class=move || format!("step {}", if step.get() >= SetupStep::Company { "step-primary" } else { "" })>{setup_step_company}</li>
                <li class=move || format!("step {}", if step.get() >= SetupStep::Resources { "step-primary" } else { "" })>{setup_step_resources}</li>
            </ul>

            <div class="card w-full max-w-lg bg-base-100 shadow-2xl border border-base-300">
                <div class="card-body">
                    {move || {
                        let current_lang = lang.get();
                        let i18n_for_t = i18n.clone();
                        let t = move |key: &str| setup_t(&i18n_for_t, current_lang, key);
                        let setup_admin_title = t("setup_admin_title");
                        let setup_admin_desc = t("setup_admin_desc");
                        let setup_tenant_title = t("setup_tenant_title");
                        let setup_tenant_desc = t("setup_tenant_desc");
                        let setup_company_title = t("setup_company_title");
                        let setup_company_desc = t("setup_company_desc");
                        let setup_resources_title = t("setup_resources_title");
                        let setup_resources_desc = t("setup_resources_desc");
                        let setup_resource_type = t("setup_resource_type");
                        let setup_equipment = t("setup_equipment");
                        let setup_first_machine = t("setup_first_machine");
                        let setup_first_machine_placeholder = t("first_machine_placeholder");
                        let first_name_label = t("first_name");
                        let last_name_label = t("last_name");
                        let email_label = t("email");
                        let admin_password_label = t("admin_password");
                        let tenant_name_label = t("tenant_name");
                        let tenant_slug_label = t("tenant_slug");
                        let company_name_label = t("company_name");
                        let company_address_label = t("company_address");
                        let company_country_label = t("company_country");
                        let company_email_label = t("company_email");
                        let company_phone_label = t("company_phone");
                        let field_management_label = t("field_management");
                        let task_protection_label = t("task_protection");
                        let resources_label = t("resources");
                        let continue_label = t("continue");
                        let required_error = t("validation_required");
                        let invalid_email_error = t("validation_invalid_email");
                        let invalid_phone_error = t("validation_invalid_phone");
                        let invalid_slug_error = t("validation_invalid_slug");
                        let starting_setup_label = t("starting_setup");
                        let submit_i18n = i18n.clone();
                        let submit_i18n_status = i18n.clone();
                        match step.get() {
                            SetupStep::Admin => view! {
                                <h2 class="card-title text-2xl font-bold mb-4">
                                    {setup_admin_title.clone()}
                                </h2>
                                <p class="mb-6 text-base-content/70">
                                    {setup_admin_desc.clone()}
                                </p>

                                {move || setup_error.get().map(|err| view! {
                                    <div class="alert alert-error mb-4">
                                        <span>{err}</span>
                                    </div>
                                })}

                                <div class="grid grid-cols-2 gap-4">
                                    <div class="form-control w-full mb-4">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                {first_name_label.clone()}
                                            </RequiredLabel>
                                        </label>
                                        <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_admin_firstname.set(event_target_value(&ev)) />
                                    </div>
                                    <div class="form-control w-full mb-4">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                {last_name_label.clone()}
                                            </RequiredLabel>
                                        </label>
                                        <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_admin_lastname.set(event_target_value(&ev)) />
                                    </div>
                                </div>
                                <div class="form-control w-full mb-4">
                                    <label class="label">
                                        <RequiredLabel required=true>
                                            {email_label.clone()}
                                        </RequiredLabel>
                                    </label>
                                    <input type="email" class="input input-bordered w-full" required on:input=move |ev| set_admin_email.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full mb-6">
                                    <label class="label">
                                        <RequiredLabel required=true>
                                            {admin_password_label.clone()}
                                        </RequiredLabel>
                                    </label>
                                    <input type="password" class="input input-bordered w-full" required minlength="8" on:input=move |ev| set_admin_password.set(event_target_value(&ev)) />
                                </div>
                                <button
                                    class="btn btn-primary w-full"
                                    on:click=move |_| {
                                        let firstname = admin_firstname.get();
                                        let lastname = admin_lastname.get();
                                        let email = admin_email.get();
                                        let password = admin_password.get();
                                        if firstname.trim().is_empty()
                                            || lastname.trim().is_empty()
                                            || email.trim().is_empty()
                                            || password.trim().is_empty()
                                        {
                                            set_setup_error.set(Some(required_error.clone()));
                                            return;
                                        }
                                        if !is_valid_email(&email) {
                                            set_setup_error.set(Some(invalid_email_error.clone()));
                                            return;
                                        }
                                        set_setup_error.set(None);
                                        set_step.set(SetupStep::Tenant);
                                    }
                                >
                                    {continue_label.clone()}
                                </button>
                            }
                            .into_any(),

                            SetupStep::Tenant => view! {
                                <h2 class="card-title text-2xl font-bold mb-4">
                                    {setup_tenant_title.clone()}
                                </h2>
                                <p class="mb-6 text-base-content/70">
                                    {setup_tenant_desc.clone()}
                                </p>

                                <div class="form-control w-full mb-4">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                {tenant_name_label.clone()}
                                            </RequiredLabel>
                                        </label>
                                    <input type="text" placeholder="AgroCorp" class="input input-bordered w-full" required on:input=move |ev| set_tenant_name.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full mb-6">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                {tenant_slug_label.clone()}
                                            </RequiredLabel>
                                        </label>
                                    <input type="text" placeholder="agrocorp" class="input input-bordered w-full" required on:input=move |ev| set_tenant_slug.set(event_target_value(&ev)) />
                                </div>
                                <button
                                    class="btn btn-primary w-full"
                                    on:click=move |_| {
                                        let name = tenant_name.get();
                                        let slug = tenant_slug.get();
                                        if name.trim().is_empty() || slug.trim().is_empty() {
                                            set_setup_error.set(Some(required_error.clone()));
                                            return;
                                        }
                                        if !slug_is_valid(&slug) {
                                            set_setup_error.set(Some(invalid_slug_error.clone()));
                                            return;
                                        }
                                        set_setup_error.set(None);
                                        set_step.set(SetupStep::Company);
                                    }
                                >
                                    {continue_label.clone()}
                                </button>
                            }
                            .into_any(),

                            SetupStep::Company => view! {
                                <h2 class="card-title text-2xl font-bold mb-4">
                                    {setup_company_title.clone()}
                                </h2>
                                <p class="mb-6 text-base-content/70">
                                    {setup_company_desc.clone()}
                                </p>

                                <div class="form-control w-full mb-4">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                {company_name_label.clone()}
                                            </RequiredLabel>
                                        </label>
                                    <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_company_name.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full mb-4">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                {company_address_label.clone()}
                                            </RequiredLabel>
                                        </label>
                                    <textarea class="textarea textarea-bordered w-full" required on:input=move |ev| set_company_address.set(event_target_value(&ev))></textarea>
                                </div>
                                <div class="grid grid-cols-2 gap-4 mb-6">
                                    <div class="form-control w-full">
                                        <label class="label flex items-center justify-between">
                                            <RequiredLabel required=true>
                                                {company_country_label.clone()}
                                            </RequiredLabel>
                                            <span class="badge badge-outline">{move || country_flag(&company_country.get())}</span>
                                        </label>
                                        <input type="text" class="input input-bordered w-full" required on:input=move |ev| set_company_country.set(event_target_value(&ev)) />
                                    </div>
                                    <div class="form-control w-full">
                                        <label class="label">
                                            <span class="label-text">{company_email_label.clone()}</span>
                                        </label>
                                        <input type="email" class="input input-bordered w-full" on:input=move |ev| set_company_email.set(event_target_value(&ev)) />
                                    </div>
                                    <div class="form-control w-full">
                                        <label class="label">
                                            <span class="label-text flex items-center gap-2">
                                                <span>{company_phone_label.clone()}</span>
                                                <span class="badge badge-ghost badge-sm">{move || company_phone_prefix.get()}</span>
                                            </span>
                                        </label>
                                        <div class="grid grid-cols-[8rem_1fr] gap-2">
                                            <select class="select select-bordered w-full" prop:value=move || company_phone_prefix.get() on:change=move |ev| set_company_phone_prefix.set(event_target_value(&ev))>
                                                {PHONE_PREFIXES.iter().map(|(prefix, country)| {
                                                    view! {
                                                        <option value=*prefix>{format!("{prefix} {country}")}</option>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </select>
                                            <input type="tel" class="input input-bordered w-full" on:input=move |ev| set_company_phone_local.set(event_target_value(&ev)) />
                                        </div>
                                    </div>
                                </div>
                                <button
                                    class="btn btn-primary w-full"
                                    on:click=move |_| {
                                        let name = company_name.get();
                                        let address = company_address.get();
                                        let country = company_country.get();
                                        if name.trim().is_empty() || address.trim().is_empty() || country.trim().is_empty() {
                                            set_setup_error.set(Some(required_error.clone()));
                                            return;
                                        }
                                        if let Some(email) = {
                                            let value = company_email.get();
                                            if value.trim().is_empty() {
                                                None
                                            } else {
                                                Some(value)
                                            }
                                        } {
                                            if !is_valid_email(&email) {
                                                set_setup_error.set(Some(invalid_email_error.clone()));
                                                return;
                                            }
                                        }
                                        let phone_local = company_phone_local.get();
                                        if !phone_local.trim().is_empty() {
                                            let prefix = company_phone_prefix.get();
                                            let phone = normalize_phone(&prefix, &phone_local);
                                            if phone.is_none() {
                                                set_setup_error.set(Some(invalid_phone_error.clone()));
                                                return;
                                            }
                                        }
                                        set_setup_error.set(None);
                                        set_step.set(SetupStep::Resources);
                                    }
                                >
                                    {continue_label.clone()}
                                </button>
                            }
                            .into_any(),

                            SetupStep::Resources => view! {
                                <h2 class="card-title text-2xl font-bold mb-4">
                                    {setup_resources_title.clone()}
                                </h2>
                                <p class="mb-6 text-base-content/70">
                                    {setup_resources_desc.clone()}
                                </p>

                                <div class="form-control w-full mb-4">
                                    <label class="label"><span class="label-text">{setup_resource_type.clone()}</span></label>
                                    <select class="select select-bordered w-full" on:change=move |ev| set_resource_type.set(event_target_value(&ev))>
                                        <option value="field_management">{field_management_label.clone()}</option>
                                        <option value="PlantProtection">{task_protection_label.clone()}</option>
                                        <option value="CostTracking">{resources_label.clone()}</option>
                                    </select>
                                </div>

                                <div class="divider">{setup_equipment.clone()}</div>
                                <div class="form-control w-full mb-6">
                                    <label class="label"><span class="label-text">{setup_first_machine.clone()}</span></label>
                                    <input type="text" placeholder={setup_first_machine_placeholder.clone()} class="input input-bordered w-full" />
                                </div>

                                <button
                                    class="btn btn-success w-full"
                                    class:btn-disabled=move || setup_status.get().map(|s| s.is_ok()).unwrap_or(false)
                                    on:click=move |_| {
                                        submit_setup(
                                            submit_i18n.clone(),
                                            lang.get(),
                                            admin_firstname.get(),
                                            admin_lastname.get(),
                                            admin_email.get(),
                                            admin_password.get(),
                                            tenant_name.get(),
                                            tenant_slug.get(),
                                            company_name.get(),
                                            company_address.get(),
                                            company_country.get(),
                                            company_email.get(),
                                            company_phone_prefix.get(),
                                            company_phone_local.get(),
                                            set_setup_status,
                                            set_setup_error,
                                        );
                                    }
                                >
                                    {finish_label.clone()}
                                </button>
                                {move || match setup_status.get() {
                                    Some(Err(msg)) if msg == starting_setup_label => view! {
                                        <p class="mt-3 text-sm text-info">{setup_t(&submit_i18n_status, lang.get(), "redirecting")}</p>
                                    }.into_any(),
                                    Some(Err(msg)) => view! {
                                        <div class="alert alert-error mt-3">
                                            <span>{msg}</span>
                                        </div>
                                    }.into_any(),
                                    _ => view! {
                                        <div class="hidden"></div>
                                    }.into_any(),
                                }}
                            }
                            .into_any(),
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
