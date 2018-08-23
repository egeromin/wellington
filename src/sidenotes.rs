use itertools::Itertools;
use pulldown_cmark::Event;
use regex::Regex;
use std::borrow::Cow;
use std::vec::IntoIter;

/// check that sidenotes are properly formatted.
/// There shouldn't be any nested curly braces,
/// and all braces should match.
///
/// # Examples
///
/// * this is correct: "this {has sidenotes} {more than once}"
/// * this is not: "this {has {nested sidenotes} that don't } match}"
fn are_sidenotes_formatted(text: &str) -> bool {
    let mut counter: i32 = 0;
    for c in text.chars() {
        if c == '{' {
            if counter > 0 {
                return false;
            }
            counter += 1;
        } else if c == '}' {
            if counter < 1 {
                return false;
            }
            counter -= 1;
        }
    }
    counter == 0
}

/// compile sidenotes
/// if correctly formatted, then replace '{' and '}' with tags
/// otherwise, return text as is
pub fn compile_sidenotes<'a, 'b: 'a>(text: &'a str) -> IntoIter<Event<'b>> {
    if !are_sidenotes_formatted(text) {
        return vec![Event::Text(Cow::from(text.to_string()))].into_iter();
    }

    let re = Regex::new(r"[{}]").unwrap();

    let text_events = re.split(text)
        .map(String::from)
        .map(Cow::from)
        .map(Event::Text);

    // need to collect and into_iter above because `Split` object
    // doesn't implement Clone

    let start_stop_tags = vec!["<span>", "</span>"];
    let start_stop_events = start_stop_tags
        .into_iter()
        .map(Cow::from)
        .map(Event::InlineHtml)
        .cycle();

    let mut all_events = text_events
        .interleave_shortest(start_stop_events)
        .collect::<Vec<Event>>();

    all_events.pop(); // remove last item
    all_events.into_iter()

    // TODO: modify this function and eliminate unnecessary copies by
    // taking as argument a `Cow` directly?
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    #[test]
    fn can_use_regex() {
        let re = Regex::new(r"[{}]").unwrap();
        let expected = vec!["a", "separated", "string"];
        let actual: Vec<&str> = re.split("a{separated}string").collect();
        assert_eq!(expected, actual);
    }
}
