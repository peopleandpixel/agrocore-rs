use crate::api;
// use icondata::*;
use leptos::prelude::*;
// use leptos_icons::Icon;

#[component]
pub fn WorkerTasksPage() -> impl IntoView {
    let t = crate::i18n::use_i18n();
    let tasks = LocalResource::new(|| async move { api::fetch_worker_tasks().await.unwrap_or_default() });

    view! {
        <div class="flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold">{crate::t!(t, "nav_my_tasks")}</h1>
                <p class="text-base-content/60">{crate::t!(t, "worker_tasks_desc")}</p>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table table-sm">
                        <thead>
                            <tr>
                                <th>{crate::t!(t, "order_type")}</th>
                                <th>{crate::t!(t, "description")}</th>
                                <th>{crate::t!(t, "status")}</th>
                                <th>{crate::t!(t, "actions")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || tasks.get().unwrap_or_default()
                                key=|task| task.id
                                children=move |task| {
                                    let order_type = task.order_type.clone();
                                    let label = task.label.clone();
                                    let status = task.status.clone();
                                    view! {
                                        <tr>
                                            <td>{move || t(&format!("order_type_{}", order_type))}</td>
                                            <td>{label}</td>
                                            <td><div class="badge badge-sm">{status}</div></td>
                                            <td>
                                                <button class="btn btn-xs btn-primary">{crate::t!(t, "start_task")}</button>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
