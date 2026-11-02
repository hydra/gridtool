//! The language of everything the application says, with the words kept in one translation file
//! for each language. The files are compiled in, so an installed application needs nothing else
//! and can be used from any working directory.
//!
//! Nothing here depends on the GUI, and `egui-i18n` does not depend on `egui` either.

use std::sync::LazyLock;

use egui_i18n::fluent::FluentArgs;

/// The translation of `key`, in the current language, with any values it needs.
///
/// ```
/// use gridtool::tr;
///
/// assert_eq!(tr!("object-area", { number: 2 }), "Area 2");
/// ```
#[macro_export]
macro_rules! tr {
    ($key:expr) => {
        $crate::i18n::translate($key, &[])
    };
    ($key:expr, { $($name:ident: $value:expr),+ $(,)? }) => {
        $crate::i18n::translate($key, &[$((stringify!($name), $value.to_string())),+])
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    EnUs,
    EsEs,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::EnUs, Language::EsEs];

    /// The name of its translation folder, and how a choice of language is remembered.
    pub fn identifier(self) -> &'static str {
        match self {
            Language::EnUs => "en-US",
            Language::EsEs => "es-ES",
        }
    }

    pub fn from_identifier(identifier: &str) -> Option<Language> {
        Language::ALL
            .into_iter()
            .find(|language| language.identifier() == identifier)
    }

    /// What the language calls itself, so that it can be found whatever language is in use.
    pub fn name(self) -> String {
        match self {
            Language::EnUs => tr!("language-en-US"),
            Language::EsEs => tr!("language-es-ES"),
        }
    }

    fn translations(self) -> &'static str {
        match self {
            Language::EnUs => include_str!("../assets/translations/en-US/translations.ftl"),
            Language::EsEs => include_str!("../assets/translations/es-ES/translations.ftl"),
        }
    }
}

/// Loads the translations the first time anything needs them, so the core works without being
/// set up first.
static LOADED: LazyLock<()> = LazyLock::new(|| {
    // Neither language is written right to left, so the marks Fluent puts around values for that
    // would only be extra characters in the text.
    egui_i18n::set_use_isolating(false);
    for language in Language::ALL {
        egui_i18n::load_translations_from_text(language.identifier(), language.translations())
            .expect("the bundled translations are valid");
    }
    let default = Language::default().identifier();
    egui_i18n::set_fallback(default);
    egui_i18n::set_language(default);
});

pub fn language() -> Language {
    LazyLock::force(&LOADED);
    Language::from_identifier(&egui_i18n::get_language()).unwrap_or_default()
}

pub fn set_language(language: Language) {
    LazyLock::force(&LOADED);
    egui_i18n::set_language(language.identifier());
}

/// What [`tr!`] calls. A language without the message uses the default language's.
pub fn translate(key: &str, values: &[(&str, String)]) -> String {
    LazyLock::force(&LOADED);
    let mut args = FluentArgs::new();
    for (name, value) in values {
        args.set(*name, value.as_str());
    }
    egui_i18n::translate_fluent(key, &args)
}

#[cfg(test)]
mod tests;
