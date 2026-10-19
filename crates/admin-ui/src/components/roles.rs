use leptos::prelude::*;

/// Roles & Permissions matrix page
#[component]
pub fn RolesPage() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    view! {
        <div class="flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold">{t("role_permissions")}</h1>
                <p class="text-base-content/60">{t("role_permissions_desc")}</p>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title">{t("roles_overview")}</h2>

                    <div class="overflow-x-auto">
                        <table class="table table-zebra">
                            <thead>
                                <tr>
                                    <th>{t("role")}</th>
                                    <th>{t("description")}</th>
                                    <th>{t("sites")}</th>
                                    <th>{t("equipment")}</th>
                                    <th>{t("orders")}</th>
                                    <th>{t("users")}</th>
                                    <th>{t("finance")}</th>
                                    <th>{t("analytics")}</th>
                                    <th>{t("impersonation")}</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td><strong class="text-primary">{t("role_superadmin")}</strong></td>
                                    <td class="text-xs">{t("role_superadmin_desc")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all_tenants")}</td>
                                </tr>
                                <tr>
                                    <td><strong class="text-secondary">{t("role_admin")}</strong></td>
                                    <td class="text-xs">{t("role_admin_desc")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_tenant_users")}</td>
                                </tr>
                                <tr>
                                    <td><strong class="text-accent">{t("role_manager")}</strong></td>
                                    <td class="text-xs">{t("role_manager_desc")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_all")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_read_update")}</td>
                                    <td class="text-xs">{t("permissions_read_create")}</td>
                                    <td class="text-xs">{t("permissions_none")}</td>
                                </tr>
                                <tr>
                                    <td><strong>{t("role_worker")}</strong></td>
                                    <td class="text-xs">{t("role_worker_desc")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_none")}</td>
                                    <td class="text-xs">{t("permissions_none")}</td>
                                    <td class="text-xs">{t("permissions_none")}</td>
                                </tr>
                                <tr>
                                    <td><strong class="text-base-content/50">{t("role_viewer")}</strong></td>
                                    <td class="text-xs">{t("role_viewer_desc")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_none")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_read")}</td>
                                    <td class="text-xs">{t("permissions_none")}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    <div class="mt-4 alert">
                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-info h-6 w-6 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <span>{t("role_permissions_note")}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
