use crate::api;
use crate::components::toast::{ToastContext, ToastType};
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// CustomersPage — customer management with search and lookup.
/// Consumes orphaned API routes:
///   GET /api/v1/customers/number/{number}
///   GET /api/v1/customers/search/{query}
///   GET /api/v1/customers/{id}/orders
#[component]
pub fn CustomersPage() -> impl IntoView {
    let t = crate::i18n::use_i18n();
    let toast_context = use_context::<ToastContext>().expect("ToastContext not provided");

    // Input signals
    let (search_query, set_search_query) = signal(String::new());
    let (search_mode, set_search_mode) = signal("search");
    let (customer_number, set_customer_number) = signal(String::new());

    // Results signals
    let (customers, set_customers) = signal(Vec::<api::CustomerDto>::new());
    let (selected_customer, set_selected_customer) = signal(None::<api::CustomerDto>);
    let (customer_orders, set_customer_orders) = signal(Vec::<api::OrderDto>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let on_search_by_name = move |_| {
        let q = search_query.get();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match api::search_customers(&q).await {
                Ok(results) => {
                    set_customers.set(results);
                    set_selected_customer.set(None);
                    set_customer_orders.set(Vec::new());
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    let on_search_by_number = move |_| {
        let num = customer_number.get();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match api::fetch_customer_by_number(&num).await {
                Ok(customer) => {
                    set_selected_customer.set(Some(customer.clone()));
                    set_customers.set(vec![customer]);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    let on_view_orders = move |customer: api::CustomerDto| {
        set_selected_customer.set(Some(customer.clone()));
        spawn_local(async move {
            match api::fetch_customer_orders(customer.id).await {
                Ok(orders) => {
                    set_customer_orders.set(orders);
                    toast_context
                        .add_toast
                        .run((t("orders_loaded").to_string(), ToastType::Success));
                }
                Err(e) => {
                    set_customer_orders.set(Vec::new());
                    toast_context.add_toast.run((e, ToastType::Error));
                }
            }
        });
    };

    let on_customer_select = move |customer: api::CustomerDto| {
        set_selected_customer.set(Some(customer.clone()));
        spawn_local(async move {
            match api::fetch_customer_orders(customer.id).await {
                Ok(orders) => set_customer_orders.set(orders),
                Err(e) => {
                    set_customer_orders.set(Vec::new());
                    toast_context.add_toast.run((e, ToastType::Error));
                }
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">{crate::t!(t, "customer_management")}</h1>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert alert-error">
                    <Icon icon=LuCircleAlert width="20" height="20" />
                    <span>{err}</span>
                </div>
            })}

            <div class="tabs tabs-box">
                <button
                    class={move || if search_mode.get() == "search" { "tab tab-active" } else { "tab" }}
                    on:click=move |_| set_search_mode.set("search")
                >
                    {crate::t!(t, "search_by_name")}
                </button>
                <button
                    class={move || if search_mode.get() == "number" { "tab tab-active" } else { "tab" }}
                    on:click=move |_| set_search_mode.set("number")
                >
                    {crate::t!(t, "search_by_number")}
                </button>
            </div>

            <Show when=move || search_mode.get() == "search">
                <div class="flex gap-2">
                    <input
                        type="text"
                        class="input input-bordered flex-1"
                        placeholder={crate::t!(t, "search_customers")}
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                    <button class="btn btn-primary" on:click=on_search_by_name>
                        <Icon icon=LuSearch width="16" height="16" />
                        {crate::t!(t, "search")}
                    </button>
                </div>
            </Show>

            <Show when=move || search_mode.get() == "number">
                <div class="flex gap-2">
                    <input
                        type="text"
                        class="input input-bordered flex-1"
                        placeholder={crate::t!(t, "customer_number")}
                        prop:value=move || customer_number.get()
                        on:input=move |ev| set_customer_number.set(event_target_value(&ev))
                    />
                    <button class="btn btn-primary" on:click=on_search_by_number>
                        <Icon icon=LuSearch width="16" height="16" />
                        {crate::t!(t, "search")}
                    </button>
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div class="flex justify-center py-8">
                    <div class="loading loading-spinner loading-lg"></div>
                </div>
            </Show>

            <Show when=move || !loading.get() && customers.get().is_empty() && selected_customer.get().is_none()>
                <div class="text-center py-8 text-base-content/60">
                    <Icon icon=LuUsers width="48" height="48" />
                    <p>{crate::t!(t, "no_customers_found")}</p>
                </div>
            </Show>

            <Show when=move || !loading.get() && selected_customer.get().is_some() || !customers.get().is_empty()>
                {move || {
                    let selected = selected_customer.get().clone();
                    let list = customers.get().clone();
                    if let Some(c) = selected {
                        let c_clone = c.clone();
                        let c_name = c.name;
                        let c_email = c.email.unwrap_or_default();
                        let c_id = c.id.to_string();
                        let c_tenant = c.tenant_id.to_string();
                        view! {
                            <div class="card bg-base-100 shadow">
                                <div class="card-body">
                                    <h2 class="card-title">{c_name}</h2>
                                    <p class="text-base-content/60">{c_email}</p>
                                    <div class="grid grid-cols-2 md:grid-cols-3 gap-4 mt-4">
                                        <div class="stat">
                                            <div class="stat-label">{crate::t!(t, "customer_id")}</div>
                                            <div class="stat-value text-sm">{c_id}</div>
                                        </div>
                                        <div class="stat">
                                            <div class="stat-label">{crate::t!(t, "tenant")}</div>
                                            <div class="stat-value text-sm">{c_tenant}</div>
                                        </div>
                                    </div>
                                    <div class="mt-4">
                                        <button class="btn btn-outline btn-primary" on:click=move |_| on_view_orders(c_clone.clone())>
                                            <Icon icon=LuShoppingBag width="16" height="16" />
                                            {crate::t!(t, "view_orders")}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else if !list.is_empty() {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-hover">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "name")}</th>
                                            <th>{crate::t!(t, "email")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For each=move || list.clone() key=|c| c.id children=move |c| {
                                            view! {
                                                <tr>
                                                    <td>{c.name.clone()}</td>
                                                    <td>{c.email.clone().unwrap_or_default()}</td>
                                                    <td>
                                                        <button class="btn btn-sm btn-ghost" on:click=move |_| on_customer_select(c.clone())>
                                                            {crate::t!(t, "view_details")}
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        } />
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </Show>

            <Show when=move || !customer_orders.get().is_empty()>
                <div class="card bg-base-100 shadow mt-4">
                    <div class="card-body">
                        <h3 class="font-bold">{crate::t!(t, "customer_orders")}</h3>
                        <div class="overflow-x-auto">
                            <table class="table table-sm">
                                <thead>
                                    <tr>
                                        <th>{crate::t!(t, "order_type")}</th>
                                        <th>{crate::t!(t, "status")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For each=move || customer_orders.get().clone() key=|o| o.id children=move |order| {
                                        view! {
                                            <tr>
                                                <td>{order.order_type}</td>
                                                <td><div class="badge badge-outline">{order.status}</div></td>
                                            </tr>
                                        }
                                    } />
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
