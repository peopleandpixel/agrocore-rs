use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn WizardView() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = |key: &str| i18n.t(lang.get().as_str(), key);
    let title = t("wizard_tasks");
    let task_planting = t("task_planting");
    let task_protection = t("task_protection");
    let task_harvest = t("task_harvest");
    let wizard_livestock = t("wizard_livestock");
    let wizard_finance = t("wizard_finance");
    let wizard_bbch = t("wizard_bbch");
    let wizard_start = t("wizard_start");
    let wizard_more_tasks = t("wizard_more_tasks");
    let wizard_explore = t("wizard_explore");
    let planting_desc = t("wizard_planting_desc");
    let protection_desc = t("wizard_protection_desc");
    let harvest_desc = t("wizard_harvest_desc");
    let livestock_desc = t("wizard_livestock_desc");
    let finance_desc = t("wizard_finance_desc");
    let bbch_desc = t("wizard_bbch_desc");

    view! {
        <div class="flex flex-col gap-8 items-center py-10">
            <h1 class="text-4xl font-extrabold text-center mb-8">{title}</h1>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 w-full max-w-4xl">
                <WizardCard
                    title=task_planting.clone()
                    icon=LuSprout
                    color="bg-success"
                    description=planting_desc.clone()
                    action_label=wizard_start.clone()
                    href="/sites"
                />
                <WizardCard
                    title=task_protection.clone()
                    icon=LuDroplets
                    color="bg-info"
                    description=protection_desc.clone()
                    action_label=wizard_start.clone()
                    href="/tasks"
                />
                <WizardCard
                    title=task_harvest.clone()
                    icon=LuWheat
                    color="bg-warning"
                    description=harvest_desc.clone()
                    action_label=wizard_start.clone()
                    href="/tasks"
                />
                <WizardCard title=wizard_livestock.clone() icon=LuSyringe color="bg-error" description=livestock_desc.clone() action_label=wizard_start.clone() href="/livestock" />
                <WizardCard title=wizard_finance.clone() icon=LuWallet color="bg-accent" description=finance_desc.clone() action_label=wizard_start.clone() href="/finance" />
                <WizardCard title=wizard_bbch.clone() icon=LuEye color="bg-info" description=bbch_desc.clone() action_label=wizard_start.clone() href="/weather" />
            </div>

            <div class="mt-12 p-6 bg-base-200 rounded-2xl border-2 border-dashed border-base-300 w-full max-w-2xl text-center">
                <h2 class="text-xl font-bold mb-2">{wizard_more_tasks}</h2>
                <p class="text-base-content/70">{wizard_explore}</p>
            </div>
        </div>
    }
}

#[component]
fn WizardCard(
    title: String,
    description: String,
    action_label: String,
    icon: icondata::Icon,
    color: &'static str,
    href: &'static str,
) -> impl IntoView
where
{
    view! {
        <a href=href class="card bg-base-100 shadow-xl hover:shadow-2xl transition-all hover:-translate-y-2 cursor-pointer border-2 border-transparent hover:border-primary text-left overflow-hidden group">
            <div class=format!("h-2 w-full {}", color)></div>
            <div class="card-body">
                <div class="flex items-center gap-4 mb-2">
                    <div class=format!("p-3 rounded-xl {} text-white group-hover:scale-110 transition-transform", color)>
                        <Icon icon=icon width="32" height="32" />
                    </div>
                    <h2 class="card-title text-2xl">{title}</h2>
                </div>
                <p class="text-base-content/70">{description}</p>
                <div class="card-actions justify-end mt-4">
                    <span class="btn btn-primary btn-sm rounded-lg">{action_label}</span>
                </div>
            </div>
        </a>
    }
}
