use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use csv::{WriterBuilder, ReaderBuilder};

use parser::html_from_markdown;


#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
struct IndexedBlogPost {
    path: PathBuf,
    last_updated: SystemTime,
    first_published: SystemTime,
    #[serde(skip)]
    checked: bool,
    title: Option<String>
} 


#[derive(Debug)]
struct BlogPost {
    path: PathBuf,
    last_updated: SystemTime
}


impl From<BlogPost> for IndexedBlogPost {
    fn from(post: BlogPost) -> Self {
        IndexedBlogPost {
            path: post.path,
            last_updated: post.last_updated,
            first_published: post.last_updated,
            checked: false,
            title: None
        }
    }
}


impl IndexedBlogPost {

    fn get_filename_path(&self, file: &str) -> Result<String, BlogError> {
        let mut input_path = self.path.clone();
        input_path.push(file);
        match input_path.to_str() {
            Some(s) => Ok(s.to_string()),
            None => 
                Err(BlogError::CantReadDir(self.path.clone(),
                    format!("can't get full path for {}", file)))
        }
    }

    fn convert(&mut self) -> Result<(), BlogError> {
        let input_filename = self.get_filename_path("index.md")?;
        let output_filename = self.get_filename_path("index.html")?;
        if let Ok(input) = fs::read_to_string(&input_filename) {
            let output = match html_from_markdown(&input, true) {
                Ok(ht) => ht,
                Err(err) => {
                    return Err(BlogError::ConvertError(format!("{}", err)));
                }
            };
            match fs::write(&output_filename, output.html) {
                Err(_) => {
                    return Err(BlogError::WriteError(output_filename));
                },
                _ => ()
            };
            self.title = output.title;
        } else {
            return Err(BlogError::ReadError(input_filename))
        }
        Ok(())
    }

}


pub struct Blog {
    index: Vec<IndexedBlogPost>,
    path: PathBuf
}


#[derive(Debug)]
pub enum BlogError {
    CantReadDir(PathBuf, String),
    ReadError(String),
    ConvertError(String),
    WriteError(String),
    ReadIndexError(String),
    WriteIndexError(String)
}

impl fmt::Display for BlogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlogError::CantReadDir(path, err) => write!(f, "Can't read the blog specified at {}: {}", 
                                                   match path.to_str() {Some(s) => s, None => ""}, err),
            BlogError::ReadError(err) => write!(f, "Encountered a read error: {}", err),
            BlogError::WriteError(err) => write!(f, "Encountered a write error: {}", err),
            BlogError::WriteIndexError(err) => write!(f, "Encountered an error while writing the index: {}", err),
            BlogError::ConvertError(err) => write!(f, "Encountered an error while converting: {}", err),
            BlogError::ReadIndexError(err) => write!(f, "Encountered an error while reading the index: {}", err),
        }
    }
}


impl Blog {

    pub fn new(path: PathBuf) -> Blog { Blog{path, index: vec![]} }

    fn get_index_path(&self) -> PathBuf {
        let mut index_path = self.path.clone(); index_path.push(".index.csv");
        index_path
    }

    fn load(&mut self) -> Result<(), BlogError> {
        let reader = match ReaderBuilder::new()
            .has_headers(false)
            .from_path(self.get_index_path()) {
            Ok(w) => w,
            _ => {return Ok(());} 
            // if opening the file causes error, assume it's because the file
            // doesn't exist, and ignore
        };

        for post in reader.into_deserialize() {
            self.index.push(match post {
                Ok(p) => p,
                Err(e) => {
                    return Err(BlogError::ReadIndexError(
                        format!("Could not parse index file: {:?}", e.kind())));
                }
            });
        }
        Ok(())      
    }

    pub fn sync(&mut self) -> Result<usize, BlogError> {
        self.load()?;
        let num_updated = self.update(false)?;
        self.persist()?;
        Ok(num_updated)
    }

    fn persist(&self) -> Result<(), BlogError> {
        let mut writer = match WriterBuilder::new()
            .has_headers(false)
            .from_path(self.get_index_path()) {
            Ok(w) => w,
            _ => {
                return Err(BlogError::WriteIndexError(format!(
                    "Failed to open index file {:?}", &self.path)));
            }
        };
        for post in self.index.iter() {
            writer.serialize(post).expect("Can't serialize");
            // match writer.serialize(post) {
            //     Ok(_) => (),
            //     _ => {
            //         return Err(BlogError::WriteIndexError(format!(
            //             "Couldn't serialize {:?}", post)));
            //     }
            // };
        }
        Ok(())
    }

    fn list_entries(path: &PathBuf, only_dir: bool) -> Result<Vec<BlogPost>, BlogError> {
        let mut posts: Vec<BlogPost> = vec![];

        let entries = match fs::read_dir(path) {
            Ok(s) => s,
            Err(e) => {
                return Err(BlogError::CantReadDir(path.clone(),
                    format!("failed to list directory entries: {:?}", e.kind())))
            }
        };
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() || (! only_dir ) {
                        if let Ok(last_updated) = metadata.modified() {
                            posts.push(BlogPost{path: entry.path(), 
                                                last_updated});
                        } else {
                            return Err(BlogError::CantReadDir(path.clone(),
                                "failed to get time of last update".to_string()))
                        }
                    }
                } else {
                    return Err(BlogError::CantReadDir(path.clone(),
                        "failed to read metadata".to_string()))
                }
            } else {
                return Err(BlogError::CantReadDir(path.clone(),
                    "failed to read directory entry".to_string()))
            }
        }
        Ok(posts)
    }

    /// filter out those subdirectories which contain "index.md" 
    /// or "index.html"
    fn list_posts(&self) -> Result<Vec<BlogPost>, BlogError> {
        let subdirs = Blog::list_entries(&self.path, true)?;
        let mut posts: Vec<BlogPost> = vec![];
        for subdir in subdirs {
            let contents = Blog::list_entries(&subdir.path, false)?;
            for post in contents {
                if let Some(file_name) = post.path.file_name() {
                    if let Some(file_name) = file_name.to_str() {
                        if "index.md" == file_name {
                            posts.push(subdir);
                            break;
                        }
                    }
                } else {
                    return Err(BlogError::CantReadDir(self.path.clone(),
                        "can't extract file name for pathbuf".to_string()))
                }
            }
        }
        Ok(posts)
    }

    // perform a linear search in index
    // TODO: replace with a more efficient method, when there are many posts
    fn find_in_index(&self, post: &BlogPost) -> Option<usize> {
        for (i, b) in self.index.iter().enumerate() {
            if b.path == post.path {
                return Some(i);
            }
        }
        None
    }

    fn update(&mut self, dry_run: bool) -> Result<usize, BlogError> {
        let all_posts = self.list_posts()?;
        let mut num_updated: usize = 0;
        for post in all_posts {
            if let Some(i) = self.find_in_index(&post) {
                self.index[i].checked = true;
                if self.index[i].last_updated < post.last_updated {
                    self.index[i].last_updated = post.last_updated;
                    if ! dry_run {
                        self.index[i].convert()?;
                    }
                    num_updated += 1;
                }
            } else {
                let now = SystemTime::now();
                let mut new_post = IndexedBlogPost{
                    path: post.path, last_updated: now,
                    first_published: now, checked: true,
                    title: None
                };
                if ! dry_run {
                    new_post.convert()?;
                }
                self.index.push(new_post);
                num_updated += 1;
            }
        }
        let old_index = self.index.clone(); 
        // TODO: avoid this unnecessary clone

        self.index = vec![];
        for post in old_index.into_iter() {
            if post.checked {
                self.index.push(post);
            } else {
                num_updated += 1;
            }
        }
        Ok(num_updated)
    }
}


#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use super::{Blog, IndexedBlogPost, BlogPost};

    static POSTS: &[&'static str] = &["irkutsk", "krasnoyarsk", "yekaterinburg"];

    fn create_fake_dirs(name: &str) -> PathBuf {
        let mut temp_dir = env::temp_dir();
        temp_dir.push(name);
        match fs::create_dir(&temp_dir) {
            Err(_) => {
                println!("Failed to create dir! Temp_dir: {}", 
                         temp_dir.to_str().unwrap());
                assert!(false);
            },
            _ => ()
        };

        let mut file_path = temp_dir.clone();
        for post in POSTS.iter() {
            file_path.push(post);
            fs::create_dir(&file_path).expect("Should be able to create subdir!");
            file_path.push("index.md");
            fs::File::create(&file_path).expect("Should be able to create file!");
            file_path.pop(); 
            file_path.pop();
        }

        file_path.push("garbage.txt");
        fs::File::create(&file_path).expect("Should be able to create file!");
        file_path.pop();

        file_path.push("ghosttown");
        fs::create_dir(&file_path).expect("vla3");

        temp_dir
    }

    fn cleanup(temp_dir: &PathBuf) {
        fs::remove_dir_all(temp_dir).expect("bla2");
    }

    #[test]
    fn can_list_dirs() {
       let blog = Blog::new(create_fake_dirs("blog"));
        let posts = blog.list_posts().expect("bla");
        let post_names = posts.iter()
            .map(|x| x.path
                 .file_name().expect("2")
                 .to_str()
                 .expect("3")
                 .to_string())
            .collect::<Vec<String>>();
        cleanup(&blog.path);
        assert_eq!(post_names, POSTS);
    }

    #[test]
    fn can_update() {
        let mut blog = Blog::new(create_fake_dirs("blog2"));
        let posts = blog.list_posts().expect("can't list posts");
        blog.index = vec![
            IndexedBlogPost::from(BlogPost{
                path: posts[1].path.clone(),
                last_updated: UNIX_EPOCH,
            }),
            IndexedBlogPost::from(BlogPost{
                path: posts[0].path.clone(),
                last_updated: posts[0].last_updated,
            }),
        ];
        let num_updated;
        {
            num_updated = blog.update(true).expect("can't update");
        }
        cleanup(&blog.path);
        assert_eq!(num_updated, posts.len() - 1);
        let expected_new_index_paths = vec![
            posts[1].path.clone(),
            posts[0].path.clone(),
            posts[2].path.clone(),
        ];
        let new_index_paths = blog.index.clone()
            .into_iter()
            .map(|x| x.path)
            .collect::<Vec<PathBuf>>();
        assert_eq!(new_index_paths, expected_new_index_paths);
    }

    #[test]
    fn can_compute_input_output_filename() {
        let blogpost = IndexedBlogPost::from(BlogPost{
            path: PathBuf::from("/example"),
            last_updated: SystemTime::now(),
        });
        let i = blogpost.get_filename_path("index.md").expect("Should get input!");
        let o = blogpost.get_filename_path("index.html").expect("Should get output!");
        assert_eq!(i, "/example/index.md");
        assert_eq!(o, "/example/index.html");
    }

    #[test]
    fn write_read_index() {
        let blog_path = create_fake_dirs("blog");
        let mut blog = Blog::new(blog_path.clone());
        let posts = blog.list_posts().expect("can't list posts");
        blog.index = vec![
            IndexedBlogPost::from(BlogPost{
                path: posts[0].path.clone(),
                last_updated: SystemTime::now(),
            }),
            IndexedBlogPost::from(BlogPost{
                path: posts[1].path.clone(),
                last_updated: UNIX_EPOCH,
            })
        ];
        blog.index[1].title = Some("Some title with \"quotes".to_string());

        blog.persist().expect("can't persist");
        let mut blog2 = Blog::new(blog_path.clone());
        blog2.load().expect("can't load");
        cleanup(&blog_path);
        assert_eq!(blog.index, blog2.index);
    }
}
