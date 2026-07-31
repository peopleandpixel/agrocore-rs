//! SIGPAC Parcel Management Component
//!
//! Provides UI for browsing and searching Spanish SIGPAC cadastral parcels
//! imported from official fiboa GeoParquet data.

use crate::api::{
    NearPointQuery, SigpacParcelDto, SigpacParcelQuery, list_sigpac_parcels,
    search_parcels_near_point,
};
use crate::i18n::use_i18n;
use icondata as I;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn SigpacParcels() -> impl IntoView {
    let t = use_i18n();

    // State
    let (parcels, set_parcels) = signal(Vec::<SigpacParcelDto>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (total, set_total) = signal(0u64);
    let (page, set_page) = signal(1u64);
    let (total_pages, set_total_pages) = signal(1u64);

    // Filter state
    let (filters, set_filters) = signal(SigpacParcelQuery {
        province: None,
        municipality: None,
        aggregate: None,
        zone: None,
        polygon: None,
        parcel: None,
        enclosure: None,
        sigpac_reference: None,
        page: Some(1),
        per_page: Some(50),
    });

    // Selected parcel for detail view
    let (selected_parcel, set_selected_parcel) = signal(None::<SigpacParcelDto>);
    let (show_detail, set_show_detail) = signal(false);

    // Spatial search
    let (search_coords, set_search_coords) = signal(None::<(f64, f64)>);
    let (search_radius, set_search_radius) = signal(1000f64);

    // Fetch parcels
    let fetch_parcels = move || {
        let filters = filters.get();
        set_loading.set(true);
        set_error.set(None);

        let query = SigpacParcelQuery {
            page: Some(filters.page.unwrap_or(1)),
            per_page: Some(filters.per_page.unwrap_or(50)),
            province: filters.province,
            municipality: filters.municipality,
            aggregate: filters.aggregate,
            zone: filters.zone,
            polygon: filters.polygon,
            parcel: filters.parcel,
            enclosure: filters.enclosure,
            sigpac_reference: filters.sigpac_reference.clone(),
        };

        leptos::task::spawn_local(async move {
            match list_sigpac_parcels(query).await {
                Ok(response) => {
                    set_parcels.set(response.data);
                    set_total.set(response.total);
                    set_total_pages.set(response.total_pages);
                    set_page.set(response.page);
                }
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    // Initial load
    Effect::new(move |_| {
        fetch_parcels();
    });

    // Handlers
    let handle_filter_change = move |field: &'static str, value: Option<String>| {
        set_filters.update(|f| {
            match field {
                "province" => f.province = value.and_then(|v| v.parse().ok()),
                "municipality" => f.municipality = value.and_then(|v| v.parse().ok()),
                "aggregate" => f.aggregate = value.and_then(|v| v.parse().ok()),
                "zone" => f.zone = value.and_then(|v| v.parse().ok()),
                "polygon" => f.polygon = value.and_then(|v| v.parse().ok()),
                "parcel" => f.parcel = value.and_then(|v| v.parse().ok()),
                "enclosure" => f.enclosure = value.and_then(|v| v.parse().ok()),
                "sigpac_reference" => {
                    f.sigpac_reference = if value.as_deref() == Some("") {
                        None
                    } else {
                        value
                    }
                }
                _ => {}
            }
            f.page = Some(1);
        });
        fetch_parcels();
    };

    let handle_page_change = move |new_page: u64| {
        set_filters.update(|f| f.page = Some(new_page));
        fetch_parcels();
    };

    let handle_per_page_change = move |new_per_page: u64| {
        set_filters.update(|f| {
            f.per_page = Some(new_per_page);
            f.page = Some(1);
        });
        fetch_parcels();
    };

    let handle_select_parcel = move |parcel: SigpacParcelDto| {
        set_selected_parcel.set(Some(parcel));
        set_show_detail.set(true);
    };

    let handle_spatial_search = move || {
        if let Some((lng, lat)) = search_coords.get() {
            set_loading.set(true);
            set_error.set(None);
            let radius = search_radius.get();

            leptos::task::spawn_local(async move {
                let query = NearPointQuery {
                    lng,
                    lat,
                    radius_m: Some(radius),
                };
                match search_parcels_near_point(query).await {
                    Ok(response) => {
                        set_parcels.set(response.data);
                        set_total.set(response.total);
                        set_total_pages.set(1);
                        set_page.set(1);
                    }
                    Err(e) => set_error.set(Some(e.to_string())),
                }
                set_loading.set(false);
            });
        }
    };

    view! {
        <div class="container mx-auto p-4">
            <div class="mb-6">
                <h1 class="text-2xl font-bold">{crate::t!(t, "sigpac.title")}</h1>
                <p class="text-base-content/70 mt-1">{crate::t!(t, "sigpac.subtitle")}</p>
            </div>

            {move || error.get().map(|e| view! {
                <div class="alert alert-error mb-4" role="alert">
                    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                    <span>{e}</span>
                </div>
            })}

            {/* Filters */}
            <div class="card bg-base-100 shadow-sm mb-6">
                <div class="card-body">
                    <h2 class="card-title">{crate::t!(t, "sigpac.filters")}</h2>
                    <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-7 gap-4 mt-4">
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.province")}</span></label>
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder="01-52"
                                min="1"
                                max="52"
                                prop:value=move || filters.get().province.map(|v| v.to_string()).unwrap_or_default()
                                on:change=move |ev| handle_filter_change("province", Some(event_target_value(&ev)))
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.municipality")}</span></label>
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder="001-999"
                                min="1"
                                max="999"
                                prop:value=move || filters.get().municipality.map(|v| v.to_string()).unwrap_or_default()
                                on:change=move |ev| handle_filter_change("municipality", Some(event_target_value(&ev)))
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.aggregate")}</span></label>
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder="001-999"
                                min="1"
                                max="999"
                                prop:value=move || filters.get().aggregate.map(|v| v.to_string()).unwrap_or_default()
                                on:change=move |ev| handle_filter_change("aggregate", Some(event_target_value(&ev)))
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.zone")}</span></label>
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder="001-999"
                                min="1"
                                max="999"
                                prop:value=move || filters.get().zone.map(|v| v.to_string()).unwrap_or_default()
                                on:change=move |ev| handle_filter_change("zone", Some(event_target_value(&ev)))
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.polygon")}</span></label>
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder="001-999"
                                min="1"
                                max="999"
                                prop:value=move || filters.get().polygon.map(|v| v.to_string()).unwrap_or_default()
                                on:change=move |ev| handle_filter_change("polygon", Some(event_target_value(&ev)))
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.parcel")}</span></label>
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder="00001-99999"
                                min="1"
                                max="99999"
                                prop:value=move || filters.get().parcel.map(|v| v.to_string()).unwrap_or_default()
                                on:change=move |ev| handle_filter_change("parcel", Some(event_target_value(&ev)))
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.enclosure")}</span></label>
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder="001-999"
                                min="1"
                                max="999"
                                prop:value=move || filters.get().enclosure.map(|v| v.to_string()).unwrap_or_default()
                                on:change=move |ev| handle_filter_change("enclosure", Some(event_target_value(&ev)))
                            />
                        </div>
                    </div>
                    <div class="mt-4 flex flex-wrap gap-4">
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.sigpac_reference")}</span></label>
                            <input
                                type="text"
                                class="input input-bordered w-full max-w-xs"
                                placeholder="ES411234567890123"
                                prop:value=move || filters.get().sigpac_reference.clone().unwrap_or_default()
                                on:change=move |ev| handle_filter_change("sigpac_reference", Some(event_target_value(&ev)))
                            />
                        </div>
                        <div class="form-control mt-auto">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.per_page")}</span></label>
                            <select
                                class="select select-bordered w-auto"
                                prop:value=move || filters.get().per_page.unwrap_or(50).to_string()
                                on:change=move |ev| handle_per_page_change(event_target_value(&ev).parse().unwrap_or(50))
                            >
                                <option value="25">25</option>
                                <option value="50">50</option>
                                <option value="100">100</option>
                                <option value="200">200</option>
                            </select>
                        </div>
                        <button class="btn btn-primary mt-auto" on:click=move |_| fetch_parcels()>
                            <Icon icon=I::LuRefreshCw width="20" height="20" attr:class="mr-2"/>
                            {crate::t!(t, "common.refresh")}
                        </button>
                    </div>
                </div>
            </div>

            {/* Spatial Search */}
            <div class="card bg-base-100 shadow-sm mb-6">
                <div class="card-body">
                    <h2 class="card-title">{crate::t!(t, "sigpac.spatial_search")}</h2>
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mt-4">
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.longitude")}</span></label>
                            <input
                                type="number"
                                step="0.000001"
                                class="input input-bordered w-full"
                                placeholder="-9.5"
                                prop:value=move || search_coords.get().map(|(lng, _)| lng.to_string()).unwrap_or_default()
                                on:change=move |ev| {
                                    let val = event_target_value(&ev).parse::<f64>();
                                    if let Ok(lng) = val {
                                        set_search_coords.set(Some((lng, search_coords.get().map(|(_, lat)| lat).unwrap_or(40.0))));
                                    }
                                }
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.latitude")}</span></label>
                            <input
                                type="number"
                                step="0.000001"
                                class="input input-bordered w-full"
                                placeholder="40.0"
                                prop:value=move || search_coords.get().map(|(_, lat)| lat.to_string()).unwrap_or_default()
                                on:change=move |ev| {
                                    let val = event_target_value(&ev).parse::<f64>();
                                    if let Ok(lat) = val {
                                        set_search_coords.set(Some((search_coords.get().map(|(lng, _)| lng).unwrap_or(-3.0), lat)));
                                    }
                                }
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">{crate::t!(t, "sigpac.radius_meters")}</span></label>
                            <input
                                type="number"
                                step="100"
                                min="100"
                                max="10000"
                                class="input input-bordered w-full"
                                prop:value=move || search_radius.get().to_string()
                                on:change=move |ev| set_search_radius.set(event_target_value(&ev).parse().unwrap_or(1000.0))
                            />
                        </div>
                        <div class="form-control mt-auto">
                            <button class="btn btn-secondary w-full" on:click=move |_| handle_spatial_search()>
                                <Icon icon=I::LuSearch width="20" height="20" attr:class="mr-2"/>
                                {crate::t!(t, "sigpac.search_near_point")}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            {/* Results Table */}
            <div class="card bg-base-100 shadow-sm">
                <div class="card-body">
                    <div class="flex justify-between items-center mb-4">
                        <h2 class="card-title">
                            {crate::t!(t, "sigpac.results")} " (" {move || total.get()} ")"
                        </h2>
                        <div class="flex gap-2">
                            {move || show_detail.get().then(|| view! {
                                <button class="btn btn-outline" on:click=move |_| set_show_detail.set(false)>
                                    <Icon icon=I::LuX width="18" height="18" attr:class="mr-2"/>
                                    {crate::t!(t, "common.close_detail")}
                                </button>
                            })}
                        </div>
                    </div>

                    {move || {
                        if loading.get() {
                            view! {
                                <div class="flex justify-center py-8">
                                    <span class="loading loading-spinner loading-lg"></span>
                                </div>
                            }.into_any()
                        } else if parcels.get().is_empty() {
                            view! {
                                <div class="text-center py-8 text-base-content/50">
                                    <Icon icon=I::LuMapPinOff width="48" height="48" attr:class="mx-auto mb-4 opacity-50"/>
                                    <p>{crate::t!(t, "sigpac.no_results")}</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="table table-zebra w-full">
                                        <thead>
                                            <tr>
                                                <th>{crate::t!(t, "sigpac.sigpac_reference")}</th>
                                                <th>{crate::t!(t, "sigpac.province")}</th>
                                                <th>{crate::t!(t, "sigpac.municipality")}</th>
                                                <th>{crate::t!(t, "sigpac.aggregate")}</th>
                                                <th>{crate::t!(t, "sigpac.zone")}</th>
                                                <th>{crate::t!(t, "sigpac.polygon")}</th>
                                                <th>{crate::t!(t, "sigpac.parcel")}</th>
                                                <th>{crate::t!(t, "sigpac.enclosure")}</th>
                                                <th>{crate::t!(t, "sigpac.usage_code")}</th>
                                                <th>{crate::t!(t, "sigpac.area_hectares")}</th>
                                                <th>{crate::t!(t, "common.actions")}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {move || parcels.get().into_iter().map(|parcel| {
                                                let parcel_clone = parcel.clone();
                                                let btn_parcel = parcel.clone();

                                                let sigpac_ref = parcel.sigpac_reference.clone();
                                                let usage_badge = parcel.usage_code.clone().map(|c| {
                                                    view! { <span class="badge badge-sm badge-outline">{c}</span> }
                                                });

                                                view! {
                                                    <tr class="hover:bg-base-200 cursor-pointer" on:click=move |_| handle_select_parcel(parcel_clone.clone())>
                                                        <td class="font-mono text-sm">{sigpac_ref}</td>
                                                        <td>{format!("{:02}", parcel.province)}</td>
                                                        <td>{format!("{:03}", parcel.municipality)}</td>
                                                        <td>{format!("{:03}", parcel.aggregate)}</td>
                                                        <td>{format!("{:03}", parcel.zone)}</td>
                                                        <td>{format!("{:03}", parcel.polygon)}</td>
                                                        <td>{format!("{:05}", parcel.parcel)}</td>
                                                        <td>{format!("{:03}", parcel.enclosure)}</td>
                                                        <td>{usage_badge}</td>
                                                        <td>{parcel.area_hectares.map(|a| format!("{:.4}", a)).unwrap_or_default()}</td>
                                                        <td>
                                                            <div class="flex gap-1">
                                                                <button class="btn btn-xs btn-ghost" on:click=move |ev| {
                                                                    ev.stop_propagation();
                                                                    handle_select_parcel(btn_parcel.clone());
                                                                }>
                                                                    <Icon icon=I::LuEye width="16" height="16"/>
                                                                </button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>

                                {/* Pagination */}
                                <div class="flex justify-center mt-4 gap-2">
                                    <button
                                        class="btn btn-sm"
                                        disabled=move || page.get() <= 1
                                        on:click=move |_| handle_page_change(page.get().saturating_sub(1))
                                    >
                                        <Icon icon=I::LuChevronLeft width="18" height="18"/>
                                    </button>
                                    <span class="flex items-center px-4">
                                        {crate::t!(t, "common.page")} {page} {crate::t!(t, "common.of")} {total_pages}
                                    </span>
                                    <button
                                        class="btn btn-sm"
                                        disabled=move || page.get() >= total_pages.get()
                                        on:click=move |_| handle_page_change(page.get() + 1)
                                    >
                                        <Icon icon=I::LuChevronRight width="18" height="18"/>
                                    </button>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            {/* Detail Modal */}
            {move || show_detail.get().then(|| selected_parcel.get().map(|parcel| {
                let usage_badge = parcel.usage_code.clone().map(|c| view! { <span class="badge badge-outline">{c}</span> });
                let source_str = parcel.source_dataset.clone().unwrap_or_else(|| "-".to_string());

                view! {
                    <div class="modal modal-open">
                        <div class="modal-box max-w-3xl">
                            <div class="flex justify-between items-start mb-4">
                                <h3 class="text-xl font-bold">{crate::t!(t, "sigpac.parcel_detail")} " - " {parcel.sigpac_reference.clone()}</h3>
                                <button class="btn btn-sm btn-ghost" on:click=move |_| set_show_detail.set(false)>
                                    <Icon icon=I::LuX width="24" height="24"/>
                                </button>
                            </div>

                            <div class="grid grid-cols-2 gap-4 mb-4">
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.province")}</span></label>
                                    <div class="font-mono text-lg">{format!("{:02}", parcel.province)}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.municipality")}</span></label>
                                    <div class="font-mono text-lg">{format!("{:03}", parcel.municipality)}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.aggregate")}</span></label>
                                    <div class="font-mono text-lg">{format!("{:03}", parcel.aggregate)}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.zone")}</span></label>
                                    <div class="font-mono text-lg">{format!("{:03}", parcel.zone)}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.polygon")}</span></label>
                                    <div class="font-mono text-lg">{format!("{:03}", parcel.polygon)}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.parcel")}</span></label>
                                    <div class="font-mono text-lg">{format!("{:05}", parcel.parcel)}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.enclosure")}</span></label>
                                    <div class="font-mono text-lg">{format!("{:03}", parcel.enclosure)}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.usage_code")}</span></label>
                                    <div>{usage_badge}</div>
                                </div>
                            </div>

                            <div class="divider mb-4"></div>

                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.area_hectares")}</span></label>
                                    <div class="text-lg">{parcel.area_hectares.map(|a| format!("{:.4} ha", a)).unwrap_or_else(|| (crate::t!(t, "common.n_a"))())}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.official_area_ha")}</span></label>
                                    <div class="text-lg">{parcel.official_area_ha.map(|a| format!("{:.4} ha", a)).unwrap_or_else(|| (crate::t!(t, "common.n_a"))())}</div>
                                </div>
                                <div>
                                    <label class="label"><span class="label-text font-medium">{crate::t!(t, "sigpac.source_dataset")}</span></label>
                                    <div>{source_str}</div>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }))}
        </div>
    }
}
