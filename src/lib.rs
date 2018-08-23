extern crate itertools;
extern crate pulldown_cmark;
extern crate regex;

mod sidenotes;
mod parser;

pub use parser::html_from_markdown;

