use std::{env, io, process};
use atty::Stream;
use chrono::Utc;
use chrono::offset::TimeZone;
use getopts::Options;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [OPTIONS] [INPUT]...", program);
    print!("{}", opts.usage(&brief));

    println!("");
    println!("If stdin has been redirected then each line of stdin will be separately decoded and printed");
}

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

        print_usage(program, opts_spec);
        return;
    }

    let use_millis = opts.opt_present("m");
    let use_iso_format = opts.opt_present("s");
    let is_stdin_tty = atty::is(Stream::Stdin);

    if is_stdin_tty && opts.free.is_empty() {
        eprintln!("No input values provided");
        eprintln!("");
        print_usage(program, opts_spec);
        process::exit(1);
    }

    for arg in opts.free {
        process_input(&arg, use_millis, use_iso_format);
    }

    if is_stdin_tty {
        // NOTE: We only read from stdin (by default, we could conceivably add a flag for it)
        //       so that the user doesn't run the program and then sit confused as to whether the
        //       program is stuck or just waiting for input.
        return;
    }

    let mut input = String::new();
    loop {
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Error: {}", e.to_string());
            break;
        }

        let trimmed = input.trim();
        if trimmed.len() == 0 {
            break;
        }

        process_input(trimmed, use_millis, use_iso_format);
        input.clear();
    }
}

fn process_input(input: &str, use_millis: bool, use_iso_format: bool) {
    let input_timestamp = match input.parse::<i64>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Bad input: \"{}\" is not a valid unix timestamp. Error: '{}'", input, e);
            return;
        }
    };

    let output = match use_millis {
        true => {
            let seconds = input_timestamp/1000;
            let nanos = ((input_timestamp % 1000) * 1_000_000) as u32;
            Utc.timestamp(seconds, nanos)
        },
        false => Utc.timestamp(input_timestamp, 0)
    };

    if use_iso_format {
        println!("{}", output.to_rfc3339());
    } else {
        println!("{}", output.to_string());
    }
}

