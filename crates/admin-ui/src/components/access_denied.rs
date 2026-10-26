/// Access denied page shown when user doesn't have permission to view a resource

use crate::i18n::use_i18n;
use icondata_lu::LuShieldAlert;
use leptos::prelude::*;
use crate::icon::Icon;

#[component]
pub fn AccessDenied() -> impl IntoView {
    let t = use_i18n();

    view! {
        <div class="flex flex-col items-center justify-center min-h-[60vh] px-4">
            <div class="card bg-base-100 shadow-sm border border-base-200 rounded-2xl max-w-md w-full">
                <div class="card-body items-center text-center">
                    <div class="text-error mb-4">
                        <Icon icon=&LuShieldAlert width="64" height="64" />
                    </div>
                    <h2 class="card-title text-2xl font-bold mb-2">{t("access_denied")}</h2>
                    <p class="text-base-content/60 mb-6">
                        {t("access_denied_desc")}
                    </p>
                    <a href="/" class="btn btn-primary">
                        {t("go_to_dashboard")}
                    </a>
                </div>
            </div>
        </div>
    }
}
