use crate::api;
use crate::i18n;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn WorkerTasksPage() -> impl IntoView {
    let t = i18n::use_i18n();
    let navigate = use_navigate();

    // Fetch tasks on mount
    let (tasks, set_tasks) = signal(Vec::<api::OrderDto>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        set_loading.set(true);
        leptos::task::spawn_local(async move {
            match api::fetch_worker_tasks().await {
                Ok(t) => {
                    set_tasks.set(t);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });

    view! {
        <div class="flex flex-col gap-6 p-4 max-w-2xl mx-auto">
            <div>
                <h1 class="text-3xl font-bold">{crate::t!(t, "nav_my_tasks")}</h1>
                <p class="text-base-content/60">{crate::t!(t, "worker_tasks_desc")}</p>
            </div>

            <Suspense fallback=move || view! { <div class="flex justify-center py-12"><div class="loading loading-spinner loading-lg"></div></div> }.into_any()>
                {move || {
                    if loading.get() {
                        view! { <div class="flex justify-center py-12"><div class="loading loading-spinner loading-lg"></div></div> }.into_any()
                    } else if let Some(err) = error.get() {
                        view! {
                            <div class="alert alert-error">
                                <div class="flex items-center gap-3">
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77-1.333.192 3 1.732 3z"></path>
                                    </svg>
                                    <span>{err}</span>
                                </div>
                            </div>
                        }.into_any()
                    } else if tasks.get().is_empty() {
                        view! {
                            <div class="card bg-base-100 shadow text-center py-12">
                                <svg class="mx-auto text-6xl opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 012-2h2a2 2 0 012 2v10a2 2 0 01-2 2H9a2 2 0 01-2-2V5z"></path>
                                </svg>
                                <h3 class="text-xl font-bold mt-4 mb-2">{crate::t!(t, "no_tasks_assigned")}</h3>
                                <p class="text-base-content/60">{crate::t!(t, "no_tasks_desc")}</p>
                            </div>
                        }.into_any()
                    } else {
                        // Wir generieren die Zeilen direkt hier im Zustand, um Ownership-Konflikte zu vermeiden
                        let rows = tasks.get().into_iter().map(|task| {
                            let order_type = task.order_type.clone();
                            let label = task.label.clone();
                            let status = task.status.clone();
                            let task_id = task.id;
                            let nav = navigate.clone(); // Klonen für die Click-Closure

                            view! {
                                <tr>
                                    // FIX 1: Kein "move ||" hier! Direkt den übersetzten String rendern.
                                    <td>{crate::t!(t, &format!("order_type_{}", order_type.to_lowercase()))}</td>
                                    <td>{label}</td>
                                    <td>
                                        <div class="badge badge-sm">
                                            {status}
                                        </div>
                                    </td>
                                    <td>
                                        <button
                                            class="btn btn-xs btn-primary"
                                            on:click=move |_| {
                                                nav(&format!("/worker/tasks/{}", task_id), Default::default());
                                            }
                                        >
                                            {crate::t!(t, "start_task")}
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>();

                        view! {
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
                                            {rows}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>
        </div>
    }
}
