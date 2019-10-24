use std::env;
use std::process;
use chrono::Utc;
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
    let use_iso_format = opts.opt_present("s");
    // TODO: Print usage if no other args given (and not interactive?)
    for arg in opts.free {
        process_input(&arg, use_millis, use_iso_format);
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

