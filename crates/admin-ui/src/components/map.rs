use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["google", "maps"])]
    pub type Map;

    #[wasm_bindgen(constructor, js_namespace = ["google", "maps"])]
    fn new(el: &web_sys::HtmlDivElement, options: &js_sys::Object) -> Map;

    #[wasm_bindgen(js_namespace = ["google", "maps"])]
    pub type Marker;

    #[wasm_bindgen(constructor, js_namespace = ["google", "maps"])]
    fn new(options: &js_sys::Object) -> Marker;
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

#[component]
pub fn MapView() -> impl IntoView {
    let map_ref = NodeRef::<leptos::html::Div>::new();

    let (locations, _set_locations) = signal(Vec::<WorkerLocation>::new());

    Effect::new(move |_| {
        if let Some(el) = map_ref.get() {
            let options = js_sys::Object::new();
            js_sys::Reflect::set(&options, &"center".into(), &{
                let center = js_sys::Object::new();
                js_sys::Reflect::set(&center, &"lat".into(), &48.8566.into()).unwrap();
                js_sys::Reflect::set(&center, &"lng".into(), &2.3522.into()).unwrap();
                center
            }.into()).unwrap();
            js_sys::Reflect::set(&options, &"zoom".into(), &13.into()).unwrap();

            let map = Map::new(&el, &options);

            for loc in locations.get() {
                let marker_options = js_sys::Object::new();
                js_sys::Reflect::set(&marker_options, &"position".into(), &{
                    let pos = js_sys::Object::new();
                    js_sys::Reflect::set(&pos, &"lat".into(), &loc.lat.into()).unwrap();
                    js_sys::Reflect::set(&pos, &"lng".into(), &loc.lng.into()).unwrap();
                    pos
                }.into()).unwrap();
                js_sys::Reflect::set(&marker_options, &"map".into(), &map).unwrap();
                js_sys::Reflect::set(&marker_options, &"title".into(), &format!("Worker {}", loc.worker_id).into()).unwrap();
                
                // Farbe basierend auf Status (einfache Google Maps Marker Farben sind begrenzt ohne SVGs)
                // Für diese Demo nutzen wir Standard-Marker.
                
                Marker::new(&marker_options);
            }
        }
    });

    view! {
        <div class="flex flex-col gap-4 h-full">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">"Mitarbeiter-Karte (Google Maps)"</h1>
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
            >
            </div>
            
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
