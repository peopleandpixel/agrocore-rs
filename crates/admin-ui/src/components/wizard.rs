use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;
use crate::i18n::{I18n, Language};

#[component]
pub fn WizardView() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");

    view! {
        <div class="flex flex-col gap-8 items-center py-10">
            <h1 class="text-4xl font-extrabold text-center mb-8">{
                let i18n = i18n.clone();
                move || i18n.t(lang.get().as_str(), "wizard_tasks")
            }</h1>
            
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 w-full max-w-4xl">
                <WizardCard 
                    title={
                        let i18n = i18n.clone();
                        move || i18n.t(lang.get().as_str(), "task_planting")
                    }
                    icon=LuSprout
                    color="bg-success"
                    description="Planen Sie die nächste Aussaat auf Ihren Flächen."
                />
                <WizardCard 
                    title={
                        let i18n = i18n.clone();
                        move || i18n.t(lang.get().as_str(), "task_protection")
                    }
                    icon=LuDroplets
                    color="bg-info"
                    description="Dokumentieren Sie Düngung oder Pflanzenschutz."
                />
                <WizardCard 
                    title={
                        let i18n = i18n.clone();
                        move || i18n.t(lang.get().as_str(), "task_harvest")
                    }
                    icon=LuWheat
                    color="bg-warning"
                    description="Erfassen Sie Erntemengen und Qualität."
                />
                <WizardCard 
                    title=|| "Tierbehandlung".to_string()
                    icon=LuSyringe
                    color="bg-error"
                    description="Dokumentieren Sie medizinische Maßnahmen."
                />
                <WizardCard 
                    title=|| "Finanz-Buchung".to_string()
                    icon=LuWallet
                    color="bg-accent"
                    description="Einnahmen oder Ausgaben schnell erfassen."
                />
                <WizardCard 
                    title=|| "BBCH-Beobachtung".to_string()
                    icon=LuEye
                    color="bg-info"
                    description="Aktuelle Wachstumsstadien protokollieren."
                />
            </div>

            <div class="mt-12 p-6 bg-base-200 rounded-2xl border-2 border-dashed border-base-300 w-full max-w-2xl text-center">
                <h2 class="text-xl font-bold mb-2">"Weitere Aufgaben?"</h2>
                <p class="text-base-content/70">"Wählen Sie einen Bereich aus, um Schritt für Schritt durch den Prozess geführt zu werden."</p>
            </div>
        </div>
    }
}

#[component]
fn WizardCard<F>(
    title: F,
    icon: icondata::Icon,
    color: &'static str,
    description: &'static str,
) -> impl IntoView 
where 
    F: Fn() -> String + 'static + std::marker::Send
{
    view! {
        <button class="card bg-base-100 shadow-xl hover:shadow-2xl transition-all hover:-translate-y-2 cursor-pointer border-2 border-transparent hover:border-primary text-left overflow-hidden group">
            <div class=format!("h-2 w-full {}", color)></div>
            <div class="card-body">
                <div class="flex items-center gap-4 mb-2">
                    <div class=format!("p-3 rounded-xl {} text-white group-hover:scale-110 transition-transform", color)>
                        <Icon icon=icon width="32" height="32" />
                    </div>
                    <h2 class="card-title text-2xl">{move || title()}</h2>
                </div>
                <p class="text-base-content/70">{description}</p>
                <div class="card-actions justify-end mt-4">
                    <div class="btn btn-primary btn-sm rounded-lg">"Starten"</div>
                </div>
            </div>
        </button>
    }
}
