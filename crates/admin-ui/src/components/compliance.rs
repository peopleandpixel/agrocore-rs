use crate::api;
use crate::i18n::{I18n, Language};
use chrono::Utc;
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[component]
pub fn CompliancePage() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = |key: &str| i18n.t(lang.get().as_str(), key);
    let title = t("compliance_certification");
    let description = t("manage_requirements");
    let export_report = t("export_report");
    let new_checklist = t("new_checklist");
    let compliance_score = t("compliance_score");
    let score_from_checks = t("score_from_checks");
    let open_actions = t("open_actions");
    let active_checks = t("active_checks");
    let next_audit = t("next_audit");
    let next_audit_desc = t("next_audit_desc");
    let certification_status = t("certification_status");
    let certification_status_desc = t("certification_status_desc");
    let application_history = t("application_history");
    let entry_label = t("entry");
    let status_label = t("status");
    let recorded = t("recorded");
    let no_entries = t("no_protection_entries");
    let create_checklist = t("create_checklist");
    let type_label = t("type");
    let due_date_label = t("due_date");
    let cancel_label = t("cancel");
    let save_label = t("save");
    let no_site_for_checklist = t("compliance_no_site_for_checklist");
    let no_site_for_checklist_label: &'static str = Box::leak(no_site_for_checklist.into_boxed_str());
    let sites = LocalResource::new(|| async move { api::fetch_sites().await.ok() });
    let checklists =
        LocalResource::new(|| async move { api::fetch_compliance_checklists().await.ok() });
    let plant_protection =
        LocalResource::new(|| async move { api::fetch_plant_protection_records().await.ok() });
    let (show_modal, set_show_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (due_date, set_due_date) = signal(Utc::now().date_naive().to_string());
    let (checklist_type, set_checklist_type) = signal(String::from("GlobalGAP"));

    let on_export = move |_| {
        spawn_local(async move {
            if let Ok(bytes) = api::export_pac_sip().await {
                let _ = api::download_bytes(
                    "pac_sip_report.xlsx",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    &bytes,
                );
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">{title}</h1>
                    <p class="text-base-content/60">{description}</p>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-outline" on:click=on_export>
                        <Icon icon=LuDownload width="20" height="20" />
                        {export_report}
                    </button>
                    <button class="btn btn-primary" on:click=move |_| set_show_modal.set(true)>
                        <Icon icon=LuPlus width="20" height="20" />
                        {new_checklist}
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">{compliance_score.clone()}</div>
                        <div class="stat-value text-success">"—"</div>
                        <div class="stat-desc">{score_from_checks.clone()}</div>
                    </div>
                </div>
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">{open_actions.clone()}</div>
                        <div class="stat-value text-warning">
                            {move || checklists.read().as_ref().map(|page| page.as_ref().map(|p| p.total).unwrap_or(0)).unwrap_or(0)}
                        </div>
                        <div class="stat-desc">{active_checks.clone()}</div>
                    </div>
                </div>
                <div class="stats shadow">
                    <div class="stat">
                        <div class="stat-title">{next_audit.clone()}</div>
                        <div class="stat-value">"—"</div>
                        <div class="stat-desc">{next_audit_desc.clone()}</div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title mb-4">{certification_status}</h2>
                    <div class="alert alert-info">
                        <span>{certification_status_desc}</span>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title mb-4">{application_history}</h2>
                    <div class="overflow-x-auto">
                        {move || plant_protection.read().as_ref().and_then(|page| page.as_ref()).map(|page| view! {
                            <table class="table table-zebra w-full">
                                <thead>
                                    <tr>
                                        <th>{entry_label.clone()}</th>
                                        <th>{status_label.clone()}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {page.data.iter().map(|entry| {
                                        view! {
                                            <tr>
                                                <td>{entry.to_string()}</td>
                                                <td><span class="badge badge-ghost">{recorded.clone()}</span></td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        }.into_any()).unwrap_or_else(|| view! {
                            <div class="alert alert-ghost">
                                        <span>{no_entries.clone()}</span>
                            </div>
                        }.into_any())}
                    </div>
                </div>
            </div>

            <Show when=move || show_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">{create_checklist.clone()}</h3>
                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}
                        <div class="grid grid-cols-2 gap-4 mt-4">
                            <div class="form-control">
                                <label class="label"><span class="label-text">{type_label.clone()}</span></label>
                                <select class="select select-bordered w-full" prop:value=move || checklist_type.get() on:change=move |ev| set_checklist_type.set(event_target_value(&ev))>
                                    <option value="GlobalGAP">"GlobalGAP"</option>
                                    <option value="GAP">"GAP"</option>
                                    <option value="Organic">"Organic"</option>
                                    <option value="HACCP">"HACCP"</option>
                                </select>
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{due_date_label.clone()}</span></label>
                                <input type="date" class="input input-bordered w-full" prop:value=move || due_date.get() on:input=move |ev| set_due_date.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_modal.set(false)>{cancel_label.clone()}</button>
                            <button class="btn btn-primary" on:click=move |_| {
                                let due_date_value = due_date.get();
                                let checklist_type_value = checklist_type.get();
                                let site_id = sites
                                    .read()
                                    .clone()
                                    .flatten()
                                    .and_then(|page| page.data.first().map(|site| site.id));
                                let no_site_message = no_site_for_checklist_label.to_string();
                                set_error.set(None);

                                spawn_local(async move {
                                    let Some(site_id) = site_id else {
                                        set_error.set(Some(no_site_message));
                                        return;
                                    };

                                    match api::create_compliance_checklist(api::CreateComplianceChecklistRequest {
                                        site_id,
                                        checklist_type: checklist_type_value,
                                        due_date: Some(format!("{}T00:00:00Z", due_date_value)),
                                    })
                                    .await
                                    {
                                        Ok(_) => {
                                            let _ = leptos::prelude::window().location().reload();
                                        }
                                        Err(e) => set_error.set(Some(e)),
                                    }
                                });
                            }>{save_label.clone()}</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
