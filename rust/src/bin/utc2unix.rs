use std::env;
use std::process;
use chrono::{DateTime, Utc};
use chrono::offset::TimeZone;
use getopts::Options;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts_spec = Options::new();
    opts_spec.optflag("h", "help", "print this help menu");
    opts_spec.optflag("m", "millis", "use the input as the number of milliseconds since the unix epoch, rather than the number of seconds");
    opts_spec.optflag("s", "iso", "use the standard ISO-8601/RFC-3339 format when formatting the output text");
    let opts = match opts_spec.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e.to_string());
            eprintln!("Try '{} --help' for more information", program);
            process::exit(1);
        }
    };

    if opts.opt_present("h") {
        println!("Convert a unix timestamp into an equivalent human-readable date-time string");
        println!("");

        let brief = format!("Usage: {} [OPTIONS] [INPUT]...", program);
        print!("{}", opts_spec.usage(&brief));
        return;
    }

    let use_millis = opts.opt_present("m");
    // TODO: Print usage if no other args given (and not interactive?)
    for arg in opts.free {
        process_input(&arg, use_millis);
    }

    // TODO: Read from stdin so we can pipe things together
}

fn process_input(input: &str, use_millis: bool) {
    match input.parse::<DateTime<Utc>>() {
        Ok(dt) => {
            // TODO: This should parse the output of non-iso unix2utc output?
            output_timestamp(dt, use_millis);
            return;
        },
        Err(_e) => {}
    };

    match DateTime::parse_from_rfc3339(input) {
        Ok(dt) => {
            output_timestamp(dt, use_millis);
            return;
        },
        Err(_e) => {}
    };

    eprintln!("Bad input: {} is not a time string with a recognized format", input);
}

fn output_timestamp(instant: DateTime<impl TimeZone>, use_millis: bool) {
    if use_millis {
        let timestamp = (instant.timestamp() * 1000) + (instant.timestamp_subsec_millis() as i64);
        println!("{}", timestamp);
    } else {
        println!("{}", instant.timestamp());
    }
}

