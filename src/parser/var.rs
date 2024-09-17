use crate::{
    ast::{Location, Var, VarLhs},
    lexer::{token::TokenKind as TK, Lexer},
    parser::error::ParseError,
};

use super::{
    error::ParseResult,
    Input,
    _type::_type,
    expr::{expr, expression},
};

pub fn var(input: &mut Input) -> (Var, bool) {
    let mut product = Var::default();

    let (lhs, mut errors_lhs, is_eof) = var_lhs(input);
    if is_eof {
        return (product, true);
    }

    product.errors.append(&mut errors_lhs);
    product.lhs = lhs;

    match input.peek() {
        Some(t) if t.kind() == TK::Column => {
            input.eat();
            product.is_decl = true;

            let (inner_type, mut errors, is_eof) = _type(input);

            product._type = inner_type;
            product.errors.append(&mut errors);

            (product, is_eof)
        }
        Some(t) if t.kind() == TK::Eq => {
            input.eat();

            let (rhs, mut errors, is_eof) = expression(input);

            product.errors.append(&mut errors);
            product.rhs = rhs;

            (product, is_eof)
        }
        Some(t) if t.kind() == TK::Coleq => {
            input.eat();
            product.is_decl = true;

            let (rhs, mut errors, is_eof) = expression(input);

            product.errors.append(&mut errors);
            product.rhs = rhs;

            (product, is_eof)
        }
        Some(_) => todo!(),
        _ => todo!(),
    }

    // (product, false)
}

#[test]
fn test_var_parser() {
    // let input = "{a, b} := (50 * 2) / 5";
    let input = "a := (50 * 2) / 5";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);

    let (var, is_eof) = var(&mut input);
    assert_eq!(var.errors.len(), 0);
    assert_eq!(is_eof, false);
}

pub fn var_lhs(input: &mut Input) -> ParseResult<VarLhs> {
    let (first_kind, first_pos, first_row) = match input.peek() {
        Some(t) => (t.kind(), t.pos(), t.row_col().0),
        None => todo!(),
    };

    if first_kind == TK::OpenCurly {
        input.eat();

        let (product, mut errors, _is_eof) = separated_identifiers(input);

        let mut var_lhs = VarLhs::default();
        var_lhs.name = vec![input.eat().unwrap().literal()];
        let location = Location::new(
            first_pos.start..input.prev_pos.end,
            (first_row, input.prev_row),
        );
        var_lhs.location = location;

        return match input.peek() {
            Some(t) if t.kind() == TK::ClosedCurly => {
                input.eat();
                (var_lhs, errors, false)
            }
            Some(t) => {
                let span = t.pos();
                let rows = (t.row_col().0, t.row_col().0);
                let location = Location::new(span, rows);
                let message = format!("Expected `}}` at the end of list but found `{:?}`.", t.kind());
                let error = ParseError::new(message, location);

                errors.push(error);

                (var_lhs, errors, false)
            }
            _ => (var_lhs, errors, true),
        };
    }

    if first_kind == TK::Identifier {
        let mut var_lhs = VarLhs::default();
        var_lhs.name = vec![input.eat().unwrap().literal()];
        let location = Location::new(first_pos, (first_row, first_row));
        var_lhs.location = location;
        return (var_lhs, vec![], false);
    }

    let mut errors: Vec<ParseError> = vec![];

    let span = first_pos;
    let rows = (first_row, first_row);
    let location = Location::new(span, rows);
    let message = format!("Expected an identifier at the start of list.");
    let error = ParseError::new(message, location);

    errors.push(error);

    (VarLhs::default(), errors, false)
}

pub fn separated_identifiers(input: &mut Input) -> ParseResult<Vec<String>> {
    let mut product: Vec<String> = vec![];
    let mut errors: Vec<ParseError> = vec![];
    match input.peek() {
        Some(t) if t.kind() == TK::Identifier => product.push(input.eat().unwrap().literal()),
        Some(t) => {
            let span = t.pos();
            let rows = (t.row_col().0, t.row_col().0);
            let location = Location::new(span, rows);
            let message = format!("Expected an identifier at the start of list.");
            let error = ParseError::new(message, location);

            errors.push(error);
        }
        None => return (product, errors, true),
    };

    loop {
        let kind = match input.peek() {
            Some(t) => t.kind(),
            None => break,
        };

        match kind {
            _ if input.match_pattern(vec![TK::Comma, TK::Identifier]) => {
                input.eat();
                product.push(input.eat().unwrap().literal());
            }
            TK::Comma => {
                let comma = input.eat().unwrap();

                let span = comma.pos();
                let rows = (comma.row_col().0, comma.row_col().0);
                let location = Location::new(span, rows);
                let message =
                    format!("Got a `,` at the end of list, expected a `,` and an `identifier`.");
                let error = ParseError::new(message, location);

                errors.push(error);
                break;
            }
            TK::Identifier => {
                let identifier = input.eat().unwrap();
                product.push(identifier.literal());

                let span = identifier.pos();
                let rows = (identifier.row_col().0, identifier.row_col().0);
                let location = Location::new(span, rows);
                let message = format!("Expected a comma and an identifier, got `identifier`");
                let error = ParseError::new(message, location);

                errors.push(error);
            }
            _ => break,
        }
    }

    (product, errors, false)
}

#[test]
fn test_separated_identifiers_parser() {
    let input = "user, summary";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);

    let (_identifiers, errors, is_eof) = separated_identifiers(&mut input);
    assert_eq!(errors.len(), 0);
    assert_eq!(is_eof, false);
}
