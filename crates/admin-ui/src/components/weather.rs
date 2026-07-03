use crate::api;
use crate::i18n::I18n;
use crate::i18n::Language;
use chrono::Utc;
use icondata::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

fn station_label(station: &serde_json::Value) -> String {
    station
        .get("label")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("—"))
}

fn station_status(station: &serde_json::Value) -> String {
    station
        .get("station_type")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("—"))
}

#[component]
pub fn WeatherManagement() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let title = i18n.t(lang.get().as_str(), "weather_and_phenology");
    let current_weather = i18n.t(lang.get().as_str(), "weather_current_data");
    let temperature = i18n.t(lang.get().as_str(), "temperature");
    let humidity = i18n.t(lang.get().as_str(), "humidity");
    let wind = i18n.t(lang.get().as_str(), "wind");
    let precipitation = i18n.t(lang.get().as_str(), "precipitation");
    let source = i18n.t(lang.get().as_str(), "source");
    let weather_no_external_data = i18n.t(lang.get().as_str(), "weather_no_external_data");
    let weather_stations_title = i18n.t(lang.get().as_str(), "weather_stations_title");
    let no_weather_stations = i18n.t(lang.get().as_str(), "no_weather_stations");
    let phenology_title = i18n.t(lang.get().as_str(), "phenology_title");
    let weather_bbch_desc = i18n.t(lang.get().as_str(), "weather_bbch_desc");
    let observe = i18n.t(lang.get().as_str(), "observe");
    let last_report = i18n.t(lang.get().as_str(), "last_report");
    let report_observation = i18n.t(lang.get().as_str(), "report_observation");
    let weather_observation_title = i18n.t(lang.get().as_str(), "weather_observation_title");
    let observation_note = i18n.t(lang.get().as_str(), "observation_note");
    let cancel = i18n.t(lang.get().as_str(), "cancel");
    let save = i18n.t(lang.get().as_str(), "save");
    let weather_no_site_for_observation = i18n.t(lang.get().as_str(), "weather_no_site_for_observation");
    let weather_no_site_for_observation_label: &'static str =
        Box::leak(weather_no_site_for_observation.into_boxed_str());
    let sites = LocalResource::new(|| async move { api::fetch_sites().await.ok() });
    let stations = LocalResource::new(|| async move { api::fetch_weather_stations().await.ok() });
    let fallback_weather = LocalResource::new(|| async move {
        api::fetch_weather_for_company_profile()
            .await
            .ok()
            .flatten()
    });
    let (show_observation_modal, set_show_observation_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (notes, set_notes) = signal(String::new());

    view! {
        <div class="flex flex-col gap-6">
            <h1 class="text-3xl font-bold">{title}</h1>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div class="lg:col-span-2 space-y-6">
                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title">{current_weather}</h2>
                            {move || fallback_weather.read().as_ref().and_then(|snapshot| snapshot.clone()).map(|snapshot| view! {
                                <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                                    <WeatherStat icon=LuThermometer label=temperature.clone() value=snapshot.temperature_c.map(|value| format!("{:.1} °C", value)).unwrap_or_else(|| String::from("—")) />
                                    <WeatherStat icon=LuDroplets label=humidity.clone() value=snapshot.humidity_percent.map(|value| format!("{:.0}%", value)).unwrap_or_else(|| String::from("—")) />
                                    <WeatherStat icon=LuWind label=wind.clone() value=snapshot.wind_kmh.map(|value| format!("{:.0} km/h", value)).unwrap_or_else(|| String::from("—")) />
                                    <WeatherStat icon=LuCloudRain label=precipitation.clone() value=snapshot.precipitation_mm.map(|value| format!("{:.1} mm", value)).unwrap_or_else(|| String::from("—")) />
                                </div>
                                <p class="text-sm opacity-70 mt-4">
                                    {format!("{}: {}", source, snapshot.location_label)}
                                </p>
                            }.into_any()).unwrap_or_else(|| view! {
                                <div class="alert alert-info mt-4">
                                    <span>{weather_no_external_data.as_str().to_string()}</span>
                                </div>
                            }.into_any())}
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title mb-4">{weather_stations_title}</h2>
                            {move || {
                                stations.read().as_ref().and_then(|result| result.as_ref()).and_then(|page| {
                                    if page.data.is_empty() {
                                        None
                                    } else {
                                        Some(view! {
                                            <div class="space-y-2">
                                                {page.data.iter().map(|station| {
                                                    let label = station_label(station);
                                                    let status = station_status(station);
                                                    view! {
                                                        <div class="flex justify-between items-center p-3 bg-base-200 rounded-lg">
                                                            <div>
                                                                <div class="font-bold">{label}</div>
                                                                <div class="text-xs opacity-60">{status}</div>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any())
                                    }
                                }).unwrap_or_else(|| view! {
                                    <div class="alert alert-ghost">
                                        <span>{no_weather_stations.as_str().to_string()}</span>
                                    </div>
                                }.into_any())
                            }}
                        </div>
                    </div>
                </div>

                <div class="space-y-6">
                    <div class="card bg-primary text-primary-content shadow">
                        <div class="card-body">
                            <h2 class="card-title">{phenology_title}</h2>
                            <p class="text-sm">{weather_bbch_desc}</p>
                            <div class="divider divider-neutral"></div>
                            <div class="space-y-3">
                                <div class="flex justify-between items-center">
                                    <span class="font-bold">{observe}</span>
                                    <span class="badge badge-secondary">"—"</span>
                                </div>
                                <div class="flex justify-between items-center">
                                    <span class="font-bold">{last_report}</span>
                                    <span class="badge badge-secondary">"—"</span>
                                </div>
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <button class="btn btn-sm btn-outline btn-neutral" on:click=move |_| set_show_observation_modal.set(true)>{report_observation}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <Show when=move || show_observation_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">{weather_observation_title.as_str().to_string()}</h3>
                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}
                        <div class="form-control w-full mt-4">
                            <label class="label"><span class="label-text">{observation_note.as_str().to_string()}</span></label>
                            <textarea class="textarea textarea-bordered w-full" prop:value=move || notes.get() on:input=move |ev| set_notes.set(event_target_value(&ev))></textarea>
                        </div>
                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_observation_modal.set(false)>{cancel.as_str().to_string()}</button>
                            <button class="btn btn-primary" on:click=move |_| {
                                let notes_value = notes.get();
                                let site_id = sites
                                    .read()
                                    .clone()
                                    .flatten()
                                    .and_then(|page| page.data.first().map(|site| site.id));
                                let no_site_message = weather_no_site_for_observation_label.to_string();
                                set_error.set(None);

                                spawn_local(async move {
                                    let Some(site_id) = site_id else {
                                        set_error.set(Some(no_site_message));
                                        return;
                                    };

                                    match api::create_phenology_record(api::CreatePhenologyRecordRequest {
                                        site_id,
                                        observation_date: Utc::now().to_rfc3339(),
                                        stage: String::from("75"),
                                        notes: if notes_value.is_empty() { None } else { Some(notes_value) },
                                        photo_url: None,
                                    })
                                    .await
                                    {
                                        Ok(_) => {
                                            let _ = leptos::prelude::window().location().reload();
                                        }
                                        Err(e) => set_error.set(Some(e)),
                                    }
                                });
                            }>{save.clone()}</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn WeatherStat(icon: icondata::Icon, label: String, value: String) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center p-2 bg-base-200 rounded-box">
            <Icon icon=icon width="24" height="24" attr:class="mb-1 opacity-70" />
            <div class="text-xs opacity-60 uppercase">{label}</div>
            <div class="font-bold text-lg">{value}</div>
        </div>
    }
}
