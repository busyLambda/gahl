use crate::{
    ast::{Expr, FuncNode, Location},
    lexer::{token::TokenKind as TK, Lexer},
    parser::error::ParseError,
};

use super::{error::ParseResult, Input};

pub fn primary(input: &mut Input) -> ParseResult<Expr> {
    let (first_kind, start_pos, start_row) = match input.peek() {
        Some(t) => (t.kind(), t.pos(), t.row_col().0),
        None => todo!(),
    };

    match first_kind {
        TK::Integer => {
            let int = input.eat().unwrap();
            let location = Location::new(start_pos, (start_row, start_row));
            (
                Expr::Int(int.literal().parse().unwrap(), location),
                vec![],
                false,
            )
        }
        TK::OpenParen => {
            input.eat();
            let (expr, mut errors, is_eof) = expr(input);
            if is_eof {
                return (expr, errors, true);
            }

            let close_paren = match input.peek() {
                Some(t) => t,
                // TODO: Wrap expr in parens
                None => return (expr, errors, true),
            };

            if close_paren.kind() == TK::ClosedParen {
                input.eat().unwrap();
            } else {
                let span = close_paren.pos();
                let rows = (close_paren.row_col().0, close_paren.row_col().0);
                let location = Location::new(span, rows);
                let message = format!("Expected closing parenthesis, found {:?}", close_paren);
                let error = ParseError::new(message, location);
                errors.push(error);
            }

            let location = Location::new(
                start_pos.start..input.prev_pos.end,
                (start_row, input.prev_row),
            );

            (Expr::Paren(Box::new(expr), location), errors, false)
        }
        TK::Min => {
            input.eat();
            let (expr, errors, is_eof) = factor(input);
            let location = Location::new(
                start_pos.start..input.prev_pos.end,
                (start_row, input.prev_row),
            );

            (Expr::Neg(Box::new(expr), location), errors, is_eof)
        }
        _ => todo!(),
    }
}

fn term(input: &mut Input) -> ParseResult<Expr> {
    let (expr, mut errors, is_eof) = factor(input);
    loop {
        let tok = match input.peek() {
            Some(t) => t,
            None => return (expr, errors, is_eof),
        };

        match tok.kind() {
            TK::Add => {
                input.eat();
                let (rhs, mut lhs_errors, is_eof) = term(input);
                errors.append(&mut lhs_errors);
                return (
                    Expr::Add(Box::new(expr), Box::new(rhs), Location::default()),
                    errors,
                    is_eof,
                );
            }
            TK::Min => {
                input.eat();
                let (rhs, mut lhs_errors, is_eof) = term(input);
                errors.append(&mut lhs_errors);
                return (
                    Expr::Min(Box::new(expr), Box::new(rhs), Location::default()),
                    errors,
                    is_eof,
                );
            }
            TK::Mul => {
                input.eat();
                let (rhs, mut lhs_errors, is_eof) = factor(input);
                errors.append(&mut lhs_errors);
                return (
                    Expr::Mul(Box::new(expr), Box::new(rhs), Location::default()),
                    errors,
                    is_eof,
                );
            }
            TK::Div => {
                input.eat();
                let (rhs, mut lhs_errors, is_eof) = factor(input);
                errors.append(&mut lhs_errors);
                return (
                    Expr::Div(Box::new(expr), Box::new(rhs), Location::default()),
                    errors,
                    is_eof,
                );
            }
            _ => return (expr, errors, is_eof),
        }
    }
}

pub fn expr(input: &mut Input) -> ParseResult<Expr> {
    let (expr, mut errors, is_eof) = term(input);

    loop {
        let tok = match input.peek() {
            Some(t) => t,
            None => return (expr, errors, is_eof),
        };

        match tok.kind() {
            TK::Add => {
                input.eat();
                let (rhs, mut lhs_errors, is_eof) = term(input);
                errors.append(&mut lhs_errors);
                return (
                    Expr::Add(Box::new(expr), Box::new(rhs), Location::default()),
                    errors,
                    is_eof,
                );
            }
            TK::Min => {
                input.eat();
                let (rhs, mut lhs_errors, is_eof) = term(input);
                errors.append(&mut lhs_errors);
                return (
                    Expr::Min(Box::new(expr), Box::new(rhs), Location::default()),
                    errors,
                    is_eof,
                );
            }
            TK::Mul => {
                input.eat();
                let (rhs, mut lhs_errors, is_eof) = term(input);
                errors.append(&mut lhs_errors);
                return (
                    Expr::Mul(Box::new(expr), Box::new(rhs), Location::default()),
                    errors,
                    is_eof,
                );
            }
            TK::Div => {
                input.eat();
                let (rhs, mut lhs_errors, is_eof) = factor(input);
                errors.append(&mut lhs_errors);
                return (
                    Expr::Div(Box::new(expr), Box::new(rhs), Location::default()),
                    errors,
                    is_eof,
                );
            }
            _ => return (expr, errors, is_eof),
        }
    }
}

fn factor(input: &mut Input) -> ParseResult<Expr> {
    let (expr, mut errors, is_eof) = primary(input);

    let tok = match input.peek() {
        Some(t) => t,
        None => return (expr, errors, is_eof),
    };

    if tok.kind() == TK::Caret {
        input.eat();
        let (rhs, mut lhs_errors, is_eof) = factor(input);
        errors.append(&mut lhs_errors);
        return (
            Expr::Power(Box::new(expr), Box::new(rhs), Location::default()),
            errors,
            is_eof,
        );
    }

    (expr, errors, is_eof)
}

// Expr, Expr
pub fn tuple_expr(input: &mut Input) -> ParseResult<Expr> {
    todo!()
}

pub fn function_expr(input: &mut Input) -> ParseResult<FuncNode> {
    input.eat().unwrap();

    let mut product = FuncNode::default();

    match input.expect(TK::OpenParen) {
        Err((error, is_stmt)) => {
            product.errors.push(error);
            if is_stmt {
                return (product, vec![], false);
            } else {
                input.eat();
            }
        }
        Ok(None) => return (product, vec![], false),
        _ => (),
    };

    loop {
        match input.peek() {
            Some(t) if t.kind() == TK::ClosedParen => {
                input.eat();
                break;
            }
            None => return (product, vec![], true),
            _ => (),
        };

        match input.expect(TK::Identifier) {
            Ok(Some(_)) => {
                product.args.push(input.eat().unwrap().literal());
            }
            Err((error, is_stmt)) => {
                product.errors.push(error);
                if is_stmt {
                    return (product, vec![], false);
                } else {
                    input.eat();
                }
            },
            Ok(None) => return (product, vec![], true),
        };
    }

    (product, vec![], false)
}

#[test]
fn test_function_parser() {
    let input = "fn (a b)";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);

    let (expr, errors, is_eof) = function_expr(&mut input);
    assert_eq!(errors.len(), 0);
}

#[test]
fn test_expr_parser() {
    let input = "(5 * 64) / 2 * 5 + 3 ^ 2";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);

    let (expr, errors, is_eof) = expr(&mut input);
    assert_eq!(errors.len(), 0);
}
