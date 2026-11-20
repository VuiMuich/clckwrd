use std::{thread::sleep, time::Duration};

use crate::template::Template;
use rand::seq::SliceRandom;
use time::{format_description, OffsetDateTime, Time};

// TODO
// - clean up 'err' and make fn panic
// - import language templates
// - default to english
pub(crate) fn get_time_in_words(template: &Template, local: Time) -> String {
    let delta_minutes = local.minute() % 5;
    let start_sentence = template.start_sentence.choose(&mut rand::thread_rng());
    let preposition = get_preposition(delta_minutes, template);
    // let minutes = get_proxy_minutes(local);
    // let hours = get_proxy_hours(minutes, local);
    let (minute_slot, hour_overflow) = gex_proximity(local);
    let minutes_string = get_minutes_words(minute_slot, template);
    let hour_string = get_hours_words(hour_overflow, template);
    let special_cases = get_special_case_words(local, template);

    if let Some(is_special_case) = special_cases {
        is_special_case.to_string()
    } else if minute_slot == 0 {
        format!(
            "{} {} {}.",
            start_sentence.unwrap(),
            preposition.unwrap(),
            hour_string
        )
    } else if preposition == Some(&String::new()) {
        format!(
            "{} {} {}.",
            start_sentence.unwrap(),
            minutes_string,
            hour_string
        )
    } else {
        format!(
            "{} {} {} {}.",
            start_sentence.unwrap(),
            preposition.unwrap(),
            minutes_string,
            hour_string
        )
    }
}

fn get_special_case_words(local: Time, template: &Template) -> Option<&String> {
    // match &*format!("{:02}:{:02}", local.hour(), local.minute()) {
    match (local.hour(), local.minute()) {
        (23, 58) | (23, 59) => Some(&template.special_cases.before_midnight),
        (00, 00) => Some(&template.special_cases.midnight),
        (00, 01) | (00, 02) => Some(&template.special_cases.after_midnight),
        (00, 58) | (12, 58) => Some(&template.special_cases.two_to_one),
        (00, 59) | (12, 59) => Some(&template.special_cases.one_to_one),
        (01, 00) | (13, 00) => Some(&template.special_cases.exactly_one),
        (01, 01) | (13, 01) => Some(&template.special_cases.one_past_one),
        (01, 02) | (13, 02) => Some(&template.special_cases.two_past_one),
        (12, 00) => Some(&template.special_cases.high_noon),
        _ => None,
    }
}

pub(crate) fn get_hours_words(hours: u8, template: &Template) -> &String {
    match hours {
        0 | 12 => &template.hours.twelve,
        1 => &template.hours.one,
        2 => &template.hours.two,
        3 => &template.hours.three,
        4 => &template.hours.four,
        5 => &template.hours.five,
        6 => &template.hours.six,
        7 => &template.hours.seven,
        8 => &template.hours.eight,
        9 => &template.hours.nine,
        10 => &template.hours.ten,
        11 => &template.hours.eleven,
        _ => &template.hours.hour_err,
    }
}

fn get_minutes_words(minutes: u8, template: &Template) -> &String {
    match minutes {
        1 => &template.minutes.five_past,
        2 => &template.minutes.ten_past,
        3 => &template.minutes.quarter_past,
        4 => &template.minutes.twenty_past,
        5 => &template.minutes.twenty_five_past,
        6 => &template.minutes.half_past,
        7 => &template.minutes.twenty_five_to,
        8 => &template.minutes.twenty_to,
        9 => &template.minutes.quarter_to,
        10 => &template.minutes.ten_to,
        11 | 12 => &template.minutes.five_to,
        _ => &template.minutes.mini_err,
    }
}

// We want to make sure we only use precise wording, when needed and round or use the upcoming hour otherwise
fn gex_proximity(local: Time) -> (u8, u8) {
    let minute_slot = (local.minute() + 2) / 5;
    let (minute_slot, hour_overflow) = if minute_slot == 12 {
        (0, 1)
    } else {
        (minute_slot, 0)
    };
    let next_hour = u8::from(minute_slot >= 4); //TODO: take it from the template
    (
        minute_slot,
        (local.hour() as u8 + hour_overflow + next_hour) % 12,
    )
}

fn get_preposition(delta_minute: u8, template: &Template) -> Option<&String> {
    match delta_minute {
        3 | 4 => template.prepositions.almost.choose(&mut rand::thread_rng()),
        0 => template
            .prepositions
            .exactly
            .choose(&mut rand::thread_rng()),
        1 | 2 => template
            .prepositions
            .roughly
            .choose(&mut rand::thread_rng()),
        _ => template
            .prepositions
            .prepo_err
            .choose(&mut rand::thread_rng()),
    }
}

pub(crate) fn time_loop_template(template: &Template) -> ! {
    loop {
        let now = OffsetDateTime::now_local().unwrap();
        println!("{}", get_time_in_words(template, now.time()));
        let wait = Duration::from_secs(u64::from(60 - now.second()))
            - Duration::from_nanos(u64::from(now.nanosecond()))
            + Duration::from_millis(50);
        sleep(wait);
    }
}

pub(crate) fn time_loop_simple() -> ! {
    loop {
        let now = OffsetDateTime::now_local().unwrap();
        println!("{}", get_simple_time());
        let wait = Duration::from_secs(u64::from(60 - now.second()))
            - Duration::from_nanos(u64::from(now.nanosecond()))
            + Duration::from_millis(50);
        sleep(wait);
    }
}

pub(crate) fn get_simple_time() -> String {
    let format = format_description::parse("[hour]:[minute]").unwrap();
    OffsetDateTime::now_local()
        .unwrap()
        .time()
        .format(&format)
        .unwrap()
}

#[test]
fn every_minute_is_sane() {
    let tpl =
        crate::file_handler::load_template(Some("./time-in-words-templates/de.ron".to_string())); // or whatever the loader is
    for h in 0..24u8 {
        for m in 0..60u8 {
            let s = get_time_in_words(&tpl, Time::from_hms(h, m, 0).unwrap());
            assert!(!s.is_empty(), "{h:02}:{m:02}");
            assert!(!s.contains("err"), "{h:02}:{m:02} -> {s}"); // hour_err / mini_err / prepo_err
        }
    }
}
