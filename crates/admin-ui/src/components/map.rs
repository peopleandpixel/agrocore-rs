use chrono::{DateTime, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

#[wasm_bindgen(module = "/src/leaflet.js")]
extern "C" {
    #[wasm_bindgen(js_name = initPolygonEditor)]
    fn init_polygon_editor(el: &web_sys::HtmlDivElement, on_change: &js_sys::Function);

    #[wasm_bindgen(js_name = initWorkerMap)]
    fn init_worker_map(el: &web_sys::HtmlDivElement);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerLocation {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub lat: f64,
    pub lng: f64,
    pub current_task_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

fn js_point_to_geo_point(value: &JsValue) -> Option<crate::api::GeoPoint> {
    if value.is_null() || value.is_undefined() {
        return None;
    }

    let lat = js_sys::Reflect::get(value, &"lat".into()).ok()?.as_f64()?;
    let lng = js_sys::Reflect::get(value, &"lng".into()).ok()?.as_f64()?;
    Some(crate::api::GeoPoint { lat, lng })
}

fn js_points_to_geo_points(value: &JsValue) -> Option<Vec<crate::api::GeoPoint>> {
    if value.is_null() || value.is_undefined() {
        return None;
    }

    let points = js_sys::Array::from(value);
    let parsed: Vec<_> = points
        .iter()
        .filter_map(|point| js_point_to_geo_point(&point))
        .collect();

    if parsed.len() >= 3 {
        Some(parsed)
    } else {
        None
    }
}

fn js_number_to_option(value: &JsValue) -> Option<f64> {
    if value.is_null() || value.is_undefined() {
        None
    } else {
        value.as_f64()
    }
}

#[component]
pub fn FieldPolygonEditor<F>(on_change: F) -> impl IntoView
where
    F: Fn(Option<Vec<crate::api::GeoPoint>>, Option<f64>, Option<crate::api::GeoPoint>) + 'static,
{
    let map_ref = NodeRef::<leptos::html::Div>::new();
    let on_change = std::rc::Rc::new(on_change);

    Effect::new(move |_| {
        if let Some(el) = map_ref.get() {
            let on_change = on_change.clone();
            let callback = Closure::wrap(Box::new(
                move |boundary: JsValue, area: JsValue, center: JsValue| {
                    let boundary = js_points_to_geo_points(&boundary);
                    let area = js_number_to_option(&area);
                    let center = js_point_to_geo_point(&center);
                    on_change(boundary, area, center);
                },
            )
                as Box<dyn FnMut(JsValue, JsValue, JsValue)>);

            init_polygon_editor(&el, callback.as_ref().unchecked_ref());
            callback.forget();
        }
    });

    view! {
        <div class="space-y-3">
            <div
                node_ref=map_ref
                class="w-full h-[600px] rounded-box overflow-hidden border border-base-300 bg-base-200"
            ></div>
        </div>
    }
}

#[component]
pub fn MapView() -> impl IntoView {
    let map_ref = NodeRef::<leptos::html::Div>::new();
    let (locations, _set_locations) = signal(Vec::<WorkerLocation>::new());

    Effect::new(move |_| {
        if let Some(el) = map_ref.get() {
            init_worker_map(&el);
        }
    });

    view! {
        <div class="flex flex-col gap-4 h-full">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">"Mitarbeiter-Karte (OSM)"</h1>
                <div class="flex gap-4">
                    <div class="flex items-center gap-2">
                        <div class="w-3 h-3 rounded-full bg-success"></div>
                        <span class="text-sm">"In Aufgabe"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-3 h-3 rounded-full bg-neutral"></div>
                        <span class="text-sm">"Bereit / Idle"</span>
                    </div>
                </div>
            </div>

            <div
                node_ref=map_ref
                class="w-full h-[600px] bg-base-300 rounded-box overflow-hidden shadow-inner"
            ></div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-4">
                {move || locations.get().into_iter().map(|loc| {
                    let status_class = if loc.current_task_id.is_some() { "badge-success" } else { "badge-ghost" };
                    let status_text = if loc.current_task_id.is_some() { "In Aufgabe" } else { "Bereit" };
                    view! {
                        <div class="card bg-base-100 shadow-xl compact">
                            <div class="card-body">
                                <div class="flex items-center gap-3">
                                    <div class=format!("w-3 h-3 rounded-full {}", if loc.current_task_id.is_some() { "bg-success" } else { "bg-neutral" })></div>
                                    <h2 class="card-title text-sm">"Mitarbeiter " {loc.worker_id.to_string()[..8].to_string()}</h2>
                                    <span class=format!("badge badge-xs ml-auto {}", status_class)>{status_text}</span>
                                </div>
                                <p class="text-[10px] opacity-50">"Pos: " {format!("{:.4}, {:.4}", loc.lat, loc.lng)}</p>
                                <p class="text-[10px] opacity-50">"Zuletzt gemeldet: " {loc.timestamp.format("%H:%M:%S").to_string()}</p>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
