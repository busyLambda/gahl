use crate::{
    ast::{DocComment, Expr, Stmt},
    lexer::{token::TokenKind as TK, Lexer},
    parser::{expr::expression, var::var},
};

use super::{
    error::{ParseError, ParseResult},
    Input,
};

fn is_var(input: &mut Input) -> bool {
    input.match_pattern(vec![TK::Identifier, TK::Coleq])
        || input.match_pattern(vec![TK::Identifier, TK::Column])
        || input.match_pattern(vec![TK::Identifier, TK::Eq])
        || input.match_pattern(vec![TK::OpenCurly, TK::Identifier])
}

pub fn stmt(input: &mut Input) -> Option<(Stmt, bool)> {
    let first_kind = match input.peek() {
        Some(t) => t.kind(),
        None => return None,
    };

    match first_kind {
        k if is_var(input) => {
            let (var, is_eof) = var(input);
            Some((Stmt::Var(var), is_eof))
        }
        k if k.is_expr() => {
            let (expr, errors, is_eof) = expression(input);
            Some((Stmt::Expr(expr, errors), is_eof))
        }
        TK::DocComment => {
            let doc_comment = input.eat().unwrap().literal();
            Some((Stmt::DocComment(DocComment { md: doc_comment }), false))
        }
        k => {
            // TODO: Report error
            println!("Got: {:?}", k);
            todo!()
        }
    }
}

pub fn block(input: &mut Input) -> ParseResult<Vec<Stmt>> {
    let mut product: Vec<Stmt> = vec![];
    let mut errors: Vec<ParseError> = vec![];

    input.eat();

    loop {
        match input.peek() {
            Some(t) if t.kind() == TK::ClosedCurly => {
                input.eat();
                break;
            }
            None => return (product, vec![], true),
            _ => (),
        }

        let (stmt, is_eof) = match stmt(input) {
            Some(res) => res,
            None => return (product, vec![], true),
        };

        product.push(stmt);

        if is_eof {
            return (product, vec![], is_eof);
        }
    }

    (product, errors, false)
}

// #[test]
// fn test_stmt_parser() {
//     let input = "transform : fn(i32) fn(i32) bool";
//     let mut lexer = Lexer::new(input);
//     let tokens = lexer.lex();
//     let mut input = Input::new(tokens);

//     let (stmt, is_eof) = stmt(&mut input).unwrap();
//     assert_eq!(is_eof, false);
// }
