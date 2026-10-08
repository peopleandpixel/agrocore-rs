use crate::api::GeoPoint;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

#[wasm_bindgen(module = "/vendor/maplibre-wrapper.js")]
extern "C" {
    type MapLibreMap;

    #[wasm_bindgen(constructor)]
    fn new(containerId: &str, options: &JsValue) -> MapLibreMap;

    #[wasm_bindgen(method, js_name = "init")]
    fn init(this: &MapLibreMap) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    fn addWorkerLocation(this: &MapLibreMap, workerId: &str, lat: f64, lng: f64, options: &JsValue);

    #[wasm_bindgen(method)]
    fn updateWorkerLocation(this: &MapLibreMap, workerId: &str, lat: f64, lng: f64, options: &JsValue);

    #[wasm_bindgen(method)]
    fn removeWorkerLocation(this: &MapLibreMap, workerId: &str);

    #[wasm_bindgen(method)]
    fn addTaskLayer(this: &MapLibreMap, taskId: &str, geojson: &JsValue, options: &JsValue);

    #[wasm_bindgen(method)]
    fn updateTaskLayer(this: &MapLibreMap, taskId: &str, geojson: &JsValue, options: &JsValue);

    #[wasm_bindgen(method)]
    fn removeTaskLayer(this: &MapLibreMap, taskId: &str);

    #[wasm_bindgen(method)]
    fn addSiteLayer(this: &MapLibreMap, siteId: &str, geojson: &JsValue, options: &JsValue);

    #[wasm_bindgen(method)]
    fn fitBounds(this: &MapLibreMap, bounds: &JsValue, padding: f64);

    #[wasm_bindgen(method)]
    fn flyTo(this: &MapLibreMap, center: &JsValue, zoom: f64, options: &JsValue);

    #[wasm_bindgen(method)]
    fn destroy(this: &MapLibreMap);
}

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
    let map_instance = std::rc::Rc::new(std::cell::RefCell::new(None::<MapLibreMap>));

    // State for reactive data
    let (worker_locations, set_worker_locations) = signal(Vec::<WorkerLocation>::new());
    let (tasks, set_tasks) = signal(Vec::<MapTaskData>::new());
    let (sites, set_sites) = signal(Vec::<MapSiteData>::new());

    let show_workers = show_workers.unwrap_or(true);
    let show_tasks = show_tasks.unwrap_or(true);
    let show_sites = show_sites.unwrap_or(true);

    // Initialize map
    Effect::new({
        let map_ref = map_ref.clone();
        let map_instance = map_instance.clone();
        move |_| {
            if let Some(_el) = map_ref.get() {
                let options = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "center": initial_center.map(|c| vec![c.lng, c.lat]).unwrap_or(vec![-9.1393, 38.7223]),
                    "zoom": initial_zoom.unwrap_or(13.0),
                    "style": "https://demotiles.maplibre.org/style.json"
                })).unwrap_or(JsValue::NULL);

                let map = MapLibreMap::new("map-container", &options);
                let map_clone = map.clone();
                let map_instance = map_instance.clone();

                let promise = map_clone.init();
                let future = wasm_bindgen_futures::JsFuture::from(promise);
                leptos::task::spawn_local(async move {
                    if let Ok(_) = future.await {
                        *map_instance.borrow_mut() = Some(map_clone);
                    }
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
                Err(e) => log::error!("Failed to fetch worker locations: {}", e),
            }
        });
    };

    // Fetch tasks
    let fetch_tasks = move || {
        if !show_tasks { return; }
        leptos::task::spawn_local(async move {
            match crate::api::get_json::<Vec<MapTaskData>>("/api/v1/map/tasks", true).await {
                Ok(tasks_data) => set_tasks.set(tasks_data),
                Err(e) => log::error!("Failed to fetch tasks: {}", e),
            }
        });
    };

    // Fetch sites
    let fetch_sites = move || {
        if !show_sites { return; }
        leptos::task::spawn_local(async move {
            match crate::api::get_json::<Vec<MapSiteData>>("/api/v1/map/sites", true).await {
                Ok(sites_data) => set_sites.set(sites_data),
                Err(e) => log::error!("Failed to fetch sites: {}", e),
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

    // Update map when data changes
    Effect::new({
        let map_instance = map_instance.clone();
        move |_| {
            let workers = worker_locations.get();
            let tasks = tasks.get();
            let sites = sites.get();

            if let Some(map) = map_instance.borrow().as_ref() {
                // Update worker markers
                for worker in &workers {
                    let options = serde_wasm_bindgen::to_value(&serde_json::json!({
                        "inTask": worker.current_task_id.is_some(),
                        "popup": format!(
                            "<b>Worker:</b> {}<br><b>Status:</b> {}<br><b>Last update:</b> {}",
                            &worker.worker_id.to_string()[..8],
                            if worker.current_task_id.is_some() { "In Task" } else { "Idle" },
                            worker.timestamp.format("%H:%M:%S")
                        )
                    })).unwrap_or(JsValue::NULL);

                    map.addWorkerLocation(
                        &worker.worker_id.to_string(),
                        worker.lat,
                        worker.lng,
                        &options
                    );
                }

                // Update task layers
                for task in &tasks {
                    let geojson = serde_wasm_bindgen::to_value(&task.geometry).unwrap_or(JsValue::NULL);
                    let options = serde_wasm_bindgen::to_value(&serde_json::json!({
                        "status": task.status
                    })).unwrap_or(JsValue::NULL);

                    map.addTaskLayer(&task.id.to_string(), &geojson, &options);
                }

                // Add site layers
                for site in &sites {
                    let geojson = serde_wasm_bindgen::to_value(&site.geometry).unwrap_or(JsValue::NULL);
                    let options = serde_wasm_bindgen::to_value(&serde_json::json!({
                        "color": site.color.clone().unwrap_or_else(|| "#3b82f6".to_string())
                    })).unwrap_or(JsValue::NULL);

                    map.addSiteLayer(&site.id.to_string(), &geojson, &options);
                }
            }
        }
    });

    // Cleanup on unmount
    on_cleanup({
        let map_instance = map_instance.clone();
        move || {
            if let Some(map) = map_instance.borrow_mut().take() {
                map.destroy();
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