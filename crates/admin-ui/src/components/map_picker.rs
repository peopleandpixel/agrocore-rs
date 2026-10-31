/// Map component for selecting a company address by clicking on a map.
/// Uses MapLibre GL JS. User can click on the map to select a location.
use leptos::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast, prelude::*};

/// Callback type for when a location is selected on the map
pub type LocationSelectedCallback = WriteSignal<(f64, f64)>;

#[component]
pub fn MapPicker(
    /// Initial center point (lat, lng). If None, uses Portugal center.
    #[prop(optional)] initial_center: Option<(f64, f64)>,
    /// Initial zoom level
    #[prop(default = 13.0)] initial_zoom: f64,
    /// Callback when a location is selected
    on_location_selected: LocationSelectedCallback,
    /// Current selected location (lat, lng) - used to show marker
    #[prop(optional)] selected_location: Option<(f64, f64)>,
    /// Label shown above the map
    #[prop(optional)] label: Option<String>,
) -> impl IntoView {
    let map_ref = NodeRef::<leptos::html::Div>::new();
    let map_instance = std::rc::Rc::new(std::cell::RefCell::new(None::<JsValue>));
    let (loading, set_loading) = signal(true);

    // Initialize map
    Effect::new({
        let map_ref = map_ref.clone();
        let map_instance = map_instance.clone();
        let on_location_selected = on_location_selected.clone();
        let selected_location = selected_location.clone();
        move |_| {
            if let Some(_el) = map_ref.get() {
                let center = initial_center
                    .or(selected_location)
                    .unwrap_or((38.7223, -9.1393));

                let json_str = serde_json::to_string(&serde_json::json!({
                    "center": [center.1, center.0],
                    "zoom": initial_zoom,
                    "style": "https://demotiles.maplibre.org/style.json"
                })).unwrap_or_default();
                let options: JsValue = js_sys::JSON::parse(&json_str).unwrap_or(JsValue::NULL);

                // Create map via JS: new maplibregl.Map({container, style, center, zoom})
                let window = web_sys::window().unwrap();
                let maplibregl = window.get("maplibregl").unwrap();
                let constructor = js_sys::Reflect::get(&maplibregl, &"Map".into()).unwrap();
                
                let map_options = js_sys::Object::new();
                js_sys::Reflect::set(&map_options, &"container".into(), &"map-picker-container".into()).ok();
                js_sys::Reflect::set(&map_options, &"style".into(), &"https://demotiles.maplibre.org/style.json".into()).ok();
                let center_arr = js_sys::Array::new();
                center_arr.push(&JsValue::from_f64(center.1));
                center_arr.push(&JsValue::from_f64(center.0));
                js_sys::Reflect::set(&map_options, &"center".into(), &center_arr).ok();
                js_sys::Reflect::set(&map_options, &"zoom".into(), &JsValue::from_f64(initial_zoom)).ok();

                let map = match js_sys::Reflect::construct(&constructor, &js_sys::Array::of1(&map_options)) {
                    Ok(m) => m,
                    Err(_) => return,
                };
                let map_clone = map.clone();
                let map_instance = map_instance.clone();
                let on_location_selected = on_location_selected.clone();

                // Use map.on('click', callback) to add click listener
                if let Some(add_listener) = js_sys::Reflect::get(&map_clone, &"on".into())
                    .ok()
                    .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                {
                    let inner_cb = Closure::wrap(Box::new(move |e: JsValue| {
                        // e is a MapLibre MouseEvent with lngLat property
                        if let Some(lng_lat) = js_sys::Reflect::get(&e, &"lngLat".into()).ok() {
                            if let Some(lat_lng) = js_sys::Reflect::get(&lng_lat, &"lat".into()).ok()
                                .and_then(|v| v.as_f64())
                            {
                                if let Some(lng_val) = js_sys::Reflect::get(&lng_lat, &"lng".into()).ok()
                                    .and_then(|v| v.as_f64())
                                {
                                    on_location_selected.set((lat_lng, lng_val));
                                }
                            }
                        }
                    }) as Box<dyn FnMut(JsValue)>);
                    inner_cb.forget(); // Keep alive

                    let _ = add_listener.call2(&map_clone, &"click".into(), inner_cb.as_ref().as_ref());
                }

                let map_instance2 = map_instance.clone();
                leptos::task::spawn_local(async move {
                    *map_instance2.borrow_mut() = Some(map_clone);
                    set_loading.set(false);
                });
            }
        }
    });

    // Update marker when selected location changes
    Effect::new({
        let map_instance = map_instance.clone();
        move |_| {
            if let Some((lat, lng)) = selected_location {
                if let Some(map) = map_instance.borrow().as_ref() {
                    // Set marker via JS
                    let marker = js_sys::Reflect::get(map, &"setMarker".into()).ok();
                    if let Some(set_marker_fn) = marker.and_then(|f| f.dyn_ref::<js_sys::Function>()) {
                        set_marker_fn.call3(map, &JsValue::NULL, &JsValue::from(lat), &JsValue::from(lng)).ok();
                    }
                }
            }
        }
    });

    // Cleanup on unmount
    on_cleanup({
        let map_instance = map_instance.clone();
        move || {
            if let Some(map) = map_instance.borrow_mut().take() {
                let _ = js_sys::Reflect::get(&map, &"remove".into());
            }
        }
    });

    view! {
        <div class="space-y-2">
            {label.map(|l| view! {
                <label class="label">
                    <span class="label-text font-medium">{l}</span>
                </label>
            })}
            
            <div class="relative">
                {move || {
                    if loading.get() {
                        view! {
                            <div class="w-full h-64 bg-base-300 rounded-box overflow-hidden flex items-center justify-center">
                                <span class="loading loading-spinner loading-lg text-primary"></span>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div
                                id="map-picker-container"
                                node_ref=map_ref
                                class="w-full h-64 bg-base-300 rounded-box overflow-hidden shadow-inner cursor-crosshair"
                            ></div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="flex items-center gap-2 text-sm text-base-content/60">
                <span class="text-primary">"&#9873;"</span>
                <span>"Klicken Sie auf die Karte, um den Standort auszuwaehlen"</span>
            </div>

            {move || {
                selected_location.map(|(lat, lng)| {
                    view! {
                        <div class="alert alert-info shadow-lg border-info/20">
                            <span class="text-primary">"&#9873;"</span>
                            <span class="text-sm">
                                "Ausgewaehlter Standort: " {format!("{:.6}", lat)} ", " {format!("{:.6}", lng)}
                            </span>
                        </div>
                    }.into_any()
                })
            }}
        </div>
    }
}
