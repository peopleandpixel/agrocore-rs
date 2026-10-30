//! H10: Worker Task Detail page — allows workers to update task status
//! and see aggregated progress across all workers.

use crate::api;
use crate::i18n;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::params::Params;
use leptos_router::hooks::use_params;

#[component]
pub fn WorkerTaskDetailPage() -> impl IntoView {
    let t = i18n::use_i18n();
    let params = use_params();
    let task_id_str = Memo::new(move |_| {
        let params = match params.get() {
            Ok(p) => p,
            Err(_) => return String::new(),
        };
        params.get("id").cloned().unwrap_or_default()
    });

    let (task, set_task) = signal(None::<api::OrderDto>);
    let (status, set_status) = signal(None::<api::WorkerTaskStatusDto>);
    let (aggregate, set_aggregate) = signal(None::<api::WorkerTaskStatusAggregateDto>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let worker_id = api::user_id();

    Effect::new(move |_| {
        let task_id_str = task_id_str.get();
        let worker_id = worker_id;
        if task_id_str.is_empty() {
            return;
        }
        set_loading.set(true);
        spawn_local(async move {
            let task_id = match task_id_str.parse::<uuid::Uuid>() {
                Ok(id) => id,
                Err(_) => {
                    set_error.set(Some("Invalid task ID".to_string()));
                    set_loading.set(false);
                    return;
                }
            };

            // Fetch task details
            let tasks = match api::fetch_worker_tasks().await {
                Ok(t) => t,
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                    return;
                }
            };
            if let Some(task) = tasks.into_iter().find(|t| t.id == task_id) {
                set_task.set(Some(task));
            }

            // Fetch current worker's status for this task
            if let Some(wid) = worker_id {
                if let Ok(st) = api::fetch_worker_task_status(task_id, wid).await {
                    set_status.set(Some(st));
                }
            }

            // Fetch aggregated status
            if let Ok(agg) = api::fetch_aggregated_task_status(task_id).await {
                set_aggregate.set(Some(agg));
            }

            set_loading.set(false);
        });
    });

    let on_update_status = move |new_status: String| {
        let task_id_str = task_id_str.get();
        let worker_id = worker_id;
        if task_id_str.is_empty() {
            return;
        }
        let task_id = match task_id_str.parse::<uuid::Uuid>() {
            Ok(id) => id,
            Err(_) => return,
        };
        let wid = match worker_id {
            Some(id) => id,
            None => return,
        };

        spawn_local(async move {
            let req = api::UpdateWorkerTaskStatusRequest {
                status: new_status,
            };
            match api::update_worker_task_status(task_id, wid, req).await {
                Ok(_) => {
                    let _ = leptos::prelude::window().location().reload();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_start_task = move |_| {
        on_update_status("in_progress".to_string());
    };

    let on_complete_task = move |_| {
        on_update_status("completed".to_string());
    };

    let on_pause_task = move |_| {
        on_update_status("paused".to_string());
    };

    view! {
        <div class="flex flex-col gap-6 p-4 max-w-2xl mx-auto">
            <div>
                <a href="/worker/tasks" class="text-primary hover:underline">
                    "{i18n::t!(t, "back")} ←"
                </a>
            </div>

            {move || {
                if loading.get() {
                    view! { <div class="flex justify-center py-12"><div class="loading loading-spinner loading-lg"></div></div> }.into_any()
                } else if let Some(err) = error.get() {
                    view! { <div class="alert alert-error">{err}</div> }.into_any()
                } else {
                    let current_status = status.get().map(|s| s.status.clone());
                    let agg = aggregate.get().map(|a| a.aggregated_status.clone());

                    view! {
                        <div class="card bg-base-100 shadow">
                            <div class="card-body">
                                {move || {
                                    if let Some(task) = task.get() {
                                        view! {
                                            <h2 class="card-title text-2xl">{task.label.clone()}</h2>
                                            <p class="text-base-content/60">
                                                {crate::t!(t, &format!("order_type_{}", task.order_type.to_lowercase()))}
                                            </p>
                                        }.into_any()
                                    } else {
                                        view! { <h2 class="card-title text-2xl">"Task not found"</h2> }.into_any()
                                    }
                                }}

                                <div class="divider"></div>

                                // Aggregated status
                                {move || {
                                    if let Some(status) = agg.clone() {
                                        view! {
                                            <div class="alert alert-info">
                                                <div>
                                                    <h3 class="font-bold">"Overall Progress"</h3>
                                                    <p class="text-sm">{status}</p>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }
                                }}

                                // Current worker status
                                <div class="space-y-2">
                                    <h3 class="font-bold">"Your Status"</h3>
                                    {move || {
                                        if let Some(st) = current_status.clone() {
                                            view! {
                                                <div class="badge badge-lg badge-outline">
                                                    {st}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="badge badge-lg badge-ghost">
                                                    "Not started"
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>

                                <div class="divider"></div>

                                // Action buttons
                                <div class="flex flex-wrap gap-3">
                                    <button
                                        class="btn btn-primary"
                                        on:click=on_start_task
                                    >
                                        "Start Task"
                                    </button>
                                    <button
                                        class="btn btn-warning"
                                        on:click=on_pause_task
                                    >
                                        "Pause"
                                    </button>
                                    <button
                                        class="btn btn-success"
                                        on:click=on_complete_task
                                    >
                                        "Complete"
                                    </button>
                                </div>

                                // Task details
                                {move || {
                                    if let Some(task) = task.get() {
                                        view! {
                                            <div class="mt-4 space-y-2 text-sm">
                                                <p><strong>"Task ID:"</strong> {task.id.to_string()}</p>
                                                <p><strong>"Status:"</strong> {task.status.clone()}</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }
                                }}
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
