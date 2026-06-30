use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::*;
use crate::i18n::I18n;
use crate::i18n::Language;

#[component]
pub fn WeatherManagement() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);

    view! {
        <div class="flex flex-col gap-6">
            <h1 class="text-3xl font-bold">{move || t("weather_and_phenology")}</h1>
            
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div class="lg:col-span-2 space-y-6">
                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title">"Aktuelle Wetterdaten"</h2>
                            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                                <WeatherStat icon=LuThermometer label="Temperatur" value="24.5 °C" />
                                <WeatherStat icon=LuDroplets label="Feuchtigkeit" value="65%" />
                                <WeatherStat icon=LuWind label="Wind" value="12 km/h" />
                                <WeatherStat icon=LuCloudRain label="Niederschlag" value="0.0 mm" />
                            </div>
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow">
                        <div class="card-body">
                            <h2 class="card-title mb-4">"Wetterstationen"</h2>
                            <div class="space-y-2">
                                <div class="flex justify-between items-center p-3 bg-base-200 rounded-lg">
                                    <div>
                                        <div class="font-bold">"Station Nordhang"</div>
                                        <div class="text-xs opacity-60">"IoT - Online"</div>
                                    </div>
                                    <button class="btn btn-ghost btn-xs">"Details"</button>
                                </div>
                                <div class="flex justify-between items-center p-3 bg-base-200 rounded-lg">
                                    <div>
                                        <div class="font-bold">"Station Olivenhain A"</div>
                                        <div class="text-xs opacity-60">"Manuell - Letztes Update: Vor 2h"</div>
                                    </div>
                                    <button class="btn btn-ghost btn-xs">"Details"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="space-y-6">
                    <div class="card bg-primary text-primary-content shadow">
                        <div class="card-body">
                            <h2 class="card-title">"Phänologie"</h2>
                            <p class="text-sm">"Aktuelle BBCH-Stadien Ihrer Kulturen."</p>
                            <div class="divider divider-neutral"></div>
                            <div class="space-y-3">
                                <div class="flex justify-between items-center">
                                    <span class="font-bold">"Wein (Merlot)"</span>
                                    <span class="badge badge-secondary">"BBCH 75"</span>
                                </div>
                                <div class="flex justify-between items-center">
                                    <span class="font-bold">"Oliven"</span>
                                    <span class="badge badge-secondary">"BBCH 61"</span>
                                </div>
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <button class="btn btn-sm btn-outline btn-neutral">"Beobachtung melden"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn WeatherStat(icon: icondata::Icon, label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center p-2 bg-base-200 rounded-box">
            <Icon icon=icon width="24" height="24" attr:class="mb-1 opacity-70" />
            <div class="text-xs opacity-60 uppercase">{label}</div>
            <div class="font-bold text-lg">{value}</div>
        </div>
    }
}
