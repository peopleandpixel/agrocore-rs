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
        && (6..=15).contains(&digits)
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
        "it" => "🇮🇹",
        "pl" => "🇵🇱",
        "ro" => "🇷🇴",
        "uk" => "🇺🇦",
        "nl" => "🇳🇱",
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

#[cfg(test)]
mod tests {
    use super::{
        country_flag, is_valid_email, is_valid_phone, language_flag, normalize_phone, split_phone,
    };

    #[test]
    fn email_validation_accepts_normal_addresses() {
        assert!(is_valid_email("alice@example.com"));
        assert!(is_valid_email(" alice@example.com "));
    }

    #[test]
    fn email_validation_rejects_invalid_addresses() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("alice"));
        assert!(!is_valid_email("alice@"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("alice@example"));
        assert!(!is_valid_email("alice@.example.com"));
        assert!(!is_valid_email("alice@example..com"));
    }

    #[test]
    fn phone_validation_accepts_normalized_international_numbers() {
        assert!(is_valid_phone("+351912345678"));
        assert!(is_valid_phone("+49 151 234 5678"));
        assert!(is_valid_phone("+1 (555) 123-4567"));
    }

    #[test]
    fn phone_validation_rejects_invalid_values() {
        assert!(!is_valid_phone(""));
        assert!(!is_valid_phone("123456"));
        assert!(!is_valid_phone("+12"));
        assert!(!is_valid_phone("+351-abc"));
    }

    #[test]
    fn normalize_phone_combines_prefix_and_local_digits() {
        assert_eq!(
            normalize_phone("+351", "912 345 678"),
            Some(String::from("+351912345678"))
        );
        assert_eq!(normalize_phone("", "912345678"), None);
        assert_eq!(normalize_phone("+351", "123"), None);
    }

    #[test]
    fn split_phone_uses_known_prefixes_and_fallback() {
        assert_eq!(
            split_phone("+351912345678", "+49"),
            (String::from("+351"), String::from("912345678"))
        );
        assert_eq!(
            split_phone("912345678", "+49"),
            (String::from("+49"), String::from("912345678"))
        );
    }

    #[test]
    fn country_and_language_flags_map_expected_values() {
        assert_eq!(country_flag("Portugal"), "🇵🇹");
        assert_eq!(country_flag("Deutschland"), "🇩🇪");
        assert_eq!(country_flag("United States"), "🇺🇸");
        assert_eq!(language_flag("de"), "🇩🇪");
        assert_eq!(language_flag("pt"), "🇵🇹");
        assert_eq!(language_flag("it"), "🇮🇹");
        assert_eq!(language_flag("pl"), "🇵🇱");
        assert_eq!(language_flag("ro"), "🇷🇴");
        assert_eq!(language_flag("uk"), "🇺🇦");
        assert_eq!(language_flag("nl"), "🇳🇱");
        assert_eq!(language_flag("xx"), "🏳");
    }
}
