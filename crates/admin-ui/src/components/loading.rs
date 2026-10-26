use leptos::prelude::*;

/// Simple loading/splash screen shown while the app determines whether to
/// display the login form or the initial setup wizard.
#[component]
pub fn LoadingScreen() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-base-200 flex flex-col items-center justify-center p-4">
            <div class="text-center">
                <div class="loader-container">
                    <div class="loader"></div>
                    <img
                        src="/logo_only.png"
                        alt="AgroCore Logo"
                        class="loader-logo"
                    />
                </div>
            </div>
        </div>
    }
}
