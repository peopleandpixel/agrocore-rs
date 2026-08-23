use crate::api;
use crate::i18n;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use icondata::*;

/// TaskDetailPage — shows a single task with start/stop actions.
/// Consumes orphaned API routes:
///   POST /api/v1/tasks/{id}/start-for-worker
///   POST /api/v1/tasks/{id}/stop-for-worker
///   POST /api/v1/orders/{id}/start
///   POST /api/v1/orders/{id}/complete
#[component]
pub fn TaskDetailPage() -> impl IntoView {
    let t = i18n::use_i18n();

    let path = window().location().pathname().unwrap_or_default();
    let task_id_str = path.rsplit('/').next().unwrap_or_default();
    let invalid_id = uuid::Uuid::parse_str(task_id_str).is_err();
    let tid = uuid::Uuid::parse_str(task_id_str).unwrap_or_default();

    let (task, set_task) = signal(None::<api::TaskData>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(None::<String>);

    Effect::new(move |_| {
        set_loading.set(true);
        spawn_local({
            async move {
                match api::fetch_task(tid).await {
                    Ok(data) => {
                        set_task.set(Some(data));
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(e));
                        set_loading.set(false);
                    }
                }
            }
        });
    });

    view! {
        <div class="flex flex-col gap-6">
            {move || {
                if invalid_id {
                    view! {
                        <div class="p-4"><div class="alert alert-error"><span>{crate::t!(t, "invalid_task_id")}</span></div></div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex items-center gap-4">
                            <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                let _ = window().location().set_href("/tasks");
                            }>
                                <Icon icon=LuArrowLeft width="16" height="16" />
                            </button>
                            <h1 class="text-3xl font-bold">{crate::t!(t, "task_detail")}</h1>
                        </div>

                        {move || success.get().map(|msg| view! {
                            <div class="alert alert-success">
                                <Icon icon=LuCircleCheck width="20" height="20" />
                                <span>{msg}</span>
                            </div>
                        })}
                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error">
                                <Icon icon=LuCircleAlert width="20" height="20" />
                                <span>{err}</span>
                            </div>
                        })}

                        {move || {
                            match (loading.get(), error.get().as_ref(), task.get()) {
                                (true, _, _) => view! {
                                    <div class="flex justify-center py-12">
                                        <div class="loading loading-spinner loading-lg"></div>
                                    </div>
                                }.into_any(),
                                (false, Some(e), _) => {
                                    let err = e.clone();
                                    view! {
                                        <div class="alert alert-error"><span>{err}</span></div>
                                    }.into_any()
                                },
                                (false, None, Some(d)) => {
                                    let label = d.label.clone();
                                    let description = d.description.clone();
                                    let order_type_val = d.order_type.clone();
                                    let status_val = d.status.clone();
                                    let planned = d.planned_date.clone();
                                    let deadline = d.deadline_date.clone();
                                    view! {
                                        <div class="card bg-base-100 shadow">
                                            <div class="card-body">
                                                <h2 class="card-title">{label}</h2>
                                                <p class="text base-content/60">{crate::t!(t, "task_description")}: {description}</p>
                                                <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                                                    <div class="stat">
                                                        <div class="stat-label">{crate::t!(t, "task_type")}</div>
                                                        <div class="stat-value text-sm">{crate::t!(t, &format!("order_type_{}", order_type_val.to_lowercase()))}</div>
                                                    </div>
                                                    <div class="stat">
                                                        <div class="stat-label">{crate::t!(t, "status")}</div>
                                                        <div class="stat-value"><div class="badge badge-lg">{status_val}</div></div>
                                                    </div>
                                                    {planned.as_ref().map(|p| view! {
                                                        <div class="stat">
                                                            <div class="stat-label">{crate::t!(t, "planned_date")}</div>
                                                            <div class="stat-value text-sm">{p.clone()}</div>
                                                        </div>
                                                    })}
                                                    {deadline.as_ref().map(|dl| view! {
                                                        <div class="stat">
                                                            <div class="stat-label">{crate::t!(t, "deadline")}</div>
                                                            <div class="stat-value text-sm">{dl.clone()}</div>
                                                        </div>
                                                    })}
                                                </div>
                                                <div class="card-actions justify-start mt-6">
                                                    <button class="btn btn-outline btn-primary" on:click=move |_| {
                                                        set_error.set(None);
                                                        set_success.set(None);
                                                        spawn_local({
                                                            async move {
                                                                match api::start_task_for_worker(tid).await {
                                                                    Ok(_) => set_success.set(Some((crate::t!(t, "task_started_success"))().to_string())),
                                                                    Err(e) => set_error.set(Some(e)),
                                                                }
                                                            }
                                                        });
                                                    }>
                                                        <Icon icon=LuPlay width="16" height="16" />
                                                        {crate::t!(t, "start_task")}
                                                    </button>
                                                    <button class="btn btn-outline btn-warning" on:click=move |_| {
                                                        spawn_local({
                                                            async move {
                                                                match api::stop_task_for_worker(tid).await {
                                                                    Ok(_) => set_success.set(Some((crate::t!(t, "task_stopped_success"))().to_string())),
                                                                    Err(e) => set_error.set(Some(e)),
                                                                }
                                                            }
                                                        });
                                                    }>
                                                        <Icon icon=LuPause width="16" height="16" />
                                                        {crate::t!(t, "stop_task")}
                                                    </button>
                                                    <button class="btn btn-outline btn-accent" on:click=move |_| {
                                                        spawn_local({
                                                            async move {
                                                                match api::start_order(tid).await {
                                                                    Ok(_) => set_success.set(Some((crate::t!(t, "order_started_success"))().to_string())),
                                                                    Err(e) => set_error.set(Some(e)),
                                                                }
                                                            }
                                                        });
                                                    }>
                                                        <Icon icon=LuRocket width="16" height="16" />
                                                        {crate::t!(t, "start_order")}
                                                    </button>
                                                    <button class="btn btn-outline btn-success" on:click=move |_| {
                                                        spawn_local({
                                                            async move {
                                                                match api::complete_order(tid).await {
                                                                    Ok(_) => set_success.set(Some((crate::t!(t, "order_completed_success"))().to_string())),
                                                                    Err(e) => set_error.set(Some(e)),
                                                                }
                                                            }
                                                        });
                                                    }>
                                                        <Icon icon=LuSquareCheck width="16" height="16" />
                                                        {crate::t!(t, "complete_order")}
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                },
                                (false, None, None) => view! {
                                    <div class="alert alert-error"><span>{crate::t!(t, "task_load_failed")}</span></div>
                                }.into_any(),
                            }
                        }}
                    }.into_any()
                }
            }}
        </div>
    }
}
