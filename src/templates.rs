use std::fmt;
use std::fs;
use std::io;
use handlebars::{Handlebars, no_escape};
use serde::Serialize;


pub const TOC_TEMPLATE: &[u8]  = include_bytes!("../templates/toc.html");
pub const POST_TEMPLATE: &[u8]  = include_bytes!("../templates/post.html");

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

    pub fn make_template(template_str: &str, path: &str) -> Result<Handlebars, TemplateError> {
        let mut template = Handlebars::new();
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

        Ok(AllTemplates{
            post: post_template,
            index: AllTemplates::make(&index_path, TOC_TEMPLATE)?
        })
    }

    pub fn new() -> Result<Self, TemplateError> {
        AllTemplates::make_from_paths(None, None)
    }
}

impl From<(Handlebars, Handlebars)> for AllTemplates {
    fn from(templates: (Handlebars, Handlebars)) -> Self {
        AllTemplates{post: templates.0, index: templates.1}
    }
}


#[cfg(test)]
mod tests {
    use toc::Blog;
    use parser::PostData;
    use std::path::PathBuf;

    use super::AllTemplates;

    #[test]
    fn make_without_error() {
        let templates = AllTemplates::new().expect("Can't get templates");
        let article = "some article";
        let test_post = PostData::new(&article);
        let test_index = Blog::new(PathBuf::from("/example")).unwrap();
        assert!(templates.validate_both::<PostData<'static>, Blog>(
                &test_post, &test_index).is_ok());
    }
}

