use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn GroupManagement() -> impl IntoView {
    let (label, set_label) = signal(String::new());
    let (group_type, set_group_type) = signal(String::from("herd"));
    view! {
        <div>
            <h2>"Gruppen"</h2>
            <form on:submit=move|ev|{ev.prevent_default(); spawn_local(async move {let _ = (label.get(), group_type.get());})}>
                <label>"Name" <input prop:value=move||label.get() on:input=move|ev|set_label.set(event_target_value(&ev))/></label>
                <label>"Typ"
                    <select prop:value=move||group_type.get() on:change=move|ev|set_group_type.set(event_target_value(&ev))>
                        <option value="herd">"Herde"</option><option value="grove">"Baumgruppe"</option><option value="coop">"Stall"</option>
                    </select>
                </label>
                <button type="submit">"Gruppe erstellen"</button>
            </form>
        </div>
    }
}
