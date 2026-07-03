use crate::api;
use crate::components::form::RequiredLabel;
use crate::i18n::I18n;
use crate::i18n::Language;
use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn FinanceManagement() -> impl IntoView {
    let i18n = use_context::<I18n>().expect("i18n context");
    let lang = use_context::<ReadSignal<Language>>().expect("lang signal");
    let t = move |key: &str| i18n.t(lang.get().as_str(), key);
    let pac_applications =
        LocalResource::new(|| async move { api::fetch_pac_applications().await.ok() });
    let cost_centers = LocalResource::new(|| async move { api::fetch_cost_centers().await.ok() });
    let financial_records =
        LocalResource::new(|| async move { api::fetch_financial_records().await.ok() });
    let (show_pac_modal, set_show_pac_modal) = signal(false);
    let (show_booking_modal, set_show_booking_modal) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (application_number, set_application_number) = signal(String::new());
    let (eligible_area, set_eligible_area) = signal(String::new());
    let (booking_description, set_booking_description) = signal(String::new());
    let (booking_amount, set_booking_amount) = signal(String::new());

    let on_create_pac = move |_| {
        let application_number = application_number.get();
        let total_eligible_area = eligible_area.get().parse::<f64>().unwrap_or(0.0);
        set_error.set(None);

        if application_number.trim().is_empty() || total_eligible_area <= 0.0 {
            set_error.set(Some(String::from(
                "Bitte Antragsnummer und Fläche ausfüllen.",
            )));
            return;
        }

        spawn_local(async move {
            match api::create_pac_application(api::CreatePACApplicationRequest {
                year: 2026,
                application_number,
                total_eligible_area,
                eco_schemes: vec![],
            })
            .await
            {
                Ok(_) => {
                    let _ = leptos::prelude::window().location().reload();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_create_booking = move |_| {
        let booking_description = booking_description.get();
        let amount = booking_amount.get().parse::<f64>().unwrap_or(-450.0);
        set_error.set(None);

        if booking_description.trim().is_empty() || amount == -450.0 {
            set_error.set(Some(String::from(
                "Bitte Beschreibung und Betrag ausfüllen.",
            )));
            return;
        }

        spawn_local(async move {
            let cost_center_id = match api::create_cost_center(api::CreateCostCenterRequest {
                label: String::from("Allgemein"),
                code: String::from("GEN"),
                cost_center_type: String::from("General"),
                reference_id: None,
            })
            .await
            {
                Ok(value) => value
                    .get("id")
                    .and_then(|value| value.as_str())
                    .and_then(|value| uuid::Uuid::parse_str(value).ok()),
                Err(e) => {
                    set_error.set(Some(e));
                    return;
                }
            };

            let Some(cost_center_id) = cost_center_id else {
                set_error.set(Some(String::from(
                    "Kostenstelle konnte nicht bestimmt werden.",
                )));
                return;
            };

            match api::create_financial_record(api::CreateFinancialRecordRequest {
                cost_center_id,
                date: Utc::now().to_rfc3339(),
                amount,
                currency: String::from("EUR"),
                record_type: String::from("Expense"),
                category: String::from("Betriebsmittel"),
                description: booking_description,
                reference_id: None,
            })
            .await
            {
                Ok(_) => {
                    let _ = leptos::prelude::window().location().reload();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <h1 class="text-3xl font-bold">{move || t("finance_and_pac")}</h1>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title">"PAC-Anträge"</h2>
                        <div class="space-y-4 mt-4">
                            <div class="alert alert-info">
                                <span>{move || format!("{} Anträge vorhanden.", pac_applications.read().as_ref().map(|page| page.as_ref().map(|p| p.total).unwrap_or(0)).unwrap_or(0))}</span>
                            </div>
                            <button class="btn btn-outline btn-sm w-full" on:click=move |_| set_show_pac_modal.set(true)>"Neuer Antrag"</button>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title">"Kostenstellen"</h2>
                        <div class="overflow-x-auto mt-4">
                            <table class="table table-xs">
                                <thead>
                                    <tr>
                                        <th>"Name"</th>
                                        <th>"Typ"</th>
                                        <th>"Saldo"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || cost_centers.read().as_ref().and_then(|result| result.as_ref()).map(|page| {
                                        page.data.clone().into_iter().map(|center| {
                                            let label = center
                                                .get("label")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("Kostenstelle")
                                                .to_string();
                                            let center_type = center
                                                .get("cost_center_type")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("-")
                                                .to_string();
                                            let balance = center
                                                .get("balance")
                                                .and_then(|v| v.as_f64())
                                                .map(|value| format!("{:+.2} €", value))
                                                .unwrap_or_else(|| String::from("—"));
                                            view! {
                                                <tr>
                                                    <td>{label}</td>
                                                    <td>{center_type}</td>
                                                    <td>{balance}</td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()
                                    }).unwrap_or_else(Vec::new)}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-100 shadow border-t-4 border-accent">
                    <div class="card-body">
                        <h2 class="card-title">"Finanz-Quicklink"</h2>
                        <p class="text-sm">"Erfassen Sie schnell neue Einnahmen oder Ausgaben."</p>
                        <div class="card-actions justify-end mt-4">
                            <button class="btn btn-accent btn-sm" on:click=move |_| set_show_booking_modal.set(true)>"Buchung erstellen"</button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title mb-4">"Letzte Finanzaufzeichnungen"</h2>
                    <div class="overflow-x-auto">
                        {move || financial_records.read().as_ref().and_then(|result| result.as_ref()).map(|page| view! {
                            <table class="table w-full">
                                <thead>
                                    <tr>
                                        <th>"Datum"</th>
                                        <th>"Beschreibung"</th>
                                        <th>"Betrag"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {page.data.iter().map(|record| {
                                        view! {
                                            <tr>
                                                <td>{record.get("date").and_then(|v| v.as_str()).unwrap_or("-")}</td>
                                                <td>{record.get("description").and_then(|v| v.as_str()).unwrap_or("-")}</td>
                                                <td>{record.get("amount").and_then(|v| v.as_f64()).map(|value| format!("{:+.2} €", value)).unwrap_or_else(|| String::from("—"))}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        }.into_any()).unwrap_or_else(|| view! {
                            <div class="alert alert-ghost">
                                <span>"Noch keine Finanzbuchungen vorhanden."</span>
                            </div>
                        }.into_any())}
                    </div>
                </div>
            </div>

            <Show when=move || show_pac_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">"PAC-Antrag erstellen"</h3>
                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}
                        <div class="form-control w-full mt-4">
                            <label class="label"><RequiredLabel required=true>{"Antragsnummer"}</RequiredLabel></label>
                            <input type="text" class="input input-bordered w-full" required prop:value=move || application_number.get() on:input=move |ev| set_application_number.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-control w-full mt-4">
                            <label class="label"><RequiredLabel required=true>{"Förderfähige Fläche (ha)"}</RequiredLabel></label>
                            <input type="number" step="0.1" class="input input-bordered w-full" required prop:value=move || eligible_area.get() on:input=move |ev| set_eligible_area.set(event_target_value(&ev)) />
                        </div>
                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_pac_modal.set(false)>"Abbrechen"</button>
                            <button class="btn btn-primary" on:click=on_create_pac>"Speichern"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_booking_modal.get()>
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">"Buchung erstellen"</h3>
                        {move || error.get().map(|err| view! {
                            <div class="alert alert-error mt-4">
                                <span>{err}</span>
                            </div>
                        })}
                        <div class="form-control w-full mt-4">
                            <label class="label"><RequiredLabel required=true>{"Beschreibung"}</RequiredLabel></label>
                            <input type="text" class="input input-bordered w-full" required prop:value=move || booking_description.get() on:input=move |ev| set_booking_description.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-control w-full mt-4">
                            <label class="label"><RequiredLabel required=true>{"Betrag"}</RequiredLabel></label>
                            <input type="number" step="0.01" class="input input-bordered w-full" required prop:value=move || booking_amount.get() on:input=move |ev| set_booking_amount.set(event_target_value(&ev)) />
                        </div>
                        <div class="modal-action">
                            <button class="btn" on:click=move |_| set_show_booking_modal.set(false)>"Abbrechen"</button>
                            <button class="btn btn-primary" on:click=on_create_booking>"Speichern"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
