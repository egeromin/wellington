extern crate itertools;
extern crate pulldown_cmark;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate csv;

mod sidenote_error;
mod parser;
mod sidenotes;
mod toc;

pub use parser::{html_from_markdown, ParsedMarkdown};
pub use toc::Blog;

