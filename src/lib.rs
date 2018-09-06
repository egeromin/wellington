extern crate itertools;
extern crate pulldown_cmark;
extern crate regex;

mod sidenote_error;
mod parser;
mod sidenotes;

pub use parser::html_from_markdown;

