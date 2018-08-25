extern crate getopts;
extern crate wellington;

use getopts::Options;
use std::env;
use std::fs;
// use std::io::prelude::*;

use wellington::html_from_markdown;

fn brief(program: &str, opts: Options) -> String {
    let brie = format!("Usage: {} [options]", program);
    opts.usage(&brie)
}


fn main() {
    let args :Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.reqopt("i", "input", "Input markdown file", "INPUT");
    opts.reqopt("o", "output", "Output html file", "HTML");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}\n{}", e.to_string(), brief(&args[0], opts));
            std::process::exit(1);
        }
    };

    let input_filename = matches.opt_str("input").expect("Error with filename");
    let input = fs::read_to_string(input_filename).expect("Error reading input file");

    let output = match html_from_markdown(&input) {
        Ok(ht) => ht,
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };

    let output_filename = matches.opt_str("output").expect("Error with filename");
    fs::write(output_filename, output).expect("Error writing result");
}

