use std::{borrow::Borrow, collections::HashMap, ops::Range};

use error::ParseError;
use stmt::stmt;

use crate::{
    ast::{Expr, FuncNode, Location, Module, Stmt, Type, TypeValue, VarLhs},
    lexer::{
        token::{Token, TokenKind},
        Lexer,
    },
};

pub mod _type;
pub mod error;
pub mod expr;
pub mod name;
pub mod stmt;
pub mod var;

pub struct Input {
    stream: Vec<Token>,
    pos: usize,
    prev_pos: Range<usize>,
    prev_row: usize,
}

impl Input {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            stream: tokens,
            pos: 0,
            prev_pos: 0..0,
            prev_row: 0,
        }
    }

    pub fn eat(&mut self) -> Option<&Token> {
        let result = if self.pos > self.stream.len() - 1 {
            None
        } else {
            let tok = &self.stream[self.pos];

            self.prev_pos = tok.pos();
            self.prev_row = tok.row_col().0;

            Some(tok)
        };

        self.pos += 1;

        result
    }

    pub fn eat_x(&mut self, x: usize) -> Option<&Token> {
        if self.pos + x + 1 >= self.stream.len() {
            None
        } else {
            self.pos += x;
            Some(&self.stream[self.pos])
        }
    }

    pub fn peek_x(&self, x: usize) -> Option<&Token> {
        if self.pos + x > self.stream.len() - 1 {
            None
        } else {
            Some(&self.stream[self.pos + x])
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        if self.pos > self.stream.len() - 1 {
            None
        } else {
            Some(&self.stream[self.pos])
        }
    }

    pub fn peek_vec(&self, x: usize) -> Option<Vec<&Token>> {
        let mut results: Vec<&Token> = vec![];

        for i in 0..x {
            let tok = self.peek_x(i)?;
            results.push(tok);
        }

        Some(results)
    }

    pub fn match_pattern(&mut self, pattern: Vec<TokenKind>) -> bool {
        let found = match self.peek_vec(pattern.len()) {
            Some(tks) => tks,
            None => {
                return false;
            }
        };

        let mut i = 0;
        loop {
            if i > pattern.len() - 1 {
                break;
            }

            if found[i].kind() != pattern[i] {
                return false;
            }

            i += 1;
        }

        true
    }

    pub fn match_pattern_ref(&mut self, pattern: &Vec<TokenKind>) -> bool {
        let found = match self.peek_vec(pattern.len()) {
            Some(tks) => tks,
            None => {
                return false;
            }
        };

        let mut i = 0;
        loop {
            if i >= pattern.len() - 1 {
                break;
            }

            if found[i].kind() != pattern[i] {
                return false;
            }

            i += 1;
        }

        true
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<Option<&Token>, (ParseError, bool)> {
        match self.peek() {
            Some(t) if t.kind() == kind => Ok(Some(self.eat().unwrap())),
            Some(t) => {
                let span = t.pos();
                let rows = (t.row_col().0, t.row_col().0);
                let location = Location::new(span, rows);
                let message = format!("Expected {:?} but found {:?}", kind, t.kind());
                let error = ParseError::new(message, location);

                if t.kind().is_stmt() {
                    Err((error, true))
                } else {
                    Err((error, false))
                }
            }
            None => Ok(None),
        }
    }

    pub fn recover_to_stmt_breaks(&mut self, patterns: Vec<Vec<TokenKind>>) -> Option<&Token> {
        loop {
            for pattern in &patterns {
                match self.peek() {
                    Some(t) if &t.kind() == &pattern[0] => {
                        if self.match_pattern_ref(pattern) {
                            return self.eat_x(pattern.len());
                        }
                    }
                    None => return None,
                    _ => todo!(),
                }
            }

            self.eat();
        }

        todo!()
    }
}

pub fn module(input: &mut Input, name: String) -> Module {
    let mut fn_decls = HashMap::<String, (Type, Location)>::new();
    let mut fn_defns = HashMap::<String, (FuncNode, Location)>::new();

    loop {
        let (stmt, is_eof) = match stmt(input) {
            Some(res) => res,
            None => break,
        };

        let is_func_def = |var_rhs: &Expr| {
            if let Expr::Func(_) = var_rhs {
                true
            } else {
                false
            }
        };

        let is_func_decl = |var_type: &Type| {
            if let TypeValue::Func(_, _) = var_type.type_value {
                true
            } else {
                false
            }
        };

        match stmt {
            Stmt::Var(var) if is_func_def(&var.rhs) => {
                if let Expr::Func(fn_node) = var.rhs {
                    if var.lhs.name.len() != 1 {
                        // TODO: Raise error.
                        continue;
                    }

                    fn_defns.insert(var.lhs.name[0].clone(), (fn_node, var.lhs.location));
                }
            }
            Stmt::Var(var) if is_func_decl(&var._type) => {
                if var.lhs.name.len() != 1 {
                    // TODO: Raise error.
                    continue;
                }

                fn_decls.insert(var.lhs.name[0].clone(), (var._type, var.lhs.location));
            }
            _ => continue,
        };

        if is_eof {
            // TODO: Report error
            break;
        }
    }

    Module {
        name,
        fn_decls,
        fn_defns,
    }
}

#[test]
fn test_module_parser() {
    let input = "\
    transform : fn(i32) i32 \
    transform = fn(x) { \
        5 + x \
    } \
    \
    add : fn(i32 i32) i32 \
    add = fn(a b) { \
        a + b \
    } \
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);
    let module = module(&mut input, String::from("main.gh"));
}
