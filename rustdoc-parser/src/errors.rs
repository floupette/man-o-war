use std::{
    error::Error,
    fmt::{
        Display,
        Formatter,
        Result
        }
};

#[derive(Debug)]
pub enum Herr {
    Parsing(&'static str),
}

impl Error for Herr {}
impl Display for Herr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Herr::Parsing(err) => write!(f, "{}", err),
        }
    }
}
