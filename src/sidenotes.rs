use itertools::Itertools;
use pulldown_cmark::Event;
use regex::Regex;
use std::borrow::Cow;
use std::vec::IntoIter;
use std::cmp::{max, min};
use std::fmt;


/// sidenote errors. The possible errors are:
/// 
/// * not matched, e.g. "bla { bla" or "bla } {bla}"
/// * nested, e.g. "{ bla { }"
#[derive(Debug)]
pub enum SidenoteError{
    NotMatched{
        context: String
    },
    Nested {
        first: String,
        second: String
    }
}


impl fmt::Display for SidenoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SidenoteError::NotMatched{context} => {
                write!(f, "Error: a sidenote delimiter was not matched: ..{}..", context)
            },
            SidenoteError::Nested{first, second} => {
            write!(f, "Error: encountered a nested sidenote: ..{}.. is enclosed by ..{}..",
                   first, second)
            }
        }
    }
}


/// check that sidenotes are properly formatted.
/// There shouldn't be any nested curly braces,
/// and all braces should match.
///
/// # Examples
///
/// * this is correct: "this {has sidenotes} {more than once}"
/// * this is not: "this {has {nested sidenotes} that don't } match}"
fn check_sidenote_formatting(text: &str) -> Result<(), SidenoteError> {
    let mut start: Option<usize> = None;
    for (i, c) in text.chars().enumerate() {
        match c {
            '{' => {
                match start {
                    Some(j) => {
                        return Err(SidenoteError::Nested{
                            first: String::from(&text[max(j-5,0)..min(j+5,text.len())]),
                            second: String::from(&text[max(i-5,0)..min(i+5,text.len())]),
                        });
                    },
                    None => {
                        start = Some(i);
                    }
                };
            },
            '}' => {
                match start {
                    None => {
                        return Err(SidenoteError::NotMatched{
                            context: String::from(&text[max(i-5,0)..min(i+5,text.len())]),
                        });
                    }
                    _ => {
                        start = None;
                    }
                };
            },
            _ => {}
        };
    }
    match start {
        Some(j) => Err(SidenoteError::NotMatched{
            context: String::from(&text[max(j-5,0)..min(j+5,text.len())]),
        }),
        None => Ok(())
    }
}

/// compile sidenotes
/// if correctly formatted, then replace '{' and '}' with tags
/// otherwise, return text as is
pub fn compile_sidenotes(text: Cow<str>) -> Result<IntoIter<Event>, 
    SidenoteError> {
    check_sidenote_formatting(&text)?;

    let re = Regex::new(r"[{}]").unwrap();

    let text_events = re.split(&text)
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
    Ok(all_events.into_iter())

    // TODO: is it possible to modify this function and eliminate unnecessary copies?
    // I don't see how, because the events own the string slices they refer to
    // Check the mechanics of Cow and smart pointers, because often the data of the 
    // Cow is borrowed.
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
