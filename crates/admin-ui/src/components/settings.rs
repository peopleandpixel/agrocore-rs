use crate::api;
use crate::components::form::{
    country_flag, is_valid_email, language_flag, normalize_phone, split_phone, RequiredLabel,
    PHONE_PREFIXES,
};
use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
#[component]
pub fn SettingsPage() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n engine");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let set_lang = use_context::<WriteSignal<Language>>().expect("set lang signal");
    let initial_profile = api::load_company_profile().unwrap_or_default();

    let settings_title = i18n.t(lang.get().as_str(), "settings");
    let company_profile = i18n.t(lang.get().as_str(), "company_profile");
    let company_name_label = i18n.t(lang.get().as_str(), "company_name");
    let tax_id_label = i18n.t(lang.get().as_str(), "tax_id");
    let office_email_label = i18n.t(lang.get().as_str(), "office_email");
    let international_phone = i18n.t(lang.get().as_str(), "international_phone");
    let address_label = i18n.t(lang.get().as_str(), "address");
    let website_label = i18n.t(lang.get().as_str(), "website");
    let country_label = i18n.t(lang.get().as_str(), "country");
    let update_profile = i18n.t(lang.get().as_str(), "update_profile");
    let display_and_language = i18n.t(lang.get().as_str(), "display_and_language");
    let system_language = i18n.t(lang.get().as_str(), "system_language");
    let timezone_label = i18n.t(lang.get().as_str(), "timezone");
    let timezone_europe_berlin = i18n.t(lang.get().as_str(), "timezone_europe_berlin");
    let timezone_utc = i18n.t(lang.get().as_str(), "timezone_utc");
    let system_status = i18n.t(lang.get().as_str(), "system_status");
    let api_connection = i18n.t(lang.get().as_str(), "api_connection");
    let online = i18n.t(lang.get().as_str(), "online");
    let database = i18n.t(lang.get().as_str(), "database");
    let connected = i18n.t(lang.get().as_str(), "connected");
    let version = i18n.t(lang.get().as_str(), "version");
    let export_system_log = i18n.t(lang.get().as_str(), "export_system_log");
    let danger_zone = i18n.t(lang.get().as_str(), "danger_zone");
    let danger_zone_desc = i18n.t(lang.get().as_str(), "danger_zone_desc");
    let export_all_data = i18n.t(lang.get().as_str(), "export_all_data");
    let delete_tenant = i18n.t(lang.get().as_str(), "delete_tenant");
    let validation_required_company = i18n.t(lang.get().as_str(), "validation_required_company");
    let validation_invalid_email = i18n.t(lang.get().as_str(), "validation_invalid_email");
    let validation_invalid_phone = i18n.t(lang.get().as_str(), "validation_invalid_phone");
    let profile_saved = i18n.t(lang.get().as_str(), "profile_saved");
    let system_log_title = i18n.t(lang.get().as_str(), "system_log_title");
    let time_label = i18n.t(lang.get().as_str(), "time");
    let initialized_label = i18n.t(lang.get().as_str(), "initialized");
    let (company_name, set_company_name) = signal(initial_profile.company_name.unwrap_or_default());
    let (tax_id, set_tax_id) = signal(initial_profile.tax_id.unwrap_or_default());
    let (office_email, set_office_email) = signal(initial_profile.office_email.unwrap_or_default());
    let (phone_prefix, set_phone_prefix) = signal({
        let (prefix, _) = split_phone(initial_profile.phone.as_deref().unwrap_or_default(), "+351");
        prefix
    });
    let (phone_local, set_phone_local) = signal({
        let (_, local) = split_phone(initial_profile.phone.as_deref().unwrap_or_default(), "+351");
        local
    });
    let (address, set_address) = signal(initial_profile.address.unwrap_or_default());
    let (website, set_website) = signal(initial_profile.website.unwrap_or_default());
    let (country, set_country) = signal(initial_profile.country.unwrap_or_default());
    let (status_message, set_status_message) = signal(None::<String>);

    let blank_to_none = |value: String| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    };

    let on_update_profile = move |_| {
        let company_name_value = company_name.get();
        let address_value = address.get();
        let country_value = country.get();
        let office_email_value = office_email.get();
        let phone_local_value = phone_local.get();
        let phone_prefix_value = phone_prefix.get();

        if company_name_value.trim().is_empty()
            || address_value.trim().is_empty()
            || country_value.trim().is_empty()
        {
            set_status_message.set(Some(validation_required_company.clone()));
            return;
        }

        if !office_email_value.trim().is_empty() && !is_valid_email(&office_email_value) {
            set_status_message.set(Some(validation_invalid_email.clone()));
            return;
        }

        let phone = if phone_local_value.trim().is_empty() {
            None
        } else {
            match normalize_phone(&phone_prefix_value, &phone_local_value) {
                Some(value) => Some(value),
                None => {
                    set_status_message.set(Some(validation_invalid_phone.clone()));
                    return;
                }
            }
        };

        let profile = api::CompanyProfile {
            company_name: blank_to_none(company_name_value),
            tax_id: blank_to_none(tax_id.get()),
            office_email: blank_to_none(office_email_value),
            phone,
            address: blank_to_none(address_value),
            website: blank_to_none(website.get()),
            country: blank_to_none(country_value),
        };
        api::save_company_profile(&profile);
        set_status_message.set(Some(profile_saved.clone()));
    };

    let tenant_name_log_label = i18n.t(lang.get().as_str(), "tenant_name");
    let email_log_label = i18n.t(lang.get().as_str(), "email");
    let address_log_label = address_label.clone();
    let country_log_label = country_label.clone();

    let on_export_system_log = move |_| {
        let company_name = company_name.get();
        let office_email = office_email.get();
        let address = address.get();
        let country = country.get();
        let system_log_title = system_log_title.clone();
        let time_label = time_label.clone();
        let initialized_label = initialized_label.clone();
        let tenant_name_log_label = tenant_name_log_label.clone();
        let email_log_label = email_log_label.clone();
        let address_log_label = address_log_label.clone();
        let country_log_label = country_log_label.clone();

        spawn_local(async move {
            let mut lines = vec![
                system_log_title,
                format!("{}: {}", time_label, chrono::Utc::now().to_rfc3339()),
            ];

            if let Ok(status) = api::fetch_system_status().await {
                lines.push(format!("{}: {}", initialized_label, status.initialized));
            }
            lines.push(format!("{}: {}", tenant_name_log_label, company_name));
            lines.push(format!("{}: {}", email_log_label, office_email));
            lines.push(format!("{}: {}", address_log_label, address));
            lines.push(format!("{}: {}", country_log_label, country));

            let _ =
                api::download_bytes("system-log.txt", "text/plain", lines.join("\n").as_bytes());
        });
    };

    let on_export_all_data = move |_| {
        spawn_local(async move {
            if let Ok(bytes) = api::export_orders_excel().await {
                let _ = api::download_bytes(
                    "orders.xlsx",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    &bytes,
                );
            }
            if let Ok(text) = api::export_sites_geojson().await {
                let _ =
                    api::download_bytes("sites.geojson", "application/geo+json", text.as_bytes());
            }
            if let Ok(bytes) = api::export_pac_sip().await {
                let _ = api::download_bytes(
                    "pac_sip_report.xlsx",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    &bytes,
                );
            }
            if let Ok(bytes) = api::export_veterinary().await {
                let _ = api::download_bytes(
                    "veterinary_report.xlsx",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    &bytes,
                );
            }
        });
    };

    let on_delete_tenant = move |_| {
        spawn_local(async move {
            match api::delete_tenant().await {
                Ok(_) => {
                    api::clear_auth_token();
                    let _ = leptos::prelude::window().location().reload();
                }
                Err(e) => set_status_message.set(Some(e)),
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <h1 class="text-3xl font-bold">{settings_title}</h1>

            {move || status_message.get().map(|message| view! {
                <div class="alert alert-success">
                    <span>{message}</span>
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div class="lg:col-span-2 space-y-6">
                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title mb-4"><Icon icon=LuBriefcase width="20" height="20" /> {company_profile}</h2>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="form-control w-full">
                                    <label class="label"><RequiredLabel required=true>{company_name_label}</RequiredLabel></label>
                                    <input type="text" class="input input-bordered w-full" required prop:value=move || company_name.get() on:input=move |ev| set_company_name.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">{tax_id_label}</span></label>
                                    <input type="text" class="input input-bordered w-full" prop:value=move || tax_id.get() on:input=move |ev| set_tax_id.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">{office_email_label}</span></label>
                                    <input type="email" class="input input-bordered w-full" prop:value=move || office_email.get() on:input=move |ev| set_office_email.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">{international_phone}</span></label>
                                    <div class="grid grid-cols-[8rem_1fr] gap-2">
                                        <select class="select select-bordered w-full" prop:value=move || phone_prefix.get() on:change=move |ev| set_phone_prefix.set(event_target_value(&ev))>
                                            {PHONE_PREFIXES.iter().map(|(prefix, country)| view! {
                                                <option value=*prefix>{format!("{prefix} {country}")}</option>
                                            }).collect::<Vec<_>>()}
                                        </select>
                                        <input type="tel" class="input input-bordered w-full" prop:value=move || phone_local.get() on:input=move |ev| set_phone_local.set(event_target_value(&ev)) />
                                    </div>
                                </div>
                                <div class="form-control w-full md:col-span-2">
                                    <label class="label"><RequiredLabel required=true>{address_label}</RequiredLabel></label>
                                    <input type="text" class="input input-bordered w-full" required prop:value=move || address.get() on:input=move |ev| set_address.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full md:col-span-2">
                                    <label class="label"><span class="label-text">{website_label}</span></label>
                                    <input type="url" class="input input-bordered w-full" prop:value=move || website.get() on:input=move |ev| set_website.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full md:col-span-2">
                                    <label class="label flex items-center justify-between">
                                        <RequiredLabel required=true>{country_label}</RequiredLabel>
                                        <span class="badge badge-outline">{move || country_flag(&country.get())}</span>
                                    </label>
                                    <input type="text" class="input input-bordered w-full" required prop:value=move || country.get() on:input=move |ev| set_country.set(event_target_value(&ev)) />
                                </div>
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <button class="btn btn-primary" on:click=on_update_profile>{update_profile}</button>
                            </div>
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title mb-4"><Icon icon=LuGlobe width="20" height="20" /> {display_and_language}</h2>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">{system_language}</span></label>
                                    <select
                                        class="select select-bordered"
                                        on:change=move |ev| {
                                            let val = event_target_value(&ev);
                                            set_lang.set(Language::from_str(&val));
                                            if let Some(storage) = leptos::prelude::window().local_storage().ok().flatten() {
                                                let _ = storage.set_item("agrocore.lang", &val);
                                            }
                                        }
                                    >
                                        <option value="de" selected=move || lang.get() == Language::DE>{format!("{} {}", language_flag("de"), i18n.t(lang.get().as_str(), "language_de"))}</option>
                                        <option value="en" selected=move || lang.get() == Language::EN>{format!("{} {}", language_flag("en"), i18n.t(lang.get().as_str(), "language_en"))}</option>
                                        <option value="es" selected=move || lang.get() == Language::ES>{format!("{} {}", language_flag("es"), i18n.t(lang.get().as_str(), "language_es"))}</option>
                                        <option value="fr" selected=move || lang.get() == Language::FR>{format!("{} {}", language_flag("fr"), i18n.t(lang.get().as_str(), "language_fr"))}</option>
                                        <option value="pt" selected=move || lang.get() == Language::PT>{format!("{} {}", language_flag("pt"), i18n.t(lang.get().as_str(), "language_pt"))}</option>
                                    </select>
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">{timezone_label}</span></label>
                                    <select class="select select-bordered">
                                        <option selected>{timezone_europe_berlin}</option>
                                        <option>{timezone_utc}</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="space-y-6">
                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title"><Icon icon=LuShieldCheck width="20" height="20" /> {system_status}</h2>
                            <ul class="space-y-2 mt-2">
                                <li class="flex justify-between items-center text-sm">
                                    <span>{api_connection}</span>
                                    <span class="badge badge-success badge-sm">{online}</span>
                                </li>
                                <li class="flex justify-between items-center text-sm">
                                    <span>{database}</span>
                                    <span class="badge badge-success badge-sm">{connected}</span>
                                </li>
                                <li class="flex justify-between items-center text-sm">
                                    <span>{version}</span>
                                    <span class="font-mono text-xs">"v0.4.2-stable"</span>
                                </li>
                            </ul>
                            <div class="divider"></div>
                            <button class="btn btn-outline btn-sm w-full" on:click=on_export_system_log>{export_system_log}</button>
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow border-2 border-error/20">
                        <div class="card-body">
                            <h2 class="card-title text-error"><Icon icon=LuTrash2 width="20" height="20" /> {danger_zone}</h2>
                            <p class="text-xs opacity-70">{danger_zone_desc}</p>
                            <div class="space-y-2 mt-4">
                                <button class="btn btn-outline btn-error btn-sm w-full" on:click=on_export_all_data>{export_all_data}</button>
                                <button class="btn btn-error btn-sm w-full" on:click=on_delete_tenant>{delete_tenant}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
