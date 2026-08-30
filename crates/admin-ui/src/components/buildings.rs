use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn BuildingManagement() -> impl IntoView {
    let (label, set_label) = signal(String::new());
    let (building_type, set_building_type) = signal(String::from("barn"));
    let (plot_id, set_plot_id) = signal(String::new());
    let toast =
        use_context::<crate::components::toast::ToastContext>().expect("ToastContext not provided");
    view! {
        <div>
            <h2>"Gebäude"</h2>
            <form on:submit=move |ev| {
                ev.prevent_default();
                if label.get().trim().is_empty() || plot_id.get().trim().is_empty() {
                    toast.add_toast.run(("Label und Plot-ID erforderlich".to_string(), crate::components::toast::ToastType::Warning));
                    return;
                }
                spawn_local(async move {
                    toast.add_toast.run((format!("Gebäude '{}' angelegt (Typ: {}, Plot: {})", label.get(), building_type.get(), plot_id.get()), crate::components::toast::ToastType::Success));
                });
            }>
                <label>"Label" <input prop:value=move||label.get() on:input=move|ev|set_label.set(event_target_value(&ev))/></label>
                <label>"Typ" <select prop:value=move||building_type.get() on:change=move|ev|set_building_type.set(event_target_value(&ev))>
                    <option value="barn">"Scheune"</option><option value="coop">"Stall"</option><option value="other">"Andere"</option>
                </select></label>
                <label>"Plot-ID (UUID)" <input prop:value=move||plot_id.get() on:input=move|ev|set_plot_id.set(event_target_value(&ev))/></label>
                <button type="submit">"Speichern"</button>
            </form>
        </div>
    }
}
