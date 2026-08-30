use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn TreeManagement() -> impl IntoView {
    let (label, set_label) = signal(String::new());
    let (count, set_count) = signal(1u32);
    view! {
        <div>
            <h2>"Bäume"</h2>
            <form on:submit=move|ev|{ev.prevent_default(); spawn_local(async move {let _ = (label.get(), count.get());})}>
                <label>"Bezeichnung" <input prop:value=move||label.get() on:input=move|ev|set_label.set(event_target_value(&ev))/></label>
                <label>"Anzahl" <input type="number" prop:value=move||count.get().to_string() on:input=move|ev|set_count.set(event_target_value(&ev).parse().unwrap_or(1))/></label>
                <button type="submit">"Baum anlegen"</button>
            </form>
        </div>
    }
}
