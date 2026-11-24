// Clckwrd is a CLI tool to print 'time in words'
// It is heavily inspired by TickeTack (www.ticketack.de) and actuallrand::seq::SliceRandom::choose(&template.addendum, &mutrand::thread_rng())ight 2021, Johannes Mayrhofer
// License MIT

mod default;
mod file_handler;
mod hours;
mod minutes;
mod prepositions;
mod special_cases;
mod template;
mod tests;
mod time_processing;

use std::path::Path;

use crate::{
    template::Template,
    time_processing::{render, run_loop},
};
use clap::{arg, command};
use time::{format_description::parse_borrowed, OffsetDateTime, Time};

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Words,
    Digital,
    Combined,
}

// TODO implement structs properly
// - write Tests for special cases, testing randomness of prepositions, check some random times.
// - clean up 'err' and make fn panic
// - add 'write to file'
// - load template files
// - handle Errors
fn main() {
    let args = command!("Clckwrd")
        .author("Johannes Mayrhofer <vuimuich@quantentunnel.de>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("prints current system time in words continuously")
        .after_help(templates_help())
        .args(&[
            arg!( -q --quit "Prints time in words only once"),
            arg!( -a --addendum "Add a random selection from the templates `add_text` array." ),
            arg!( -d --digital "Prints time as digital clock in HH:MM (24h) format"),
            arg!( -c --combined "Combines regular output with digital clock output (see '-d|--digital' flag)" ), 
            arg!( -i --input <TIME> "Manually input a time in HH:MM (24h) format, implies `-q|--quit`" )
                .conflicts_with("digital"),
            arg!( -l --language "Chose language. Available Languages see below"),
            arg!( -t --template <NAME> "Use template file <NAME>.ron from $XDG_CONFIG_HOME/clckwrd/templates")
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
        (Some(path), _) => file_handler::load_template(Some(Path::new(path))),
        (None, Some(name)) => {
            if let Some(p) = file_handler::find_template(name) {
                file_handler::load_template(Some(&p))
            } else {
                eprintln!("clckwrd: template '{name}.ron' not found in config dir");
                std::process::exit(1);
            }
        }
        (None, None) => Template::default(),
    };
    let addendum = args.get_flag("addendum");
    let mode = if args.get_flag("combined") {
        Mode::Combined
    } else if args.get_flag("digital") {
        Mode::Digital
    } else {
        Mode::Words
    };

    let fixed_time = args.get_one::<String>("input").map(|s| {
        let fmt = parse_borrowed::<1>("[hour]:[minute]").unwrap();
        Time::parse(s, &fmt).expect("Wrong format, use `HH:MM`")
    });

    if args.get_flag("quit") || fixed_time.is_some() {
        let time = fixed_time.unwrap_or_else(|| OffsetDateTime::now_local().unwrap().time());
        println!("{}", render(mode, &template, time, addendum));
        return;
    }

    run_loop(mode, &template, addendum)
}

fn templates_help() -> String {
    let dir = file_handler::template_dir()
        .map_or_else(|| "<no config dir>".into(), |d| d.display().to_string());
    let names = file_handler::list_templates();

    if names.is_empty() {
        format!("Templates in {dir}:\n  (none found)")
    } else {
        format!("Templates in {dir}:\n  {}", names.join("\n  "))
    }
}
