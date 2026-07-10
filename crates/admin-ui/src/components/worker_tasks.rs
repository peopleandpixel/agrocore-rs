use crate::api::{fetch_my_tasks, TaskData};
use crate::i18n::{I18n, Language};
use icondata::*;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn WorkerTasksPage() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");

    let title = i18n.t(lang.get().as_str(), "my_tasks");
    let start_label = i18n.t(lang.get().as_str(), "start");
    let stop_label = i18n.t(lang.get().as_str(), "stop");
    let complete_label = i18n.t(lang.get().as_str(), "complete");

    let tasks = LocalResource::new(|| async move { fetch_my_tasks().await.ok() });

    view! {
        <div class="flex flex-col gap-4">
            <h1 class="text-2xl font-bold">{title}</h1>

            <Suspense fallback=move || view! { <div class="loading loading-spinner"></div> }>
                <div class="grid gap-4">
                    <For
                        each=move || {
                            tasks
                                .read()
                                .as_ref()
                                .map(|opt| {
                                    opt.as_ref()
                                        .map(|t| t.data.clone())
                                        .unwrap_or_default()
                                })
                                .unwrap_or_default()
                        }
                        key=|task| task.id
                        children=move |task: TaskData| {
                            let description = task.description;
                            let start_l = start_label.clone();
                            let stop_l = stop_label.clone();
                            let complete_l = complete_label.clone();
                            view! {
                                <div class="card bg-base-100 shadow">
                                    <div class="card-body">
                                        <h2 class="card-title">{description}</h2>
                                        <div class="card-actions justify-end">
                                            <button class="btn btn-success btn-sm">
                                                <Icon icon=LuPlay width="16" height="16" />
                                                {start_l}
                                            </button>
                                            <button class="btn btn-error btn-sm">
                                                <Icon icon=LuPause width="16" height="16" />
                                                {stop_l}
                                            </button>
                                            <button class="btn btn-primary btn-sm">
                                                <Icon icon=LuCheck width="16" height="16" />
                                                {complete_l}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </Suspense>
        </div>
    }
}