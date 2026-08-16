use crate::api;
use leptos::prelude::{window, *};
use leptos::task::spawn_local;
use leptos_icons::Icon;

// Re-export commonly used icons
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

    // Form state for add item
    let (name, set_name) = signal(String::new());
    let (category, set_category) = signal(String::from("seed"));
    let (unit, set_unit) = signal(String::from("kg"));
    let (minimum_stock, set_minimum_stock) = signal(String::from("0"));
    let (sku, set_sku) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (inventory_method, set_inventory_method) = signal(String::from("FIFO"));

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
                                                            <button
                                                                class="btn btn-sm btn-ghost"
                                                                on:click=move |_| on_delete(item_id)
                                                            >
                                                                <Icon icon=LuTrash2 width="14" height="14" />
                                                            </button>
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
