use crate::api;
use leptos::prelude::*;

#[component]
pub fn AuditLogPage() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let logs = LocalResource::new(|| async move { api::fetch_audit_logs().await.ok() });

    view! {
        <div class="flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold">{crate::t!(t, "nav_audit")}</h1>
                <p class="text-base-content/60">{crate::t!(t, "audit_desc")}</p>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table table-sm">
                        <thead>
                            <tr>
                                <th>{crate::t!(t, "timestamp")}</th>
                                <th>{crate::t!(t, "entity")}</th>
                                <th>{crate::t!(t, "action")}</th>
                                <th>{crate::t!(t, "user")}</th>
                                <th>{crate::t!(t, "changes")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || {
                                    logs
                                        .read()
                                        .as_ref()
                                        .map(|l| {
                                            l.as_ref()
                                                .map(|page| page.data.clone())
                                                .unwrap_or_default()
                                        })
                                        .unwrap_or_default()
                                }
                                key=|log| log.id
                                children=move |log| {
                                    let action_class = match log.action.as_str() {
                                        "CREATE" => "badge-success",
                                        "UPDATE" => "badge-info",
                                        "DELETE" => "badge-error",
                                        _ => "badge-ghost",
                                    };
                                    let entity_type = log.entity_type.clone();
                                    let entity_id = log.entity_id.to_string();
                                    let action = log.action.clone();
                                    let user_id = log.user_id.to_string();
                                    let timestamp = log.timestamp.clone();
                                    let old_value = log.old_value.clone();
                                    let new_value = log.new_value.clone();

                                    view! {
                                        <tr>
                                            <td>{timestamp}</td>
                                            <td>
                                                <div class="font-bold">{entity_type}</div>
                                                <div class="text-xs opacity-50">{entity_id}</div>
                                            </td>
                                            <td><div class=format!("badge badge-sm {}", action_class)>{action}</div></td>
                                            <td>{user_id}</td>
                                            <td>
                                                <div class="flex flex-col gap-1 max-w-md">
                                                    {old_value.as_ref().map(|v| {
                                                        let val_str = v.to_string();
                                                        view! {
                                                            <div class="text-xs line-through opacity-50 truncate">{val_str}</div>
                                                        }
                                                    })}
                                                    {new_value.as_ref().map(|v| {
                                                        let val_str = v.to_string();
                                                        view! {
                                                            <div class="text-xs truncate">{val_str}</div>
                                                        }
                                                    })}
                                                </div>
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
