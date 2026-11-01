//! IoT Device Management — CRUD, telemetry, and commands.

use crate::api;
use crate::icon::Icon;
use icondata_lu::{LuActivity, LuPlus, LuTrash2, LuZap};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

#[component]
pub fn IotDevicesPage() -> impl IntoView {
    let devices = LocalResource::new(|| async move { api::fetch_iot_devices().await.ok() });

    let (show_create, set_show_create) = signal(false);
    let (selected_device, set_selected_device) = signal::<Option<api::IoTDeviceDto>>(None);
    let (show_detail, set_show_detail) = signal(false);

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">"IoT Devices"</h1>
                <button
                    class="btn btn-primary"
                    on:click=move |_| set_show_create.set(true)
                >
                    <Icon icon=&LuPlus width="18" height="18" />
                    "New Device"
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                <IotDeviceGrid devices=devices set_selected_device=set_selected_device set_show_detail=set_show_detail />
            </div>

            // Create Device Modal
            <Show when=move || show_create.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <CreateDeviceModal on_close=move || set_show_create.set(false) />
                </div>
            </Show>

            // Device Detail Modal
            <Show when=move || show_detail.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    {move || selected_device.get().map(|device| {
                        let on_close_cb = move || {
                            set_show_detail.set(false);
                            set_selected_device.set(None);
                        };
                        view! {
                            <DeviceDetailModal device=device.clone() on_close=on_close_cb />
                        }
                    })}
                </div>
            </Show>
        </div>
    }
}

#[component]
fn IotDeviceGrid(
    devices: LocalResource<Option<Vec<api::IoTDeviceDto>>>,
    set_selected_device: WriteSignal<Option<api::IoTDeviceDto>>,
    set_show_detail: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        {move || {
            let devs: Vec<api::IoTDeviceDto> = devices.read().as_ref().and_then(|o| o.as_ref()).cloned().unwrap_or_default();
            if devs.is_empty() {
                view! {
                    <div class="col-span-full alert alert-info">
                        <span>"No IoT devices configured yet."</span>
                    </div>
                }.into_any()
            } else {
                let set_sel = set_selected_device.clone();
                let set_det = set_show_detail.clone();
                let children = move |device: api::IoTDeviceDto| {
                    let sel = set_sel.clone();
                    let det = set_det.clone();
                    render_device_card(device, sel, det)
                };
                view! {
                    <For each=move || devs.clone() key=|d| d.device_id.clone() children=children />
                }.into_any()
            }
        }}
    }
}

fn render_device_card(
    device: api::IoTDeviceDto,
    set_selected_device: WriteSignal<Option<api::IoTDeviceDto>>,
    set_show_detail: WriteSignal<bool>,
) -> impl IntoView {
    let device_id_for_delete = device.device_id.clone();
    let status_badge = match device.status.as_str() {
        "Online" => "badge-success",
        "Offline" => "badge-ghost",
        "Error" => "badge-error",
        "Maintenance" => "badge-warning",
        "Updating" => "badge-info",
        _ => "badge-ghost",
    };
    view! {
        <div class="card bg-base-100 shadow hover:shadow-lg transition-shadow">
            <div class="card-body">
                <div class="flex justify-between items-start">
                    <div>
                        <h3 class="card-title text-lg">{device.device_id.clone()}</h3>
                        <p class="text-sm opacity-70">{device.device_type.clone()}</p>
                    </div>
                    <span class=move || format!("badge {}", status_badge)>{device.status.clone()}</span>
                </div>

                <div class="flex flex-wrap gap-2 mt-2">
                    {device.capabilities.iter().map(|cap| {
                        view! {
                            <span class="badge badge-outline">{cap.capability_type.clone()}</span>
                        }
                    }).collect_view()}
                </div>

                <div class="flex gap-4 text-xs opacity-60 mt-2">
                    {device.battery_level.map(|b| format!("Battery: {:.0}%", b))}
                    {device.signal_strength.map(|s| format!("Signal: {} dBm", s))}
                </div>

                {device.last_seen.as_ref().map(|ts| {
                    view! {
                        <div class="text-xs opacity-50">"Last seen: " {ts.clone()}</div>
                    }
                })}

                <div class="card-actions justify-end mt-2">
                    <button
                        class="btn btn-sm btn-outline"
                        on:click=move |_| {
                            set_selected_device.set(Some(device.clone()));
                            set_show_detail.set(true);
                        }
                    >
                        <Icon icon=&LuActivity width="14" height="14" />
                        "Details"
                    </button>
                    <button
                        class="btn btn-sm btn-ghost text-error"
                        on:click=move |_| {
                            let device_id = device_id_for_delete.clone();
                            spawn_local(async move {
                                if let Err(e) = api::delete_iot_device(&device_id).await {
                                    eprintln!("Failed to delete device: {}", e);
                                }
                            });
                        }
                    >
                        <Icon icon=&LuTrash2 width="14" height="14" />
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn CreateDeviceModal(on_close: impl Fn() + Clone + 'static) -> impl IntoView {
    let (device_id, set_device_id) = signal(String::new());
    let (device_type, set_device_type) = signal(String::new());
    let (capabilities, set_capabilities) = signal(String::new());

    let on_submit = {
        let device_id = device_id.clone();
        let device_type = device_type.clone();
        let capabilities = capabilities.clone();
        let on_close = on_close.clone();
        move |_| {
            let id = device_id.get().trim().to_string();
            let typ = device_type.get().trim().to_string();
            let caps_str = capabilities.get().trim().to_string();
            let caps: Vec<String> = caps_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if id.is_empty() || typ.is_empty() {
                return;
            }

            let req = api::CreateIoTDeviceRequest {
                device_id: id,
                device_type: typ,
                site_id: None,
                capabilities: caps,
                topic_prefix: None,
            };

            let on_close = on_close.clone();
            spawn_local(async move {
                if let Err(e) = api::create_iot_device(&req).await {
                    eprintln!("Failed to create device: {}", e);
                } else {
                    on_close();
                }
            });
        }
    };

    view! {
        <div class="card bg-base-100 shadow-2xl w-full max-w-md m-4">
            <div class="card-body">
                <h2 class="card-title">"New IoT Device"</h2>

                <div class="form-control w-full">
                    <label class="label"><span class="label-text">"Device ID"</span></label>
                    <input
                        type="text"
                        class="input input-bordered w-full"
                        placeholder="sensor-001"
                        prop:value=device_id.get()
                        on:input=move |e| set_device_id.set(event_target_value(&e))
                    />
                </div>

                <div class="form-control w-full">
                    <label class="label"><span class="label-text">"Device Type"</span></label>
                    <input
                        type="text"
                        class="input input-bordered w-full"
                        placeholder="Weather Station"
                        prop:value=device_type.get()
                        on:input=move |e| set_device_type.set(event_target_value(&e))
                    />
                </div>

                <div class="form-control w-full">
                    <label class="label"><span class="label-text">"Capabilities (comma-separated)"</span></label>
                    <input
                        type="text"
                        class="input input-bordered w-full"
                        placeholder="Temperature, Humidity, SoilMoisture"
                        prop:value=capabilities.get()
                        on:input=move |e| set_capabilities.set(event_target_value(&e))
                    />
                </div>

                <div class="card-actions justify-end">
                    <button class="btn btn-ghost" on:click=move |_| on_close()>"Cancel"</button>
                    <button class="btn btn-primary" on:click=on_submit>"Create"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DeviceDetailModal(
    device: api::IoTDeviceDto,
    on_close: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let device_id_str = device.device_id.clone();
    let device_type_str = device.device_type.clone();
    let device_status = device.status.clone();
    let firmware = device
        .firmware_version
        .clone()
        .unwrap_or_else(|| "—".to_string());
    let battery = device.battery_level;
    let signal_strength = device.signal_strength;
    let topic = device
        .topic_prefix
        .clone()
        .unwrap_or_else(|| "—".to_string());
    let last_seen = device.last_seen.clone().unwrap_or_else(|| "—".to_string());

    let device_id_for_telemetry = device_id_str.clone();
    let measurements = LocalResource::new(move || {
        let device_id = device_id_for_telemetry.clone();
        async move { api::fetch_iot_device_telemetry(&device_id).await.ok() }
    });

    let (command_type, set_command_type) = signal(String::new());
    let (command_payload, set_command_payload) = signal(String::new());

    let send_command = {
        let device_id = device_id_str.clone();
        let command_type = command_type.clone();
        let command_payload = command_payload.clone();
        move |_| {
            let cmd_type = command_type.get().clone();
            let payload_str = command_payload.get().clone();
            let payload: serde_json::Value =
                serde_json::from_str(&payload_str).unwrap_or(serde_json::json!({}));
            let req = api::IoTCommandRequest {
                command_type: cmd_type,
                payload,
                timeout_seconds: None,
            };
            let device_id = device_id.clone();
            spawn_local(async move {
                if let Err(e) = api::send_iot_device_command(&device_id, &req).await {
                    eprintln!("Failed to send command: {}", e);
                }
            });
        }
    };

    view! {
        <div class="card bg-base-100 shadow-2xl w-full max-w-2xl m-4 max-h-[80vh] overflow-y-auto">
            <div class="card-body">
                <div class="flex justify-between items-start">
                    <div>
                        <h2 class="card-title text-xl">{device_id_str}</h2>
                        <p class="text-sm opacity-70">{device_type_str}</p>
                    </div>
                    <button class="btn btn-sm btn-ghost" on:click=move |_| on_close()>"✕"</button>
                </div>

                <div class="divider"></div>

                <h3 class="font-bold flex items-center gap-2">
                    <Icon icon=&LuActivity width="18" height="18" />
                    "Telemetry"
                </h3>

                {move || {
                    let meas: Vec<api::IoTMeasurementDto> = measurements.read().as_ref().and_then(|o| o.as_ref()).cloned().unwrap_or_default();
                    if meas.is_empty() {
                        Some(view! {
                            <div class="text-sm opacity-50">"No telemetry data available."</div>
                        }.into_any())
                    } else {
                        Some(view! {
                            <div class="overflow-x-auto">
                                <table class="table table-sm w-full">
                                    <thead>
                                        <tr>
                                            <th>"Capability"</th>
                                            <th>"Value"</th>
                                            <th>"Unit"</th>
                                            <th>"Timestamp"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {meas.iter().map(|m| {
                                            view! {
                                                <tr>
                                                    <td>{m.capability.capability_type.clone()}</td>
                                                    <td>{format!("{:.2}", m.value)}</td>
                                                    <td>{m.unit.clone()}</td>
                                                    <td class="text-xs opacity-60">{m.timestamp.clone()}</td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        }.into_any())
                    }
                }}

                <div class="divider"></div>

                <h3 class="font-bold flex items-center gap-2">
                    <Icon icon=&LuZap width="18" height="18" />
                    "Send Command"
                </h3>

                <div class="form-control w-full">
                    <label class="label"><span class="label-text">"Command Type"</span></label>
                    <input
                        type="text"
                        class="input input-bordered w-full"
                        placeholder="reboot"
                        prop:value=command_type.get()
                        on:input=move |e| set_command_type.set(event_target_value(&e))
                    />
                </div>

                <div class="form-control w-full">
                    <label class="label"><span class="label-text">"Payload (JSON)"</span></label>
                    <textarea
                        class="textarea textarea-bordered w-full font-mono"
                        placeholder="{}"
                        prop:value=command_payload.get()
                        on:input=move |e| set_command_payload.set(event_target_value(&e))
                    />
                </div>

                <div class="card-actions justify-end">
                    <button class="btn btn-warning btn-sm" on:click=send_command>
                        <Icon icon=&LuZap width="14" height="14" />
                        "Send Command"
                    </button>
                </div>

                <div class="divider"></div>

                <h3 class="font-bold">"Device Info"</h3>
                <div class="grid grid-cols-2 gap-2 text-sm">
                    <div><span class="opacity-60">"Status: "</span> {device_status}</div>
                    <div><span class="opacity-60">"Firmware: "</span> {firmware}</div>
                    <div><span class="opacity-60">"Battery: "</span> {battery.map(|b| format!("{:.0}%", b)).unwrap_or_else(|| "—".to_string())}</div>
                    <div><span class="opacity-60">"Signal: "</span> {move || signal_strength.map(|s| format!("{} dBm", s)).unwrap_or_else(|| "—".to_string())}</div>
                    <div><span class="opacity-60">"Topic: "</span> {topic}</div>
                    <div><span class="opacity-60">"Last seen: "</span> {last_seen}</div>
                </div>
            </div>
        </div>
    }
}

fn event_target_value(e: &web_sys::Event) -> String {
    e.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.value())
        .or_else(|| {
            e.target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                .map(|el| el.value())
        })
        .unwrap_or_default()
}
