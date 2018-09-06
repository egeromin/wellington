use std::fmt;


/// sidenote errors. The possible errors are:
/// 
/// * not matched, e.g. "bla { bla" or "bla } {bla}"
/// * nested, e.g. "{ bla { }"
#[derive(Debug)]
pub enum SidenoteError{
    NotMatched,
    Nested
}


impl fmt::Display for SidenoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SidenoteError::NotMatched => {
                write!(f, "Error: a sidenote delimiter was not matched")
            },
            SidenoteError::Nested => {
                write!(f, "Error: encountered a nested sidenote")
            }
        }
    }
}

