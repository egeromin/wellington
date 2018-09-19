use pulldown_cmark::{Event, Tag, html, Parser};
use std::borrow::Cow;

use sidenote_error::SidenoteError;


static FOOTER: &str = r#"
</section></article></body></html>
"#;

pub struct SidenoteParser<'a> {
    parser: Parser<'a>,
    pub in_code_block: bool,
    pub in_sidenote_block: bool,
    pub remaining_text: String,
    pub title: &'a mut Option<String>,
    pub in_title: bool,
    pub in_image: bool,
    pub remaining_events: Vec<Event<'a>>
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
    pub fn new(parser: Parser<'a>, title: &'a mut Option<String>) -> SidenoteParser<'a> {
        SidenoteParser{
            parser,
            title,
            in_code_block: false,
            in_sidenote_block: false,
            remaining_text: String::from(""),
            in_title: false,
            in_image: false,
            remaining_events: vec![]
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
                Event::InlineHtml(Cow::from("<br /><br />\n"))
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

    fn start_codeblock() -> Event<'a> {
        Event::InlineHtml(Cow::from("<pre><code class=\"code\">"))
    }

    fn parse_next_event(&mut self, event: Event<'a>) -> 
        Result<Event<'a>, SidenoteError> {
        match event {
            Event::Text(text) => Ok(self.parse_text_block(text)),
            Event::Start(tag) => match tag {
                Tag::Code => self.parse_code_tag(true, Event::Start(Tag::Code)),
                Tag::CodeBlock(_lang) => self.parse_code_tag(true, 
                    SidenoteParser::start_codeblock()),
                Tag::Paragraph => Ok(self.parse_paragraph_tag(true)),
                Tag::Header(1) => {
                    self.in_title = true;
                    Ok(Event::Start(Tag::Header(1)))
                },
                Tag::Image(url, title) => {
                    self.in_image = true;
                    Ok(Event::Start(Tag::Image(url, title)))
                },
                _ => Ok(Event::Start(tag))
            },
            Event::End(tag) => match tag {
                Tag::Code => self.parse_code_tag(false, Event::End(Tag::Code)),
                Tag::CodeBlock(lang) => self.parse_code_tag(false, 
                    Event::End(Tag::CodeBlock(lang))),
                Tag::Paragraph => Ok(self.parse_paragraph_tag(false)),
                Tag::Header(1) => Ok(Event::InlineHtml(Cow::from("</h1><section>"))),
                Tag::Image(url, title) => {
                    self.in_image = false;
                    Ok(Event::End(Tag::Image(url, title)))
                },
                _ => Ok(Event::End(tag))
            },
            _ => Ok(event)
        }
    }
} 


impl<'a> Iterator for SidenoteParser<'a> {
    type Item = Result<Event<'a>, SidenoteError>;

    fn next(&mut self) -> Option<Result<Event<'a>, SidenoteError>> {
        match self.remaining_events.pop() {
            Some(e) => Some(Ok(e)),
            None => {
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
    }
} 


pub struct ParsedMarkdown {
    pub html: String,
    pub title: Option<String>
}


/// Main function to convert markdown to html
pub fn html_from_markdown(md: &str, write_header: bool) -> Result<ParsedMarkdown, SidenoteError> {
    let mut title: Option<String> = None;
    let mut html_buf = String::new();
    {
        let parser = SidenoteParser::new(Parser::new(md), &mut title);
        for event in parser {
            html::push_html(&mut html_buf, vec![event?].into_iter());
        }
    }

    let title = match title {
        Some(t) => match t.len() {
            0 => None,  // don't allow empty titles
            _ => Some(t)
        },
        None => None
    };

    if write_header {
        let title_with_default = match &title {
            Some(t) => t.clone(),
            None => "No Title".to_string()
        };
        let header = format!(r#"
<html>
<head>
<link rel="stylesheet" href="tufte.css" />
<title>{}</title>
</head>
<body>
<article>
"#, title_with_default);

        // let header = format!(HEADER, title);
        html_buf = format!("{}{}{}", header, html_buf, FOOTER);
    }
    Ok(ParsedMarkdown{html: html_buf, title})

}


#[cfg(test)]
mod tests {
    use pulldown_cmark::Parser;
    use super::{html_from_markdown, SidenoteParser};

    #[test]
    fn check_catch_sidenote_errors() {
        let markdown_str = r#"
hello
=====

Here is some text with { badly formatted {sidenotes}.

* alpha
* beta

"#;

        let html_buf = html_from_markdown(markdown_str, false);
        assert!(html_buf.is_err());
    }

    #[test]
    fn check_fail_nested_code_sidenote() {

        let markdown_str = r#"
hello
=====

Here is some text with { a sidenote `and code nested`
    }"#;

        assert!(html_from_markdown(markdown_str, false).is_err());
    }

    #[test]
    fn check_nested_sidenote_code() {
        let markdown_str = r#"
hello
=====

Here is some text with ` code {and curly braces nested`
"#;
        assert_eq!(html_from_markdown(markdown_str, false).expect("Should succeed").html,
            r#"<h1>hello</h1><section>
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

        let html_buf = html_from_markdown(markdown_str, false).expect("Should succeed");
        assert_eq!(
            html_buf.html,
            r#"<h1>hello</h1><section>
<p>Here is some text with <label class="sidenote-number"></label><span class="sidenote"> a sidenote<br /><br />
spanning multiple lines, which is also supported<br /><br />
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
        let html_buf = html_from_markdown(markdown_str, false).expect("Shouldn't fail!");

        assert_eq!(
            html_buf.html,
            r#"<h1>hello</h1><section>
<p>Here is some text with <label class="sidenote-number"></label><span class="sidenote">sidenotes</span>.</p>
<ul>
<li>alpha</li>
<li>beta</li>
</ul>
<p>And also some <code>inline_code</code> as well as</p>
<pre><code class="code">code_with{
    curly_braces();
}
</code></pre>
"#
        );
    }

    #[test]
    fn can_get_title() {
        let md = r#"
hello
=====

Here is some text with {sidenotes}.
"#;
        let mut title: Option<String> = None;
        {
            let parser = SidenoteParser::new(Parser::new(md), &mut title);
            for _ in parser {}
        }
        assert_eq!(title.expect("Should work!"), "hello")
    }

    #[test]
    fn can_parse_image() {
        let md = r#"
hello
=====

![image](https://image)
"#;
        assert_eq!(html_from_markdown(md, false).expect("should work!").html, r#"<h1>hello</h1><section>
<p><img src="https://image" alt="" /><br /><span class="image-caption">image</span></p>
"#);
    }
}
