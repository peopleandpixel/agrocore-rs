use crate::api;
use crate::components::form::{
    PHONE_PREFIXES, RequiredLabel, country_flag, is_valid_email, language_flag, normalize_phone,
};
use crate::components::toast::{ToastContext, ToastType};
use crate::i18n::LANGUAGE_OPTIONS;
use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum SetupStep {
    Admin,
    Tenant,
    Company,
}

fn setup_t(i18n: &I18n, lang: Language, key: &str) -> String {
    i18n.t(lang.as_str(), key)
}

fn setup_logo_data_url() -> &'static str {
    use std::sync::OnceLock;

    static LOGO_DATA_URL: OnceLock<String> = OnceLock::new();
    LOGO_DATA_URL
        .get_or_init(|| {
            let bytes = include_bytes!("../../public/logo_trans.png");
            const TABLE: &[u8; 64] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

            for chunk in bytes.chunks(3) {
                let b0 = chunk[0];
                let b1 = *chunk.get(1).unwrap_or(&0);
                let b2 = *chunk.get(2).unwrap_or(&0);
                let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

                encoded.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
                encoded.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
                if chunk.len() > 1 {
                    encoded.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
                } else {
                    encoded.push('=');
                }
                if chunk.len() > 2 {
                    encoded.push(TABLE[(n & 0x3f) as usize] as char);
                } else {
                    encoded.push('=');
                }
            }

            format!("data:image/png;base64,{encoded}")
        })
        .as_str()
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
    toast_context: crate::components::toast::ToastContext,
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
                    let primary_role = auth
                        .roles
                        .iter()
                        .find(|role| {
                            matches!(
                                role.to_lowercase().as_str(),
                                "admin" | "manager" | "worker" | "viewer"
                            )
                        })
                        .cloned()
                        .unwrap_or_else(|| String::from("Viewer"));
                    api::set_user_role(&primary_role);
                    api::save_company_profile(&company_profile);
                    if let Some(storage) = window().local_storage().ok().flatten() {
                        let _ = storage.set_item("agrocore.lang", &selected_language);
                    }
                    set_setup_status.set(Some(Ok(())));
                    toast_context.add_toast.run((
                        setup_t(&i18n, lang, "setup_complete"),
                        crate::components::toast::ToastType::Success,
                    ));
                    let _ = window().location().reload();
                }
                Err(e) => {
                    toast_context
                        .add_toast
                        .run((e.clone(), crate::components::toast::ToastType::Error));
                    set_setup_status.set(Some(Err(e)));
                }
            },
            Err(e) => {
                toast_context
                    .add_toast
                    .run((e.clone(), crate::components::toast::ToastType::Error));
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

    let toast_context = use_context::<ToastContext>().expect("ToastContext not provided");

    let i18n_for_title = i18n;
    let i18n_for_admin_step = i18n;
    let i18n_for_tenant_step = i18n;
    let i18n_for_company_step = i18n;
    let i18n_for_language_options = i18n;
    let setup_title = move || setup_t(&i18n_for_title, lang.get(), "setup_title");
    let finish_label = move || setup_t(&i18n, lang.get(), "finish_setup");
    let i18n_for_welcome = i18n;
    let _setup_welcome = move || setup_t(&i18n_for_welcome, lang.get(), "setup_welcome");
    let starting_setup_label_loading = move || setup_t(&i18n, lang.get(), "starting_setup");
    let starting_setup_label_loading_memo = starting_setup_label_loading;
    let is_submitting = Memo::new(
        move |_| matches!(setup_status.get(), Some(Err(ref msg)) if msg == &starting_setup_label_loading_memo()),
    );
    let setup_progress = move || match step.get() {
        SetupStep::Admin => 33,
        SetupStep::Tenant => 66,
        SetupStep::Company => 100,
    };
    view! {
        <div class="min-h-screen bg-base-200 flex flex-col items-center justify-center p-4 relative">
            <div class="absolute top-6 right-6 z-50">
                <select
                    class="select select-bordered select-sm"
                    prop:value=move || lang.get().as_str().to_string()
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        set_lang.set(Language::from_str(&value));
                        if let Some(storage) = window().local_storage().ok().flatten() {
                            let _ = storage.set_item("agrocore.lang", &value);
                        }
                    }
                >
                    {LANGUAGE_OPTIONS.iter().map(|(language, code, label_key)| {
                        let i18n = i18n_for_language_options;
                        let selected = move || lang.get() == *language;
                        view! {
                            <option value=*code selected=selected>
                                {move || format!("{} {}", language_flag(code), i18n.t(lang.get().as_str(), label_key))}
                            </option>
                        }
                    }).collect::<Vec<_>>()}
                </select>
            </div>

            <div class="text-center space-y-4 mb-8">
                <div class="flex flex-col items-center gap-4">

                                    <img
                                        src=setup_logo_data_url()
                                        alt="AgroCore Logo"
                                        class=move || format!("w-48 h-48 p-3 object-contain {}", if is_submitting.get() { "pulse" } else { "" })
                                    />




                    <div>
                        <p class="text-lg text-base-content/60 font-medium mt-2">{move || setup_title()}</p>
                    </div>
                </div>
            </div>

            <div class="w-full max-w-2xl space-y-8">
                <div class="card bg-base-100 shadow-2xl border border-base-300 overflow-hidden">
                    <div class="h-2 bg-base-200 w-full">
                        <div
                            class="h-full bg-primary transition-all duration-500 ease-in-out"
                            style:width=move || format!("{}%", setup_progress())
                        ></div>
                    </div>
                    <div class="card-body p-8 gap-6">
                            {move || {
                        let current_lang = lang.get();
                        let i18n_for_t = i18n;
                        let t = move |key: &str| setup_t(&i18n_for_t, current_lang, key);
                        let setup_admin_title = t("setup_admin_title");
                        let setup_admin_desc = t("setup_admin_desc");
                        let setup_tenant_title = t("setup_tenant_title");
                        let setup_tenant_desc = t("setup_tenant_desc");
                        let setup_company_title = t("setup_company_title");
                        let setup_company_desc = t("setup_company_desc");
                        let setup_admin_help = t("setup_admin_help");
                        let setup_tenant_help = t("setup_tenant_help");
                        let setup_company_help = t("setup_company_help");
                        let optional_label = t("optional_label");
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
                        let continue_label = t("continue");
                        let required_error = t("validation_required");
                        let invalid_email_error = t("validation_invalid_email");
                        let invalid_phone_error = t("validation_invalid_phone");
                        let invalid_slug_error = t("validation_invalid_slug");
                        let _starting_setup_label_loading_class = starting_setup_label_loading;
                        let _starting_setup_label_loading_disabled = starting_setup_label_loading;
                        let starting_setup_label_status = starting_setup_label_loading;
                        let submit_i18n = i18n;
                        let submit_i18n_status = i18n;
                        match step.get() {
                            SetupStep::Admin => view! {
                                <div class="space-y-5 animate-slide-up">
                                    <div>
                                        <h3 class="text-lg font-semibold">{setup_admin_title.clone()}</h3>
                                        <p class="text-sm text-base-content/70">{setup_admin_desc.clone()}</p>
                                        <p class="mt-2 text-sm leading-6 text-base-content/60">
                                            {setup_admin_help.clone()}
                                        </p>
                                    </div>

                                    <div class="grid gap-4 md:grid-cols-2">
                                        <div class="form-control w-full">
                                            <label class="label">
                                                <RequiredLabel required=true>
                                                    {first_name_label.clone()}
                                                </RequiredLabel>
                                            </label>
                                            <input type="text" class="input input-bordered w-full" autocomplete="given-name" required on:input=move |ev| set_admin_firstname.set(event_target_value(&ev)) />
                                        </div>
                                        <div class="form-control w-full">
                                            <label class="label">
                                                <RequiredLabel required=true>
                                                    {last_name_label.clone()}
                                                </RequiredLabel>
                                            </label>
                                            <input type="text" class="input input-bordered w-full" autocomplete="family-name" required on:input=move |ev| set_admin_lastname.set(event_target_value(&ev)) />
                                        </div>
                                    </div>

                                    <div class="grid gap-4 md:grid-cols-2">
                                        <div class="form-control w-full">
                                            <label class="label">
                                                <RequiredLabel required=true>
                                                    {email_label.clone()}
                                                </RequiredLabel>
                                            </label>
                                            <input type="email" class="input input-bordered w-full" autocomplete="email" required on:input=move |ev| set_admin_email.set(event_target_value(&ev)) />
                                        </div>
                                        <div class="form-control w-full">
                                            <label class="label">
                                                <RequiredLabel required=true>
                                                    {admin_password_label.clone()}
                                                </RequiredLabel>
                                            </label>
                                            <input type="password" class="input input-bordered w-full" autocomplete="new-password" required minlength="8" on:input=move |ev| set_admin_password.set(event_target_value(&ev)) />
                                        </div>
                                    </div>

                                    <div class="flex justify-end">
                                        <button
                                            class="btn btn-primary min-w-40"
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
                                                    toast_context.add_toast.run((required_error.clone(), ToastType::Warning));
                                                    return;
                                                }
                                                if !is_valid_email(&email) {
                                                    toast_context.add_toast.run((invalid_email_error.clone(), ToastType::Warning));
                                                    return;
                                                }
                                                set_setup_error.set(None);
                                                set_step.set(SetupStep::Tenant);
                                                toast_context.add_toast.run((setup_tenant_title.clone(), ToastType::Info));
                                            }
                                            disabled=move || is_submitting.get()
                                        >
                                            {continue_label.clone()}
                                        </button>
                                    </div>
                                </div>
                            }
                            .into_any(),

                            SetupStep::Tenant => view! {
                                <div class="space-y-5 animate-slide-up">
                                    <div>
                                        <h3 class="text-lg font-semibold">{setup_tenant_title.clone()}</h3>
                                        <p class="text-sm text-base-content/70">{setup_tenant_desc.clone()}</p>
                                        <p class="mt-2 text-sm leading-6 text-base-content/60">
                                            {setup_tenant_help.clone()}
                                        </p>
                                    </div>

                                    <div class="form-control w-full">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                {tenant_name_label.clone()}
                                            </RequiredLabel>
                                        </label>
                                        <input type="text" placeholder="AgroCorp" class="input input-bordered w-full" autocomplete="organization" required on:input=move |ev| set_tenant_name.set(event_target_value(&ev)) />
                                    </div>

                                    <div class="form-control w-full">
                                        <label class="label">
                                            <RequiredLabel required=true>
                                                {tenant_slug_label.clone()}
                                            </RequiredLabel>
                                        </label>
                                        <input type="text" placeholder="agrocorp" class="input input-bordered w-full" autocomplete="off" autocapitalize="off" spellcheck="false" required on:input=move |ev| set_tenant_slug.set(event_target_value(&ev)) />
                                    </div>

                                    <div class="flex justify-end">
                                        <button
                                            class="btn btn-primary min-w-40"
                                            on:click=move |_| {
                                                let name = tenant_name.get();
                                                let slug = tenant_slug.get();
                                                if name.trim().is_empty() || slug.trim().is_empty() {
                                                    toast_context.add_toast.run((required_error.clone(), ToastType::Warning));
                                                    return;
                                                }
                                                if !slug_is_valid(&slug) {
                                                    toast_context.add_toast.run((invalid_slug_error.clone(), ToastType::Warning));
                                                    return;
                                                }
                                                set_setup_error.set(None);
                                                set_step.set(SetupStep::Company);
                                                toast_context.add_toast.run((setup_company_title.clone(), ToastType::Info));
                                            }
                                            disabled=move || is_submitting.get()
                                        >
                                            {continue_label.clone()}
                                        </button>
                                    </div>
                                </div>
                            }
                            .into_any(),

                            SetupStep::Company => view! {
                                <div class="space-y-5 animate-slide-up">
                                    <div>
                                        <h3 class="text-lg font-semibold">{setup_company_title.clone()}</h3>
                                        <p class="text-sm text-base-content/70">{setup_company_desc.clone()}</p>
                                        <p class="mt-2 text-sm leading-6 text-base-content/60">
                                            {setup_company_help.clone()}
                                        </p>
                                    </div>

                                    <div class="grid gap-4">
                                        <div class="form-control w-full">
                                            <label class="label">
                                                <RequiredLabel required=true>
                                                    {company_name_label.clone()}
                                                </RequiredLabel>
                                            </label>
                                            <input type="text" class="input input-bordered w-full" autocomplete="organization" required on:input=move |ev| set_company_name.set(event_target_value(&ev)) />
                                        </div>
                                        <div class="form-control w-full">
                                            <label class="label">
                                                <RequiredLabel required=true>
                                                    {company_address_label.clone()}
                                                </RequiredLabel>
                                            </label>
                                            <textarea class="textarea textarea-bordered w-full min-h-28" autocomplete="street-address" required on:input=move |ev| set_company_address.set(event_target_value(&ev))></textarea>
                                        </div>
                                    </div>

                                    <div class="grid gap-4 md:grid-cols-2">
                                        <div class="form-control w-full">
                                            <label class="label flex items-center justify-between">
                                                <RequiredLabel required=true>
                                                    {company_country_label.clone()}
                                                </RequiredLabel>
                                                <span class="badge">{move || country_flag(&company_country.get())}</span>
                                            </label>
                                            <input type="text" class="input input-bordered w-full" autocomplete="country-name" required on:input=move |ev| set_company_country.set(event_target_value(&ev)) />
                                        </div>
                                        <div class="form-control w-full">
                                            <label class="label">
                                                <span class="label-text flex items-center gap-2">
                                                    <span>{company_email_label.clone()}</span>
                                                    <span class="badge badge-ghost badge-sm">{optional_label.clone()}</span>
                                                </span>
                                            </label>
                                            <input type="email" class="input input-bordered w-full" autocomplete="email" on:input=move |ev| set_company_email.set(event_target_value(&ev)) />
                                        </div>
                                    </div>

                                    <div class="form-control w-full">
                                        <label class="label">
                                            <span class="label-text flex items-center gap-2">
                                                <span>{company_phone_label.clone()}</span>
                                                <span class="badge badge-ghost badge-sm">{move || company_phone_prefix.get()}</span>
                                            </span>
                                        </label>
                                        <div class="join w-full">
                                            <select class="select select-bordered join-item w-36" prop:value=move || company_phone_prefix.get() on:change=move |ev| set_company_phone_prefix.set(event_target_value(&ev))>
                                                {PHONE_PREFIXES.iter().map(|(prefix, country)| {
                                                    view! {
                                                        <option value=*prefix>{format!("{prefix} {country}")}</option>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </select>
                                            <input type="tel" class="input input-bordered join-item w-full" autocomplete="tel" on:input=move |ev| set_company_phone_local.set(event_target_value(&ev)) />
                                        </div>
                                    </div>

                                    <div class="flex justify-end">
                                        <button
                                            class="btn btn-primary min-w-40"
                                            on:click=move |_| {
                                                if is_submitting.get() { return; }
                                                let name = company_name.get();
                                                let address = company_address.get();
                                                let country = company_country.get();
                                                if name.trim().is_empty() || address.trim().is_empty() || country.trim().is_empty() {
                                                    toast_context.add_toast.run((required_error.clone(), ToastType::Warning));
                                                    return;
                                                }
                                                let email_opt = {
                                                    let value = company_email.get();
                                                    if value.trim().is_empty() {
                                                        None
                                                    } else {
                                                        Some(value)
                                                    }
                                                };
                                                if let Some(email) = email_opt
                                                    && !is_valid_email(&email) {
                                                        toast_context.add_toast.run((invalid_email_error.clone(), ToastType::Warning));
                                                        return;
                                                }
                                                let phone_local = company_phone_local.get();
                                                if !phone_local.trim().is_empty() {
                                                    let prefix = company_phone_prefix.get();
                                                    let phone = normalize_phone(&prefix, &phone_local);
                                                    if phone.is_none() {
                                                        toast_context.add_toast.run((invalid_phone_error.clone(), ToastType::Warning));
                                                        return;
                                                    }
                                                }
                                                set_setup_error.set(None);
                                                submit_setup(
                                                    submit_i18n,
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
                                                    toast_context,
                                                );
                                            }
                                            disabled=move || is_submitting.get()
                                        >
                                            {move || if is_submitting.get() {
                                                view! { <span class="loading loading-spinner loading-sm"></span> }.into_any()
                                            } else {
                                                view! { <Icon icon=LuCircleCheck width="20" height="20" /> }.into_any()
                                            }}
                                            {move || finish_label()}
                                        </button>
                                    </div>

                                    {move || match setup_status.get() {
                                        Some(Err(msg)) if msg == starting_setup_label_status() => view! {
                                            <div class="alert alert-info shadow-lg border-info/20">
                                                <div class="loading loading-spinner loading-md"></div>
                                                <span>{setup_t(&submit_i18n_status, lang.get(), "redirecting")}</span>
                                            </div>
                                        }.into_any(),
                                        Some(Err(msg)) => view! {
                                            <div class="alert alert-error shadow-lg border-error/20">
                                                <Icon icon=LuCircleAlert width="20" height="20" />
                                                <span>{msg}</span>
                                            </div>
                                        }.into_any(),
                                        _ => view! {
                                            <div class="hidden"></div>
                                        }.into_any(),
                                    }}
                                </div>
                            }
                            .into_any(),
                            }}}
                        </div>
                    </div>

                    <ul class="steps steps-horizontal w-full">
                        <li class=move || format!("step {}", if step.get() >= SetupStep::Admin { "step-primary" } else { "" })>{move || setup_t(&i18n_for_admin_step, lang.get(), "setup_step_admin")}</li>
                        <li class=move || format!("step {}", if step.get() >= SetupStep::Tenant { "step-primary" } else { "" })>{move || setup_t(&i18n_for_tenant_step, lang.get(), "setup_step_tenant")}</li>
                        <li class=move || format!("step {}", if step.get() >= SetupStep::Company { "step-primary" } else { "" })>{move || setup_t(&i18n_for_company_step, lang.get(), "setup_step_company")}</li>
                    </ul>

                    {move || setup_error.get().map(|err| view! {
                        <div class="alert alert-error shadow-lg">
                            <Icon icon=LuTriangleAlert width="20" height="20" />
                            <span>{err}</span>
                        </div>
                    })}
                </div>
            </div>
    }
}
