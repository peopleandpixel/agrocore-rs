use crate::api;
use crate::icon::Icon;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;

// Re-export commonly used icons
use icondata_lu::{
    LuArrowDownToLine, LuArrowRightLeft, LuArrowUpToLine, LuPencil, LuPlus, LuSlidersHorizontal,
    LuTrash2, LuTriangleAlert as LuAlertTriangle,
};

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
    let (success_msg, set_success_msg) = signal(None::<String>);

    // Form state for add item
    let (name, set_name) = signal(String::new());
    let (category, set_category) = signal(String::from("seed"));
    let (unit, set_unit) = signal(String::from("kg"));
    let (minimum_stock, set_minimum_stock) = signal(String::from("0"));
    let (sku, set_sku) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (inventory_method, set_inventory_method) = signal(String::from("FIFO"));

    // Edit modal state
    let (show_edit_modal, set_show_edit_modal) = signal(false);
    let edit_item_id = RwSignal::<Option<uuid::Uuid>>::new(None);
    let set_edit_item_id = edit_item_id;
    let (edit_name, set_edit_name) = signal(String::new());
    let (edit_category, set_edit_category) = signal(String::from("seed"));
    let (edit_unit, set_edit_unit) = signal(String::from("kg"));
    let (edit_minimum_stock, set_edit_minimum_stock) = signal(String::from("0"));
    let (edit_sku, set_edit_sku) = signal(String::new());
    let (edit_description, set_edit_description) = signal(String::new());
    let (edit_inventory_method, set_edit_inventory_method) = signal(String::from("FIFO"));

    // Stock-In modal state
    let (show_stock_in_modal, set_show_stock_in_modal) = signal(false);
    let stock_in_item_id = RwSignal::<Option<uuid::Uuid>>::new(None);
    let set_stock_in_item_id = stock_in_item_id;
    let (stock_in_item_name, set_stock_in_item_name) = signal(String::new());
    let (stock_in_quantity, set_stock_in_quantity) = signal(String::new());
    let (stock_in_unit_cost, set_stock_in_unit_cost) = signal(String::new());
    let (stock_in_batch, set_stock_in_batch) = signal(String::new());
    let (stock_in_location, set_stock_in_location) = signal(String::new());
    let (stock_in_notes, set_stock_in_notes) = signal(String::new());

    // Stock-Out modal state
    let (show_stock_out_modal, set_show_stock_out_modal) = signal(false);
    let stock_out_item_id = RwSignal::<Option<uuid::Uuid>>::new(None);
    let set_stock_out_item_id = stock_out_item_id;
    let (stock_out_item_name, set_stock_out_item_name) = signal(String::new());
    let (stock_out_quantity, set_stock_out_quantity) = signal(String::new());
    let (stock_out_location, set_stock_out_location) = signal(String::new());
    let (stock_out_notes, set_stock_out_notes) = signal(String::new());

    // Transfer modal state
    let (show_transfer_modal, set_show_transfer_modal) = signal(false);
    let transfer_item_id = RwSignal::<Option<uuid::Uuid>>::new(None);
    let set_transfer_item_id = transfer_item_id;
    let (transfer_item_name, set_transfer_item_name) = signal(String::new());
    let (transfer_quantity, set_transfer_quantity) = signal(String::new());
    let (transfer_from, set_transfer_from) = signal(String::new());
    let (transfer_to, set_transfer_to) = signal(String::new());

    // Adjust modal state
    let (show_adjust_modal, set_show_adjust_modal) = signal(false);
    let adjust_item_id = RwSignal::<Option<uuid::Uuid>>::new(None);
    let set_adjust_item_id = adjust_item_id;
    let (adjust_item_name, set_adjust_item_name) = signal(String::new());
    let (adjust_quantity, set_adjust_quantity) = signal(String::new());
    let (adjust_notes, set_adjust_notes) = signal(String::new());

    // Active tab navigation
    let (active_tab, set_active_tab) = signal("items");

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            if api::delete_inventory_item(id).await.is_ok() {
                let _ = window().location().reload();
            }
        });
    };

    let on_add_item = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let cat = category.get();
        let min_str = minimum_stock.get();

        if name.get().is_empty() {
            set_error.set(Some("Item name is required".to_string()));
            return;
        }
        if min_str.parse::<f64>().is_err() {
            set_error.set(Some("Minimum stock must be a valid number".to_string()));
            return;
        }

        let req = api::CreateInventoryItemRequest {
            category: cat,
            name: name.get(),
            sku: if sku.get().is_empty() {
                None
            } else {
                Some(sku.get())
            },
            description: if description.get().is_empty() {
                None
            } else {
                Some(description.get())
            },
            unit: unit.get(),
            minimum_stock: minimum_stock.get().parse::<f64>().unwrap_or(0.0),
            inventory_method: inventory_method.get(),
        };

        spawn_local(async move {
            match api::create_inventory_item(req).await {
                Ok(_) => {
                    let _ = window().location().reload();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    // Edit item handler
    let on_edit = move |item: api::InventoryItemDto| {
        set_edit_item_id.set(Some(item.id));
        set_edit_name.set(item.name);
        set_edit_category.set(item.category);
        set_edit_unit.set(item.unit);
        set_edit_minimum_stock.set(item.minimum_stock.to_string());
        set_edit_sku.set(item.sku.unwrap_or_default());
        set_edit_description.set(item.description.unwrap_or_default());
        set_edit_inventory_method.set(item.inventory_method);
        set_show_edit_modal.set(true);
    };

    let on_update_item = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let item_id = edit_item_id.get();
        let min_str = edit_minimum_stock.get();

        if let Some(id) = item_id {
            if min_str.parse::<f64>().is_err() {
                set_error.set(Some("Minimum stock must be a valid number".to_string()));
                return;
            }

            let req = api::UpdateInventoryItemRequest {
                category: edit_category.get(),
                name: edit_name.get(),
                sku: if edit_sku.get().is_empty() {
                    None
                } else {
                    Some(edit_sku.get())
                },
                description: if edit_description.get().is_empty() {
                    None
                } else {
                    Some(edit_description.get())
                },
                unit: edit_unit.get(),
                minimum_stock: min_str.parse::<f64>().unwrap_or(0.0),
                inventory_method: edit_inventory_method.get(),
            };

            spawn_local(async move {
                match api::update_inventory_item(id, req).await {
                    Ok(_) => {
                        let _ = window().location().reload();
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // Stock-In handler
    let on_stock_in = move |item: api::InventoryItemDto| {
        set_stock_in_item_id.set(Some(item.id));
        set_stock_in_item_name.set(item.name);
        set_stock_in_quantity.set(String::new());
        set_stock_in_unit_cost.set(String::new());
        set_stock_in_batch.set(String::new());
        set_stock_in_location.set(String::new());
        set_stock_in_notes.set(String::new());
        set_show_stock_in_modal.set(true);
    };

    let on_submit_stock_in = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let item_id = stock_in_item_id.get();
        let qty_str = stock_in_quantity.get();
        let qty: f64 = qty_str.parse().unwrap_or(0.0);

        if let Some(id) = item_id {
            if qty <= 0.0 {
                set_error.set(Some("Quantity must be positive".to_string()));
                return;
            }

            let req = api::StockInRequest {
                item_id: id,
                quantity: qty,
                unit_cost: if stock_in_unit_cost.get().is_empty() {
                    None
                } else {
                    Some(stock_in_unit_cost.get().parse().unwrap_or(0.0))
                },
                batch_number: if stock_in_batch.get().is_empty() {
                    None
                } else {
                    Some(stock_in_batch.get())
                },
                expiration_date: None,
                location: if stock_in_location.get().is_empty() {
                    None
                } else {
                    Some(stock_in_location.get())
                },
                notes: if stock_in_notes.get().is_empty() {
                    None
                } else {
                    Some(stock_in_notes.get())
                },
            };

            spawn_local(async move {
                match api::stock_in(req).await {
                    Ok(_) => {
                        set_success_msg.set(Some("Stock-in successful".to_string()));
                        set_show_stock_in_modal.set(false);
                        let _ = window().location().reload();
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // Stock-Out handler
    let on_stock_out = move |item: api::InventoryItemDto| {
        set_stock_out_item_id.set(Some(item.id));
        set_stock_out_item_name.set(item.name);
        set_stock_out_quantity.set(String::new());
        set_stock_out_location.set(String::new());
        set_stock_out_notes.set(String::new());
        set_show_stock_out_modal.set(true);
    };

    let on_submit_stock_out = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let item_id = stock_out_item_id.get();
        let qty_str = stock_out_quantity.get();
        let qty: f64 = qty_str.parse().unwrap_or(0.0);

        if let Some(id) = item_id {
            if qty <= 0.0 {
                set_error.set(Some("Quantity must be positive".to_string()));
                return;
            }

            let req = api::StockOutRequest {
                item_id: id,
                quantity: qty,
                location: if stock_out_location.get().is_empty() {
                    None
                } else {
                    Some(stock_out_location.get())
                },
                notes: if stock_out_notes.get().is_empty() {
                    None
                } else {
                    Some(stock_out_notes.get())
                },
            };

            spawn_local(async move {
                match api::stock_out(req).await {
                    Ok(_) => {
                        set_success_msg.set(Some("Stock-out successful".to_string()));
                        set_show_stock_out_modal.set(false);
                        let _ = window().location().reload();
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // Transfer handler
    let on_transfer = move |item: api::InventoryItemDto| {
        set_transfer_item_id.set(Some(item.id));
        set_transfer_item_name.set(item.name);
        set_transfer_quantity.set(String::new());
        set_transfer_from.set(String::new());
        set_transfer_to.set(String::new());
        set_show_transfer_modal.set(true);
    };

    let on_submit_transfer = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let item_id = transfer_item_id.get();
        let qty_str = transfer_quantity.get();
        let qty: f64 = qty_str.parse().unwrap_or(0.0);
        let from_loc = transfer_from.get();
        let to_loc = transfer_to.get();

        if let Some(id) = item_id {
            if qty <= 0.0 {
                set_error.set(Some("Quantity must be positive".to_string()));
                return;
            }
            if from_loc.is_empty() || to_loc.is_empty() {
                set_error.set(Some("Both locations are required".to_string()));
                return;
            }

            let req = api::TransferRequest {
                item_id: id,
                quantity: qty,
                from_location: from_loc,
                to_location: to_loc,
            };

            spawn_local(async move {
                match api::transfer_inventory(req).await {
                    Ok(_) => {
                        set_success_msg.set(Some("Transfer successful".to_string()));
                        set_show_transfer_modal.set(false);
                        let _ = window().location().reload();
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // Adjust handler
    let on_adjust = move |item: api::InventoryItemDto| {
        set_adjust_item_id.set(Some(item.id));
        set_adjust_item_name.set(item.name);
        set_adjust_quantity.set(String::new());
        set_adjust_notes.set(String::new());
        set_show_adjust_modal.set(true);
    };

    let on_submit_adjust = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let item_id = adjust_item_id.get();
        let qty_str = adjust_quantity.get();
        let notes = adjust_notes.get();

        if let Some(id) = item_id {
            if notes.is_empty() {
                set_error.set(Some("Adjustment notes are required".to_string()));
                return;
            }

            let req = api::AdjustRequest {
                item_id: id,
                quantity: qty_str.parse().unwrap_or(0.0),
                notes,
            };

            spawn_local(async move {
                match api::adjust_inventory(req).await {
                    Ok(_) => {
                        set_success_msg.set(Some("Adjustment successful".to_string()));
                        set_show_adjust_modal.set(false);
                        let _ = window().location().reload();
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    view! {
        <div class="p-6">
            <div class="flex justify-between items-center mb-4">
                <h1 class="text-2xl font-bold">{crate::t!(t, "nav_inventory")}</h1>
                <button
                    class="btn btn-primary"
                    on:click=move |_| set_show_add_modal.set(true)
                >
                    <Icon icon=&LuPlus width="16" height="16" />
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
                let msg = success_msg.get();
                if let Some(m) = msg {
                    view! { <div class="alert alert-success mb-4">{m}</div> }.into_any()
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
                                <Icon icon=&LuAlertTriangle width="16" height="16" />
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

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                {move || {
                    let loaded = balances.read();
                    if let Some(balances) = loaded.as_ref() {
                        let balances = balances.clone().unwrap_or_default();
                        view! {
                            <For
                                each=move || balances.clone()
                                key=|b: &api::InventoryBalanceDto| b.item_id
                                children=move |b: api::InventoryBalanceDto| {
                                    let item_name = b.item_name.clone();
                                    let unit = b.unit.clone();
                                    let total_qty = b.total_quantity;
                                    let avail_qty = b.available_quantity;
                                    let is_low = b.is_below_minimum;
                                    let total_value = b.total_value;

                                    view! {
                                        <div class="stat-card card bg-base-200 shadow-xl p-4">
                                            <h2 class="card-title text-lg">{item_name}</h2>
                                            <div class="stat">
                                                <span class="stat-label">{crate::t!(t, "current_stock")}</span>
                                                <span class="stat-value">{format!("{:.1}", total_qty)}</span>
                                                <span class="stat-desc">{unit}</span>
                                            </div>
                                            <div class="stat">
                                                <span class="stat-label">{crate::t!(t, "available")}</span>
                                                <span class="stat-value text-sm">{format!("{:.1}", avail_qty)}</span>
                                            </div>
                                            {move || {
                                                if is_low {
                                                    view! { <div class="badge badge-warning mt-1">{crate::t!(t, "below_minimum")}</div> }.into_any()
                                                } else {
                                                    ().into_any()
                                                }
                                            }}
                                            {move || {
                                                if let Some(val) = total_value {
                                                    view! { <p class="text-xs opacity-70 mt-1">{format!("{:.2}", val)}</p> }.into_any()
                                                } else {
                                                    ().into_any()
                                                }
                                            }}
                                        </div>
                                    }
                                }
                            />
                        }.into_any()
                    } else {
                        view! { <div class="text-center py-4">{crate::t!(t, "loading")}</div> }.into_any()
                    }
                }}
            </div>

            <div class="tabs tabs-boxed mb-4">
                <a
                    class="tab"
                    class:tab-active={move || active_tab.get() == "items"}
                    on:click=move |_| set_active_tab.set("items")
                >
                    {crate::t!(t, "all_items")}
                </a>
                <a
                    class="tab"
                    class:tab-active={move || active_tab.get() == "locations"}
                    on:click=move |_| set_active_tab.set("locations")
                >
                    {crate::t!(t, "storage_locations")}
                </a>
            </div>

            <div class="overflow-x-auto">
                {move || {
                    let active = active_tab.get();
                    if active == "items" {
                        let loaded = items.read();
                        if let Some(items) = loaded.as_ref() {
                            let items = items.clone().unwrap_or_default();
                            view! {
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "item_name")}</th>
                                            <th>{crate::t!(t, "category")}</th>
                                            <th>{crate::t!(t, "unit")}</th>
                                            <th>{crate::t!(t, "inventory_method")}</th>
                                            <th>{crate::t!(t, "min_stock")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || items.data.clone()
                                            key=|i: &api::InventoryItemDto| i.id
                                            children=move |item: api::InventoryItemDto| {
                                                let item_id = item.id;
                                                let item_name = item.name.clone();
                                                let item_cat = item.category.clone();
                                                let item_unit = item.unit.clone();
                                                let item_method = item.inventory_method.clone();
                                                let item_min = item.minimum_stock;
                                                let item_for_edit = item.clone();
                                                let item_for_stock_in = item.clone();
                                                let item_for_stock_out = item.clone();
                                                let item_for_transfer = item.clone();
                                                let item_for_adjust = item.clone();

                                                view! {
                                                    <tr>
                                                        <td>{item_name}</td>
                                                        <td>{item_cat}</td>
                                                        <td>{item_unit}</td>
                                                        <td>
                                                            <span class="badge badge-ghost">{item_method}</span>
                                                        </td>
                                                        <td>{format!("{:.1}", item_min)}</td>
                                                        <td>
                                                            <div class="flex gap-1">
                                                                <button
                                                                    class="btn btn-sm btn-ghost btn-info"
                                                                    on:click=move |_| on_edit(item_for_edit.clone())
                                                                >
                                                                    <Icon icon=&LuPencil width="14" height="14" />
                                                                </button>
                                                                <button
                                                                    class="btn btn-sm btn-ghost btn-success"
                                                                    on:click=move |_| on_stock_in(item_for_stock_in.clone())
                                                                >
                                                                    <Icon icon=&LuArrowUpToLine width="14" height="14" />
                                                                </button>
                                                                <button
                                                                    class="btn btn-sm btn-ghost btn-warning"
                                                                    on:click=move |_| on_stock_out(item_for_stock_out.clone())
                                                                >
                                                                    <Icon icon=&LuArrowDownToLine width="14" height="14" />
                                                                </button>
                                                                <button
                                                                    class="btn btn-sm btn-ghost"
                                                                    on:click=move |_| on_transfer(item_for_transfer.clone())
                                                                >
                                                                    <Icon icon=&LuArrowRightLeft width="14" height="14" />
                                                                </button>
                                                                <button
                                                                    class="btn btn-sm btn-ghost"
                                                                    on:click=move |_| on_adjust(item_for_adjust.clone())
                                                                >
                                                                    <Icon icon=&LuSlidersHorizontal width="14" height="14" />
                                                                </button>
                                                                <button
                                                                    class="btn btn-sm btn-ghost btn-error"
                                                                    on:click=move |_| on_delete(item_id)
                                                                >
                                                                    <Icon icon=&LuTrash2 width="14" height="14" />
                                                                </button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            }.into_any()
                        } else {
                            view! { <p class="text-center py-4">{crate::t!(t, "loading")}</p> }.into_any()
                        }
                    } else {
                        let loaded = locations.read();
                        if let Some(locs) = loaded.as_ref() {
                            let locs = locs.clone().unwrap_or_default();
                            view! {
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "location_name")}</th>
                                            <th>{crate::t!(t, "location_code")}</th>
                                            <th>{crate::t!(t, "description")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || locs.data.clone()
                                            key=|l: &api::InventoryLocationDto| l.id
                                            children=move |loc: api::InventoryLocationDto| {
                                                let loc_name = loc.name.clone();
                                                let loc_code = loc.code.clone();
                                                let loc_desc = loc.description.clone();

                                                view! {
                                                    <tr>
                                                        <td>{loc_name}</td>
                                                        <td>{move || loc_code.clone().unwrap_or_default()}</td>
                                                        <td>{move || loc_desc.clone().unwrap_or_default()}</td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            }.into_any()
                        } else {
                            view! { <p class="text-center py-4">{crate::t!(t, "loading")}</p> }.into_any()
                        }
                    }
                }}
            </div>

            // Edit Modal
            {move || {
                if show_edit_modal.get() {
                    view! {
                        <dialog class="modal" open>
                            <div class="modal-box">
                                <h3 class="font-bold text-lg mb-4">"Edit Item"</h3>
                                <form on:submit=on_update_item class="space-y-4">
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">{crate::t!(t, "item_name")}</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=edit_name
                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">{crate::t!(t, "category")}</span></label>
                                        <select
                                            class="select select-bordered w-full"
                                            prop:value=edit_category
                                            on:change=move |ev| set_edit_category.set(event_target_value(&ev))
                                        >
                                            <option value="seed">{crate::t!(t, "category_seed")}</option>
                                            <option value="fertilizer">{crate::t!(t, "category_fertilizer")}</option>
                                            <option value="pesticide">{crate::t!(t, "category_pesticide")}</option>
                                            <option value="herbicide">{crate::t!(t, "category_herbicide")}</option>
                                            <option value="fuel">{crate::t!(t, "category_fuel")}</option>
                                            <option value="feed">{crate::t!(t, "category_feed")}</option>
                                            <option value="medicine">{crate::t!(t, "category_medicine")}</option>
                                            <option value="spare_part">{crate::t!(t, "category_spare_part")}</option>
                                            <option value="equipment">{crate::t!(t, "category_equipment")}</option>
                                            <option value="other">{crate::t!(t, "category_other")}</option>
                                        </select>
                                    </div>
                                    <div class="grid grid-cols-2 gap-4">
                                        <div class="form-control">
                                            <label class="label"><span class="label-text">{crate::t!(t, "unit")}</span></label>
                                            <select
                                                class="select select-bordered w-full"
                                                prop:value=edit_unit
                                                on:change=move |ev| set_edit_unit.set(event_target_value(&ev))
                                            >
                                                <option value="kg">kg</option>
                                                <option value="l">l</option>
                                                <option value="bag">Bag</option>
                                                <option value="unit">Stück</option>
                                                <option value="box">Box</option>
                                                <option value="bale">Bale</option>
                                            </select>
                                        </div>
                                        <div class="form-control">
                                            <label class="label"><span class="label-text">{crate::t!(t, "min_stock")}</span></label>
                                            <input
                                                type="number"
                                                step="0.1"
                                                class="input input-bordered w-full"
                                                prop:value=edit_minimum_stock
                                                on:input=move |ev| set_edit_minimum_stock.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"SKU"</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=edit_sku
                                            on:input=move |ev| set_edit_sku.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">{crate::t!(t, "inventory_method")}</span></label>
                                        <select
                                            class="select select-bordered w-full"
                                            prop:value=edit_inventory_method
                                            on:change=move |ev| set_edit_inventory_method.set(event_target_value(&ev))
                                        >
                                            <option value="FIFO">FIFO</option>
                                            <option value="FEFO">FEFO</option>
                                        </select>
                                    </div>
                                    <div class="modal-action">
                                        <button type="button" class="btn" on:click=move |_| set_show_edit_modal.set(false)>
                                            {crate::t!(t, "btn_cancel")}
                                        </button>
                                        <button type="submit" class="btn btn-primary">
                                            {crate::t!(t, "btn_save")}
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </dialog>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            // Stock-In Modal
            {move || {
                if show_stock_in_modal.get() {
                    let item_name = stock_in_item_name.get();
                    view! {
                        <dialog class="modal" open>
                            <div class="modal-box">
                                <h3 class="font-bold text-lg mb-4">{format!("Stock-In: {}", item_name)}</h3>
                                <form on:submit=on_submit_stock_in class="space-y-4">
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Quantity"</span></label>
                                        <input
                                            type="number"
                                            step="0.01"
                                            class="input input-bordered w-full"
                                            prop:value=stock_in_quantity
                                            on:input=move |ev| set_stock_in_quantity.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Unit Cost (€)"</span></label>
                                        <input
                                            type="number"
                                            step="0.01"
                                            class="input input-bordered w-full"
                                            prop:value=stock_in_unit_cost
                                            on:input=move |ev| set_stock_in_unit_cost.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Batch Number"</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=stock_in_batch
                                            on:input=move |ev| set_stock_in_batch.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Location"</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=stock_in_location
                                            on:input=move |ev| set_stock_in_location.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Notes"</span></label>
                                        <textarea
                                            class="textarea textarea-bordered w-full"
                                            prop:value=stock_in_notes
                                            on:input=move |ev| set_stock_in_notes.set(event_target_value(&ev))
                                        ></textarea>
                                    </div>
                                    <div class="modal-action">
                                        <button type="button" class="btn" on:click=move |_| set_show_stock_in_modal.set(false)>
                                            {crate::t!(t, "btn_cancel")}
                                        </button>
                                        <button type="submit" class="btn btn-success">
                                            "Stock In"
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </dialog>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            // Stock-Out Modal
            {move || {
                if show_stock_out_modal.get() {
                    let item_name = stock_out_item_name.get();
                    view! {
                        <dialog class="modal" open>
                            <div class="modal-box">
                                <h3 class="font-bold text-lg mb-4">{format!("Stock-Out: {}", item_name)}</h3>
                                <form on:submit=on_submit_stock_out class="space-y-4">
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Quantity"</span></label>
                                        <input
                                            type="number"
                                            step="0.01"
                                            class="input input-bordered w-full"
                                            prop:value=stock_out_quantity
                                            on:input=move |ev| set_stock_out_quantity.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Location"</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=stock_out_location
                                            on:input=move |ev| set_stock_out_location.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Notes"</span></label>
                                        <textarea
                                            class="textarea textarea-bordered w-full"
                                            prop:value=stock_out_notes
                                            on:input=move |ev| set_stock_out_notes.set(event_target_value(&ev))
                                        ></textarea>
                                    </div>
                                    <div class="modal-action">
                                        <button type="button" class="btn" on:click=move |_| set_show_stock_out_modal.set(false)>
                                            {crate::t!(t, "btn_cancel")}
                                        </button>
                                        <button type="submit" class="btn btn-warning">
                                            "Stock Out"
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </dialog>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            // Transfer Modal
            {move || {
                if show_transfer_modal.get() {
                    let item_name = transfer_item_name.get();
                    view! {
                        <dialog class="modal" open>
                            <div class="modal-box">
                                <h3 class="font-bold text-lg mb-4">{format!("Transfer: {}", item_name)}</h3>
                                <form on:submit=on_submit_transfer class="space-y-4">
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Quantity"</span></label>
                                        <input
                                            type="number"
                                            step="0.01"
                                            class="input input-bordered w-full"
                                            prop:value=transfer_quantity
                                            on:input=move |ev| set_transfer_quantity.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"From Location"</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=transfer_from
                                            on:input=move |ev| set_transfer_from.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"To Location"</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=transfer_to
                                            on:input=move |ev| set_transfer_to.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                    <div class="modal-action">
                                        <button type="button" class="btn" on:click=move |_| set_show_transfer_modal.set(false)>
                                            {crate::t!(t, "btn_cancel")}
                                        </button>
                                        <button type="submit" class="btn btn-primary">
                                            "Transfer"
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </dialog>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            // Adjust Modal
            {move || {
                if show_adjust_modal.get() {
                    let item_name = adjust_item_name.get();
                    view! {
                        <dialog class="modal" open>
                            <div class="modal-box">
                                <h3 class="font-bold text-lg mb-4">{format!("Adjust: {}", item_name)}</h3>
                                <form on:submit=on_submit_adjust class="space-y-4">
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"New Quantity"</span></label>
                                        <input
                                            type="number"
                                            step="0.01"
                                            class="input input-bordered w-full"
                                            prop:value=adjust_quantity
                                            on:input=move |ev| set_adjust_quantity.set(event_target_value(&ev))
                                            required
                                        />
                                        <label class="label"><span class="label-text-alt">"Positive = add, Negative = remove"</span></label>
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">"Reason"</span></label>
                                        <textarea
                                            class="textarea textarea-bordered w-full"
                                            prop:value=adjust_notes
                                            on:input=move |ev| set_adjust_notes.set(event_target_value(&ev))
                                            required
                                        ></textarea>
                                    </div>
                                    <div class="modal-action">
                                        <button type="button" class="btn" on:click=move |_| set_show_adjust_modal.set(false)>
                                            {crate::t!(t, "btn_cancel")}
                                        </button>
                                        <button type="submit" class="btn btn-primary">
                                            "Adjust"
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </dialog>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            // Add Modal
            {move || {
                if show_add_modal.get() {
                    view! {
                        <dialog class="modal" open>
                            <div class="modal-box">
                                <h3 class="font-bold text-lg mb-4">{crate::t!(t, "btn_add_item")}</h3>
                                <form on:submit=on_add_item class="space-y-4">
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">{crate::t!(t, "item_name")}</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=name
                                            on:input=move |ev| set_name.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">{crate::t!(t, "category")}</span></label>
                                        <select
                                            class="select select-bordered w-full"
                                            prop:value=category
                                            on:change=move |ev| set_category.set(event_target_value(&ev))
                                        >
                                            <option value="seed">{crate::t!(t, "category_seed")}</option>
                                            <option value="fertilizer">{crate::t!(t, "category_fertilizer")}</option>
                                            <option value="pesticide">{crate::t!(t, "category_pesticide")}</option>
                                            <option value="herbicide">{crate::t!(t, "category_herbicide")}</option>
                                            <option value="fuel">{crate::t!(t, "category_fuel")}</option>
                                            <option value="feed">{crate::t!(t, "category_feed")}</option>
                                            <option value="medicine">{crate::t!(t, "category_medicine")}</option>
                                            <option value="spare_part">{crate::t!(t, "category_spare_part")}</option>
                                            <option value="equipment">{crate::t!(t, "category_equipment")}</option>
                                            <option value="other">{crate::t!(t, "category_other")}</option>
                                        </select>
                                    </div>
                                    <div class="grid grid-cols-2 gap-4">
                                        <div class="form-control">
                                            <label class="label"><span class="label-text">{crate::t!(t, "unit")}</span></label>
                                            <select
                                                class="select select-bordered w-full"
                                                prop:value=unit
                                                on:change=move |ev| set_unit.set(event_target_value(&ev))
                                            >
                                                <option value="kg">kg</option>
                                                <option value="l">l</option>
                                                <option value="bag">Bag</option>
                                                <option value="unit">Stück</option>
                                                <option value="box">Box</option>
                                                <option value="bale">Bale</option>
                                            </select>
                                        </div>
                                        <div class="form-control">
                                            <label class="label"><span class="label-text">{crate::t!(t, "min_stock")}</span></label>
                                            <input
                                                type="number"
                                                step="0.1"
                                                class="input input-bordered w-full"
                                                prop:value=minimum_stock
                                                on:input=move |ev| set_minimum_stock.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">SKU</span></label>
                                        <input
                                            type="text"
                                            class="input input-bordered w-full"
                                            prop:value=sku
                                            on:input=move |ev| set_sku.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-control">
                                        <label class="label"><span class="label-text">{crate::t!(t, "inventory_method")}</span></label>
                                        <select
                                            class="select select-bordered w-full"
                                            prop:value=inventory_method
                                            on:change=move |ev| set_inventory_method.set(event_target_value(&ev))
                                        >
                                            <option value="FIFO">FIFO</option>
                                            <option value="FEFO">FEFO</option>
                                        </select>
                                    </div>
                                    <div class="modal-action">
                                        <button type="button" class="btn" on:click=move |_| set_show_add_modal.set(false)>
                                            {crate::t!(t, "btn_cancel")}
                                        </button>
                                        <button type="submit" class="btn btn-primary">
                                            {crate::t!(t, "btn_save")}
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </dialog>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}
