extern crate getopts;
extern crate wellington;

use std::env;
use std::fs;
use std::path::PathBuf;

use wellington::{html_from_markdown, Blog};


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
    fs::write(output_filename, output.html).expect("Error writing result");
}


fn sync(blog_path: &str) {
    let mut blog = Blog::new(PathBuf::from(blog_path));
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
        if args.len() < 3 {
            eprintln!("Please give me 1 argument: the path to the blog");
            std::process::exit(1);
        } 
        sync(&args[2]);
    } else {
        eprintln!("I don't recognise this command :(");
    }
}
