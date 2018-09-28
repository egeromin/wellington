use std::fmt;
use std::fs;
use std::io;
use std::str::from_utf8;
use std::time::SystemTime;
use handlebars::{Handlebars, no_escape};
use handlebars::{RenderContext, Helper, Context, HelperResult, Output, RenderError};
use chrono::{DateTime, Utc};

use serde::{Serialize, Deserialize};

use rss::RssData;


pub const TOC_TEMPLATE: &[u8]  = include_bytes!("../templates/toc.html");
pub const POST_TEMPLATE: &[u8]  = include_bytes!("../templates/post.html");
pub const RSS_TEMPLATE: &[u8]  = include_bytes!("../templates/rss.xml");

pub const PATH_POST: &str = ".post_template.html";
pub const PATH_INDEX: &str = ".index_template.html";

#[derive(Debug, Copy, Clone)]
pub enum ErrorKind {
    CantRead,
    FailValidation,
    InvalidSyntax
}


#[derive(Debug)]
pub struct TemplateError {
    msg: String,
    kind: ErrorKind
}


impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad template: {}", &self.msg)
    }
}


impl TemplateError {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}


pub struct AllTemplates {
    pub post: Handlebars,
    pub index: Handlebars,
    pub rss: Handlebars,
}



impl AllTemplates {

    fn read_template(path: &str, fallback: &[u8]) -> Result<String, TemplateError> {
        match fs::read_to_string(path) {
            Ok(s) => Ok(s),
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => Ok(String::from_utf8_lossy(fallback).to_string()),
                _ => Err(TemplateError{
                    msg: format!("Couldn't read file {}", path),
                    kind: ErrorKind::CantRead
                })
            }
        }
    }

    fn as_date(h: &Helper, 
               _: &Handlebars, 
               _: &Context, 
               _: &mut RenderContext, 
               out: &mut Output) -> HelperResult {

        let param = match h.param(0) {
            Some(p) => p,
            _ => {
                return Err(RenderError::new(
                    "You must provide a parameter to the as-date template helper"));
            }
        };

        let unpack_error = RenderError::new(
            "Couldn't unpack value passed to as-date. Are you sure it's a SystemTime object?"
            );

        let stime = match SystemTime::deserialize(param.value()) {
            Ok(t) => t,
            _ => { return Err(unpack_error); }
        };

        let format_str = match h.param(1) {
            None => "%d %B %Y at %H:%M UTC", // display
            _ => "%a, %d %b %Y %T GMT", // RSS
        };

        let datetime = DateTime::<Utc>::from(stime);
        match out.write(&format!("{}", datetime.format(format_str))) {
            Ok(_) => Ok(()),
            _ => Err(RenderError::new(
                "Coultn't write"))
        }
    }

    pub fn make_template(template_str: &str, path: &str) -> Result<Handlebars, TemplateError> {
        let mut template = Handlebars::new();
        template.register_helper("as-date", Box::new(AllTemplates::as_date));
        match template.register_template_string("t1", template_str) {
            Ok(_) => Ok(template),
            Err(_) => Err(TemplateError{
                msg: format!("Template at {} has bad syntax", path),
                kind: ErrorKind::InvalidSyntax
            })
        }
    }

    pub fn validate<T>(template: &Handlebars, test: &T, path: &str) -> Result<(), TemplateError>
        where T: Serialize {
        match template.render("t1", test) {
            Ok(_) => Ok(()),
            Err(_) => Err(TemplateError{
                msg: format!("Template at {} didn't pass validation. Are all of the fields correct?", path),
                kind: ErrorKind::FailValidation
            })
        }
    }

    pub fn validate_both<T, U>(&self, test_post: &T, test_index: &U) -> Result<(), TemplateError>
        where T: Serialize, U: Serialize {
        AllTemplates::validate::<T>(&self.post, test_post, &PATH_POST)?;
        AllTemplates::validate::<U>(&self.index, test_index, &PATH_INDEX)?;
        Ok(())
    }

    fn make(path: &str, fallback: &[u8]) -> Result<Handlebars, TemplateError> {
        let template_str = AllTemplates::read_template(path, fallback)?;
        let template = AllTemplates::make_template(&template_str, path)?;
        Ok(template)
    }

    pub fn make_from_paths(path_post: Option<String>, 
                           path_index: Option<String>) -> Result<Self, TemplateError> {
        let post_path = path_post.unwrap_or(PATH_POST.to_string());
        let index_path = path_index.unwrap_or(PATH_INDEX.to_string());
        let mut post_template = AllTemplates::make(&post_path, POST_TEMPLATE)?;
        post_template.register_escape_fn(no_escape);

        let rss = match AllTemplates::make_template(match from_utf8(RSS_TEMPLATE) {
            Ok(s) => s,
            Err(e) => {
                return Err(TemplateError{
                msg: format!("Couldn't read rss template: {}", e),
                kind: ErrorKind::InvalidSyntax
            });}
        }, "rss-path") {
            Ok(h) => h,
            Err(e) => {
                return Err(TemplateError{
                msg: format!("Couldn't read rss template: {}", e),
                kind: ErrorKind::InvalidSyntax
            });}
        };

        let rss_test = RssData::example();

        AllTemplates::validate::<RssData>(&rss, &rss_test, "rss-path")?;

        Ok(AllTemplates{
            post: post_template,
            index: AllTemplates::make(&index_path, TOC_TEMPLATE)?,
            rss
        })
    }

    pub fn new() -> Result<Self, TemplateError> {
        AllTemplates::make_from_paths(None, None)
    }
}

impl From<(Handlebars, Handlebars, Handlebars)> for AllTemplates {
    fn from(templates: (Handlebars, Handlebars, Handlebars)) -> Self {
        AllTemplates{
            post: templates.0, 
            index: templates.1,
            rss: templates.2
        }
    }
}


#[cfg(test)]
mod tests {
    use toc::{Blog, IndexedBlogPost};
    use parser::PostData;
    use std::path::PathBuf;

    use super::AllTemplates;

    #[test]
    fn make_without_error() {
        let templates = AllTemplates::new().expect("Can't get templates");
        let article = "some article";
        let test_post = PostData::new(&article);
        let mut test_index = Blog::new(PathBuf::from("/example")).unwrap();
        test_index.push(IndexedBlogPost::example());
        assert!(templates.validate_both::<PostData<'static>, Blog>(
                &test_post, &test_index).is_ok());
    }
}

