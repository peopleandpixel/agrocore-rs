//! Data Import Component
//!
//! Provides UI for importing sites from GeoJSON or Shapefile formats
//! with optional LPIS validation against SIGPAC reference data.

use crate::api::{GeoJsonImportRequest, ImportResult, ShapefileImportRequest};
use crate::i18n::use_i18n;
use agrocore_shared::lpis::LpisCountry;
use base64::Engine;
use leptos::prelude::*;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{File, HtmlInputElement};

#[component]
pub fn DataImport() -> impl IntoView {
    let t = use_i18n();

    // Upload state
    let (geojson_file, set_geojson_file) = signal::<Option<File>>(None);
    let (shapefile_file, set_shapefile_file) = signal::<Option<File>>(None);

    // Import options
    let (skip_duplicates, set_skip_duplicates) = signal(true);
    let (update_existing, set_update_existing) = signal(false);
    let (validate_lpis, set_validate_lpis) = signal(true);

    // LPIS Country selection
    let (_lpis_country, _set_lpis_country) = signal::<LpisCountry>(LpisCountry::Es);

    // Result state
    let (result, set_result) = signal::<Option<ImportResult>>(None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // Handle GeoJSON file selection
    let on_geojson_change = move |event: web_sys::Event| {
        if let Some(input) = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Some(files) = input.files()
            && let Some(file) = files.get(0)
        {
            set_geojson_file.set(Some(file.unchecked_into()));
        }
    };

    // Handle Shapefile file selection
    let on_shapefile_change = move |event: web_sys::Event| {
        if let Some(input) = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Some(files) = input.files()
            && let Some(file) = files.get(0)
        {
            set_shapefile_file.set(Some(file.unchecked_into()));
        }
    };

    // Import GeoJSON
    let on_import_geojson = move |_| {
        set_error.set(None);
        set_result.set(None);

        if let Some(file) = geojson_file.get() {
            set_loading.set(true);

            let skip_dups = skip_duplicates.get();
            let update = update_existing.get();
            let validate = validate_lpis.get();

            spawn_local(async move {
                let file_text_result = file.text().await;
                let file_text = match file_text_result {
                    Ok(text) => text.as_string().unwrap_or_default(),
                    Err(e) => {
                        set_error.set(Some(format!("Failed to read file: {:?}", e)));
                        set_loading.set(false);
                        return;
                    }
                };

                let features: serde_json::Value = match serde_json::from_str(&file_text) {
                    Ok(v) => v,
                    Err(e) => {
                        set_error.set(Some(format!("Invalid GeoJSON: {}", e)));
                        set_loading.set(false);
                        return;
                    }
                };

                let features_array = match features.get("features").and_then(|f| f.as_array()) {
                    Some(arr) => arr.clone(),
                    None => {
                        set_error.set(Some("No features found in GeoJSON".to_string()));
                        set_loading.set(false);
                        return;
                    }
                };

                let geojson_request = GeoJsonImportRequest {
                    features: features_array
                        .into_iter()
                        .filter_map(|f| {
                            f.get("geometry").and_then(|g| {
                                let is_valid = g.get("type").and_then(|t| t.as_str())
                                    .map(|s| s == "Polygon" || s == "MultiPolygon")
                                    .unwrap_or(false);
                                if is_valid {
                                    Some(serde_json::json!({
                                        "type": "Feature",
                                        "geometry": f.get("geometry").cloned().unwrap_or_default(),
                                        "properties": f.get("properties").cloned().unwrap_or_default()
                                    }))
                                } else {
                                    None
                                }
                            })
                        })
                        .collect(),
                    skip_duplicates: Some(skip_dups),
                    update_existing: Some(update),
                    validate_lpis: Some(validate),
                };

                match crate::api::import_geojson(geojson_request).await {
                    Ok(res) => set_result.set(Some(res)),
                    Err(e) => set_error.set(Some(e)),
                }

                set_loading.set(false);
            });
        } else {
            set_error.set(Some(t("select_geojson").to_string()));
        }
    };

    // Import Shapefile
    let on_import_shapefile = move |_| {
        set_error.set(None);
        set_result.set(None);

        if let Some(file) = shapefile_file.get() {
            set_loading.set(true);

            let skip_dups = skip_duplicates.get();
            let update = update_existing.get();
            let validate = validate_lpis.get();

            spawn_local(async move {
                let file_bytes_result = file.array_buffer().await;
                let file_bytes = match file_bytes_result {
                    Ok(buf) => buf,
                    Err(e) => {
                        set_error.set(Some(format!("Failed to read file: {:?}", e)));
                        set_loading.set(false);
                        return;
                    }
                };

                let base64_str = if let Some(uint8) = file_bytes.dyn_ref::<js_sys::Uint8Array>() {
                    base64::engine::general_purpose::STANDARD.encode(uint8.to_vec())
                } else {
                    set_error.set(Some("Invalid file type".to_string()));
                    set_loading.set(false);
                    return;
                };

                let shapefile_request = ShapefileImportRequest {
                    file_base64: base64_str,
                    skip_duplicates: Some(skip_dups),
                    update_existing: Some(update),
                    validate_lpis: Some(validate),
                    encoding: None,
                };

                match crate::api::import_shapefile(shapefile_request).await {
                    Ok(res) => set_result.set(Some(res)),
                    Err(e) => set_error.set(Some(e)),
                }

                set_loading.set(false);
            });
        } else {
            set_error.set(Some(t("select_file").to_string()));
        }
    };

    // Result display
    let result_view = move || {
        result.get().map(|r| {
            view! {
                <div class="card bg-base-100 shadow-sm mt-6">
                    <div class="card-body">
                        <h3 class="card-title">"Import Result"</h3>
                        <div class="stats stats-vertical md:stats-horizontal gap-4 w-full">
                            <div class="stat">
                                <div class="stat-label">"Total"</div>
                                <div class="stat-value text-primary">{r.total}</div>
                            </div>
                            <div class="stat">
                                <div class="stat-label">"Created"</div>
                                <div class="stat-value text-success">{r.created}</div>
                            </div>
                            <div class="stat">
                                <div class="stat-label">"Updated"</div>
                                <div class="stat-value text-info">{r.updated}</div>
                            </div>
                            <div class="stat">
                                <div class="stat-label">"Skipped"</div>
                                <div class="stat-value text-warning">{r.skipped}</div>
                            </div>
                        </div>

                        {if !r.errors.is_empty() {
                            view! {
                                <div class="mt-4">
                                    <h4 class="font-bold mb-2">"Errors:"</h4>
                                    <ul class="list-disc list-inside text-error">
                                        {move || r.errors.iter().map(|e| {
                                            view! {
                                                <li>{format!("{}: {}", e.label, e.error)}</li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div class="mt-4 text-success"><span>"No errors"</span></div> }.into_any()
                        }}

                        {if !r.warnings.is_empty() {
                            view! {
                                <div class="mt-4">
                                    <h4 class="font-bold mb-2">"Warnings:"</h4>
                                    <ul class="list-disc list-inside text-warning">
                                        {move || r.warnings.iter().map(|w| {
                                            view! {
                                                <li>{format!("{}: {}", w.label, w.warning)}</li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div class="mt-4 text-base-content"><ul class="list-disc list-inside text-warning"><li>"No warnings"</li></ul></div> }.into_any()
                        }}
                    </div>
                </div>
            }
        })
    };

    view! {
        <div class="container mx-auto p-4">
            <div class="mb-6">
                <h1 class="text-3xl font-bold">"Data Import"</h1>
                <p class="text-base-content/70 mt-1">
                    {"Import sites from GeoJSON or Shapefile files with LPIS validation."}
                </p>
            </div>

            {move || error.get().map(|e| view! {
                <div class="alert alert-error mb-4" role="alert">
                    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                    <span>{e}</span>
                </div>
            })}

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* GeoJSON Import Card */}
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title">
                            {"GeoJSON Import"}
                        </h2>
                        <p class="text-base-content/60 mb-4">
                            {"Import sites from GeoJSON files (.geojson, .json)"}
                        </p>

                        <div class="form-control mb-4">
                            <input
                                type="file"
                                accept=".geojson,.json,application/geo+json"
                                on:change=on_geojson_change
                            />
                        </div>

                        {move || geojson_file.get().map(|f| view! {
                            <div class="badge badge-outline mb-4">
                                {f.name()}
                            </div>
                        })}
                    </div>
                </div>

                {/* Shapefile Import Card */}
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title">
                            {"Shapefile Import"}
                        </h2>
                        <p class="text-base-content/60 mb-4">
                            {"Import from ESRI Shapefile (ZIP containing .shp, .shx, .dbf)"}
                        </p>

                        <div class="form-control mb-4">
                            <input
                                type="file"
                                accept=".zip"
                                on:change=on_shapefile_change
                            />
                        </div>

                        {move || shapefile_file.get().map(|f| view! {
                            <div class="badge badge-outline mb-4">
                                {f.name()}
                            </div>
                        })}
                    </div>
                </div>
            </div>

            {/* Import Options */}
            <div class="card bg-base-100 shadow mt-6">
                <div class="card-body">
                    <h3 class="card-title">"Import Options"</h3>

                    <div class="form-control">
                        <label class="label cursor-pointer">
                            <span class="label-text">"Skip duplicates"</span>
                            <input
                                type="checkbox"
                                class="checkbox checkbox-primary"
                                checked=skip_duplicates.get()
                                on:change=move |ev| set_skip_duplicates.set(event_target_checked(&ev))
                            />
                        </label>
                    </div>

                    <div class="form-control">
                        <label class="label cursor-pointer">
                            <span class="label-text">"Update existing sites"</span>
                            <input
                                type="checkbox"
                                class="checkbox checkbox-primary"
                                checked=update_existing.get()
                                on:change=move |ev| set_update_existing.set(event_target_checked(&ev))
                            />
                        </label>
                    </div>

                    <div class="form-control">
                        <label class="label cursor-pointer">
                            <span class="label-text">"Validate against LPIS (SIGPAC reference)"</span>
                            <input
                                type="checkbox"
                                class="checkbox checkbox-primary"
                                checked=validate_lpis.get()
                                on:change=move |ev| set_validate_lpis.set(event_target_checked(&ev))
                            />
                        </label>
                        <p class="text-xs text-base-content/60 mt-1">
                            {"Validates parcel boundaries and areas against official Spanish SIGPAC data"}
                        </p>
                    </div>
                </div>
            </div>

            {/* Import Buttons */}
            <div class="flex gap-4 mt-6">
                <button
                    class="btn btn-primary flex-1"
                    on:click=move |_| on_import_geojson(())
                    disabled=loading.get() || geojson_file.get().is_none()
                >
                    {move || {
                        if loading.get() {
                            "Importing..."
                        } else {
                            "Import GeoJSON"
                        }
                    }}
                </button>

                <button
                    class="btn btn-secondary flex-1"
                    on:click=move |_| on_import_shapefile(())
                    disabled=loading.get() || shapefile_file.get().is_none()
                >
                    {move || {
                        if loading.get() {
                            "Importing..."
                        } else {
                            "Import Shapefile"
                        }
                    }}
                </button>
            </div>

            {/* Result View */}
            {result_view}
        </div>
    }
}
