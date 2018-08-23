use pulldown_cmark::{Event, html, Parser};
use sidenotes::compile_sidenotes;
use std::vec::IntoIter;

/// The main wrapper for pulldown_cmark events.
/// Wraps the events to account for sidenotes and custom
/// styling and html classes required for tufte-css.
/// Currently:
///
/// * checks text events for sidenotes, leaving the
/// others unchanged.
fn wrapper(event: Event) -> IntoIter<Event> {
    match event {
        Event::Text(text) => compile_sidenotes(&text),
        _ => vec![event].into_iter(),
    }
}


/// Main function to convert markdown to html
pub fn html_from_markdown(md: &str) -> String {
    let parser = Parser::new(md);

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser.flat_map(wrapper));
    html_buf
}


#[cfg(test)]
mod tests {
    use super::html_from_md;

    #[test]
    fn check_to_markdown() {
        let markdown_str = r#"
hello
=====

Here is some text with {sidenotes}.

* alpha
* beta
"#;
        let html_buf = html_from_md(markdown_str);

        assert_eq!(
            html_buf,
            r#"<h1>hello</h1>
<p>Here is some text with <span>sidenotes</span>.</p>
<ul>
<li>alpha</li>
<li>beta</li>
</ul>
"#
        );
    }

}
