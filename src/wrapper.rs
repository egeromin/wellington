use pulldown_cmark::Event;
use sidenotes::compile_sidenotes;
use std::vec::IntoIter;

/// The main wrapper for pulldown_cmark events.
/// Wraps the events to account for sidenotes and custom
/// styling and html classes required for tufte-css.
/// Currently:
///
/// * checks text events for sidenotes, leaving the
/// others unchanged.
fn wellington_wrapper(event: Event) -> IntoIter<Event> {
    match event {
        Event::Text(text) => compile_sidenotes(&text),
        _ => vec![event].into_iter(),
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::{html, Parser};

    use super::wellington_wrapper;

    #[test]
    fn check_to_markdown() {
        let markdown_str = r#"
hello
=====

Here is some text with {sidenotes}.

* alpha
* beta
"#;
        let parser = Parser::new(markdown_str);

        let mut html_buf = String::new();
        html::push_html(&mut html_buf, parser.flat_map(wellington_wrapper));

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
