use leptos::prelude::*;
use crate::i18n::{I18n, Language};
use crate::UserRole;

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum SetupStep {
    Login,
    TenantCreation,
    CompanyCreation,
    ResourcesCreation,
}

#[component]
pub fn SetupAssistant() -> impl IntoView {
    let (lang, set_lang) = signal(Language::DE);
    let i18n = I18n::new();
    let (step, set_step) = signal(SetupStep::Login);
    
    // Step 0: Login
    let (login_username, set_login_username) = signal(String::new());
    let (login_password, set_login_password) = signal(String::new());
    let (login_error, set_login_error) = signal(None::<String>);

    // Step 1: Tenant
    let (tenant_name, set_tenant_name) = signal(String::new());
    let (tenant_slug, set_tenant_slug) = signal(String::new());

    // Step 2: Company
    let (company_name, set_company_name) = signal(String::new());
    let (company_address, set_company_address) = signal(String::new());
    let (company_email, set_company_email) = signal(String::new());
    let (company_phone, set_company_phone) = signal(String::new());

    // Step 3: Resources
    let (resource_type, set_resource_type) = signal("Vineyard".to_string());

    let (setup_status, set_setup_status) = signal(None::<Result<(), String>>);

    let on_finish = move |_| {
        spawn_local(async move {
            let admin = serde_json::json!({
                "firstname": "Admin",
                "lastname": "User",
                "email": "admin@agrocore.local",
                "password": "agrocore"
            });
            
            let tenant = serde_json::json!({
                "name": tenant_name.get(),
                "slug": tenant_slug.get()
            });

            let req = crate::api::InitialSetupRequest {
                admin,
                tenant,
            };

            match crate::api::initial_setup(req).await {
                Ok(_) => {
                    set_setup_status.set(Some(Ok(())));
                    window().location().reload().unwrap();
                }
                Err(e) => {
                    set_setup_status.set(Some(Err(e)));
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-base-200 flex flex-col items-center justify-center p-4">
            <div class="mb-8 flex flex-col items-center">
                <h1 class="text-5xl font-black mb-2 text-primary">"AgroCore"</h1>
                <p class="text-base-content/50 uppercase tracking-widest font-bold">"Setup Wizard"</p>
            </div>

            <ul class="steps mb-8 w-full max-w-2xl">
                <li class=move || format!("step {}", if step.get() >= SetupStep::Login { "step-primary" } else { "" })>"Login"</li>
                <li class=move || format!("step {}", if step.get() >= SetupStep::TenantCreation { "step-primary" } else { "" })>"Tenant"</li>
                <li class=move || format!("step {}", if step.get() >= SetupStep::CompanyCreation { "step-primary" } else { "" })>"Company"</li>
                <li class=move || format!("step {}", if step.get() >= SetupStep::ResourcesCreation { "step-primary" } else { "" })>"Resources"</li>
            </ul>

            <div class="card w-full max-w-lg bg-base-100 shadow-2xl border border-base-300">
                <div class="card-body">
                    {move || match step.get() {
                        SetupStep::Login => view! {
                            <h2 class="card-title text-2xl font-bold mb-4">"Initial Login"</h2>
                            <p class="mb-6 text-base-content/70">"Please log in with the default administrator credentials."</p>
                            
                            {move || login_error.get().map(|err| view! {
                                <div class="alert alert-error mb-4">
                                    <span>{err}</span>
                                </div>
                            })}

                            <div class="form-control w-full mb-4">
                                <label class="label"><span class="label-text">"Username"</span></label>
                                <input type="text" placeholder="admin" class="input input-bordered w-full" 
                                    on:input=move |ev| set_login_username.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control w-full mb-6">
                                <label class="label"><span class="label-text">"Password"</span></label>
                                <input type="password" class="input input-bordered w-full" 
                                    on:input=move |ev| set_login_password.set(event_target_value(&ev)) />
                            </div>
                            <button class="btn btn-primary w-full" on:click=move |_| {
                                if login_username.get() == "admin" && login_password.get() == "agrocore" {
                                    set_step.set(SetupStep::TenantCreation);
                                    set_login_error.set(None);
                                } else {
                                    set_login_error.set(Some("Invalid credentials".to_string()));
                                }
                            }>"Login"</button>
                        }.into_any(),

                        SetupStep::TenantCreation => view! {
                            <h2 class="card-title text-2xl font-bold mb-4">"Create First Tenant"</h2>
                            <p class="mb-6 text-base-content/70">"A tenant represents your organization in the system."</p>
                            
                            <div class="form-control w-full mb-4">
                                <label class="label"><span class="label-text">"Tenant Name"</span></label>
                                <input type="text" placeholder="AgroCorp" class="input input-bordered w-full" 
                                    on:input=move |ev| set_tenant_name.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control w-full mb-6">
                                <label class="label"><span class="label-text">"URL Slug"</span></label>
                                <input type="text" placeholder="agrocorp" class="input input-bordered w-full" 
                                    on:input=move |ev| set_tenant_slug.set(event_target_value(&ev)) />
                            </div>
                            <button class="btn btn-primary w-full" on:click=move |_| {
                                set_step.set(SetupStep::CompanyCreation);
                            }>"Create Tenant"</button>
                        }.into_any(),

                        SetupStep::CompanyCreation => view! {
                            <h2 class="card-title text-2xl font-bold mb-4">"Company Details"</h2>
                            <p class="mb-6 text-base-content/70">"Configure your primary company data."</p>
                            
                            <div class="form-control w-full mb-4">
                                <label class="label"><span class="label-text">"Company Name"</span></label>
                                <input type="text" class="input input-bordered w-full" 
                                    on:input=move |ev| set_company_name.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-control w-full mb-4">
                                <label class="label"><span class="label-text">"Base Address"</span></label>
                                <textarea class="textarea textarea-bordered w-full" 
                                    on:input=move |ev| set_company_address.set(event_target_value(&ev))></textarea>
                            </div>
                            <div class="grid grid-cols-2 gap-4 mb-6">
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">"Email"</span></label>
                                    <input type="email" class="input input-bordered w-full" 
                                        on:input=move |ev| set_company_email.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-control w-full">
                                    <label class="label"><span class="label-text">"Phone"</span></label>
                                    <input type="text" class="input input-bordered w-full" 
                                        on:input=move |ev| set_company_phone.set(event_target_value(&ev)) />
                                </div>
                            </div>
                            <button class="btn btn-primary w-full" on:click=move |_| {
                                set_step.set(SetupStep::ResourcesCreation);
                            }>"Save & Continue"</button>
                        }.into_any(),

                        SetupStep::ResourcesCreation => view! {
                            <h2 class="card-title text-2xl font-bold mb-4">"Initial Resources"</h2>
                            <p class="mb-6 text-base-content/70">"Define your first field and equipment."</p>
                            
                            <div class="form-control w-full mb-4">
                                <label class="label"><span class="label-text">"Field Type"</span></label>
                                <select class="select select-bordered w-full" 
                                    on:change=move |ev| set_resource_type.set(event_target_value(&ev))>
                                    <option value="Vineyard">"Vineyard"</option>
                                    <option value="OliveGrove">"Olive Grove"</option>
                                    <option value="Arable">"Arable Land"</option>
                                </select>
                            </div>

                            <div class="divider">"Equipment"</div>
                            <div class="form-control w-full mb-6">
                                <label class="label"><span class="label-text">"First Machine Name"</span></label>
                                <input type="text" placeholder="Tractor Fendt 724" class="input input-bordered w-full" />
                            </div>

                            {move || setup_status.get().and_then(|res| res.err()).map(|err| view! {
                                <div class="alert alert-error mb-4">
                                    <span>{err}</span>
                                </div>
                            })}

                            <button 
                                class="btn btn-success w-full" 
                                class:btn-disabled=move || setup_status.get().map(|s| s.is_ok()).unwrap_or(false)
                                on:click=on_finish
                            >
                                {move || if setup_status.get().map(|s| s.is_ok()).unwrap_or(false) { "Redirecting..." } else { "Finish Setup" }}
                            </button>
                        }.into_any(),
                    }}
                </div>
            </div>
        </div>
    }
}
