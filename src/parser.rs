use pulldown_cmark::{Event, Tag, html, Parser};
use std::borrow::Cow;

use sidenote_error::SidenoteError;

pub struct SidenoteParser<'a> {
    parser: Parser<'a>,
    pub in_code_block: bool,
    pub in_sidenote_block: bool,
    pub remaining_text: String 
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
    pub fn new(parser: Parser) -> SidenoteParser {
        SidenoteParser{
            parser, 
            in_code_block: false,
            in_sidenote_block: false,
            remaining_text: String::from("")
        }
    }

    fn parse_code_tag(&mut self, start: bool, on_success_return: Event<'a>) -> 
        Result<Event<'a>, SidenoteError> {
        if self.in_sidenote_block {
            Err(SidenoteError::NotMatched)
        } else {
            self.in_code_block = start;
            Ok(on_success_return)
        }
    }

    fn parse_paragraph_tag(&mut self, start: bool) -> 
        Event<'a> {
        if self.in_sidenote_block {
            if start {
                Event::InlineHtml(Cow::from("<br />\n"))
            } else { // create empty event
                Event::Text(Cow::from(""))
                // TODO: would be cleaner to instead skip this and go straight
                // to the next event and invoke self.next()
                // but to do this need to change all return types
            }
        } else {
            if start {
                Event::Start(Tag::Paragraph)
            } else {
                Event::End(Tag::Paragraph)
            }
        }
    }

    fn parse_next_event(&mut self, event: Event<'a>) -> 
        Result<Event<'a>, SidenoteError> {
        match event {
            Event::Text(text) => Ok(self.parse_text_block(text)),
            Event::Start(tag) => match tag {
                Tag::Code => self.parse_code_tag(true, Event::Start(Tag::Code)),
                Tag::CodeBlock(lang) => self.parse_code_tag(true, 
                    Event::Start(Tag::CodeBlock(lang))),
                Tag::Paragraph => Ok(self.parse_paragraph_tag(true)),
                _ => Ok(Event::Start(tag))
            },
            Event::End(tag) => match tag {
                Tag::Code => self.parse_code_tag(false, Event::End(Tag::Code)),
                Tag::CodeBlock(lang) => self.parse_code_tag(false, 
                    Event::End(Tag::CodeBlock(lang))),
                Tag::Paragraph => Ok(self.parse_paragraph_tag(false)),
                _ => Ok(Event::End(tag))
            },
            _ => Ok(event)
        }
    }
} 


impl<'a> Iterator for SidenoteParser<'a> {
    type Item = Result<Event<'a>, SidenoteError>;

    fn next(&mut self) -> Option<Result<Event<'a>, SidenoteError>> {
        if self.remaining_text.len() > 0 {
            Some(self.parse_remaining_text())
        } else {
            let next_event = self.parser.next();
            match next_event {
                Some(event) => Some(self.parse_next_event(event)),
                None => None
            }
        }
    }
} 


/// Main function to convert markdown to html
pub fn html_from_markdown(md: &str) -> Result<String, SidenoteError> {
    let parser = SidenoteParser::new(Parser::new(md));

    let mut html_buf = String::new();
    for event in parser {
        html::push_html(&mut html_buf, vec![event?].into_iter());
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
    fn check_fail_nested_code_sidenote() {

        let markdown_str = r#"
hello
=====

Here is some text with { a sidenote `and code nested`
    }"#;

        assert!(html_from_markdown(markdown_str).is_err());
    }

    #[test]
    fn check_nested_sidenote_code() {
        let markdown_str = r#"
hello
=====

Here is some text with ` code {and curly braces nested`
"#;
        assert_eq!(html_from_markdown(markdown_str).expect("Should succeed"),
            r#"<h1>hello</h1>
<p>Here is some text with <code>code {and curly braces nested</code></p>
"#);
    }


    #[test]
    fn check_multi_line_sidenotes() {
        let markdown_str = r#"
hello
=====

Here is some text with { a sidenote

spanning multiple lines, which is also supported

}.

* alpha
* beta

"#;

        let html_buf = html_from_markdown(markdown_str).expect("Should succeed");
        assert_eq!(
            html_buf,
            r#"<h1>hello</h1>
<p>Here is some text with <span> a sidenote<br />
spanning multiple lines, which is also supported<br />
</span>.</p>
<ul>
<li>alpha</li>
<li>beta</li>
</ul>
"#
        );
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
