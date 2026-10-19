use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;

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

/// A column definition for the list
pub struct ColumnDef<T> {
    pub key: &'static str,
    pub label: String,
    pub sortable: bool,
    pub get_value: fn(&T) -> String,
}

/// Filter option for the list
pub struct FilterOption {
    pub key: &'static str,
    pub label: String,
}

/// Search configuration for the list
pub struct SearchConfig<T> {
    /// Fields to search across — returns the searchable text for a row
    pub get_search_text: fn(&T) -> String,
}

/// Action button definition
pub struct ActionButton {
    pub icon: IconType,
    pub label: String,
    pub class: String,
    pub disabled: bool,
    pub on_click: Box<dyn Fn()>,
}

#[derive(Clone, Copy)]
pub struct IconType(pub &'static dyn leptoframe::icon::Icon);

impl IconType {
    pub const PENCIL: Self = Self(&LuPencil);
    pub const TRASH: Self = Self(&LuTrash2);
    pub const USER: Self = Self(&LuCircleUser);
    pub const EYE: Self = Self(&LuEye);
}

/// Generic list component with sorting, filtering, searching, and actions
#[component]
pub fn DataList<T, F, S, A>(
    /// Human-readable singular name of the item type (e.g., "User", "Site")
    item_name: String,
    /// Human-readable plural name (e.g., "Users", "Sites")
    item_name_plural: String,
    /// Description text shown below the title
    description: String,
    /// The data items to display
    items: Signal<Vec<T>>,
    /// Unique key for each item
    key_fn: fn(&T) -> String,
    /// Column definitions
    columns: Vec<ColumnDef<T>>,
    /// Available filter options with their labels
    filters: Vec<FilterOption>,
    /// A function that returns filtered items given the current filter key
    filter_fn: fn(&[T], Option<&str>) -> Vec<T>,
    /// Search configuration
    search_config: SearchConfig<T>,
    /// Whether to show the search bar
    show_search: bool,
    /// Action buttons renderer — receives the item and should return action buttons
    actions: Option<impl Fn(&T) -> impl IntoView + 'static>,
    /// Button to add a new item (label, on_click callback)
    add_button: Option<(String, Box<dyn Fn()>)>,
    /// Additional buttons in the header
    header_buttons: Option<impl IntoView + 'static>,
    /// Custom footer content (e.g., pagination)
    footer: Option<impl IntoView + 'static>,
) -> impl IntoView
where
    T: Clone + 'static,
    F: Fn(&T) -> String + 'static,
    S: Fn(&T) -> String + 'static,
    A: Fn(&T) -> impl IntoView + 'static,
{
    let t = crate::i18n::use_i18n();

    // Sort state: (column_key, direction)
    let (sort_column, set_sort_column) = signal(None::<&'static str>);
    let (sort_direction, set_sort_direction) = signal(SortDirection::None);

    // Filter state
    let (active_filter, set_active_filter) = signal(None::<String>);

    // Search state
    let (search_query, set_search_query) = signal(String::new());

    // Apply search filter
    let filtered_by_search = Memo::new(move |_| {
        let query = search_query.get().trim().to_lowercase();
        if query.is_empty() {
            items.get().clone()
        } else {
            items
                .get()
                .into_iter()
                .filter(|item| {
                    let text = (search_config.get_search_text)(item).to_lowercase();
                    text.contains(&query)
                })
                .collect()
        }
    });

    // Apply custom filter
    let filtered_by_status = Memo::new(move |_| {
        let items = filtered_by_search.get();
        let filter_key = active_filter.get().as_deref();
        (filter_fn)(&items, filter_key)
    });

    // Apply sort
    let sorted_items = Memo::new(move |_| {
        let mut items = filtered_by_status.get();
        let col = sort_column.get();
        let dir = sort_direction.get();

        if let Some(col_key) = col {
            if dir != SortDirection::None {
                if let Some(col_def) = columns.iter().find(|c| c.key == col_key) {
                    items.sort_by(|a, b| {
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
        items
    });

    // Handle column header click for sorting
    let on_sort = move |col_key: &'static str| {
        let current_col = sort_column.get();
        let current_dir = sort_direction.get();

        if current_col == Some(col_key) {
            let next = current_dir.next();
            set_sort_direction.set(next);
            if next == SortDirection::None {
                set_sort_column.set(None);
            }
        } else {
            set_sort_column.set(Some(col_key));
            set_sort_direction.set(SortDirection::Asc);
        }
    };

    // Clear search
    let on_clear_search = move |_| {
        set_search_query.set(String::new());
    };

    // Count of filtered items
    let filtered_count = Memo::new(move |_| sorted_items.get().len());
    let total_count = Memo::new(move |_| items.get().len());

    view! {
        <div class="flex flex-col gap-4">
            // Header with title and actions
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold">{item_name_plural.clone()}</h1>
                    <p class="text-base-content/60">{description.clone()}</p>
                </div>
                <div class="flex gap-2">
                    {move || {
                        if let Some(Some(btns)) = &header_buttons {
                            Some(btns.clone().into_view().into_any())
                        } else {
                            None
                        }
                    }}
                    {move || {
                        if let Some((label, on_click)) = &add_button {
                            Some(view! {
                                <button class="btn btn-primary" on:click=(*on_click).clone()>
                                    <Icon icon=LuPlus width="20" height="20" />
                                    {t("add")} " " {label.clone()}
                                </button>
                            }.into_any())
                        } else {
                            None
                        }
                    }}
                </div>
            </div>

            // Toolbar: search, filters, count
            <div class="flex flex-col sm:flex-row gap-3 items-start sm:items-center">
                // Search input
                <Show when=move || show_search>
                    <div class="form-control flex-1 max-w-md">
                        <div class="relative">
                            <input
                                type="text"
                                class="input input-bordered input-sm w-full pr-8"
                                placeholder=move || format!("{}...", t("search"))
                                value=search_query
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            />
                            <Show when=move || !search_query.get().is_empty()>
                                <button
                                    class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
                                    on:click=on_clear_search
                                >
                                    <Icon icon=LuX width="14" height="14" />
                                </button>
                            </Show>
                        </div>
                    </div>
                </Show>

                // Filter dropdown
                <Show when=move || !filters.is_empty()>
                    <select
                        class="select select-bordered select-sm"
                        value=active_filter
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
                        {filters.iter().map(|f| {
                            view! {
                                <option value={f.key}>{f.label.clone()}</option>
                            }
                        }).collect_view()}
                    </select>
                </Show>

                // Item count
                <div class="text-sm text-base-content/60">
                    <Show
                        when=move || filtered_count.get() != total_count.get()
                        fallback=move || {
                            view! {
                                <span>{total_count.get()} {item_name_plural.to_lowercase()}</span>
                            }
                        }
                    >
                        <span>
                            {filtered_count.get()} / {total_count.get()} {item_name_plural.to_lowercase()}
                        </span>
                    </Show>
                </div>
            </div>

            // Table card
            <div class="card bg-base-100 shadow">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                {columns.iter().map(|col| {
                                    let is_sorted = move || {
                                        sort_column.get() == Some(col.key)
                                    };
                                    let sort_icon = move || {
                                        if !col.sortable {
                                            return view! {}.into_any();
                                        }
                                        if !is_sorted() {
                                            return view! {
                                                <span class="opacity-30 ml-1">↕</span>
                                            }.into_any();
                                        }
                                        match sort_direction.get() {
                                            SortDirection::Asc => view! {
                                                <span class="ml-1">↑</span>
                                            }.into_any(),
                                            SortDirection::Desc => view! {
                                                <span class="ml-1">↓</span>
                                            }.into_any(),
                                            SortDirection::None => view! {
                                                <span class="opacity-30 ml-1">↕</span>
                                            }.into_any(),
                                        }
                                    };

                                    view! {
                                        <th>
                                            <Show
                                                when=move || col.sortable
                                                fallback=move || {
                                                    view! {
                                                        <span>{col.label.clone()}</span>
                                                    }
                                                }
                                            >
                                                <button
                                                    class="flex items-center gap-1 hover:text-primary cursor-pointer"
                                                    on:click=move |_| on_sort(col.key)
                                                >
                                                    <span>{col.label.clone()}</span>
                                                    {sort_icon}
                                                </button>
                                            </Show>
                                        </th>
                                    }
                                }).collect_view()}
                                {move || {
                                    if actions.is_some() {
                                        Some(view! {
                                            <th></th>
                                        }.into_any())
                                    } else {
                                        None
                                    }
                                }}
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || {
                                    sorted_items.get().iter().map(|item| (key_fn)(item).clone(), item.clone()).collect::<Vec<_>>()
                                }
                                key=|item| (key_fn)(item)
                                children=move |item| {
                                    view! {
                                        <tr>
                                            {columns.iter().map(|col| {
                                                let value = (col.get_value)(&item);
                                                view! {
                                                    <td>{value}</td>
                                                }
                                            }).collect_view()}
                                            {move || {
                                                if let Some(ref actions_fn) = actions {
                                                    Some(actions_fn(&item).into_view().into_any())
                                                } else {
                                                    None
                                                }
                                            }}
                                        </tr>
                                    }
                                }
                            />
                            <Show when=move || sorted_items.get().is_empty()>
                                <tr>
                                    <td colspan={columns.len() + if actions.is_some() { 1 } else { 0 }} class="text-center py-8 text-base-content/50">
                                        {t("no_results")}
                                    </td>
                                </tr>
                            </Show>
                        </tbody>
                    </table>
                </div>
            </div>

            // Footer (e.g., pagination)
            {move || {
                if let Some(Some(footer_view)) = &footer {
                    Some(footer_view.clone().into_view().into_any())
                } else {
                    None
                }
            }}
        </div>
    }
}
