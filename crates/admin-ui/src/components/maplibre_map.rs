use crate::api::GeoPoint;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerLocation {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub lat: f64,
    pub lng: f64,
    pub current_task_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTaskData {
    pub id: Uuid,
    pub label: String,
    pub status: String,
    pub geometry: serde_json::Value,
    pub site_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSiteData {
    pub id: Uuid,
    pub label: String,
    pub geometry: serde_json::Value,
    pub color: Option<String>,
}

#[component]
pub fn MapLibreView(
    #[prop(optional)] initial_center: Option<GeoPoint>,
    #[prop(optional)] initial_zoom: Option<f64>,
    #[prop(optional)] show_workers: Option<bool>,
    #[prop(optional)] show_tasks: Option<bool>,
    #[prop(optional)] show_sites: Option<bool>,
) -> impl IntoView {
    let t = crate::i18n::use_i18n();
    let map_ref = NodeRef::<leptos::html::Div>::new();
    let map_instance = std::sync::Arc::new(std::cell::RefCell::new(None::<JsValue>));

    // State for reactive data
    let (worker_locations, set_worker_locations) = signal(Vec::<WorkerLocation>::new());
    let (tasks, set_tasks) = signal(Vec::<MapTaskData>::new());
    let (sites, set_sites) = signal(Vec::<MapSiteData>::new());

    let show_workers = show_workers.unwrap_or(true);
    let show_tasks = show_tasks.unwrap_or(true);
    let show_sites = show_sites.unwrap_or(true);

    // Initialize map via JS
    let init_center = initial_center.map(|c| vec![c.lng, c.lat]).unwrap_or(vec![-9.1393, 38.7223]);
    let init_zoom = initial_zoom.unwrap_or(13.0);
    Effect::new({
        let map_ref = map_ref.clone();
        let map_instance = map_instance.clone();
        move |_| {
            if let Some(_el) = map_ref.get() {
                let window = match web_sys::window() {
                    Some(w) => w,
                    None => return,
                };
                let maplibregl = match window.get("maplibregl") {
                    Some(m) => m,
                    None => return,
                };
                let constructor = match js_sys::Reflect::get(&maplibregl, &"Map".into())
                    .ok()
                    .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                {
                    Some(c) => c,
                    None => return,
                };

                let map_options = js_sys::Object::new();
                js_sys::Reflect::set(&map_options, &"container".into(), &"map-container".into()).ok();
                js_sys::Reflect::set(&map_options, &"style".into(), &"https://demotiles.maplibre.org/style.json".into()).ok();
                let center_arr = js_sys::Array::of2(&JsValue::from_f64(init_center[0]), &JsValue::from_f64(init_center[1]));
                js_sys::Reflect::set(&map_options, &"center".into(), &center_arr).ok();
                js_sys::Reflect::set(&map_options, &"zoom".into(), &JsValue::from_f64(init_zoom)).ok();

                let map = match js_sys::Reflect::construct(&constructor, &js_sys::Array::of1(&map_options)) {
                    Ok(m) => m,
                    Err(_) => return,
                };
                let map_clone = map.clone();
                let map_instance2 = map_instance.clone();
                leptos::task::spawn_local(async move {
                    *map_instance2.borrow_mut() = Some(map_clone);
                });
            }
        }
    });

    // Fetch worker locations
    let fetch_workers = move || {
        if !show_workers { return; }
        leptos::task::spawn_local(async move {
            match crate::api::get_json::<Vec<WorkerLocation>>("/api/v1/workforce/locations", true).await {
                Ok(locations) => set_worker_locations.set(locations),
                Err(e) => eprintln!("Failed to fetch worker locations: {}", e),
            }
        });
    };

    // Fetch tasks
    let fetch_tasks = move || {
        if !show_tasks { return; }
        leptos::task::spawn_local(async move {
            match crate::api::get_json::<Vec<MapTaskData>>("/api/v1/map/tasks", true).await {
                Ok(tasks_data) => set_tasks.set(tasks_data),
                Err(e) => eprintln!("Failed to fetch tasks: {}", e),
            }
        });
    };

    // Fetch sites
    let fetch_sites = move || {
        if !show_sites { return; }
        leptos::task::spawn_local(async move {
            match crate::api::get_json::<Vec<MapSiteData>>("/api/v1/map/sites", true).await {
                Ok(sites_data) => set_sites.set(sites_data),
                Err(e) => eprintln!("Failed to fetch sites: {}", e),
            }
        });
    };

    // Initial data fetch
    Effect::new({
        move |_| {
            fetch_workers();
            fetch_tasks();
            fetch_sites();
        }
    });

    // Update map when data changes (via JS wrapper)
    Effect::new({
        let map_instance = map_instance.clone();
        move |_| {
            let workers = worker_locations.get();
            let tasks = tasks.get();
            let sites = sites.get();

            if let Some(map) = map_instance.borrow().as_ref() {
                // Call JS wrapper methods via Reflect
                if let Some(add_worker) = js_sys::Reflect::get(map, &"addWorkerLocation".into())
                    .ok()
                    .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                {
                    for worker in &workers {
                        let popup = format!(
                            "<b>Worker:</b> {}<br><b>Status:</b> {}<br><b>Last update:</b> {}",
                            &worker.worker_id.to_string()[..8],
                            if worker.current_task_id.is_some() { "In Task" } else { "Idle" },
                            worker.timestamp.format("%H:%M:%S")
                        );
                        let opt_str = serde_json::to_string(&serde_json::json!({
                            "inTask": worker.current_task_id.is_some(),
                            "popup": popup
                        })).unwrap_or_default();
                        let options = js_sys::JSON::parse(&opt_str).unwrap_or(JsValue::NULL);
                        let _ = add_worker.call4(
                            map,
                            &worker.worker_id.to_string().into(),
                            &JsValue::from_f64(worker.lat),
                            &JsValue::from_f64(worker.lng),
                            &options,
                        );
                    }
                }

                if let Some(add_task) = js_sys::Reflect::get(map, &"addTaskLayer".into())
                    .ok()
                    .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                {
                    for task in &tasks {
                        let geojson_str = serde_json::to_string(&task.geometry).unwrap_or_default();
                        let geojson = js_sys::JSON::parse(&geojson_str).unwrap_or(JsValue::NULL);
                        let opt_str = serde_json::to_string(&serde_json::json!({"status": task.status})).unwrap_or_default();
                        let options = js_sys::JSON::parse(&opt_str).unwrap_or(JsValue::NULL);
                        let _ = add_task.call3(map, &task.id.to_string().into(), &geojson, &options);
                    }
                }

                // Add site layers
                if let Some(add_site) = js_sys::Reflect::get(map, &"addSiteLayer".into())
                    .ok()
                    .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                {
                    for site in &sites {
                        let geojson_str = serde_json::to_string(&site.geometry).unwrap_or_default();
                        let geojson = js_sys::JSON::parse(&geojson_str).unwrap_or(JsValue::NULL);
                        let opt_str = serde_json::to_string(&serde_json::json!({"color": site.color.clone().unwrap_or_else(|| "#3b82f6".to_string())})).unwrap_or_default();
                        let options = js_sys::JSON::parse(&opt_str).unwrap_or(JsValue::NULL);
                        let _ = add_site.call3(map, &site.id.to_string().into(), &geojson, &options);
                    }
                }
            }
        }
    });


    view! {
        <div class="flex flex-col gap-4 h-full">
            <div class="flex justify-between items-center">
                <h1 class="text-2xl font-bold">{crate::t!(t, "map_title")}</h1>
                <div class="flex gap-4">
                    <div class="flex items-center gap-2">
                        <div class="w-3 h-3 rounded-full bg-success"></div>
                        <span class="text-sm">{crate::t!(t, "in_task")}</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-3 h-3 rounded-full bg-neutral"></div>
                        <span class="text-sm">{crate::t!(t, "ready_idle")}</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-3 h-3 rounded-full bg-primary"></div>
                        <span class="text-sm">{crate::t!(t, "task_pending")}</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-3 h-3 rounded-full bg-success"></div>
                        <span class="text-sm">{crate::t!(t, "task_done")}</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-3 h-3 rounded-full bg-error"></div>
                        <span class="text-sm">{crate::t!(t, "task_stopped")}</span>
                    </div>
                </div>
            </div>

            <div
                id="map-container"
                node_ref=map_ref
                class="w-full h-[600px] bg-base-300 rounded-box overflow-hidden shadow-inner"
            ></div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-4">
                {move || {
                    let workers = worker_locations.get();
                    workers.into_iter().map(|loc| {
                        let status_class = if loc.current_task_id.is_some() { "badge-success" } else { "badge-ghost" };
                        let status_text = if loc.current_task_id.is_some() { t("in_task") } else { t("ready_idle") };
                        view! {
                            <div class="card bg-base-100 shadow-xl compact">
                                <div class="card-body">
                                    <div class="flex items-center gap-3">
                                        <div class=format!("w-3 h-3 rounded-full {}", if loc.current_task_id.is_some() { "bg-success" } else { "bg-neutral" })></div>
                                        <h2 class="card-title text-sm">{crate::t!(t, "worker")} " " {loc.worker_id.to_string()[..8].to_string()}</h2>
                                        <span class=format!("badge badge-xs ml-auto {}", status_class)>{status_text}</span>
                                    </div>
                                    <p class="text-[10px] opacity-50">{crate::t!(t, "position")}: {format!("{:.4}, {:.4}", loc.lat, loc.lng)}</p>
                                    <p class="text-[10px] opacity-50">{crate::t!(t, "last_reported")}: {loc.timestamp.format("%H:%M:%S").to_string()}</p>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}