use crate::api;
use crate::i18n;
use icondata::{LuArrowLeft, LuSave};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// EquipmentDetailPage - shows detailed equipment info, maintenance history,
/// cost summary, and a form to record new maintenance.
#[component]
pub fn EquipmentDetailPage() -> impl IntoView {
    let t = i18n::use_i18n();

    let path = window().location().pathname().unwrap_or_default();
    let equipment_id_str = path.rsplit('/').next().unwrap_or_default();
    let invalid_id = uuid::Uuid::parse_str(equipment_id_str).is_err();
    let eid = uuid::Uuid::parse_str(equipment_id_str).unwrap_or_default();

    let (equipment, set_equipment) = signal(None::<api::EquipmentDto>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(None::<String>);
    let (maintenance_log, set_maintenance_log) = signal(Vec::<api::MaintenanceLogDto>::new());
    let (cost_summary, set_cost_summary) = signal(None::<api::MaintenanceCostSummaryDto>);
    let (cost_loading, set_cost_loading) = signal(true);
    let (hours_input, set_hours_input) = signal(String::new());
    let (note_input, set_note_input) = signal(String::new());

    // Load equipment
    Effect::new(move |_| {
        set_loading.set(true);
        let id = eid;
        spawn_local(async move {
            match api::fetch_equipment_by_id(id).await {
                Ok(data) => {
                    set_equipment.set(Some(data));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });

    // Load maintenance log
    Effect::new(move |_| {
        let id = eid;
        spawn_local(async move {
            match api::fetch_equipment_maintenance_log(id).await {
                Ok(log) => set_maintenance_log.set(log),
                Err(e) => set_error.set(Some(format!("Failed to load maintenance log: {}", e))),
            }
        });
    });

    // Load maintenance cost summary
    Effect::new(move |_| {
        let id = eid;
        spawn_local(async move {
            match api::fetch_maintenance_cost_summary(id).await {
                Ok(summary) => {
                    set_cost_summary.set(Some(summary));
                    set_cost_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load cost summary: {}", e)));
                    set_cost_loading.set(false);
                }
            }
        });
    });

    let on_record = move |_: MouseEvent| {
        let id = eid;
        let hours: f64 = hours_input.get().parse().unwrap_or(0.0);
        let note = note_input.get();

        set_success.set(None);
        set_error.set(None);

        spawn_local(async move {
            match api::record_maintenance(
                id,
                hours,
                if note.is_empty() {
                    None
                } else {
                    Some(note.as_str())
                },
            )
            .await
            {
                Ok(_) => {
                    set_success.set(Some(
                        (crate::t!(t, "maintenance_recorded_success"))().to_string(),
                    ));
                    match api::fetch_equipment_by_id(id).await {
                        Ok(data) => set_equipment.set(Some(data)),
                        Err(e) => set_error.set(Some(e)),
                    }
                    match api::fetch_equipment_maintenance_log(id).await {
                        Ok(log) => set_maintenance_log.set(log),
                        Err(e) => set_error.set(Some(e)),
                    }
                    match api::fetch_maintenance_cost_summary(id).await {
                        Ok(summary) => set_cost_summary.set(Some(summary)),
                        Err(e) => set_error.set(Some(e)),
                    }
                }
                Err(e) => set_error.set(Some(e)),
            }
        });

        set_hours_input.set(String::new());
        set_note_input.set(String::new());
    };

    view! {
        <div class="container mx-auto p-6">
            <div class="flex items-center gap-4 mb-6">
                <button class="btn btn-ghost btn-sm" on:click=move |_| {
                    let _ = window().location().set_href("/equipment");
                }>
                    <Icon icon=LuArrowLeft width="16" height="16" />
                </button>
                <h1 class="text-2xl font-bold">{crate::t!(t, "equipment_detail")}</h1>
            </div>

            {move || {
                match (invalid_id, loading.get(), error.get().as_ref(), equipment.get()) {
                    (true, _, _, _) => view! {
                        <div class="alert alert-error"><span>{crate::t!(t, "invalid_equipment_id")}</span></div>
                    }.into_any(),
                    (false, true, _, _) => view! {
                        <div class="flex justify-center py-12">
                            <div class="loading loading-spinner loading-lg"></div>
                        </div>
                    }.into_any(),
                    (false, false, Some(e), _) => {
                        let err = e.clone();
                        view! {
                            <div class="alert alert-error"><span>{err}</span></div>
                        }.into_any()
                    },
                    (false, false, None, Some(eq)) => {
                        view! {
                            <EquipmentDetailView equipment=eq />

                            <Show when=move || cost_summary.get().is_some() || !cost_loading.get()>
                                {move || {
                                    match (cost_loading.get(), cost_summary.get()) {
                                        (true, _) => view! {
                                            <div class="card p-6 mt-6">
                                                <h2 class="text-xl font-semibold mb-4">{crate::t!(t, "cost_summary")}</h2>
                                                <p>{crate::t!(t, "cost_summary_loading")}</p>
                                            </div>
                                        }.into_any(),
                                        (false, Some(summary)) => view! {
                                            <div class="card p-6 mt-6">
                                                <h2 class="text-xl font-semibold mb-4">{crate::t!(t, "cost_summary")}</h2>
                                                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                                    <div>
                                                        <span class="text-2xl font-bold">{"EUR "}{format!("{:.2}", summary.total_parts_cost)}</span>
                                                        <p class="text-sm text-gray-500">{crate::t!(t, "parts_cost")}</p>
                                                    </div>
                                                    <div>
                                                        <span class="text-2xl font-bold">{format!("{:.1}", summary.total_labor_hours)}</span>
                                                        <p class="text-sm text-gray-500">{crate::t!(t, "labor_hours")}</p>
                                                    </div>
                                                    <div>
                                                        <span class="text-2xl font-bold">{format!("{:.1}", summary.total_downtime_hours)}</span>
                                                        <p class="text-sm text-gray-500">{crate::t!(t, "downtime_hours")}</p>
                                                    </div>
                                                    <div>
                                                        <span class="text-2xl font-bold">{"EUR "}{format!("{:.2}", summary.total_cost)}</span>
                                                        <p class="text-sm text-gray-500">{crate::t!(t, "total_cost")}</p>
                                                    </div>
                                                </div>
                                                <p class="text-sm text-gray-500 mt-2">{move || (crate::t!(t, "total_maintenance_count"))().to_string()}: {summary.total_maintenance_count}</p>
                                            </div>
                                        }.into_any(),
                                        (false, None) => ().into_any(),
                                    }
                                }}
                            </Show>

                            <div class="mt-8">
                                <h2 class="text-xl font-semibold mb-4">{crate::t!(t, "maintenance_history")}</h2>

                                {move || {
                                    let log = maintenance_log.get();
                                    let log_clone = log.clone();
                                    if log.is_empty() {
                                        view! {
                                            <p class="text-gray-500 mt-4">{crate::t!(t, "no_maintenance_records")}</p>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <table class="table table-sm mt-4">
                                                <thead>
                                                    <tr>
                                                        <th>{crate::t!(t, "performed_at")}</th>
                                                        <th>{crate::t!(t, "hours")}</th>
                                                        <th>{crate::t!(t, "parts_cost")}</th>
                                                        <th>{crate::t!(t, "labor_hours")}</th>
                                                        <th>{crate::t!(t, "downtime_hours")}</th>
                                                        <th>{crate::t!(t, "note")}</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {move || log_clone.iter().map(|entry| {
                                                        let performed_at = entry.performed_at.clone();
                                                        let hours_val = entry.hours;
                                                        let parts = format!("{:.2}", entry.parts_cost);
                                                        let labor = format!("{:.1}", entry.labor_hours);
                                                        let downtime = format!("{:.1}", entry.downtime_hours);
                                                        let note_val = entry.note.clone().unwrap_or_default();
                                                        view! {
                                                            <tr>
                                                                <td>{performed_at}</td>
                                                                <td>{hours_val.to_string()}</td>
                                                                <td>{parts}</td>
                                                                <td>{labor}</td>
                                                                <td>{downtime}</td>
                                                                <td>{note_val}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        }.into_any()
                                    }
                                }}

                                <div class="card p-6 mt-8">
                                    <h2 class="text-xl font-semibold mb-4">{crate::t!(t, "record_maintenance")}</h2>

                                    <Show when=move || success.get().is_some()>
                                        <div class="alert alert-success mb-4">
                                            {move || success.get()}
                                        </div>
                                    </Show>

                                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 items-end">
                                        <div>
                                            <label class="label">
                                                <span class="label-text">{crate::t!(t, "hours")}</span>
                                            </label>
                                            <input type="number" step="0.1" class="input input-bordered w-full"
                                                   prop:value=move || hours_input.get()
                                                   on:input=move |ev| { set_hours_input.set(event_target_value(&ev)); } />
                                        </div>
                                        <div>
                                            <label class="label">
                                                <span class="label-text">{crate::t!(t, "note")}</span>
                                            </label>
                                            <input type="text" class="input input-bordered w-full"
                                                   prop:value=move || note_input.get()
                                                   on:input=move |ev| { set_note_input.set(event_target_value(&ev)); } />
                                        </div>
                                        <div>
                                            <button class="btn btn-primary w-full" on:click=on_record>
                                                <Icon icon=LuSave width="16" height="16" />
                                                {crate::t!(t, "save")}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    },
                    (false, false, None, None) => view! {
                        <div class="alert alert-warning">{crate::t!(t, "equipment_not_found")}</div>
                    }.into_any(),
                }
            }}
        </div>
    }
}

#[component]
fn EquipmentDetailView(equipment: api::EquipmentDto) -> impl IntoView {
    let t = i18n::use_i18n();
    let eq = equipment;
    let eq_basic = eq.clone();
    let eq_maint = eq.clone();
    let eq_intervals = eq.clone();
    let intervals = eq_intervals
        .maintenance_intervals
        .clone()
        .unwrap_or_default();
    let has_intervals = !intervals.is_empty();

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 card p-6 mb-6">
            <div>
                <h3 class="font-semibold text-lg mb-2">{crate::t!(t, "basic_info")}</h3>
                <p><strong>{crate::t!(t, "label")}:</strong> {eq_basic.label}</p>
                <p><strong>{crate::t!(t, "code")}:</strong> {eq_basic.code.unwrap_or_default()}</p>
                <p><strong>{crate::t!(t, "type")}:</strong> {eq_basic.equipment_type}</p>
                <p><strong>{crate::t!(t, "in_usage")}:</strong> {move || if eq.in_usage { (crate::t!(t, "yes"))().to_string() } else { (crate::t!(t, "no"))().to_string() }}</p>
            </div>
            <div>
                <h3 class="font-semibold text-lg mb-2">{crate::t!(t, "maintenance_info")}</h3>
                <p><strong>{crate::t!(t, "created_at")}:</strong> {eq_maint.created_at}</p>
                <p><strong>{crate::t!(t, "updated_at")}:</strong> {eq_maint.updated_at}</p>
                <p><strong>{crate::t!(t, "maintenance_hours")}:</strong> {move || eq_maint.last_maintenance_hours.map(|h| h.to_string()).unwrap_or_else(|| (crate::t!(t, "none"))().to_string())}</p>
                <p><strong>{crate::t!(t, "next_maintenance")}:</strong> {move || eq_maint.next_maintenance_date.clone().unwrap_or_else(|| (crate::t!(t, "no_deadline"))().to_string())}</p>
            </div>
        </div>

        {move || {
            if has_intervals {
                view! {
                    <div class="card p-4 mb-4">
                        <h3 class="font-semibold mb-2">{crate::t!(t, "maintenance_intervals")}</h3>
                        <ul class="list-disc list-inside">
                            {intervals.iter().map(|iv| {
                                let label = iv.label.clone();
                                let hours_val = iv.interval_hours.map(|h| h.to_string()).unwrap_or_default();
                                let days_val = iv.interval_days.map(|d| d.to_string()).unwrap_or_default();
                                view! {
                                    <li>{label} - {crate::t!(t, "hours")}: {hours_val}, {crate::t!(t, "days")}: {days_val}</li>
                                }
                            }).collect_view()}
                        </ul>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }
        }}
    }
}
