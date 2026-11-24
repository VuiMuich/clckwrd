use std::path::{Path, PathBuf};

use time::Time;

use crate::{
    file_handler::{load_from_file, TemplateError},
    template::Template,
    time_processing::{get_hours_words, get_proximity, get_time_in_words, render},
    Mode,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn at(h: u8, m: u8) -> Time {
    Time::from_hms(h, m, 0).unwrap()
}

fn template_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("time-in-words-templates")
}

/// Loads every shipped `.ron` through the real loader.
/// Collects all failures and panics once, and refuses to return an empty list,
/// so loops over the result can never pass vacuously.
fn shipped_templates() -> Vec<(String, Template)> {
    let dir = template_dir();
    let mut loaded = Vec::new();
    let mut failures = Vec::new();

    for entry in std::fs::read_dir(&dir).expect("template dir should exist") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "ron") {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            match load_from_file(&path) {
                Ok(t) => loaded.push((name, t)),
                Err(e) => failures.push(format!("{}: {e}", path.display())),
            }
        }
    }

    assert!(
        failures.is_empty(),
        "templates failed to load:\n{}",
        failures.join("\n")
    );
    assert!(
        !loaded.is_empty(),
        "no .ron templates found in {}",
        dir.display()
    );
    loaded
}

fn shipped(name: &str) -> Template {
    shipped_templates()
        .into_iter()
        .find(|(n, _)| n == name)
        .map(|(_, t)| t)
        .unwrap_or_else(|| panic!("no shipped template named {name}"))
}

// ---------------------------------------------------------------------------
// Template loading
// ---------------------------------------------------------------------------

#[test]
fn all_shipped_templates_parse() {
    // The helper does the assertions.
    shipped_templates();
}

#[test]
fn embedded_default_matches_en_1() {
    // The embedded default deliberately ships without `add_text`.
    let strip = |t: Template| Template {
        addendum: Vec::new(),
        language: String::new(),
        language_label: String::new(),
        ..t
    };
    assert_eq!(strip(Template::default()), strip(shipped("en_1")));
}

#[test]
fn de_1_loads_with_its_own_values() {
    let t = shipped("de_1");
    assert_eq!(t.language, "deutsch_1");
    assert_eq!(t.language_label, "Deutsch Variante 1");
    assert_eq!(t.next_hour_from, 20);
}

#[test]
fn shipped_templates_keep_their_next_hour_from() {
    // Exact values: a test comparing against `t.next_hour_from` itself would
    // pass even if the field silently fell back to a default.
    assert_eq!(shipped("en_1").next_hour_from, 35);
    assert_eq!(shipped("de_1").next_hour_from, 20);
}

#[test]
fn required_sentence_lists_are_not_empty() {
    // `get_time_in_words` picks a random start sentence and unwraps it.
    for (name, t) in shipped_templates() {
        assert!(
            !t.start_sentence.is_empty(),
            "{name}: start_sentence is empty"
        );
    }
}

#[test]
fn missing_file_is_an_io_error() {
    let err = load_from_file(Path::new("/nonexistent/uhrwerk/nope.ron")).unwrap_err();
    assert!(matches!(err, TemplateError::Io(_)), "got {err}");
}

#[test]
fn template_format_is_strict() {
    let raw = std::fs::read_to_string(template_dir().join("en_1.ron")).unwrap();
    assert!(ron::from_str::<Template>(&raw).is_ok());

    // A misspelled or renamed key must fail instead of silently using a default.
    for (from, to) in [
        ("language_label", "lagnuag_label"),
        ("next_hour_from", "next_hour_slot"),
    ] {
        let broken = raw.replace(from, to);
        assert_ne!(raw, broken, "`{from}` not found in en_1.ron");
        assert!(
            ron::from_str::<Template>(&broken).is_err(),
            "`{from}` -> `{to}` should not parse"
        );
    }
}

// ---------------------------------------------------------------------------
// Hours
// ---------------------------------------------------------------------------

#[test]
fn midnight_is_twelve() {
    assert_eq!(
        get_hours_words(Time::MIDNIGHT.hour(), &Template::default()),
        "twelve"
    );
}

#[test]
fn every_hour_word_is_defined() {
    for (name, t) in shipped_templates() {
        for h in 0..12u8 {
            let word = get_hours_words(h, &t).to_string();
            assert!(
                !word.is_empty() && !word.contains("err"),
                "{name}: hour {h} -> {word:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Proximity (slot + hour)
// ---------------------------------------------------------------------------

#[test]
fn proximity_table_en() {
    // en_1: next_hour_from = 35, slot = (minute + 2) / 5.
    // (hour, minute, expected slot, expected hour)
    let cases: [(u8, u8, u8, u8); 16] = [
        (12, 0, 0, 0),
        (12, 2, 0, 0),
        (12, 3, 1, 0),
        (12, 27, 5, 0),
        (12, 28, 6, 0),
        (12, 32, 6, 0),
        (12, 33, 7, 1), // switch to the next hour
        (12, 57, 11, 1),
        (12, 58, 0, 1), // slot overflow
        (12, 59, 0, 1),
        (11, 57, 11, 0), // "five to twelve"
        (11, 58, 0, 0),
        (23, 57, 11, 0),
        (23, 58, 0, 0), // across midnight
        (0, 0, 0, 0),
        (0, 57, 11, 1),
    ];

    let t = shipped("en_1");
    for (h, m, slot, hour) in cases {
        assert_eq!(get_proximity(at(h, m), &t), (slot, hour), "{h:02}:{m:02}");
    }
}

#[test]
fn proximity_stays_in_range() {
    for (name, t) in shipped_templates() {
        for h in 0..24u8 {
            for m in 0..60u8 {
                let (slot, hour) = get_proximity(at(h, m), &t);
                assert!(slot < 12, "{name} {h:02}:{m:02}: slot {slot}");
                assert!(hour < 12, "{name} {h:02}:{m:02}: hour {hour}");
            }
        }
    }
}

#[test]
fn next_hour_switches_at_template_minute() {
    for (name, t) in shipped_templates() {
        let nhf = t.next_hour_from;
        if nhf < 5 {
            continue; // no "before the switch" minute to test
        }
        let hour = |m| get_proximity(at(5, m), &t).1; // hour 5 avoids the `% 12` wrap
        assert_eq!(hour(nhf - 3), 5, "{name}: just before the switch");
        assert_eq!(hour(nhf), 6, "{name}: at the switch");
    }
}

// ---------------------------------------------------------------------------
// Sentences
// ---------------------------------------------------------------------------

#[test]
fn every_minute_is_sane() {
    for (name, t) in shipped_templates() {
        for h in 0..24u8 {
            for m in 0..60u8 {
                // The start sentence is random, so sample a few times.
                for _ in 0..3 {
                    let s = get_time_in_words(&t, at(h, m));
                    assert!(!s.is_empty(), "{name} {h:02}:{m:02}");
                    // error placeholders: hour_err / mini_err / prepo_err
                    assert!(!s.contains("err"), "{name} {h:02}:{m:02} -> {s}");
                }
            }
        }
    }
}

#[test]
fn sentences_have_clean_spacing() {
    for (name, t) in shipped_templates() {
        for h in 0..24u8 {
            for m in 0..60u8 {
                let s = get_time_in_words(&t, at(h, m));
                assert_eq!(s.trim(), s, "{name} {h:02}:{m:02}: stray whitespace");
                assert!(
                    !s.contains("  "),
                    "{name} {h:02}:{m:02}: double space in {s:?}"
                );
            }
        }
    }
}

/// (template, hour, word used at the full hour) where it differs from the hour table.
const FULL_HOUR_OVERRIDES: [(&str, u8, &str); 1] = [("de_1", 1, "ein")];

#[test]
fn full_hours_name_their_hour() {
    // Hours 0 and 12 are skipped: they may be special cases (midnight, noon).
    for (name, t) in shipped_templates() {
        for h in (1..24u8).filter(|h| h % 12 != 0) {
            let word = FULL_HOUR_OVERRIDES
                .iter()
                .find(|(n, hr, _)| *n == name && *hr == h % 12)
                .map_or_else(
                    || get_hours_words(h % 12, &t).to_string(),
                    |(_, _, w)| (*w).to_string(),
                );
            let s = get_time_in_words(&t, at(h, 0));
            assert!(
                s.contains(&word),
                "{name} {h:02}:00 -> {s} (expected `{word}`)"
            );
        }
    }
}

#[test]
fn around_the_hour_en() {
    let t = Template::default();
    let words = |h, m| get_time_in_words(&t, at(h, m));

    assert!(words(12, 57).contains("one"), "12:57 -> {}", words(12, 57));
    assert!(!words(12, 57).contains("twelve"));
    assert!(words(12, 58).contains("one"));
    assert!(words(12, 3).contains("twelve")); // after the hour
    assert!(words(11, 57).contains("twelve")); // "to twelve" is right here
}

#[test]
fn half_past_names_the_next_hour_de() {
    let t = shipped("de_1");

    let s = get_time_in_words(&t, at(8, 30));
    assert!(s.contains("halb neun"), "08:30 -> {s}");

    let s = get_time_in_words(&t, at(12, 57));
    assert!(s.contains("eins"), "12:57 -> {s}");
}

// ---------------------------------------------------------------------------
// render(): modes and addendum
// ---------------------------------------------------------------------------

#[test]
fn digital_has_no_words() {
    let s = render(Mode::Digital, &Template::default(), at(12, 57), false);
    assert!(s.starts_with("12:57"), "{s}");
    assert!(!s.contains("one"), "{s}");
}

#[test]
fn combined_has_both_parts() {
    let t = Template::default();
    let s = render(Mode::Combined, &t, at(12, 57), false);
    assert!(s.starts_with("12:57"), "{s}");
    assert!(s.contains(&t.combine_bridge), "{s}");
    assert!(s.contains("one"), "{s}");
}

fn words_with_addendum(addendum: &[&str], enabled: bool) -> String {
    let t = Template {
        addendum: addendum.iter().map(|s| (*s).to_string()).collect(),
        ..Template::default()
    };
    render(Mode::Words, &t, at(12, 0), enabled)
}

#[test]
fn addendum_is_appended_when_enabled() {
    let s = words_with_addendum(&["Carpe diem."], true);
    assert!(s.ends_with(" Carpe diem."), "{s}");
}

#[test]
fn addendum_is_skipped_when_disabled() {
    let s = words_with_addendum(&["Carpe diem."], false);
    assert!(!s.contains("Carpe diem."), "{s}");
}

#[test]
fn empty_addendum_does_not_panic_or_pad() {
    let s = words_with_addendum(&[], true);
    assert!(!s.is_empty());
    assert!(!s.ends_with(' '), "{s:?}");
}

#[test]
fn addendum_is_part_of_combined_output() {
    let t = Template {
        addendum: vec!["Carpe diem.".to_string()],
        ..Template::default()
    };
    let s = render(Mode::Combined, &t, at(12, 0), true);
    assert!(s.ends_with("Carpe diem."), "{s}");
}

// ---------------------------------------------------------------------------
// Still to do
// ---------------------------------------------------------------------------

#[ignore = "special cases (get_special_case_words) are not covered yet"]
#[test]
fn all_special_cases() {}
