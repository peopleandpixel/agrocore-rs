//! Workers management page — Arbeitskräfte & Arbeitszeit-Erfassung
//!
//! Shows a list of workers with their hourly rate, clock-in/out status,
//! and total hours worked. Workers can clock in/out directly from here.

use crate::api;
use crate::components::toast::{ToastContext, ToastType};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn WorkersPage() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let (workers, set_workers) = signal::<Vec<api::WorkerDto>>(Vec::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);

    let toast = expect_context::<ToastContext>();

    let load_workers = move || {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match api::fetch_workers().await {
                Ok(result) => {
                    set_workers.set(result.data);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                    toast
                        .add_toast
                        .run(("Failed to load workers".to_string(), ToastType::Error));
                }
            }
        });
    };

    Effect::new(move |_| {
        load_workers();
    });

    let create_worker = move || {
        let new_worker = api::CreateWorkerRequest {
            user_id: uuid::Uuid::nil(),
            contract_type: "full_time".to_string(),
            language: None,
            skills: None,
            emergency_contact: None,
            nationality: None,
            hourly_rate: None,
        };
        spawn_local(async move {
            match api::create_worker(&new_worker).await {
                Ok(worker) => {
                    set_workers.update(|list| list.push(worker));
                    toast
                        .add_toast
                        .run(("Worker created".to_string(), ToastType::Success));
                }
                Err(e) => {
                    toast
                        .add_toast
                        .run((format!("Failed to create worker: {}", e), ToastType::Error));
                }
            }
        });
    };

    let clock_in_worker = move |worker_id: uuid::Uuid| {
        spawn_local(async move {
            let req = api::CreateClockEntryRequest {
                worker_id,
                entry_type: "ClockIn".to_string(),
                timestamp: None,
                lat: None,
                lng: None,
                task_id: None,
                notes: None,
            };
            match api::clock_in(&req).await {
                Ok(_) => toast.add_toast.run((
                    format!("Worker {} clocked in", worker_id),
                    ToastType::Success,
                )),
                Err(e) => toast
                    .add_toast
                    .run((format!("Clock-in failed: {}", e), ToastType::Error)),
            }
        });
    };

    let clock_out_worker = move |worker_id: uuid::Uuid| {
        spawn_local(async move {
            let req = api::CreateClockEntryRequest {
                worker_id,
                entry_type: "ClockOut".to_string(),
                timestamp: None,
                lat: None,
                lng: None,
                task_id: None,
                notes: None,
            };
            match api::clock_out(&req).await {
                Ok(_) => toast.add_toast.run((
                    format!("Worker {} clocked out", worker_id),
                    ToastType::Success,
                )),
                Err(e) => toast
                    .add_toast
                    .run((format!("Clock-out failed: {}", e), ToastType::Error)),
            }
        });
    };

    view! {
        <div class="p-6">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold">{crate::t!(t, "nav_workers")}</h1>
                <button class="btn btn-primary" on:click=move |_| create_worker()>
                    + {crate::t!(t, "add_worker")}
                </button>
            </div>

            <Show when=move || error.get().is_some() fallback=move || view! { () }>
                <div class="alert alert-error mb-4">
                    <span>{move || error.get().unwrap_or_default()}</span>
                </div>
            </Show>

            <Show when=move || loading.get() fallback=move || view! { () }>
                <div class="flex justify-center py-12">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            </Show>

            <Show when=move || !loading.get() && error.get().is_none()>
                <div class="overflow-x-auto">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr>
                                <th>{crate::t!(t, "worker")}</th>
                                <th>{crate::t!(t, "contract_type")}</th>
                                <th>{crate::t!(t, "hourly_rate")}</th>
                                <th>{crate::t!(t, "status")}</th>
                                <th>{crate::t!(t, "clock_actions")}</th>
                                <th>{crate::t!(t, "actions")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                workers.get().iter().map(|w| {
                                    let is_active = w.is_active;
                                    let hourly_rate = w.hourly_rate;
                                    let worker_id = w.id;
                                    view! {
                                        <tr>
                                            <td class="font-medium">{w.user_id.to_string()}</td>
                                            <td>{w.contract_type.clone()}</td>
                                            <td>
                                                {move || hourly_rate.map(|rate| format!("€{:.2}/h", rate)).unwrap_or_else(|| (crate::t!(t, "no_rate"))())}
                                            </td>
                                            <td>
                                                {if is_active { "Active" } else { "Inactive" }}
                                            </td>
                                            <td class="space-x-2">
                                                <button
                                                    class="btn btn-sm btn-outline"
                                                    on:click=move |_| clock_in_worker(worker_id)
                                                >
                                                    {crate::t!(t, "clock_in")}
                                                </button>
                                                <button
                                                    class="btn btn-sm btn-outline"
                                                    on:click=move |_| clock_out_worker(worker_id)
                                                >
                                                    {crate::t!(t, "clock_out")}
                                                </button>
                                            </td>
                                            <td>
                                                <a href="/worker/tasks">
                                                    <button class="btn btn-sm btn-ghost">
                                                        {crate::t!(t, "view_tasks")}
                                                    </button>
                                                </a>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
            </Show>
        </div>
    }
}
