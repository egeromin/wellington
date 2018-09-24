extern crate getopts;
extern crate wellington;

use std::env;
use std::fs;
use std::path::PathBuf;

use wellington::{html_from_markdown, Blog};


fn usage(program: &str) -> String {
    format!(r#"Usage: {} [command]"

Where command is one of:
    convert <input> <output>    Convert input markdown file to output html file
    init                        Initialise the current directory as a blog
    sync                        Sync all blog posts in the current blog directory, 
                                refreshing the table of contents
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
    fs::write(output_filename, output.html).expect("Error writing result");
}


fn current_dir() -> PathBuf {
    match env::current_dir() {
        Ok(p) => p,
        _ => {
            println!("Couldn't access the current directory. Do you have sufficient permissions?");
            std::process::exit(1);
        }
    }
}


fn init() {
    let blog = Blog::new(current_dir());
    match blog.init() {
        Ok(_) => println!("Initialised new empty blog"),
        _ => println!("Couldn't initialise blog. Do you write permission?")
    }
}


fn sync() {
    let mut blog = Blog::new(current_dir());
    match blog.sync() {
        Ok(i) => println!("Updated {} posts", i),
        Err(err) => {
            println!("Couldn't sync: {}", err);
            std::process::exit(1);
        }
    }
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
        sync();
    } else if command == "init" {
        init();
    } else {
        eprintln!("I don't recognise this command :(");
    }
}
