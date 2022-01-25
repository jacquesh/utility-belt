use std::cmp;
use std::env;
use std::process;
use std::string::String;

use colored::*;
use getopts::Options;
use terminal_size;

fn codepoint_char(codepoint: usize) -> Option<char>
{
    match codepoint {
        0..=32 => Some(' '), // Control characters
        127 => Some(' '), // Delete
        _ => std::char::from_u32(codepoint as u32)
    }
}

fn codepoint_name(codepoint: usize) -> Option<&'static str>
{
    match codepoint {
        0 => Some("Null"),
        1 => Some("Start of heading"),
        2 => Some("Start of text"),
        3 => Some("End of text"),
        4 => Some("End of transmission"),
        5 => Some("Enquiry"),
        6 => Some("Acknowledge"),
        7 => Some("Bell"),
        8 => Some("Backspace"),
        9 => Some("Horizontal tab"),
        10 => Some("Line feed"),
        11 => Some("Vertical tab"),
        12 => Some("Form feed"),
        13 => Some("Carriage return"),
        14 => Some("Shift out"),
        15 => Some("Shift in"),
        16 => Some("Data link escape"),
        17 => Some("Device control 1"),
        18 => Some("Device control 2"),
        19 => Some("Device control 3"),
        20 => Some("Device control 4"),
        21 => Some("Negative acknowledge"),
        22 => Some("Synchronous idle"),
        23 => Some("End of transmission block"),
        24 => Some("Cancel"),
        25 => Some("End of medium"),
        26 => Some("Substitute"),
        27 => Some("Escape"),
        28 => Some("File separator"),
        29 => Some("Group separator"),
        30 => Some("Record separator"),
        31 => Some("Unit separator"),
        32 => Some("Space"),
        33 => Some("Exclamation mark"),
        34 => Some("Quotation mark"),
        35 => Some("Hash"),
        36 => Some("Dollar sign"),
        37 => Some("Percent sign"),
        38 => Some("Ampersand"),
        39 => Some("Apostrophe"),
        40 => Some("Left Parenthesis"),
        41 => Some("Right Parenthesis"),
        42 => Some("Asterisk"),
        43 => Some("Plus sign"),
        44 => Some("Comma"),
        45 => Some("Hyphen"),
        46 => Some("Dot/Period"),
        47 => Some("Slash / Solidus"),
        48 => Some("Zero"),
        49 => Some("One"),
        50 => Some("Two"),
        51 => Some("Three"),
        52 => Some("Four"),
        53 => Some("Five"),
        54 => Some("Six"),
        55 => Some("Seven"),
        56 => Some("Eight"),
        57 => Some("Nine"),
        58 => Some("Colon"),
        59 => Some("Semi-colon"),
        60 => Some("Less than"),
        61 => Some("Equals"),
        62 => Some("Greater than"),
        63 => Some("Question mark"),
        64 => Some("At sign"),
        65 => Some("Capital A"),
        66 => Some("Capital B"),
        67 => Some("Capital C"),
        68 => Some("Capital D"),
        69 => Some("Capital E"),
        70 => Some("Capital F"),
        71 => Some("Capital G"),
        72 => Some("Capital H"),
        73 => Some("Capital I"),
        74 => Some("Capital J"),
        75 => Some("Capital K"),
        76 => Some("Capital L"),
        77 => Some("Capital M"),
        78 => Some("Capital N"),
        79 => Some("Capital O"),
        80 => Some("Capital P"),
        81 => Some("Capital Q"),
        82 => Some("Capital R"),
        83 => Some("Capital S"),
        84 => Some("Capital T"),
        85 => Some("Capital U"),
        86 => Some("Capital V"),
        87 => Some("Capital W"),
        88 => Some("Capital X"),
        89 => Some("Capital Y"),
        90 => Some("Capital Z"),
        91 => Some("Left square bracket"),
        92 => Some("Backslash"),
        93 => Some("Right square bracket"),
        94 => Some("Caret / Circumflex"),
        95 => Some("Underscore"),
        96 => Some("Grave / Backtick"),
        97 => Some("A"),
        98 => Some("B"),
        99 => Some("C"),
        100 => Some("D"),
        101 => Some("E"),
        102 => Some("F"),
        103 => Some("G"),
        104 => Some("H"),
        105 => Some("I"),
        106 => Some("J"),
        107 => Some("K"),
        108 => Some("L"),
        109 => Some("M"),
        110 => Some("N"),
        111 => Some("O"),
        112 => Some("P"),
        113 => Some("Q"),
        114 => Some("R"),
        115 => Some("S"),
        116 => Some("T"),
        117 => Some("U"),
        118 => Some("V"),
        119 => Some("W"),
        120 => Some("X"),
        121 => Some("Y"),
        122 => Some("Z"),
        123 => Some("Left curly brace"),
        124 => Some("Pipe"),
        125 => Some("Right curly brace"),
        126 => Some("Tilde"),
        127 => Some("Delete"),
        _ => Some("<Unknown>"),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts_spec = Options::new();
    opts_spec.optflag("h", "help", "print this help menu");
    opts_spec.optflag("a", "all", "include additional characters & columns in the output");
    let opts = match opts_spec.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e.to_string());
            eprintln!("Try '{} --help' for more information", program);
            process::exit(1);
        }
    };

    if opts.opt_present("h") {
        println!("Print a table of ASCII character information");
        println!("");

        let brief = format!("Usage: {} [-a]", program);
        print!("{}", opts_spec.usage(&brief));
        return;
    }

    let show_all = opts.opt_present("a");
    // TODO: Also binary? Octal?

    let first_codepoint = match show_all {
        true => 0, // Null
        false => 32, // Space, the first non-control character
    };
    let last_codepoint = 127;

    let width: usize = 10;
    let name_width = 3*width;
    let total_column_width: usize = 6*width + 3;
    let columns: usize = if let Some((terminal_size::Width(w), _)) = terminal_size::terminal_size() {
        cmp::max(1, (w as usize)/total_column_width)
    } else {
        2
    };
    let rows = (last_codepoint - first_codepoint + 1) / columns;

    for column_index in 0..columns {
        if column_index > 0 {
            print!("|");
        }
        print!("{1:^0$}|{2:^0$}|{3:^0$}|{5:^4$}", width, "Decimal", "Hex", "Char", name_width, "Name");
    }
    println!();
    println!("{1:=<0$}", columns*total_column_width + (columns-1), "");

    let highlight_char: Option<char> = match opts.free.first() {
        Some(arg) => {
            if arg.is_empty() {
                None
            } else {
                arg.chars().next()
            }
        },
        None => None
    };

    for row_index in 0..rows {
        for column_index in 0..columns {
            if column_index > 0 {
                print!("{}", "|".white());
            }
            let codepoint = first_codepoint + row_index + rows * column_index;
            let chr = codepoint_char(codepoint).unwrap();
            let codepoint_colour = match highlight_char {
                Some(c) => match chr {
                    _ if chr == c => Color::Black,
                    _ => Color::White,
                },
                None => Color::White,
            };

            let entry = format!("{1} {2} {3:^0$} {5:^4$}",
                   width,
                   format!("{:^1$}", codepoint, width).color(codepoint_colour),
                   format!("{:^#1$x}", codepoint, width).color(codepoint_colour),
                   String::from(chr).red(),
                   name_width,
                   codepoint_name(codepoint).unwrap().blue());

            match highlight_char {
                Some(c) => match chr {
                    _ if chr == c => print!("{}", entry.on_white()),
                    _ => print!("{}", entry),
                },
                None => print!("{}", entry),
            };
        }
        println!();
    }
}
