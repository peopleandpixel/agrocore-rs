//! Tree management — `/trees`.
//!
//! The backend has had `list_trees`, `create_tree`, `update_tree`, `delete_tree`
//! plus the two by-plot and by-group lookups since the entity was introduced.
//! This page collected a label and a count and discarded both (`let _ = (..)`),
//! so `/trees` had no caller at all.
//!
//! The list is loaded with an unfiltered request rather than filtered per plot, so
//! the page is useful before any plot is known.

use crate::api;
use icondata::{LuPlus, LuTrash2};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// The tree kinds the API accepts.
///
/// `TreeType` in the domain is `CorkOak | Olive | Almond | Other(String)`, and
/// `Other` carries a caller-chosen string, so the select offers the three named
/// kinds and the free-text field stays empty unless "Other" is picked.
const TREE_TYPES: [(&str, &str); 4] = [
    ("CorkOak", "cork_oak"),
    ("Olive", "olive"),
    ("Almond", "almond"),
    ("Other", "other_label"),
];

#[component]
pub fn TreeManagement() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let trees = LocalResource::new(|| async move { api::fetch_trees().await.ok() });
    let (reload, set_reload) = signal(0u32);

    let (plot_id, set_plot_id) = signal(String::new());
    let (tree_type, set_tree_type) = signal(String::from("CorkOak"));
    let (count, set_count) = signal(String::from("1"));
    let (label, set_label) = signal(String::new());
    let (error, set_error) = signal(None::<String>);

    // The row being edited. `None` means the form is in create mode.
    let (editing, set_editing) = signal(None::<uuid::Uuid>);
    let (edit_count, set_edit_count) = signal(String::new());
    let (edit_label, set_edit_label) = signal(String::new());
    let (edit_type, set_edit_type) = signal(String::from("CorkOak"));

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let raw_plot = plot_id.get().trim().to_string();
        let plot = match uuid::Uuid::parse_str(&raw_plot) {
            Ok(id) => id,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "invalid_uuid")().to_string()));
                return;
            }
        };
        let raw_count = count.get().trim().to_string();
        let count_value: u32 = match raw_count.parse() {
            Ok(n) => n,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "tree_count")().to_string()));
                return;
            }
        };

        let kind = tree_type.get();
        let text = label.get().trim().to_string();
        let req = api::CreateTreeRequest {
            plot_id: plot,
            group_id: None,
            tree_type: kind.clone(),
            count: count_value,
            label: if text.is_empty() { None } else { Some(text) },
        };

        spawn_local(async move {
            match api::create_tree(req).await {
                Ok(_) => {
                    set_plot_id.set(String::new());
                    set_label.set(String::new());
                    set_count.set(String::from("1"));
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_update = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let Some(id) = editing.get() else {
            return;
        };

        let raw_count = edit_count.get().trim().to_string();
        let count_value: u32 = match raw_count.parse() {
            Ok(n) => n,
            Err(_) => {
                set_error.set(Some(crate::t!(t, "field_required")().to_string()));
                return;
            }
        };

        let text = edit_label.get().trim().to_string();
        let req = api::UpdateTreeRequest {
            group_id: None,
            tree_type: Some(edit_type.get()),
            count: Some(count_value),
            label: if text.is_empty() { None } else { Some(text) },
        };

        spawn_local(async move {
            match api::update_tree(id, req).await {
                Ok(_) => {
                    set_editing.set(None);
                    set_reload.update(|n| *n += 1);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_start_edit = move |tree: api::TreeDto| {
        set_editing.set(Some(tree.id));
        set_edit_count.set(tree.count.to_string());
        set_edit_label.set(tree.label.unwrap_or_default());
        set_edit_type.set(tree.tree_type);
    };

    let on_delete = move |id: uuid::Uuid| {
        spawn_local(async move {
            if let Err(e) = api::delete_tree(id).await {
                set_error.set(Some(format!("{}: {e}", crate::t!(t, "delete_failed")())));
            } else {
                set_reload.update(|n| *n += 1);
            }
        });
    };

    view! {
        <div class="p-6">
            <div class="flex justify-between items-center mb-4">
                <h1 class="text-2xl font-bold">{crate::t!(t, "nav_trees")}</h1>
            </div>

            {move || {
                let err = error.get();
                if let Some(e) = err {
                    view! { <div class="alert alert-error mb-4">{e}</div> }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <form class="card bg-base-100 shadow p-4 mb-6" on:submit=on_create>
                <h2 class="font-semibold mb-3">{crate::t!(t, "btn_add")}</h2>
                <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "plot_id_field")}</span>
                        <input
                            class="input input-bordered"
                            prop:value=move || plot_id.get()
                            on:input=move |ev| set_plot_id.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "tree_type")}</span>
                        <select
                            class="select select-bordered"
                            prop:value=move || tree_type.get()
                            on:change=move |ev| set_tree_type.set(event_target_value(&ev))
                        >
                            {TREE_TYPES
                                .iter()
                                .map(|(value, label_key)| {
                                    view! {
                                        <option value=*value>{crate::t!(t, *label_key)()}</option>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </label>
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "tree_count")}</span>
                        <input
                            class="input input-bordered"
                            type="number"
                            min="1"
                            prop:value=move || count.get()
                            on:input=move |ev| set_count.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="form-control">
                        <span class="label-text">{crate::t!(t, "tree_label")}</span>
                        <input
                            class="input input-bordered"
                            prop:value=move || label.get()
                            on:input=move |ev| set_label.set(event_target_value(&ev))
                        />
                    </label>
                </div>
                <button type="submit" class="btn btn-primary mt-3">
                    <Icon icon=LuPlus width="16" height="16" />
                    {crate::t!(t, "btn_add")}
                </button>
            </form>

            {move || {
                let current = editing.get();
                if current.is_none() {
                    return ().into_any();
                }
                view! {
                    <form
                        class="card bg-base-100 shadow p-4 mb-6 border-l-4 border-primary"
                        on:submit=on_update
                    >
                        <h2 class="font-semibold mb-3">
                            {crate::t!(t, "edit_label")}
                        </h2>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                            <label class="form-control">
                                <span class="label-text">{crate::t!(t, "tree_type")}</span>
                                <select
                                    class="select select-bordered"
                                    prop:value=move || edit_type.get()
                                    on:change=move |ev| set_edit_type.set(event_target_value(&ev))
                                >
                                    {TREE_TYPES
                                        .iter()
                                        .map(|(value, label_key)| {
                                            view! {
                                                <option value=*value>
                                                    {crate::t!(t, *label_key)()}
                                                </option>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </select>
                            </label>
                            <label class="form-control">
                                <span class="label-text">{crate::t!(t, "tree_count")}</span>
                                <input
                                    class="input input-bordered"
                                    type="number"
                                    min="1"
                                    prop:value=move || edit_count.get()
                                    on:input=move |ev| set_edit_count.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="form-control">
                                <span class="label-text">{crate::t!(t, "tree_label")}</span>
                                <input
                                    class="input input-bordered"
                                    prop:value=move || edit_label.get()
                                    on:input=move |ev| set_edit_label.set(event_target_value(&ev))
                                />
                            </label>
                        </div>
                        <div class="flex gap-2 mt-3">
                            <button type="submit" class="btn btn-primary">
                                {crate::t!(t, "btn_save")}
                            </button>
                            <button
                                type="button"
                                class="btn btn-ghost"
                                on:click=move |_| set_editing.set(None)
                            >
                                {crate::t!(t, "btn_cancel")}
                            </button>
                        </div>
                    </form>
                }
                    .into_any()
            }}

            {move || {
                let _ = reload.get();
                let loaded = trees.read();
                let page = loaded.as_ref().and_then(|p| p.clone());
                match page {
                    Some(page) if !page.data.is_empty() => {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="table table-zebra">
                                    <thead>
                                        <tr>
                                            <th>{crate::t!(t, "tree_label")}</th>
                                            <th>{crate::t!(t, "tree_type")}</th>
                                            <th>{crate::t!(t, "tree_count")}</th>
                                            <th>{crate::t!(t, "plot_id_field")}</th>
                                            <th>{crate::t!(t, "actions")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || page.data.clone()
                                            key=|tree: &api::TreeDto| tree.id
                                            children=move |tree: api::TreeDto| {
                                                let id = tree.id;
                                                let tree_for_edit = tree.clone();
                                                view! {
                                                    <tr>
                                                        <td>
                                                            {tree.label.clone().unwrap_or_else(|| "-".to_string())}
                                                        </td>
                                                        <td>{tree.tree_type.clone()}</td>
                                                        <td>{tree.count}</td>
                                                        <td>{tree.plot_id.to_string()}</td>
                                                        <td class="flex gap-1">
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| on_start_edit(tree_for_edit.clone())
                                                            >
                                                                {crate::t!(t, "edit_label")}
                                                            </button>
                                                            <button
                                                                class="btn btn-ghost btn-xs"
                                                                on:click=move |_| on_delete(id)
                                                            >
                                                                <Icon icon=LuTrash2 width="14" height="14" />
                                                                {crate::t!(t, "btn_delete")}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            </div>
                        }
                            .into_any()
                    }
                    Some(_) => {
                        view! {
                            <div class="alert">{crate::t!(t, "no_records")}</div>
                        }
                            .into_any()
                    }
                    None => {
                        view! {
                            <div class="alert">{crate::t!(t, "loading")}</div>
                        }
                            .into_any()
                    }
                }
            }}
        </div>
    }
}
