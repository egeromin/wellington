extern crate getopts;
extern crate wellington;

use std::env;
use std::fs;
use std::path::PathBuf;
use getopts::Options;

use wellington::{html_from_markdown, Blog, PostData, IndexedBlogPost};
use wellington::templates::{AllTemplates, POST_TEMPLATE};
use wellington::rss::CoreData;


fn usage(program: &str, init_opts: &str) -> String {
    format!(r#"Usage: {} [command]

Where command is one of:
    convert <input> <output>    Convert input markdown file to output html file

    sync [-f]                   Sync all blog posts in the current blog directory, 
                                refreshing the table of contents. 
                                
                                If no posts were updated, the index and posts 
                                won't be re-rendered, unless you use the -f flag. 
                                Use this flag when changing templates, for example.

    init <options>              Initialise the current directory as a blog. You must 
                                provide the following options:{}
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
    let output = match html_from_markdown(&input, "".to_string()) {
        Ok(ht) => ht,
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };
    let mut bp = IndexedBlogPost::example();
    bp.set_title(&output.title);

    let data = PostData::from((output.html.as_str(), &mut bp, ""));
    let rendered = match data.render(&template) {
        Ok(ht) => ht,
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };
    fs::write(output_filename, rendered).expect("Error writing result");
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


fn init(core_data: CoreData, post: Option<String>, index: Option<String>) {
    let mut blog = match Blog::new(current_dir()) {
        Ok(b) => b,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };
    match blog.init(core_data, post, index) {
        Ok(_) => println!("Initialised new empty blog"),
        Err(e)  => println!("{}", e)
    }
}


fn sync(force: bool) {
    let mut blog = match Blog::new(current_dir()) {
        Ok(b) => b,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };
    match blog.sync(force) {
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
    init_opts.reqopt("t", "title", "Blog title: give your blog a name!", "BLOG_TITLE");
    init_opts.reqopt("u", "home_url", "The home URL where your blog will be hosted,
    for example https://myblog.com", "HOME_URL");
    init_opts.reqopt("d", "desc", "Describe your blog", "BLOG_DESCRIPTION");
    init_opts.reqopt("a", "author", "Who are you? Please give your name. This will be make public in the RSS feed", "BLOG_AUTHOR");
    init_opts.optopt("p", "post", "(Optional) Template for rendering individual posts", 
                     "POST_TEMPLATE");
    init_opts.optopt("i", "index", "(Optional) Template for rendering the table of contents", 
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
        if args.len() == 3 && args[2] == "-f" {
            sync(true);
        } else {
            sync(false);
        }
    } else if command == "init" {
        let matches = match init_opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error: {}", e.to_string());
                std::process::exit(1);
            }
        };
        let core_data = match CoreData::new(
            &matches.opt_str("title").unwrap(),
            &matches.opt_str("home_url").unwrap(),
            &matches.opt_str("desc").unwrap(),
            &matches.opt_str("author").unwrap()) {
            Ok(d) => d,
            Err(err) => {
                println!("{}", err);
                std::process::exit(1);
            }
        };
        init(core_data, matches.opt_str("post"), matches.opt_str("index"));
    } else {
        eprintln!("I don't recognise this command :(");
    }
}

