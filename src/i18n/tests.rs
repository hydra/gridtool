use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

/// Each message of a translation file, with the values it uses.
fn messages(language: Language) -> BTreeMap<String, BTreeSet<String>> {
    let mut messages = BTreeMap::new();
    for line in language.translations().lines() {
        if line.starts_with('#') {
            continue;
        }
        if let Some((key, text)) = line.split_once(" = ") {
            messages.insert(key.to_owned(), values_in(text));
        }
    }
    messages
}

/// The names of the `{ $value }` placeholders in the text of a message.
fn values_in(text: &str) -> BTreeSet<String> {
    text.split("{ $")
        .skip(1)
        .filter_map(|rest| rest.split(" }").next())
        .map(str::to_owned)
        .collect()
}

fn source_files(folder: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// The text of every source file, including this one.
fn sources() -> Vec<String> {
    let mut files = Vec::new();
    source_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    files
        .iter()
        .map(|file| fs::read_to_string(file).unwrap())
        .collect()
}

/// The keys `tr!` is given in `source`. Every key is written out where it is used, so that all
/// of them are found here. Other text that ends the same way, such as `include_str!`, has
/// characters no key has.
fn keys_used_in(source: &str) -> Vec<String> {
    source
        .split("tr!(")
        .skip(1)
        .filter_map(|rest| rest.trim_start().strip_prefix('"'))
        .filter_map(|rest| rest.split('"').next())
        .filter(|key| key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_language_has_the_same_messages_with_the_same_values() {
    let english = messages(Language::EnUs);
    assert!(!english.is_empty());

    for language in Language::ALL {
        let translated = messages(language);
        for (key, values) in &english {
            assert_eq!(translated.get(key), Some(values), "{language:?}: {key}");
        }
        for key in translated.keys() {
            assert!(english.contains_key(key), "{language:?}: unknown {key}");
        }
    }
}

#[test]
fn every_message_used_exists_and_every_message_is_used() {
    let english = messages(Language::EnUs);
    let sources = sources();

    let used: BTreeSet<_> = sources
        .iter()
        .flat_map(|source| keys_used_in(source))
        .collect();
    for key in &used {
        assert!(english.contains_key(key), "no message for {key}");
    }
    for key in english.keys() {
        assert!(used.contains(key), "{key} is not used");
    }
}

#[test]
fn values_are_filled_in_without_extra_characters() {
    assert_eq!(
        tr!("error-measurement-missing", { number: 3, count: 2 }),
        "measurement 3 does not exist (there are 2)"
    );
}

#[test]
fn a_language_is_found_from_its_identifier() {
    for language in Language::ALL {
        assert_eq!(
            Language::from_identifier(language.identifier()),
            Some(language)
        );
        assert!(!language.name().is_empty());
    }
    assert_eq!(Language::from_identifier("fr-FR"), None);
}

#[test]
fn english_is_the_default_language() {
    assert_eq!(Language::default(), Language::EnUs);
    assert_eq!(Language::default().identifier(), "en-US");
}
