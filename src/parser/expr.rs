use crate::{
    ast::{Expr, FuncNode, Location, Name},
    lexer::{token::TokenKind as TK, Lexer},
    parser::{error::ParseError, name::name},
};

use super::{error::ParseResult, stmt::block, Input};

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
        TK::Identifier => {
            let (name, errors, is_eof) = name(input);
            let location = Location::new(
                start_pos.start..input.prev_pos.end,
                (start_row, input.prev_row),
            );

            match input.peek() {
                Some(t) if t.kind() == TK::OpenParen => {
                    input.eat();
                    let (args, errors, is_eof) = separated_exprs(input);

                    if is_eof {
                        panic!()
                    }

                    let call_location = Location::new(
                        start_pos.start..input.prev_pos.end,
                        (start_row, input.prev_row),
                    );

                    (Expr::FuncCall(name, args, call_location), errors, false)
                }
                _ => (Expr::Identifier(name, location), errors, is_eof),
            }
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
        tk => {
            todo!()
        }
    }
}

fn term(input: &mut Input) -> ParseResult<Expr> {
    let (start_pos, start_row) = match input.peek() {
        Some(t) => (t.pos(), t.row_col().0),
        _ => (input.prev_pos.clone(), input.prev_row),
    };

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
                let location = Location::new(
                    start_pos.start..input.prev_pos.end,
                    (start_row, input.prev_row),
                );

                return (
                    Expr::Add(Box::new(expr), Box::new(rhs), location),
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
    let (start_pos, start_row) = match input.peek() {
        Some(t) => (t.pos(), t.row_col().0),
        _ => (input.prev_pos.clone(), input.prev_row),
    };

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

                let location = Location::new(
                    start_pos.start..input.prev_pos.end,
                    (start_row, input.prev_row),
                );

                return (
                    Expr::Add(Box::new(expr), Box::new(rhs), location),
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
            Ok(Some(ident)) => {
                product.args.push(ident.literal());
            }
            Err((error, is_stmt)) => {
                product.errors.push(error);
                if is_stmt {
                    return (product, vec![], false);
                } else {
                    input.eat();
                }
            }
            Ok(None) => return (product, vec![], true),
        };
    }

    match input.peek() {
        Some(t) if t.kind() == TK::OpenCurly => {
            let (block, errors, is_eof) = block(input);
            product.block = block;

            // Add errors.
            if is_eof {
                return (product, vec![], is_eof);
            }
        }
        Some(t) => todo!(),
        None => return (product, vec![], true),
    };

    (product, vec![], false)
}

fn separated_exprs(input: &mut Input) -> ParseResult<Vec<Expr>> {
    let mut product: Vec<Expr> = vec![];
    let mut errors: Vec<ParseError> = vec![];

    let mut is_multi_expr = false;

    loop {
        match input.peek() {
            Some(t) if t.kind() == TK::ClosedParen => {
                input.eat();
                break;
            }
            Some(t) if is_multi_expr && t.kind() == TK::Comma => {
                input.eat();
            }
            Some(t) if is_multi_expr => {
                // TODO: Report error relating to expecting `)` or `,`
                todo!()
            }
            None => return (product, errors, true),
            _ => (),
        }
        if is_multi_expr {
            match input.peek() {
                Some(t) if t.kind() == TK::ClosedParen => {
                    input.eat();
                    break;
                }
                None => {
                    // Report error here...
                    return (product, errors, true);
                }
                Some(t) => (),
            }
        }

        let (expr, mut expr_errors, is_eof) = expr(input);

        product.push(expr);
        errors.append(&mut expr_errors);

        if is_eof {
            return (product, errors, is_eof);
        }

        is_multi_expr = true;
    }

    (product, errors, false)
}

pub fn expression(input: &mut Input) -> ParseResult<Expr> {
    let mut errors: Vec<ParseError> = vec![];
    let mut product = Expr::Void;

    let first_kind = match input.peek() {
        Some(t) => t.kind(),
        None => return (product, errors, true),
    };

    if first_kind == TK::KwFn {
        let (fn_expr, mut fn_expr_errors, is_eof) = function_expr(input);

        product = Expr::Func(fn_expr);
        errors.append(&mut fn_expr_errors);

        if is_eof {
            return (product, errors, is_eof);
        }
    } else {
        let (expr, mut expr_errors, is_eof) = expr(input);

        product = expr;
        errors.append(&mut expr_errors);

        if is_eof {
            return (product, errors, is_eof);
        }
    }

    return (product, errors, false);
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

#[test]
fn test_function_call() {
    let input = "function(b)";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);

    let (expr, errors, is_eof) = expr(&mut input);
    assert_eq!(errors.len(), 0);
}
