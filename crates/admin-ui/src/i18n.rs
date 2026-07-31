use serde::Deserialize;
use std::collections::HashMap;

rust_i18n::i18n!("locales", fallback = "en");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    EN,
    DE,
    ES,
    FR,
    PT,
    IT,
    PL,
    RO,
    UK,
    NL,
}

impl Language {
    #[cfg(test)]
    pub const ALL: [Language; 10] = [
        Language::EN,
        Language::DE,
        Language::ES,
        Language::FR,
        Language::PT,
        Language::IT,
        Language::PL,
        Language::RO,
        Language::UK,
        Language::NL,
    ];

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "de" => Language::DE,
            "es" => Language::ES,
            "fr" => Language::FR,
            "pt" => Language::PT,
            "it" => Language::IT,
            "pl" => Language::PL,
            "ro" => Language::RO,
            "uk" => Language::UK,
            "nl" => Language::NL,
            _ => Language::EN,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::EN => "en",
            Language::DE => "de",
            Language::ES => "es",
            Language::FR => "fr",
            Language::PT => "pt",
            Language::IT => "it",
            Language::PL => "pl",
            Language::RO => "ro",
            Language::UK => "uk",
            Language::NL => "nl",
        }
    }
}

pub const LANGUAGE_OPTIONS: [(Language, &str, &str); 10] = [
    (Language::DE, "de", "language_de"),
    (Language::EN, "en", "language_en"),
    (Language::ES, "es", "language_es"),
    (Language::FR, "fr", "language_fr"),
    (Language::IT, "it", "language_it"),
    (Language::NL, "nl", "language_nl"),
    (Language::PL, "pl", "language_pl"),
    (Language::PT, "pt", "language_pt"),
    (Language::RO, "ro", "language_ro"),
    (Language::UK, "uk", "language_uk"),
];

#[derive(Clone, Default, Copy)]
pub struct I18n {
    translations: Option<&'static HashMap<String, HashMap<String, String>>>,
}

#[derive(Debug, Deserialize)]
struct LocaleFile {
    #[serde(rename = "_version")]
    _version: u8,
    #[serde(flatten)]
    translations: HashMap<String, HashMap<String, String>>,
}

impl I18n {
    pub fn new() -> Self {
        let file: LocaleFile = serde_yaml::from_str(include_str!("../locales/app.yml"))
            .expect("failed to load admin-ui locales");
        let translations = Box::leak(Box::new(file.translations));
        Self {
            translations: Some(translations),
        }
    }

    pub fn t(&self, lang: &str, key: &str) -> String {
        self.translations
            .and_then(|translations| translations.get(key))
            .and_then(|translations| translations.get(lang))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}

pub fn use_i18n() -> impl Fn(&str) -> String + Copy {
    use leptos::prelude::Get;
    let i18n = leptos::prelude::use_context::<I18n>().expect("i18n context");
    let lang = leptos::prelude::use_context::<leptos::prelude::ReadSignal<Language>>()
        .expect("lang signal");

    move |key: &str| {
        let lang_code = lang.get().as_str();
        i18n.t(lang_code, key)
    }
}

#[macro_export]
macro_rules! t {
    ($t:expr, $key:expr) => {{
        let t = $t.clone();
        move || t($key)
    }};
}

#[cfg(test)]
pub fn supported_language_codes() -> [&'static str; 10] {
    ["en", "de", "es", "fr", "pt", "it", "pl", "ro", "uk", "nl"]
}

#[cfg(test)]
mod tests {
    use super::{I18n, Language, supported_language_codes};
    use std::collections::HashSet;

    #[test]
    fn supported_language_codes_are_complete() {
        assert_eq!(
            supported_language_codes(),
            ["en", "de", "es", "fr", "pt", "it", "pl", "ro", "uk", "nl"]
        );
    }

    #[test]
    fn setup_wizard_keys_exist_in_all_languages() {
        let i18n = I18n::new();
        let keys = [
            "setup_title",
            "setup_welcome",
            "setup_admin_title",
            "setup_admin_desc",
            "setup_admin_help",
            "setup_tenant_title",
            "setup_tenant_desc",
            "setup_tenant_help",
            "setup_company_title",
            "setup_company_desc",
            "setup_company_help",
            "setup_resources_title",
            "setup_resources_desc",
            "setup_resources_help",
            "optional_label",
            "finish_setup",
            "continue",
            "wizard_language",
        ];

        for language in Language::ALL {
            for key in keys {
                let value = i18n.t(language.as_str(), key);
                assert_ne!(
                    value,
                    key,
                    "missing translation for {key} in {}",
                    language.as_str()
                );
                assert!(
                    !value.trim().is_empty(),
                    "empty translation for {key} in {}",
                    language.as_str()
                );
            }
        }
    }

    #[test]
    fn every_supported_language_has_the_same_key_set() {
        let i18n = I18n::new();
        let reference = i18n
            .translations
            .expect("translations")
            .get("setup_title")
            .expect("missing reference key setup_title");

        let reference_locales: HashSet<&str> = reference.keys().map(String::as_str).collect();
        assert_eq!(reference_locales.len(), supported_language_codes().len());

        for language in supported_language_codes() {
            assert!(
                reference_locales.contains(language),
                "missing locale {language} for setup_title"
            );
        }

        for (key, locales) in i18n.translations.expect("translations") {
            let locale_set: HashSet<&str> = locales.keys().map(String::as_str).collect();
            assert_eq!(
                locale_set, reference_locales,
                "translation locales differ for key {key}"
            );
        }
    }
}
