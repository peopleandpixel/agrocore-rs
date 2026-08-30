use leptos::prelude::*;

#[component]
pub fn PlotSubEntityLayout() -> impl IntoView {
    view! {
        <div>
            <h2>"Feld-Übersicht"</h2>
            <table>
                <thead><tr><th>"Typ"</th><th>"Name"</th><th>"Gruppe"</th><th>"Anzahl"</th></tr></thead>
                <tbody>
                    <tr><td>"Gruppe"</td><td>"Herde 1"</td><td>"—"</td><td>"—"</td></tr>
                    <tr><td>"Tier"</td><td>"Ziegen"</td><td>"Herde 1"</td><td>"2"</td></tr>
                    <tr><td>"Baum"</td><td>"Korkeiche"</td><td>"Kork-Gruppe"</td><td>"4"</td></tr>
                    <tr><td>"Gebäude"</td><td>"Stall"</td><td>"—"</td><td>"—"</td></tr>
                </tbody>
            </table>
        </div>
    }
}
