use std::fmt;
use std::fs;
use std::time::SystemTime;

use url::Url;
use serde_json;
use url_serde;

use toc::IndexedBlogPost;


const CORE_DATA_PATH: &str = ".meta.json";


#[derive(Serialize)]
struct RssPost {
    title: Option<String>,
    first_published: SystemTime,
    author: String,
    #[serde(with = "url_serde")]
    link: Url
}


impl RssPost {
    fn example() -> Self {
        RssPost{
            title: None,
            first_published: SystemTime::now(),
            author: "Me".to_string(),
            link: Url::parse("https://example.com").unwrap()
        }
    }
}


#[derive(Serialize)]
pub struct RssData {
    core_data: CoreData,
    posts: Vec<RssPost>
}


impl RssData {
    pub fn example() -> Self {
        RssData{
            core_data: CoreData::new("bla", "https://bla.com", "2", "3").unwrap(),
            posts: vec![RssPost::example()]
        }
    }

    pub fn new(core_data: CoreData) -> Self {
        RssData{core_data, posts: vec![]}
    }

    pub fn push_posts(&mut self, posts: &[IndexedBlogPost]) {
        for (i, post) in posts.iter().rev().enumerate() {
            let mut link = self.core_data.home.clone();
            link.set_path(&post.post_url);
            self.posts.push(RssPost{
                link, author: self.core_data.author.clone(),
                first_published: post.first_published,
                title: post.title.clone()
            });
            if i == 9 {
                break;
            }
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct CoreData {
    title: String,
    #[serde(with = "url_serde")]
    home: Url,
    description: String,
    author: String
}


#[derive(Debug, Copy, Clone)]
pub enum ErrorKind {
    CantRead,
    BadSyntax,
    WriteError
}


#[derive(Debug)]
pub struct RSSError {
    msg: String,
    kind: ErrorKind
}


impl fmt::Display for RSSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad template: {}", &self.msg)
    }
}


impl RSSError {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}


impl CoreData {

    fn strip_path(mut u: Url) -> Url {
        u.set_path("");
        u
    }
    
    pub fn new(title: &str, home_s: &str, 
           description: &str, author: &str) -> Result<Self, RSSError> {
        match Url::parse(home_s) {
            Ok(home) => if home.path().len() <= 1 {
                Ok(CoreData{
                    title: title.to_string(),
                    description: description.to_string(),
                    author: author.to_string(),
                    home
                })
            } else {
                Err(RSSError{
                    msg: format!("Please provide a URL *without path*, for example {}", 
                                 CoreData::strip_path(home)),
                    kind: ErrorKind::BadSyntax
                })
            },
            _ => Err(RSSError{
                msg: "Provided an invalid home address".to_string(),
                kind: ErrorKind::BadSyntax
            })
        }
    }

    pub fn load() -> Result<Self, RSSError> {
        let data_json = match fs::read_to_string(CORE_DATA_PATH) {
            Ok(j) => j,
            _ => { return Err(RSSError{
                msg: "Couldn't read core data file in this directory. Are you in the right place?".to_string(),
                kind: ErrorKind::CantRead,
            });}
        };
        match serde_json::from_str(&data_json) {
            Ok(c) => Ok(c),
            _ => Err(RSSError{
                msg: "Couldn't parse core data file. Run `init` again to fix and create a new one.".to_string(),
                kind: ErrorKind::BadSyntax
            })
        }
    }

    pub fn save(&self) -> Result<(), RSSError> {
        let data_json = match serde_json::to_string(&self) {
            Ok(s) => s,
            Err(e) => { return Err(RSSError{
                msg: format!("Couldn't serialize core data: {}", e),
                kind: ErrorKind::WriteError
            })}
        };
        match fs::write(CORE_DATA_PATH, data_json) {
            Ok(_) => Ok(()),
            Err(e) => { return Err(RSSError{
                msg: format!("Couldn't write to file: {}", e),
                kind: ErrorKind::WriteError
            })}
        }

    }
}


#[cfg(test)]
mod test {
    use super::CoreData;
    use url::Url;

    #[test]
    fn can_set() {
        assert!(CoreData::new("a", "b", "c", "d").is_err());
        assert!(CoreData::new("a", "https://example.com/some-path", "c", "d").is_err());
        assert_eq!(CoreData::new("a", "https://example.com/", "c", "d")
                   .expect("Can't create new coredata").home,
                   Url::parse("https://example.com/").unwrap());
    }
}
