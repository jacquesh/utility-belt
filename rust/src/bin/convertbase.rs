use std::env;
use std::io;
use std::process;
use std::string::String;
use atty::Stream;
use getopts::Options;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} -f [FROM-BASE] -t [TO-BASE] [OPTIONS] [INPUT]...", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts_spec = Options::new();
    opts_spec.optflag("h", "help", "print this help menu");
    opts_spec.optflag("i", "interactive", "interactively read from standard input, converting each line");
    opts_spec.optopt("f", "from", "the base from which to convert the input string", "FROM-BASE");
    opts_spec.optopt("t", "to", "the base to which to convert the output string", "TO-BASE");
    let opts = match opts_spec.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e.to_string());
            eprintln!("Try '{} --help' for more information", program);
            process::exit(1);
        }
    };

    if opts.opt_present("h") {
        println!("Convert a number or byte-string from one base to another.");
        println!("");

        print_usage(program, opts_spec);

        println!("");
        println!("If stdin has been redirected then each line of stdin will be separately decoded and printed");
        return;
    }

    let is_interactive = opts.opt_present("i");
    let is_stdin_tty = atty::is(Stream::Stdin);
    if is_interactive && !is_stdin_tty {
        println!("Bad argument: Interactive mode is not available with a non-interactive standard input stream");
        process::exit(1);
    }

    if is_stdin_tty && !is_interactive && opts.free.is_empty() {
        eprintln!("No input values provided");
        eprintln!("");
        print_usage(program, opts_spec);
        process::exit(1);
    }

    let input_base = match opts.opt_str("f") {
        Some(input_str) => {
            match input_str.parse::<u8>() {
                Ok(strval) => strval,
                Err(err) => {
                    eprintln!("{} is not valid as a numeric base: {}", input_str, err);
                    process::exit(1);
                }
            }
        },
        None => {
            eprintln!("A from-base is required but none was specified");
            eprintln!("");
            print_usage(program, opts_spec);
            process::exit(1);
        }
    };
    let output_base = match opts.opt_str("t") {
        Some(output_str) => {
            match output_str.parse::<u8>() {
                Ok(strval) => strval,
                Err(err) => {
                    eprintln!("{} is not valid as a numeric base: {}", output_str, err);
                    process::exit(1);
                }
            }
        },
        None => {
            eprintln!("A to-base is required by none was specified");
            eprintln!("");
            print_usage(program, opts_spec);
            process::exit(1);
        }
    };

    for arg in opts.free {
        process_input(&arg, input_base, output_base);
    }

    if !is_interactive && is_stdin_tty {
        return
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
        process_input(trimmed, input_base, output_base);
        input.clear();
    }
}

fn process_input(input: &str, input_base: u8, output_base: u8) {
    let input_bytes = match input_base {
        2 => alltools::binary::to_bytes(input),
        10 => alltools::decimal::to_bytes(input),
        16 => alltools::hex::to_bytes(input),
        64 => alltools::base64::to_bytes(input),
        _ => {
            eprintln!("{} is not a supported input base", input_base);
            return;
        }
    };

    match input_bytes {
        Some(input_bytes) => {
            let output = match output_base {
                2 => alltools::binary::from_bytes(&input_bytes, true),
                10 => alltools::decimal::from_bytes(&input_bytes),
                16 => alltools::hex::from_bytes(&input_bytes),
                64 => alltools::base64::from_bytes(&input_bytes),
                _ => {
                    eprintln!("{} is not a supported output base", output_base);
                    return;
                }
            };
            println!("{}", output);
        }
        None => {
            eprintln!("Bad input: \"{}\" is not a valid base-{} literal", input, input_base)
        }
    }
}
