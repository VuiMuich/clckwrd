#[cfg(test)]
use std::path::Path;

#[cfg(test)]
use crate::{
    file_handler::load_from_file,
    template,
    template::Template,
    time_processing::{get_proximity, get_time_in_words},
};
#[cfg(test)]
use time::Time;

// Testcases:
// - test for each hour the correct sentence array is chosen.
// - test for each minute the correct sentence array is chosen.
// - test special cases
#[test]
// fn test_hours() {
//     use crate::default;
//     use crate::hours;
//     use time;

//     let template = template::Template::default;

//     for i in 0..23 {
//         let test_hour = time::Time::from_hms(i, 0, 0);
//         let test_default = template.hours[i];
//     }
// }

fn test_midnight() {
    assert_eq!(
        crate::time_processing::get_hours_words(
            time::Time::MIDNIGHT.hour(),
            &template::Template::default()
        ),
        &String::from("twelve")
    )
}

#[test]
pub(crate) fn every_minute_is_sane() {
    let t = Template::default(); // or load the .ron
    for h in 0..24u8 {
        for m in 0..60u8 {
            let s = get_time_in_words(&t, Time::from_hms(h, m, 0).unwrap());
            assert!(!s.is_empty(), "{h:02}:{m:02}");
            assert!(!s.contains("err"), "{h:02}:{m:02} -> {s}"); // hour_err / mini_err / prepo_err
        }
    }
}

#[test]
pub(crate) fn proximity_all_slots() {
    let t = Template::default(); // or load the .ron
    for m in 0..60u8 {
        let time = Time::from_hms(12, m, 0).unwrap();
        println!("12:{m:02} -> {:?}", get_proximity(time, &t));
    }
}

#[test]
pub(crate) fn around_the_hour() {
    let t = Template::default();
    let words = |h, m| get_time_in_words(&t, Time::from_hms(h, m, 0).unwrap());

    assert!(words(12, 57).contains("one"), "12:57 -> {}", words(12, 57));
    assert!(!words(12, 57).contains("twelve"));
    assert!(words(12, 58).contains("one"));
    assert!(words(12, 3).contains("twelve")); // after the hour
    assert!(words(11, 57).contains("twelve")); // "to twelve" is right here
}

#[test]
pub(crate) fn next_hour_switches_at_template_minute() {
    let t = Template::default();
    let h = |m| get_proximity(Time::from_hms(5, m, 0).unwrap(), &t).1;
    assert_eq!(h(t.next_hour_from - 3), 5); // just before the switch
    assert_eq!(h(t.next_hour_from), 6); // at the switch
}

#[test]
fn loads_de_1_template_from_file() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("time-in-words-templates/de_1.ron");
    let t = load_from_file(&path).expect("de_1.ron should load");
    let time = time::Time::from_hms(12, 0, 0).unwrap();
    assert_ne!(
        crate::time_processing::get_time_in_words(&t, time),
        crate::time_processing::get_time_in_words(&Template::default(), time)
    );
}

#[test]
fn all_shipped_templates_parse() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("time-in-words-templates");
    let mut checked = 0;
    let mut failures = Vec::new();

    for entry in std::fs::read_dir(&dir).expect("template dir should exist") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "ron") {
            checked += 1;
            if let Err(e) = load_from_file(&path) {
                failures.push(format!("{}: {e}", path.display()));
            }
        }
    }

    assert!(checked > 0, "no .ron templates found in {}", dir.display());
    assert!(
        failures.is_empty(),
        "templates failed to load:\n{}",
        failures.join("\n")
    );
}
