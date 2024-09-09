use crate::ast::Location;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub location: Location,
}

impl ParseError {
    pub fn new(message: String, location: Location) -> Self {
        Self { message, location }
    }
}

/// (the result of the parse, the errors generated, if we have readched eof (for cascading error)).
pub type ParseResult<Product> = (Product, Vec<ParseError>, bool);
