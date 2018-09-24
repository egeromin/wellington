use std::fmt;
use std::path::PathBuf;
use std::fs;
use std::io;
use handlebars::{Handlebars, no_escape};
use serde::Serialize;

use toc::{IndexedBlogPost, Blog};


const TOC_TEMPLATE: &[u8]  = include_bytes!("../templates/toc.html");
const POST_TEMPLATE: &[u8]  = include_bytes!("../templates/post.html");


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
    post: Handlebars,
    index: Handlebars
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

    fn make_template(template_str: &str, path: &str) -> Result<Handlebars, TemplateError> {
        let mut template = Handlebars::new();
        match template.register_template_string("t1", template_str) {
            Ok(_) => Ok(template),
            Err(_) => Err(TemplateError{
                msg: format!("Template at {} has bad syntax", path),
                kind: ErrorKind::InvalidSyntax
            })
        }
    }

    fn validate<T>(template: &Handlebars, test: &T, path: &str) -> Result<(), TemplateError>
        where T: Serialize {
        match template.render("t1", test) {
            Ok(_) => Ok(()),
            Err(_) => Err(TemplateError{
                msg: format!("Template at {} didn't pass validation. Are all of the fields correct?", path),
                kind: ErrorKind::FailValidation
            })
        }
    }

    fn make_and_validate<T>(path: &str, fallback: &[u8], test: &T) -> Result<Handlebars, TemplateError> 
        where T: Serialize {
        let template_str = AllTemplates::read_template(path, fallback)?;
        let template = AllTemplates::make_template(&template_str, path)?;
        AllTemplates::validate::<T>(&template, test, path)?;
        Ok(template)
    }

    pub fn make_from_paths(post_path: &str, index_path: &str) -> Result<Self, TemplateError> {
        let test_post = IndexedBlogPost::example();
        let test_index = Blog::new(PathBuf::from("/example"));
        let mut index_template = AllTemplates::make_and_validate::<Blog>(&index_path, 
                                                                     TOC_TEMPLATE, 
                                                                     &test_index)?;
        index_template.register_escape_fn(no_escape);

        Ok(AllTemplates{
            post: AllTemplates::make_and_validate::<IndexedBlogPost>(&post_path, 
                                                                     POST_TEMPLATE, 
                                                                     &test_post)?,
            index: index_template
        })
    }

    pub fn make() -> Result<Self, TemplateError> {
        let post_path = ".template_post.html";
        let index_path = ".template_index.html";
        AllTemplates::make_from_paths(&post_path, &index_path)
    }
}


#[cfg(test)]
mod tests {
    use super::AllTemplates;

    fn make_without_error() {
        assert!(AllTemplates::make().is_ok());
    }
}

