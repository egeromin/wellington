extern crate getopts;
extern crate wellington;

use std::env;
use std::fs;
use std::path::PathBuf;
use getopts::Options;

use wellington::{html_from_markdown, Blog, PostData};
use wellington::templates::{AllTemplates, POST_TEMPLATE};


fn usage(program: &str, init_opts: &str) -> String {
    format!(r#"Usage: {} [command]"

Where command is one of:
    convert <input> <output>    Convert input markdown file to output html file

    sync                        Sync all blog posts in the current blog directory, 
                                refreshing the table of contents

    init <options>              Initialise the current directory as a blog. There
                                are the following options:{}
"#, program, init_opts)
}


fn convert(input_filename: &str, output_filename: &str) {
    let input = fs::read_to_string(input_filename).expect("Error reading input file");
    let post_template = String::from_utf8_lossy(POST_TEMPLATE);
    let template = match AllTemplates::make_template(&post_template, "default-template") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    let article = "some article";
    match AllTemplates::validate::<PostData<'static>>(&template, 
                                                      &PostData::new(&article),
                                                      "default-path") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    let output = match html_from_markdown(&input, Some(&template), "".to_string()) {
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


fn init(post: Option<String>, index: Option<String>) {
    let mut blog = match Blog::new(current_dir()) {
        Ok(b) => b,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };
    match blog.init(post, index) {
        Ok(_) => println!("Initialised new empty blog"),
        Err(e)  => println!("{}", e)
    }
}


fn sync() {
    let mut blog = match Blog::new(current_dir()) {
        Ok(b) => b,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };
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
    let mut init_opts = Options::new();
    init_opts.optopt("p", "post", "Template for rendering individual posts", 
                     "POST_TEMPLATE");
    init_opts.optopt("i", "index", "Template for rendering the table of contents", 
                     "INDEX_TEMPLATE");

    if args.len() == 1 {
        eprintln!("{}", usage(&args[0], &init_opts.usage("")));
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
        let matches = match init_opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error: {}", e.to_string());
                std::process::exit(1);
            }
        };
        init(matches.opt_str("post"), matches.opt_str("index"));
    } else {
        eprintln!("I don't recognise this command :(");
    }
}

