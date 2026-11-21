// Uhrwerk is a CLI tool to print 'time in words'
// It is heavily inspired by TickeTack (www.ticketack.de) and actually uses their language strings currently
// Copyright 2021, Johannes Mayrhofer
// License MIT

mod default;
mod file_handler;
mod hours;
mod minutes;
mod prepositions;
mod special_cases;
mod template;
mod time_processing;

use crate::{
    template::Template,
    time_processing::{get_simple_time, get_time_in_words, time_loop_simple, time_loop_template},
};
use clap::{arg, command};
use time::OffsetDateTime;

// TODO implement structs properly
// - write Tests for special cases, testing randomness of prepositions, check some random times.
// - clean up 'err' and make fn panic
// - import language templates
// - add 'write to file'
// - load template files
// - handle Errors
fn main() {
    let args = command!("Uhrwerk")
        .author("Johannes Mayrhofer <vuimuich@quantentunnel.de>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("prints current system time in words continuously")
        .after_help(templates_help())
        .args(&[
            arg!( -q --quit "Prints time in words only once"),
            arg!( -d --digital "Prints time as digital clock in HH:MM (24h)"),
            arg!( -l --language "Chose language. Available Languages see below"),
            arg!( -t --template <NAME> "Use template file <NAME>.ron from $XDG_CONFIG_HOME/uhrwerk/templates")
                .conflicts_with("path"),
            arg!( -p --path <PATH> "Use template from a custom path" ),
            arg!( -o --output "Wirte output to file with specified path"),
            arg!([INPUT] "Sets the input path"),
            arg!([OUTPUT] "Sets the output path"),
        ])
        .get_matches();

    let template = match (
        args.get_one::<String>("path"),
        args.get_one::<String>("template"),
    ) {
        (Some(path), _) => file_handler::load_template(Some(path.clone())),
        (None, Some(name)) => match file_handler::find_template(name) {
            Some(p) => file_handler::load_template(Some(p.to_string_lossy().into_owned())),
            None => {
                eprintln!("uhrwerk: template '{name}.ron' not found in config dir");
                std::process::exit(1);
            }
        },
        (None, None) => Template::default(),
    };

    if args.get_flag("digital") {
        println!("{}", get_simple_time());
        if args.get_flag("quit") {
            return;
        }

        time_loop_simple()
    };

    println!(
        "{}",
        get_time_in_words(&template, OffsetDateTime::now_local().unwrap().time())
    );

    if args.get_flag("quit") {
        return;
    }

    time_loop_template(&template)
}

fn templates_help() -> String {
    let dir = file_handler::template_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_else(|| "<no config dir>".into());
    let names = file_handler::list_templates();

    if names.is_empty() {
        format!("Templates in {dir}:\n  (none found)")
    } else {
        format!("Templates in {dir}:\n  {}", names.join("\n  "))
    }
}

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
