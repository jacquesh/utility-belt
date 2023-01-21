use atty::Stream;
use colored::Colorize;
use getopts::Options;
use std::env;
use std::io;
use std::path::Path;
use std::process;
use std::str;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [QUERY]\n\nQUERY will be fuzzy-matched against file names, requiring only that they contain the same characters in the same sequence", program);
    print!("{}", opts.usage(&brief));
}

fn find_ignore_ascii_case(haystack: &str, needle: &char) -> Option<usize> {
    for (idx, haystack_char) in haystack.chars().enumerate() {
        if haystack_char.eq_ignore_ascii_case(needle) {
            return Some(idx);
        }
    }

    return None;
}

fn include_filename_matches(query: &str, filename: &str) -> bool {
    if filename.len() < query.len() {
        return false;
    }

    let mut current_index = 0;
    for c in query.chars() {
        match find_ignore_ascii_case(&filename[current_index..], &c) {
            Some(idx) => current_index += idx,
            None => return false,
        }
    }
    return true;
}

fn format_matched_filename(query: &str, filename: &str) -> String {
    let mut result: String = String::new();
    let mut current_index = 0;
    for c in query.chars() {
        if current_index >= filename.len() {
            break;
        }
        match find_ignore_ascii_case(&filename[current_index..], &c) {
            Some(idx) => {
                result.push_str(&filename[current_index..current_index + idx]);

                // Pull the character from `filename` instead of `c` so that we get the correct casing.
                // e.g since the search is case-insensitive, if I search for "Windows.h" and get
                // "windows.h" then we want to print "windows.h" (from the haystack, not the needle).
                let coloured_char = &filename[current_index+idx..current_index+idx+1];
                result.push_str(&coloured_char.cyan().to_string());

                current_index += idx + 1;
            }
            None => break,
        }
    }
    if current_index < filename.len() {
        result.push_str(&filename[current_index..]);
    }

    result
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts_spec = Options::new();
    opts_spec.optflag("h", "help", "print this help menu");
    let opts = match opts_spec.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e.to_string());
            eprintln!("Try '{} --help' for more information", program);
            process::exit(1);
        }
    };

    if opts.opt_present("h") {
        println!("Query the compiler included-files output to find the path by which a particular header is included.");
        println!("");

        print_usage(program, opts_spec);
        return;
    }

    let is_stdin_tty = atty::is(Stream::Stdin);
    if is_stdin_tty {
        eprintln!("{} stdin is a TTY but is not expected to be. The expected usage is that you pipe in the output from the compiler directly.",
            "WARNING:".red().bold());
        process::exit(1);
    }

    if opts.free.len() > 1 {
        eprintln!("{} Multiple inputs were provided but are not supported, only the first query will be considered",
            "WARNING:".red().bold());
    }
    let query = match opts.free.first() {
        Some(q) => q,
        None => {
            eprintln!("No query input provided");
            eprintln!("");
            print_usage(program, opts_spec);
            process::exit(1);
        }
    };

    let mut include_seen = false;
    let mut last_seen: Vec<String> = Vec::new();
    let mut input = String::new();
    loop {
        input.clear();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Error: {}", e.to_string());
            break;
        }

        let input = input.trim();
        if input.len() == 0 {
            break;
        }

        let include_prefix = "Note: including file:";
        if input.starts_with(include_prefix) {
            include_seen = true;
            let input = &input[include_prefix.len()..];
            let indent = match input.find(|c: char| c != ' ') {
                Some(idx) => idx - 1,
                None => continue,
            };

            if indent >= last_seen.len() {
                last_seen.push(input.trim().into());
            } else {
                // TODO: Is this different to assigning a whole new string?
                //       I want to avoid re-allocating, does rust have move assignment operators?
                last_seen[indent].clear();
                last_seen[indent] += input.trim();
            }

            let included_path = Path::new(input.trim());
            let included_name = included_path.file_name().unwrap().to_str().unwrap();
            if include_filename_matches(query, included_name) {
                println!(
                    "\nFound matching include: {}",
                    format_matched_filename(query, included_name)
                );
                // TODO: Print the initial .cpp file name/path?
                for (idx, path_str) in last_seen[..indent + 1].iter().enumerate() {
                    println!(" {:indent$}-> {}", "", path_str, indent = 4 * idx);
                }
            }
        } else {
            //println!("Non-include line: {}", input);
        }
    }

    if !include_seen {
        eprintln!("No include paths found in the input, did you forget to add the relevant compile flag to show included files?");
        eprintln!("Note that currently only output from MSVC is supported.");
        process::exit(1);
    }
}
