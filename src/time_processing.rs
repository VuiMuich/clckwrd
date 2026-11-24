use std::{thread::sleep, time::Duration};

use crate::{template::Template, Mode};
use rand::seq::SliceRandom;
use time::{format_description::parse_borrowed, OffsetDateTime, Time};

// TODO
// - clean up 'err' and make fn panic
pub(crate) fn get_simple_time(time: Time) -> String {
    let format = parse_borrowed::<1>("[hour]:[minute]").unwrap();
    time.format(&format).unwrap()
}

pub(crate) fn get_time_in_words(template: &Template, local: Time) -> String {
    let delta_minutes = local.minute() % 5;
    let start_sentence = template.start_sentence.choose(&mut rand::thread_rng());
    let preposition = get_preposition(delta_minutes, template);
    let (minute_slot, hour_overflow) = get_proximity(local, template);
    let minutes_string = get_minutes_words(minute_slot, template);
    let hour_string = get_hours_words(hour_overflow, template);
    let special_cases = get_special_case_words(local, template);

    if let Some(is_special_case) = special_cases {
        is_special_case.clone()
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

#[allow(clippy::unnested_or_patterns)]
#[allow(clippy::zero_prefixed_literal)]
fn get_special_case_words(local: Time, template: &Template) -> Option<&String> {
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
pub(crate) fn get_proximity(local: Time, template: &Template) -> (u8, u8) {
    let minute_slot = (local.minute() + 2) / 5;
    let (minute_slot, hour_overflow) = if minute_slot == 12 {
        (0, 1)
    } else {
        (minute_slot, 0)
    };
    let next_hour = u8::from(minute_slot * 5 >= template.next_hour_from);
    (minute_slot, (local.hour() + hour_overflow + next_hour) % 12)
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

pub(crate) fn render(mode: Mode, template: &Template, time: Time, addendum: bool) -> String {
    let words = || {
        let mut s = get_time_in_words(template, time);
        if addendum {
            if let Some(a) = template.addendum.choose(&mut rand::thread_rng()) {
                s.push(' ');
                s.push_str(a);
            }
        }
        s
    };
    match mode {
        Mode::Words => words(),
        Mode::Digital => get_simple_time(time),
        Mode::Combined => format!("{} or in other words {}", get_simple_time(time), words()),
    }
}

pub(crate) fn run_loop(mode: Mode, template: &Template, addendum: bool) -> ! {
    loop {
        let now = OffsetDateTime::now_local().unwrap();
        println!("{}", render(mode, template, now.time(), addendum));
        let wait = Duration::from_secs(u64::from(60 - now.second()))
            .checked_sub(Duration::from_nanos(u64::from(now.nanosecond())))
            .unwrap()
            + Duration::from_millis(50);
        sleep(wait);
    }
}
