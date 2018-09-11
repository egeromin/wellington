use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use parser::html_from_markdown;


struct IndexedBlogPost {
    path: PathBuf,
    last_updated: SystemTime,
    first_published: SystemTime
} // TODO: change these timestamps to dates?


struct BlogPost {
    path: PathBuf,
    last_updated: SystemTime
}


impl IndexedBlogPost {

    fn get_input_output_filename(&self) -> Result<(String, String), BlogError> {
        let mut input_path = self.path.clone();
        input_path.push("index.md");
        let input_filename = match input_path.to_str() {
            Some(s) => s.to_string(),
            None => {
                return Err(BlogError::CantReadDir(self.path.clone()));
            }
        };
        input_path.pop(); input_path.push("index.html");
        let output_filename = match input_path.to_str() {
            Some(s) => s.to_string(),
            None => {
                return Err(BlogError::CantReadDir(self.path.clone()));
            }
        };
        Ok((input_filename, output_filename))
    }

    fn convert(&self) -> Result<(), BlogError> {
        let (input_filename, output_filename) = self.get_input_output_filename()?;
        if let Ok(input) = fs::read_to_string(&input_filename) {
            let output = match html_from_markdown(&input, true) {
                Ok(ht) => ht,
                Err(err) => {
                    return Err(BlogError::ConvertError(format!("{}", err)));
                }
            };
            match fs::write(&output_filename, output) {
                Err(_) => {
                    return Err(BlogError::WriteError(output_filename));
                },
                _ => ()
            }
        } else {
            return Err(BlogError::ReadError(input_filename))
        }
        Ok(())
    }
}


struct Blog {
    index: Vec<IndexedBlogPost>,
    path: PathBuf
}


#[derive(Debug)]
enum BlogError {
    CantReadDir(PathBuf),
    ReadError(String),
    ConvertError(String),
    WriteError(String)
}


impl Blog {

    fn new(path: PathBuf) -> Blog { Blog{path, index: vec![]} }

    fn list_entries(path: &PathBuf) -> Result<Vec<BlogPost>, BlogError> {
        let mut posts: Vec<BlogPost> = vec![];
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(last_updated) = metadata.modified() {
                            posts.push(BlogPost{path: entry.path(), 
                                                last_updated});
                        } else {
                            return Err(BlogError::CantReadDir(path.clone()))
                        }
                    } else {
                        return Err(BlogError::CantReadDir(path.clone()))
                    }
                } else {
                    return Err(BlogError::CantReadDir(path.clone()))
                }
            }
        } else {
            return Err(BlogError::CantReadDir(path.clone()))
        }
        Ok(posts)
    }

    /// filter out those subdirectories which contain "index.md" 
    /// or "index.html"
    fn list_posts(&self) -> Result<Vec<BlogPost>, BlogError> {
        let subdirs = Blog::list_entries(&self.path)?;
        let mut posts: Vec<BlogPost> = vec![];
        for subdir in subdirs {
            let contents = Blog::list_entries(&subdir.path)?;
            for post in contents {
                if let Some(file_name) = post.path.file_name() {
                    if let Some(file_name) = file_name.to_str() {
                        if "index.md" == file_name {
                            posts.push(subdir);
                            break;
                        }
                    }
                } else {
                    return Err(BlogError::CantReadDir(self.path.clone()))
                }
            }
        }
        Ok(posts)
    }

    fn update(&mut self, dry_run: bool) -> Result<usize, BlogError> {
        let all_posts = self.list_posts()?;
        let mut num_updated: usize = 0;
        for post in all_posts {
            match self.index.binary_search_by(|b| b.path.cmp(&post.path)) {
                Ok(i) => {
                    if (self.index[i].last_updated < post.last_updated) {
                        self.index[i].last_updated = post.last_updated;
                        if ! dry_run {
                            self.index[i].convert()?;
                        }
                        num_updated += 1;
                    }
                },
                Err(i) => {
                    let new_post = IndexedBlogPost{
                        path: post.path, last_updated: post.last_updated,
                        first_published: post.last_updated
                    };
                    if ! dry_run {
                        new_post.convert()?;
                    }
                    self.index.insert(i, new_post);
                    num_updated += 1;
                }
            }
        }
        Ok(num_updated)
    }

    // fn from_string(&mut self, text: String) -> Result<(), BlogError> {}
}


// impl Display for Blog {
// 
// }


// TODO:
// * serialize and deserialize index
// * read and list valid directories
// * 
// I must know which directories there are, and which are valid
// I must also know which ones are in my index
// I must infer which ones are new and which ones were updated
// I must compile each new or updated article
// I must return an error if any of the articles failed to convert, and abort in that case


#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use super::{Blog, IndexedBlogPost};

    static POSTS: &[&'static str] = &["irkutsk", "krasnoyarsk", "yekaterinburg"];

    fn create_fake_dirs(name: &str) -> PathBuf {
        let mut temp_dir = env::temp_dir();
        temp_dir.push(name);
        match fs::create_dir(&temp_dir) {
            Err(_) => {
                println!("Failed to create dir! Temp_dir: {}", 
                         temp_dir.to_str().unwrap());
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

        file_path.push("ghosttown");
        fs::create_dir(&file_path).expect("vla3");

        temp_dir
    }

    fn cleanup(temp_dir: &PathBuf) {
        fs::remove_dir_all(temp_dir).expect("bla2");
        // TODO: doesn't work on mac?? Check on linux. 
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
            IndexedBlogPost{
                path: posts[0].path.clone(),
                last_updated: posts[0].last_updated,
                first_published: posts[0].last_updated,
            },
            IndexedBlogPost{
                path: posts[1].path.clone(),
                last_updated: UNIX_EPOCH,
                first_published: UNIX_EPOCH,
            }
        ];
        let num_updated;
        {
            num_updated = blog.update(true).expect("can't update");
        }
        cleanup(&blog.path);
        assert_eq!(num_updated, posts.len() - 1);
    }

    #[test]
    fn can_compute_input_output_filename() {
        let blogpost = IndexedBlogPost{
            path: PathBuf::from("/example"),
            last_updated: SystemTime::now(),
            first_published: SystemTime::now()
        };
        let (i, o) = blogpost.get_input_output_filename().expect("Should get io!");
        assert_eq!(i, "/example/index.md");
        assert_eq!(o, "/example/index.html");
    }
}
