use crate::api;
use crate::components::form::{RequiredLabel, is_valid_email};
use crate::components::toast::{ToastContext, ToastType};
use crate::i18n::{I18n, Language};
use leptos::prelude::{window, *};
use leptos::task::spawn_local;

#[component]
pub fn LoginView() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let toast_context = use_context::<ToastContext>().expect("ToastContext not provided");
    let title = i18n.t(lang.get().as_str(), "login_title");
    let subtitle = i18n.t(lang.get().as_str(), "login_subtitle");
    let email_label = i18n.t(lang.get().as_str(), "email");
    let password_label = i18n.t(lang.get().as_str(), "password");
    let sign_in = i18n.t(lang.get().as_str(), "sign_in");
    let signing_in = i18n.t(lang.get().as_str(), "signing_in");
    let required_error = i18n.t(lang.get().as_str(), "validation_required");
    let invalid_email_error = i18n.t(lang.get().as_str(), "validation_invalid_email");
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let (busy, set_busy) = signal(false);

    let on_login = move |_| {
        let email = email.get();
        let password = password.get();
        set_busy.set(true);
        set_error.set(None);

        if email.trim().is_empty() || password.trim().is_empty() {
            toast_context
                .add_toast
                .run((required_error.clone(), ToastType::Warning));
            set_error.set(Some(required_error.clone()));
            set_busy.set(false);
            return;
        }

        if !is_valid_email(&email) {
            toast_context
                .add_toast
                .run((invalid_email_error.clone(), ToastType::Warning));
            set_error.set(Some(invalid_email_error.clone()));
            set_busy.set(false);
            return;
        }

        spawn_local(async move {
            match api::login(api::LoginRequest { email, password }).await {
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
                    let _ = window().location().reload();
                }
                Err(e) => {
                    toast_context.add_toast.run((e.clone(), ToastType::Error));
                    set_error.set(Some(e));
                    set_busy.set(false);
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-base-200 flex items-center justify-center p-4">
            <div class="card w-full max-w-md bg-base-100 shadow-2xl border border-base-300 animate-slide-up">
                <div class="card-body">
                    <h1 class="card-title text-2xl font-bold">{title}</h1>
                    <p class="text-base-content/70">{subtitle}</p>

                    {move || error.get().map(|err| view! {
                        <div class="alert alert-error">
                            <span>{err}</span>
                        </div>
                    })}

                    <div class="form-control w-full">
                        <label class="label">
                            <RequiredLabel required=true>{email_label}</RequiredLabel>
                        </label>
                        <input
                            type="email"
                            class="input input-bordered w-full"
                            required
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="form-control w-full">
                        <label class="label">
                            <RequiredLabel required=true>{password_label}</RequiredLabel>
                        </label>
                        <input
                            type="password"
                            class="input input-bordered w-full"
                            required
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>

                    <button class="btn btn-primary w-full mt-4" class:btn-disabled=move || busy.get() on:click=on_login>
                        {move || if busy.get() { signing_in.clone() } else { sign_in.clone() }}
                    </button>
                </div>
            </div>
        </div>
    }
}
