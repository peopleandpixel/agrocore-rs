use crate::api;
use leptos::prelude::*;

#[component]
pub fn AuditLogPage() -> impl IntoView {
    let logs = LocalResource::new(|| async move { api::fetch_audit_logs().await.ok() });

    view! {
        <div class="flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold">"Audit Logs"</h1>
                <p class="text-base-content/60">"Verfolgen Sie alle Änderungen im System."</p>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table table-sm">
                        <thead>
                            <tr>
                                <th>"Zeitstempel"</th>
                                <th>"Entität"</th>
                                <th>"Aktion"</th>
                                <th>"Benutzer"</th>
                                <th>"Änderungen"</th>
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
                                    view! {
                                        <tr>
                                            <td>{log.timestamp}</td>
                                            <td>
                                                <div class="font-bold">{log.entity_type}</div>
                                                <div class="text-xs opacity-50">{log.entity_id.to_string()}</div>
                                            </td>
                                            <td><div class=format!("badge badge-sm {}", action_class)>{log.action}</div></td>
                                            <td>{log.user_id.to_string()}</td>
                                            <td>
                                                <div class="flex flex-col gap-1 max-w-md">
                                                    {log.old_value.as_ref().map(|v| view! {
                                                        <div class="text-xs line-through opacity-50 truncate">{v.to_string()}</div>
                                                    })}
                                                    {log.new_value.as_ref().map(|v| view! {
                                                        <div class="text-xs truncate">{v.to_string()}</div>
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
