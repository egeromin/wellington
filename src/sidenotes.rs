use itertools::Itertools;
use pulldown_cmark::Event;
use regex::Regex;
use std::borrow::Cow;

use parser::SidenoteParser;
use sidenote_error::SidenoteError;


/// compile sidenotes
/// if correctly formatted, then replace '{' and '}' with tags
/// otherwise, return text as is
impl<'a> SidenoteParser<'a> { 

    fn parse_first_sidenote<'b>(&'b mut self, text: Cow<'a, str>) -> Event<'a> {
        let re = Regex::new(r"[{}]").unwrap();

        match re.find(&text) {
            Some(m) => {
                assert_eq!(m.start() + 1, m.end());
                let first = text[..m.start()].to_string();
                self.remaining_text = text[m.start()..].to_string();
                Event::Text(Cow::from(first))
            },
            None => {
                self.remaining_text = "".to_string();
                Event::Text(Cow::from(text.to_string()))
                // can I avoid this pointless copy?
                // how do I tell the compiler that if I return, then
                // first_match won't be needed anymore?
            }
        }
    }

    fn cycle_remaining_text(&mut self) -> char {
        let first_char :char;
        let remaining :String;
        {
            let mut remaining_iter = self.remaining_text.chars();
            first_char = remaining_iter.next()
                .expect("Remaining text unexpectedly empty!");
            remaining = remaining_iter.join("");
        }
        self.remaining_text = remaining;
        first_char
    }

    pub fn parse_remaining_text<'b>(&'b mut self) -> Result<Event<'a>, SidenoteError> {
        // println!("remaining_text: {}", self.remaining_text);
        let first_char = self.cycle_remaining_text();
        match first_char {
            '{' => {
                if self.in_sidenote_block {
                    Err(SidenoteError::Nested)
                } else {
                    self.in_sidenote_block = true;
                    Ok(Event::InlineHtml(Cow::from("<span>")))
                }
            },
            '}' => {
                if self.in_sidenote_block {
                    self.in_sidenote_block = false;
                    Ok(Event::InlineHtml(Cow::from("</span>")))
                } else {
                    Err(SidenoteError::NotMatched)
                }
            },
            _ => {
                let mut next_to_parse = first_char.to_string();
                next_to_parse.push_str(&self.remaining_text);
                Ok(self.parse_first_sidenote(Cow::from(next_to_parse)))
            }
        }
    }

    pub fn parse_text_block<'b>(&'b mut self, text: Cow<'a, str>) -> Event<'a> {
        if self.in_code_block {
            Event::Text(text)
        } else {
            self.parse_first_sidenote(text)
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use std::borrow::Cow;
    use pulldown_cmark::{Event, Parser};
    use super::SidenoteParser;


    #[test]
    fn can_use_regex() {
        let re = Regex::new(r"[{}]").unwrap();
        let expected = vec!["a", "separated", "string"];
        let actual: Vec<&str> = re.split("a{separated}string").collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_get_first_sidenote() {
        let text = "here is some text {with sidenotes}"; 
        let mut parser = SidenoteParser::new(Parser::new(""));
        assert_eq!(parser.parse_first_sidenote(Cow::from(text)),
            Event::Text(Cow::from("here is some text ")));
    }

    #[test]
    fn can_parse_remaining() {
        let mut parser = SidenoteParser::new(Parser::new(""));
        parser.remaining_text = String::from("some remaining { text");
        let mut event = parser.parse_remaining_text().unwrap();
        assert_eq!(event, Event::Text(Cow::from("some remaining ")));
        event = parser.parse_remaining_text().unwrap();
        assert_eq!(event, Event::InlineHtml(Cow::from("<span>")));
        event = parser.parse_remaining_text().unwrap();
        assert_eq!(event, Event::Text(Cow::from(" text")));
        assert_eq!(parser.remaining_text, "");
    }
}
