//! The theme and language the user chose, which apply to the application rather than to a
//! document, and are kept between runs.

use eframe::egui::{self, ThemePreference};
use gridtool::i18n::{self, Language};

const THEME_KEY: &str = "theme";
const LANGUAGE_KEY: &str = "language";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preferences {
    pub theme: ThemePreference,
    pub language: Language,
}

impl Preferences {
    /// What the last run saved. Anything missing, or unknown to this version, keeps its default.
    pub fn load(storage: &dyn eframe::Storage) -> Self {
        Self {
            theme: storage
                .get_string(THEME_KEY)
                .and_then(|theme| theme_from_identifier(&theme))
                .unwrap_or_default(),
            language: storage
                .get_string(LANGUAGE_KEY)
                .and_then(|language| Language::from_identifier(&language))
                .unwrap_or_default(),
        }
    }

    pub fn save(self, storage: &mut dyn eframe::Storage) {
        storage.set_string(THEME_KEY, theme_identifier(self.theme).to_owned());
        storage.set_string(LANGUAGE_KEY, self.language.identifier().to_owned());
    }

    /// Puts these into use.
    pub fn apply(self, ctx: &egui::Context) {
        ctx.set_theme(self.theme);
        i18n::set_language(self.language);
    }
}

fn theme_identifier(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::System => "system",
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
    }
}

fn theme_from_identifier(identifier: &str) -> Option<ThemePreference> {
    [
        ThemePreference::System,
        ThemePreference::Light,
        ThemePreference::Dark,
    ]
    .into_iter()
    .find(|theme| theme_identifier(*theme) == identifier)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use eframe::Storage;

    use super::*;

    #[derive(Default)]
    struct FakeStorage(HashMap<String, String>);

    impl eframe::Storage for FakeStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }

        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.to_owned(), value);
        }

        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }

        fn flush(&mut self) {}
    }

    #[test]
    fn with_nothing_saved_the_theme_follows_the_system_and_the_language_is_english() {
        let preferences = Preferences::load(&FakeStorage::default());

        assert_eq!(preferences.theme, ThemePreference::System);
        assert_eq!(preferences.language, Language::EnUs);
    }

    #[test]
    fn what_is_saved_is_loaded_again() {
        for theme in [
            ThemePreference::System,
            ThemePreference::Light,
            ThemePreference::Dark,
        ] {
            let saved = Preferences {
                theme,
                language: Language::EsEs,
            };
            let mut storage = FakeStorage::default();

            saved.save(&mut storage);

            assert_eq!(Preferences::load(&storage), saved);
        }
    }

    #[test]
    fn saved_values_this_version_does_not_know_keep_their_defaults() {
        let mut storage = FakeStorage::default();
        storage.set_string(THEME_KEY, "sepia".to_owned());
        storage.set_string(LANGUAGE_KEY, "fr-FR".to_owned());

        assert_eq!(Preferences::load(&storage), Preferences::default());
    }
}
