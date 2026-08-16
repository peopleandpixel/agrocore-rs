use crate::api;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

// Re-export commonly used icons at the top level for convenience
use icondata::{LuPlus, LuTrash2, LuTriangleAlert as LuAlertTriangle};

#[component]
#[allow(unused_variables)]
pub fn InventoryManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let items = LocalResource::new(|| async move { api::fetch_inventory_items().await.ok() });

    let balances = LocalResource::new(|| async move { api::fetch_inventory_balances().await.ok() });

    let below_minimum = LocalResource::new(|| async move { api::fetch_below_minimum().await.ok() });

    let locations =
        LocalResource::new(|| async move { api::fetch_inventory_locations().await.ok() });

    let (show_add_modal, set_show_add_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Form state
    let (name, set_name) = signal(String::new());
    let (category, set_category) = signal(String::from("seed"));
    let (unit, set_unit) = signal(String::from("kg"));
    let (minimum_stock, set_minimum_stock) = signal(String::from("0"));

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            if api::delete_inventory_item(id).await.is_ok() {
                let _ = window().location().reload();
            }
        });
    };

    view! {
        <div class="p-6">
            <div class="flex justify-between items-center mb-4">
                <h1 class="text-2xl font-bold">{crate::t!(t, "nav_inventory")}</h1>
                <button
                    class="btn btn-primary"
                    on:click=move |_| set_show_add_modal.set(true)
                >
                    <Icon icon=LuPlus width="16" height="16" />
                    {crate::t!(t, "btn_add_item")}
                </button>
            </div>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            {move || {
                let below = below_minimum.read();
                if let Some(items) = below.as_ref() {
                    let items = items.clone().unwrap_or_default();
                    if !items.is_empty() {
                        view! {
                            <div class="alert alert-warning mb-4">
                                <Icon icon=LuAlertTriangle width="16" height="16" />
                                {format!("{} {}", items.len(), crate::t!(t, "items_below_minimum")())}
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                } else {
                    ().into_any()
                }
            }}

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {move || {
                    let balances = balances.read();
                    if let Some(balances) = balances.as_ref() {
                        let balances = balances.clone().unwrap_or_default();
                        view! {
                            {balances.into_iter().map(|b| {
                                let is_low = b.is_below_minimum;
                                view! {
                                    <div class="card bg-base-200 shadow-xl">
                                        <div class="card-body">
                                            <h2 class="card-title">{b.item_name}</h2>
                                            <div class="stat">
                                                <span class="stat-label">{crate::t!(t, "current_stock")}</span>
                                                <span class="stat-value">{format!("{:.1}", b.total_quantity)}</span>
                                                <span class="stat-desc">{b.unit}</span>
                                            </div>
                                            <div class="stat">
                                                <span class="stat-label">{crate::t!(t, "available")}</span>
                                                <span class="stat-value">{format!("{:.1}", b.available_quantity)}</span>
                                            </div>
                                            {move || {
                                                if is_low {
                                                    view! { <div class="badge badge-warning">{crate::t!(t, "below_minimum")}</div> }.into_any()
                                                } else {
                                                    ().into_any()
                                                }
                                            }}
                                            {move || {
                                                if let Some(val) = b.total_value {
                                                    view! { <span class="text-sm opacity-70">{format!("Value: {:.2}", val)}</span> }.into_any()
                                                } else {
                                                    ().into_any()
                                                }
                                            }}
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        }.into_any()
                    } else {
                        view! { <div>{crate::t!(t, "loading")}</div> }.into_any()
                    }
                }}
            </div>

            <div class="mt-6">
                <h2 class="text-xl font-semibold mb-2">{crate::t!(t, "all_items")}</h2>
                <div class="overflow-x-auto">
                    <table class="table table-zebra">
                        <thead>
                            <tr>
                                <th>{crate::t!(t, "item_name")}</th>
                                <th>{crate::t!(t, "category")}</th>
                                <th>{crate::t!(t, "unit")}</th>
                                <th>{crate::t!(t, "min_stock")}</th>
                                <th>{crate::t!(t, "actions")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let loaded = items.read();
                                if let Some(items) = loaded.as_ref() {
                                    let items = items.clone().unwrap_or_default();
                                    view! {
                                        {items.data.into_iter().map(|item| {
                                            let item_id = item.id;
                                            view! {
                                                <tr>
                                                    <td>{item.name}</td>
                                                    <td>{item.category}</td>
                                                    <td>{item.unit}</td>
                                                    <td>{format!("{:.1}", item.minimum_stock)}</td>
                                                    <td>
                                                        <button
                                                            class="btn btn-sm btn-ghost"
                                                            on:click=move |_| on_delete(item_id)
                                                        >
                                                            <Icon icon=LuTrash2 width="14" height="14" />
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    }.into_any()
                                } else {
                                    view! { <tr><td colspan="5">{crate::t!(t, "loading")}</td></tr> }.into_any()
                                }
                            }}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
