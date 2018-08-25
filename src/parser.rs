use pulldown_cmark::{Event, Tag, html, Parser};
use sidenotes::{compile_sidenotes, SidenoteError};
use std::vec::IntoIter;


struct SidenoteParser<'a> {
    parser: Parser<'a>,
    in_code_block: bool
}


/// The main wrapper for pulldown_cmark events.
/// Wraps the events to account for sidenotes and custom
/// styling and html classes required for tufte-css.
/// Currently:
///
/// * checks text events for sidenotes, 
/// * checks code block tags, and remembers if we're in a 
/// code block, so as not to parse for sidenotes in that case
/// * returns the other events unchanged.
impl<'a> SidenoteParser<'a> {
    fn new(parser: Parser) -> SidenoteParser {
        SidenoteParser{parser, in_code_block: false}
    }
}


impl<'a> Iterator for SidenoteParser<'a> {
    type Item = Result<IntoIter<Event<'a>>, SidenoteError>;

    fn next(&mut self) -> Option<Result<IntoIter<Event<'a>>, SidenoteError>> {
        let next_event = self.parser.next();
        match next_event {
            Some(event) => {
                match event {
                    Event::Text(text) => {
                        if self.in_code_block {
                            Some(Ok(vec![Event::Text(text)].into_iter()))
                        } else {
                            Some(compile_sidenotes(text))
                        }
                    }
                    Event::Start(tag) => match tag {
                        Tag::Code => {
                            self.in_code_block = true;
                            Some(Ok(vec![Event::Start(Tag::Code)].into_iter()))
                        },
                        Tag::CodeBlock(lang) => {
                            self.in_code_block = true;
                            Some(Ok(vec![Event::Start(Tag::CodeBlock(lang))].into_iter()))
                        }
                        _ => {
                            Some(Ok(vec![Event::Start(tag)].into_iter()))
                        }
                    },
                    Event::End(tag) => match tag {
                        Tag::Code => {
                            self.in_code_block = false;
                            Some(Ok(vec![Event::End(Tag::Code)].into_iter()))
                        }
                        Tag::CodeBlock(lang) => {
                            self.in_code_block = true;
                            Some(Ok(vec![Event::End(Tag::CodeBlock(lang))].into_iter()))
                        }
                        _ => {
                            Some(Ok(vec![Event::End(tag)].into_iter()))
                        }
                    },
                    _ => Some(Ok(vec![event].into_iter()))
                }
            },
            None => None
        }
    }
} // TODO: refactor to avoid all this repetition


/// Main function to convert markdown to html
pub fn html_from_markdown(md: &str) -> Result<String, SidenoteError> {
    let parser = SidenoteParser::new(Parser::new(md));

    let mut html_buf = String::new();
    for events in parser {
        html::push_html(&mut html_buf, events?);
    }
    Ok(html_buf)
}


#[cfg(test)]
mod tests {
    use super::html_from_markdown;

    #[test]
    fn check_catch_sidenote_errors() {
        let markdown_str = r#"
hello
=====

Here is some text with { badly formatted {sidenotes}.

* alpha
* beta

"#;

        let html_buf = html_from_markdown(markdown_str);
        assert!(html_buf.is_err());
    }

    #[test]
    fn check_multi_line_sidenotes_no_good() {
        let markdown_str = r#"
hello
=====

Here is some text with { a sidenote

spanning multiple lines, which is no good

}

* alpha
* beta

"#;

        let html_buf = html_from_markdown(markdown_str);
        assert!(html_buf.is_err());
    }

    #[test]
    fn check_to_markdown() {
        let markdown_str = r#"
hello
=====

Here is some text with {sidenotes}.

* alpha
* beta

And also some `inline_code` as well as

```
code_with{
    curly_braces();
}
```

"#;
        let html_buf = html_from_markdown(markdown_str).expect("Shouldn't fail!");

        assert_eq!(
            html_buf,
            r#"<h1>hello</h1>
<p>Here is some text with <span>sidenotes</span>.</p>
<ul>
<li>alpha</li>
<li>beta</li>
</ul>
<p>And also some <code>inline_code</code> as well as</p>
<pre><code>code_with{
    curly_braces();
}
</code></pre>
"#
        );
    }

}
