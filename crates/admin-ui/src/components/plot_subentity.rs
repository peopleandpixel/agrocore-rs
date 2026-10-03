//! Plot sub-entity overview — `/plot/entities`.
//!
//! The page this replaces rendered four rows of invented data: a herd called
//! "Herde 1", two goats, four cork oaks and a barn, all hard-coded in the markup
//! and never read from anywhere. It looked like a working overview and showed
//! nothing about the tenant's actual plots.
//!
//! It now reads real records: pick a site, and its groups, trees and buildings
//! are listed side by side with counts. An empty plot shows an empty row rather
//! than placeholder content, because "no groups" and "four cork oaks" must not
//! look alike.

use crate::api;
use icondata::LuMap;
use leptos::prelude::*;
use leptos_icons::Icon;

/// One row in the overview, whichever entity it came from.
///
/// The three entities have no common shape — a group has a label and a type, a
/// tree has a type and a count, a building has a type and an optional label — so
/// the overview reduces them to what it actually displays instead of pretending
/// they share a DTO.
struct Row {
    kind: String,
    name: String,
    group: String,
    count: Option<i32>,
}

#[component]
pub fn PlotSubEntityLayout() -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let sites = LocalResource::new(|| async move { api::fetch_sites().await.ok() });
    let (selected, set_selected) = signal(String::new());

    let plot = uuid::Uuid::parse_str(selected.get().trim()).ok();

    let groups = LocalResource::new(move || async move {
        match uuid::Uuid::parse_str(selected.get().trim()) {
            Ok(id) => api::fetch_groups_by_plot(id).await.ok(),
            Err(_) => None,
        }
    });
    let trees = LocalResource::new(move || async move {
        match uuid::Uuid::parse_str(selected.get().trim()) {
            Ok(id) => api::fetch_trees_by_plot(id).await.ok(),
            Err(_) => None,
        }
    });
    let buildings = LocalResource::new(move || async move {
        match uuid::Uuid::parse_str(selected.get().trim()) {
            Ok(id) => api::fetch_buildings_by_plot(id).await.ok(),
            Err(_) => None,
        }
    });

    let rows = move || {
        let mut out: Vec<Row> = Vec::new();

        if let Some(page) = groups.read().as_ref().and_then(|p| p.clone()) {
            for group in page.data {
                out.push(Row {
                    kind: crate::t!(t, "kind_group")(),
                    name: group.label,
                    group: group
                        .parent_group_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| crate::t!(t, "none_label")().to_string()),
                    count: None,
                });
            }
        }

        if let Some(page) = trees.read().as_ref().and_then(|p| p.clone()) {
            for tree in page.data {
                out.push(Row {
                    kind: crate::t!(t, "kind_tree")(),
                    name: tree.tree_type,
                    group: tree
                        .group_id
                        .clone()
                        .unwrap_or_else(|| crate::t!(t, "none_label")().to_string()),
                    count: Some(tree.count),
                });
            }
        }

        if let Some(page) = buildings.read().as_ref().and_then(|p| p.clone()) {
            for building in page.data {
                out.push(Row {
                    kind: crate::t!(t, "kind_building")(),
                    name: building.building_type,
                    group: crate::t!(t, "none_label")().to_string(),
                    count: None,
                });
            }
        }

        out.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.name.cmp(&b.name)));
        out
    };

    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold mb-4">{crate::t!(t, "plot_subentity_title")}</h1>

            <label class="form-control max-w-md mb-6">
                <span class="label-text">{crate::t!(t, "plot_id_field")}</span>
                <select
                    class="select select-bordered"
                    prop:value=move || selected.get()
                    on:change=move |ev| set_selected.set(event_target_value(&ev))
                >
                    <option value="">{crate::t!(t, "plot_choose")}</option>
                    {move || {
                        let loaded = sites.read();
                        let page = loaded.as_ref().and_then(|p| p.clone());
                        match page {
                            Some(page) => page
                                .data
                                .iter()
                                .map(|site| {
                                    let id = site.id.to_string();
                                    let label = site.label.clone();
                                    view! { <option value=id>{label}</option> }
                                })
                                .collect::<Vec<_>>(),
                            None => Vec::new(),
                        }
                    }}
                </select>
            </label>

            {move || {
                if plot.is_none() {
                    return view! {
                        <div class="alert">
                            <Icon icon=LuMap width="16" height="16" />
                            {crate::t!(t, "plot_choose_hint")}
                        </div>
                    }
                        .into_any();
                }

                let list = rows();
                if list.is_empty() {
                    return view! { <div class="alert">{crate::t!(t, "no_records")}</div> }.into_any();
                }

                view! {
                    <div class="overflow-x-auto">
                        <table class="table table-zebra">
                            <thead>
                                <tr>
                                    <th>{crate::t!(t, "plot_subentity_kind")}</th>
                                    <th>{crate::t!(t, "tree_label")}</th>
                                    <th>{crate::t!(t, "parent_group")}</th>
                                    <th>{crate::t!(t, "tree_count")}</th>
                                </tr>
                            </thead>
                            <tbody>
                                {list
                                    .into_iter()
                                    .map(|row| {
                                                                let count = row.count.map(|c| c.to_string()).unwrap_or_else(|| {
                                            crate::t!(t, "none_label")().to_string()
                                        });
                                        view! {
                                            <tr>
                                                <td>{row.kind}</td>
                                                <td>{row.name}</td>
                                                <td>{row.group}</td>
                                                <td>{count}</td>
                                            </tr>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
}
