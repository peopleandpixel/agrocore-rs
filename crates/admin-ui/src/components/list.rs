use crate::icon::Icon;
use icondata_lu::*;
use leptos::prelude::*;
use std::sync::Arc;

/// Sort direction for table columns
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortDirection {
    #[default]
    None,
    Asc,
    Desc,
}

impl SortDirection {
    pub fn next(&self) -> Self {
        match self {
            Self::None => Self::Asc,
            Self::Asc => Self::Desc,
            Self::Desc => Self::None,
        }
    }
}

/// Filter option for the list
#[derive(Clone)]
pub struct FilterOption {
    pub key: &'static str,
    pub label: String,
}

// ---------------------------------------------------------------------------
// SimpleColumn – column definition for sortable tables
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SimpleColumn<T> {
    pub key: String,
    pub label: String,
    pub sortable: bool,
    pub get_value: fn(&T) -> String,
}

// ---------------------------------------------------------------------------
// use_list_state – Hook: search + filter
// ---------------------------------------------------------------------------

/// Returns reactive state for a filterable, searchable list.
pub fn use_list_state<T>(
    items: ReadSignal<Vec<T>>,
    search_text_fn: fn(&T) -> String,
    filter_fn: fn(&[T], Option<&str>) -> Vec<T>,
    filters: Vec<FilterOption>,
) -> (
    ReadSignal<String>,
    WriteSignal<String>,
    Memo<Vec<T>>,
    Memo<usize>,
    Memo<usize>,
    ReadSignal<Option<String>>,
    WriteSignal<Option<String>>,
    Vec<FilterOption>,
)
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let (search_query, set_search_query) = signal(String::new());
    let (active_filter, set_active_filter) = signal(None::<String>);

    let filtered_items = Memo::new(move |_| {
        let query = search_query.get().trim().to_lowercase();
        let filter_key_owned = active_filter.get();
        let filter_key = filter_key_owned.as_deref();

        let mut result: Vec<T> = items
            .get()
            .into_iter()
            .filter(|item| {
                if !query.is_empty() {
                    let text = (search_text_fn)(item).to_lowercase();
                    if !text.contains(&query) {
                        return false;
                    }
                }
                true
            })
            .collect();

        result = (filter_fn)(&result, filter_key);
        result
    });

    let filtered_count = Memo::new(move |_| filtered_items.get().len());
    let total_count = Memo::new(move |_| items.get().len());

    (
        search_query,
        set_search_query,
        filtered_items,
        filtered_count,
        total_count,
        active_filter,
        set_active_filter,
        filters,
    )
}

// ---------------------------------------------------------------------------
// use_sortable_list_state – Hook: search + filter + sort
// ---------------------------------------------------------------------------

/// Returns reactive state for a sortable, filterable, searchable list.
pub fn use_sortable_list_state<T>(
    items: ReadSignal<Vec<T>>,
    columns: ReadSignal<Vec<SimpleColumn<T>>>,
    search_text_fn: fn(&T) -> String,
    filter_fn: fn(&[T], Option<&str>) -> Vec<T>,
    filters: Vec<FilterOption>,
) -> (
    ReadSignal<String>,
    WriteSignal<String>,
    Memo<Vec<T>>,
    Memo<usize>,
    Memo<usize>,
    ReadSignal<Option<String>>,
    WriteSignal<Option<String>>,
    ReadSignal<Option<String>>,
    WriteSignal<Option<String>>,
    ReadSignal<SortDirection>,
    WriteSignal<SortDirection>,
    Arc<dyn Fn(String) + Send + Sync + 'static>,
    Vec<FilterOption>,
)
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let (search_query, set_search_query) = signal(String::new());
    let (active_filter, set_active_filter) = signal(None::<String>);
    let (sort_column, set_sort_column) = signal(None::<String>);
    let (sort_direction, set_sort_direction) = signal(SortDirection::None);

    let sorted_items = Memo::new(move |_| {
        let query = search_query.get().trim().to_lowercase();
        let filter_key_owned = active_filter.get();
        let filter_key = filter_key_owned.as_deref();
        let col = sort_column.get();
        let dir = sort_direction.get();
        let cols = columns.get();

        let mut result: Vec<T> = items
            .get()
            .into_iter()
            .filter(|item| {
                if !query.is_empty() {
                    let text = (search_text_fn)(item).to_lowercase();
                    if !text.contains(&query) {
                        return false;
                    }
                }
                true
            })
            .collect();

        result = (filter_fn)(&result, filter_key);

        if let Some(ref col_key) = col {
            if dir != SortDirection::None {
                if let Some(col_def) = cols.iter().find(|c| c.key == *col_key) {
                    result.sort_by(|a, b| {
                        let va = (col_def.get_value)(a);
                        let vb = (col_def.get_value)(b);
                        match dir {
                            SortDirection::Asc => va.cmp(&vb),
                            SortDirection::Desc => vb.cmp(&va),
                            SortDirection::None => va.cmp(&vb),
                        }
                    });
                }
            }
        }
        result
    });

    let on_sort = {
        let sc = sort_column;
        let sd = sort_direction;
        let set_sc = set_sort_column;
        let set_sd = set_sort_direction;
        Arc::new(move |col_key: String| {
            let current_col = sc.get();
            let current_dir = sd.get();

            if current_col == Some(col_key.clone()) {
                let next = current_dir.next();
                set_sd.set(next);
                if next == SortDirection::None {
                    set_sc.set(None);
                }
            } else {
                set_sc.set(Some(col_key));
                set_sd.set(SortDirection::Asc);
            }
        })
    };

    let filtered_count = Memo::new(move |_| sorted_items.get().len());
    let total_count = Memo::new(move |_| items.get().len());

    (
        search_query,
        set_search_query,
        sorted_items,
        filtered_count,
        total_count,
        active_filter,
        set_active_filter,
        sort_column,
        set_sort_column,
        sort_direction,
        set_sort_direction,
        on_sort,
        filters,
    )
}

// ---------------------------------------------------------------------------
// ListToolbar – reusable search + filter + count toolbar
// ---------------------------------------------------------------------------

#[component]
pub fn ListToolbar(
    /// Search placeholder signal (empty string to hide search)
    search_placeholder: ReadSignal<String>,
    /// Search signal
    search_query: ReadSignal<String>,
    set_search_query: WriteSignal<String>,
    /// Filter options as signal
    filters: ReadSignal<Vec<FilterOption>>,
    /// Active filter signal
    _active_filter: ReadSignal<Option<String>>,
    set_active_filter: WriteSignal<Option<String>>,
    /// Item counts
    filtered_count: Memo<usize>,
    total_count: Memo<usize>,
) -> impl IntoView {
    let t = crate::i18n::use_i18n();

    let on_clear_search = move |_| {
        set_search_query.set(String::new());
    };

    view! {
        <div class="flex flex-col sm:flex-row gap-3 items-start sm:items-center">
            <Show when=move || !search_placeholder.get().is_empty()>
                <div class="form-control flex-1 max-w-md">
                    <div class="relative">
                        <input
                            type="text"
                            class="input input-bordered input-sm w-full pr-8"
                            placeholder=move || {
                                let ph = search_placeholder.get();
                                format!("{}...", ph)
                            }
                            value=search_query
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                        <Show when=move || !search_query.get().is_empty()>
                            <button
                                class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
                                on:click=on_clear_search
                            >
                                <Icon icon=&LuX width="14" height="14" />
                            </button>
                        </Show>
                    </div>
                </div>
            </Show>

            <Show when=move || !filters.get().is_empty()>
                {move || {
                    let opts = filters.get();
                    Some(view! {
                        <select
                            class="select select-bordered select-sm"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if val.is_empty() {
                                    set_active_filter.set(None);
                                } else {
                                    set_active_filter.set(Some(val));
                                }
                            }
                        >
                            <option value="">{t("filter_all")}</option>
                            {opts.iter().map(|f| {
                                view! {
                                    <option value={f.key}>{f.label.clone()}</option>
                                }
                            }).collect_view()}
                        </select>
                    }.into_view())
                }}
            </Show>

            <div class="text-sm text-base-content/60">
                <Show
                    when=move || filtered_count.get() != total_count.get()
                    fallback=move || {
                        view! {
                            <span>{filtered_count.get()} {t("items")}</span>
                        }
                    }
                >
                    <span>
                        {filtered_count.get()} / {total_count.get()} {t("items")}
                    </span>
                </Show>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// ListHeader – reusable title + add button header
// ---------------------------------------------------------------------------

#[component]
pub fn ListHeader(
    title: String,
    description: Option<String>,
    add_button: Option<impl IntoView>,
    extra_buttons: Option<impl IntoView>,
) -> impl IntoView {
    view! {
        <div class="flex justify-between items-center">
            <div>
                <h1 class="text-3xl font-bold">{title}</h1>
                {description.map(|desc| view! { <p class="text-base-content/60">{desc}</p> })}
            </div>
            <div class="flex gap-2">
                {extra_buttons.map(|btns| btns.into_view())}
                {add_button.map(|btn| btn.into_view())}
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// ListEmpty – "no results" row
// ---------------------------------------------------------------------------

#[component]
pub fn ListEmpty(colspan: usize) -> impl IntoView {
    let t = crate::i18n::use_i18n();

    view! {
        <tr>
            <td colspan=colspan class="text-center py-8 text-base-content/50">
                {t("no_results")}
            </td>
        </tr>
    }
}

// ---------------------------------------------------------------------------
// AddButton – convenience button
// ---------------------------------------------------------------------------

#[component]
pub fn AddButton(label: String, on_click: Arc<dyn Fn() + Send + Sync + 'static>) -> impl IntoView {
    let t = crate::i18n::use_i18n();

    view! {
        <button class="btn btn-primary" on:click=move |_| on_click()>
            <Icon icon=&LuPlus width="20" height="20" />
            {t("add")} " " {label}
        </button>
    }
}

// ---------------------------------------------------------------------------
// SortableTableHeaderCell – single header cell component
// ---------------------------------------------------------------------------

#[component]
pub fn SortableTableHeaderCell(
    label: String,
    sortable: bool,
    is_sorted: bool,
    dir: SortDirection,
    on_sort: Arc<dyn Fn(String) + Send + Sync + 'static>,
    col_key: String,
) -> impl IntoView {
    let arrow = if is_sorted {
        match dir {
            SortDirection::Asc => "↑",
            SortDirection::Desc => "↓",
            SortDirection::None => "↕",
        }
    } else {
        "↕"
    };
    let opacity_class = if is_sorted && dir != SortDirection::None {
        ""
    } else {
        "opacity-30"
    };

    // Always use the same on:click closure to keep the View type identical
    let on_click = {
        let on_sort_for_col = on_sort.clone();
        let click_key = col_key.clone();
        move |_| {
            if sortable {
                (on_sort_for_col)(click_key.clone());
            }
        }
    };

    view! {
        <th>
            <button
                class="flex items-center gap-1 hover:text-primary cursor-pointer"
                on:click=on_click
            >
                <span>{label}</span>
                <span class={opacity_class}>{arrow}</span>
            </button>
        </th>
    }
}

// ---------------------------------------------------------------------------
// SortableTableHeader – reusable sortable column headers
// ---------------------------------------------------------------------------

#[component]
pub fn SortableTableHeader<T>(
    columns: ReadSignal<Vec<SimpleColumn<T>>>,
    sort_column: ReadSignal<Option<String>>,
    sort_direction: ReadSignal<SortDirection>,
    on_sort: Arc<dyn Fn(String) + Send + Sync + 'static>,
    has_actions: bool,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    view! {
        <thead>
            <tr>
                {move || {
                    let cols = columns.get();
                    let is_sorted_col = sort_column.get();
                    let dir = sort_direction.get();
                    let on_sort_ref = on_sort.clone();

                    cols.iter().map(|col| {
                        let col_key = col.key.clone();
                        let col_label = col.label.clone();
                        let sortable = col.sortable;
                        let is_sorted = is_sorted_col == Some(col_key.clone());
                        let on_sort_for_cell = on_sort_ref.clone();

                        view! {
                            <SortableTableHeaderCell
                                label={col_label}
                                sortable={sortable}
                                is_sorted={is_sorted}
                                dir={dir}
                                on_sort={on_sort_for_cell}
                                col_key={col_key}
                            />
                        }
                    }).collect_view()
                }}
                {move || {
                    if has_actions {
                        Some(view! { <th></th> })
                    } else {
                        None
                    }
                }}
            </tr>
        </thead>
    }
}

// ---------------------------------------------------------------------------
// DataList – generic list with search, filter, and custom row rendering
// ---------------------------------------------------------------------------

/// A generic list component that provides search, filter, and count.
/// The caller renders the table body rows using the `filtered_items` memo.
///
/// Use this with `use_list_state` or `use_sortable_list_state` hooks.
#[component]
pub fn DataListLayout(
    /// Header section
    header: impl IntoView,
    /// Toolbar section (use ListToolbar)
    toolbar: impl IntoView,
    /// Table content
    table: impl IntoView,
    /// Optional footer
    footer: Option<impl IntoView>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4">
            {header}
            {toolbar}
            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    {table}
                </div>
            </div>
            {footer.map(|fv| fv.into_view())}
        </div>
    }
}
