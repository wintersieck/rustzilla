use std::process;
use clap::{Arg, App};

fn main() {
    let cli_args = App::new("Rustzilla")
                          .version("1.0")
                          .author("Allan Wintersieck <awintersieck@gmail.com>")
                          .about("Scrapes Industry RiNo Roomzilla")
                          .arg(Arg::with_name("start")
                               .short("s")
                               .long("start")
                               .value_name("START")
                               .help("Start time in format of HH:MM (24-hour clock)")
                               .takes_value(true))
                          .arg(Arg::with_name("end")
                               .short("e")
                               .long("end")
                               .value_name("END")
                               .help("End time in format of HH:MM (24-hour clock)")
                               .takes_value(true))
                          .get_matches();

    if let Err(e) = rustzilla::run(&cli_args) {
        eprintln!("Application error: {}\n\nSee rustzilla -h for help", e);

        process::exit(1);
    }
}
