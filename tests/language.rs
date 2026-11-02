//! Changes the language, which applies to the whole process, so it has a test program of its own:
//! the unit tests run side by side and expect English.

use gridtool::action::Action;
use gridtool::calibration::Line;
use gridtool::i18n::{self, Language};
use gridtool::tr;

#[test]
fn a_chosen_language_applies_to_everything_the_core_says() {
    let action = Action::SetDistance {
        line: Line::A,
        distance: None,
    };
    assert_eq!(i18n::language(), Language::EnUs);
    assert_eq!(action.to_string(), "Clear distance A");

    i18n::set_language(Language::EsEs);

    assert_eq!(i18n::language(), Language::EsEs);
    assert_eq!(action.to_string(), "Borrar distancia A");
    assert_eq!(tr!("object-area", { number: 2 }), "Área 2");
    assert_eq!(Language::EnUs.name(), "English (United States)");
    assert_eq!(Language::EsEs.name(), "Español (España)");

    i18n::set_language(Language::EnUs);

    assert_eq!(action.to_string(), "Clear distance A");
}
