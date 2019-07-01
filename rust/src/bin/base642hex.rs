use std::env;
use std::io;
use std::process;
use std::string::String;
use atty::Stream;
use getopts::Options;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [OPTIONS] [INPUT]...", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts_spec = Options::new();
    opts_spec.optflag("h", "help", "print this help menu");
    opts_spec.optflag("i", "interactive", "interactively read from standard input, converting each line");
    let opts = match opts_spec.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e.to_string());
            eprintln!("Try '{} --help' for more information", program);
            process::exit(1);
        }
    };

    if opts.opt_present("h") {
        println!("Convert a base64-encoded string to the hex string containing the same data");
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

    for arg in opts.free {
        process_input(&arg);
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
        process_input(trimmed);
        input.clear();
    }
}

fn process_input(input: &str) {
    match alltools::base64::base64_to_bytes(input) {
        Some(input_bytes) => println!("{}", alltools::hex::bytes_to_hex(&input_bytes)),
        None => eprintln!("Bad input: \"{}\" is not valid base64", input)
    }
}
