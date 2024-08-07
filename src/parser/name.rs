use crate::{
    ast::{Location, Name},
    lexer::{token::TokenKind as TK, Lexer},
    parser::error::ParseError,
};

use super::{error::ParseResult, Input};

pub fn name(input: &mut Input) -> ParseResult<Name> {
    let mut errors: Vec<ParseError> = vec![];
    let mut names: Vec<String> = vec![];

    let (first_namespace, start_loc, start_row) = {
        let tok = input
            .eat()
            .expect("Expected the user of `name` to check if next token was identifier.");

        (tok.literal(), tok.pos().start, tok.row_col().0)
    };

    names.push(first_namespace);

    loop {
        let kind = match input.peek() {
            Some(t) => t.kind().to_owned(),
            None => break,
        };

        match kind {
            t if input.match_pattern(vec![TK::Dot, TK::Identifier]) => {
                input.eat().unwrap();
                names.push(input.eat().unwrap().literal());
            }
            TK::Dot => {
                input.eat().unwrap();

                let span = start_loc..input.prev_pos.end;
                let rows = (start_row, input.prev_row);
                let location = Location::new(span, rows);
                let message = format!("Name ended in a dot instead of an identifier.");
                let error = ParseError::new(message, location);

                errors.push(error);
                break;
            }
            _ => break,
        }
    }

    let span = start_loc..input.prev_pos.end;
    let rows = (start_row, input.prev_row);
    let location = Location::new(span, rows);
    (Name::new(names, location), errors, false)
}

#[test]
fn test_name_parser() {
    let input = "std.fs.open";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);

    let (_name, errors, is_eof) = name(&mut input);
    assert_eq!(errors.len(), 0);
    assert_eq!(is_eof, false);
}
