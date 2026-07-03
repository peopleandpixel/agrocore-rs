use leptos::prelude::*;

pub const PHONE_PREFIXES: &[(&str, &str)] = &[
    ("+351", "Portugal"),
    ("+49", "Deutschland"),
    ("+34", "Espana"),
    ("+33", "France"),
    ("+44", "United Kingdom"),
    ("+39", "Italia"),
    ("+1", "United States"),
];

pub fn is_valid_email(value: &str) -> bool {
    let value = value.trim();
    let mut parts = value.split('@');
    let Some(local) = parts.next() else {
        return false;
    };
    let Some(domain) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}

pub fn is_valid_phone(value: &str) -> bool {
    let value = value.trim();
    let digits = value.chars().filter(|c| c.is_ascii_digit()).count();
    value.starts_with('+')
        && digits >= 6
        && digits <= 15
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | ' ' | '-' | '(' | ')'))
}

pub fn normalize_phone(prefix: &str, local: &str) -> Option<String> {
    let prefix = prefix.trim();
    let digits: String = local.chars().filter(|c| c.is_ascii_digit()).collect();
    if prefix.is_empty() || digits.len() < 4 {
        return None;
    }
    Some(format!("{prefix}{digits}"))
}

pub fn split_phone(value: &str, default_prefix: &str) -> (String, String) {
    let normalized: String = value
        .trim()
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '(' | ')'))
        .collect();

    for (prefix, _) in PHONE_PREFIXES {
        if normalized.starts_with(prefix) {
            return (
                (*prefix).to_string(),
                normalized.trim_start_matches(prefix).to_string(),
            );
        }
    }

    (
        default_prefix.to_string(),
        normalized.trim_start_matches('+').to_string(),
    )
}

pub fn country_flag(value: &str) -> &'static str {
    match value.trim().to_lowercase().as_str() {
        "portugal" | "portugal." => "🇵🇹",
        "deutschland" | "germany" | "de" => "🇩🇪",
        "spanien" | "spain" | "es" => "🇪🇸",
        "frankreich" | "france" | "fr" => "🇫🇷",
        "vereinigtes königreich" | "united kingdom" | "uk" | "gb" => "🇬🇧",
        "italien" | "italy" | "it" => "🇮🇹",
        "vereinigte staaten" | "united states" | "usa" | "us" => "🇺🇸",
        _ => "🏳",
    }
}

pub fn language_flag(code: &str) -> &'static str {
    match code.trim().to_lowercase().as_str() {
        "de" => "🇩🇪",
        "en" => "🇬🇧",
        "es" => "🇪🇸",
        "fr" => "🇫🇷",
        "pt" => "🇵🇹",
        _ => "🏳",
    }
}

#[component]
pub fn RequiredLabel(required: bool, children: Children) -> impl IntoView {
    view! {
        <span class="label-text flex items-center gap-1">
            {children()}
            <Show when=move || required>
                <span class="text-error font-bold text-xs" aria-hidden="true">"*"</span>
            </Show>
        </span>
    }
}
