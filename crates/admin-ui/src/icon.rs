/// Minimal Icon component that renders inline SVG from icondata.
/// Drop-in replacement for leptos_icons::Icon (which depends on Leptos 0.8).
use icondata_core::IconData;
use leptos::prelude::*;

#[component]
pub fn Icon(
    /// The icon to render.
    icon: &'static IconData,
    /// Optional inline CSS style.
    #[prop(into, optional)]
    style: MaybeProp<String>,
    /// Icon width (default: "1em").
    #[prop(into, optional)]
    width: MaybeProp<String>,
    /// Icon height (default: "1em").
    #[prop(into, optional)]
    height: MaybeProp<String>,
) -> impl IntoView {
    // Wrap the icon data in a <g> element
    let data = format!("<g>{}</g>", icon.data);

    let combined_style = match (style.get(), icon.style) {
        (Some(s), Some(default)) => Some(format!("{} {}", default, s)),
        (Some(s), None) => Some(s),
        (None, Some(default)) => Some(default.to_string()),
        (None, None) => None,
    };

    let w = width.get().unwrap_or_else(|| "1em".to_string());
    let h = height.get().unwrap_or_else(|| "1em".to_string());

    view! {
        <svg
            style=combined_style
            x=icon.x
            y=icon.y
            width=w
            height=h
            viewBox=icon.view_box
            stroke_linecap=icon.stroke_linecap
            stroke_linejoin=icon.stroke_linejoin
            stroke_width=icon.stroke_width
            stroke=icon.stroke
            fill=icon.fill.unwrap_or("currentColor")
            role="graphics-symbol"
            inner_html=data
        />
    }
}
