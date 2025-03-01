use atty::Stream;
use std::env;
use std::io;
use std::process;
use std::string::String;

use colored::*;
use getopts::Options;

// Inspired by https://github.com/busyloop/lolcat

pub type Vec3 = [f32; 3];

fn hsv_to_rgb(hsv: Vec3) -> Vec3 {
    let hue_prime = hsv[0] * 6.0;
    let saturation = hsv[1];
    let value = hsv[2];
    let chroma = saturation * value;
    let m = value - chroma;

    let c = chroma + m;
    let x = c * (1.0 - ((hue_prime % 2.0) - 1.0).abs()) + m;
    let o = 0.0 + m;

    let hue_class = hue_prime.floor() as isize;
    match hue_class {
        0 => [c, x, o],
        1 => [x, c, o],
        2 => [o, c, x],
        3 => [o, x, c],
        4 => [x, o, c],
        5 => [c, o, x],
        _ => {
            panic!("Invalid hue class {}", hue_class);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts_spec = Options::new();
    opts_spec.optflag("h", "help", "print this help menu");
    let opts = match opts_spec.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("Try '{} --help' for more information", program);
            process::exit(1);
        }
    };

    if opts.opt_present("h") {
        println!("Turn any command output into a rainbow by piping its output");
        println!();

        let brief = format!("Usage: {} [-a]", program);
        print!("{}", opts_spec.usage(&brief));
        return;
    }

    let is_stdin_tty = atty::is(Stream::Stdin);
    if is_stdin_tty {
        println!("Interactive input isn't supported, please pipe in another command");
        process::exit(1);
    }

    let mut output_colour = [0.0, 0.8, 1.0];
    let mut input = String::new();
    loop {
        input.clear();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Error: {}", e);
            break;
        }

        if input.is_empty() {
            break;
        }
        let mut line_c = output_colour;
        for chr in input.chars() {
            let rgb = hsv_to_rgb(line_c);
            print!(
                "{}",
                String::from(chr).truecolor(
                    (rgb[0] * 255.0) as u8,
                    (rgb[1] * 255.0) as u8,
                    (rgb[2] * 255.0) as u8
                )
            );
            line_c[0] = (line_c[0] + 0.01) % 1.0;
        }

        output_colour[0] = (output_colour[0] + 0.01) % 1.0;
    }
}
