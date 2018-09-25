extern crate itertools;
extern crate pulldown_cmark;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate csv;
extern crate handlebars;
extern crate chrono;

mod sidenote_error;
mod parser;
mod sidenotes;
mod toc;
pub mod templates;

pub use parser::{html_from_markdown, ParsedMarkdown, PostData};
pub use toc::{Blog, IndexedBlogPost};

