extern crate getopts;
extern crate wellington;

use std::env;
use std::fs;

use wellington::html_from_markdown;


fn usage(program: &str) -> String {
    format!(r#"Usage: {} [command]"

Where command is one of:
    convert <input> <output>    Convert input markdown to output html
    sync <blogdir>              Sync all blog posts in blogdir, refreshing the 
                                table of contents
"#, program)
}

fn convert(input_filename: &str, output_filename: &str) {
    let input = fs::read_to_string(input_filename).expect("Error reading input file");
    let output = match html_from_markdown(&input, true) {
        Ok(ht) => ht,
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };
    fs::write(output_filename, output).expect("Error writing result");
}


fn main() {
    let args :Vec<String> = env::args().collect();

    if args.len() == 1 {
        eprintln!("{}", usage(&args[0]));
        std::process::exit(1);
    }
    
    let command = &args[1];
    if command == "convert" {
        if args.len() < 4 {
            eprintln!("Please give me 2 arguments: input and output");
            std::process::exit(1);
        } 
        convert(&args[2], &args[3]);
    } else if command == "sync" {
    } else {
        eprintln!("I don't recognise this command :(");
    }
}
