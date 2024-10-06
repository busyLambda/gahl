use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    mem::size_of,
    ops::Range,
    path::Path,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use error::ParseError;
use import::imports as imports_parser;
use stmt::stmt;

use crate::{
    ast::{DocComment, Expr, FuncNode, Location, Module, Name, Stmt, Type, TypeValue},
    lexer::{
        token::{Token, TokenKind},
        Lexer,
    },
};

pub mod _type;
pub mod error;
pub mod expr;
pub mod import;
pub mod name;
pub mod stmt;
pub mod var;

pub struct Parser {
    parse_jobs: Vec<Result<Module, String>>,
    name: String,
}

// Traverse the path to find the file, because the end can be a function or struct.
fn seek_file(name: Name) -> String {
    match name.name.len() {
        1 => name.name[0].clone(),
        len => {
            let file_route = &name.name[0..len - 2];
            let joined = file_route.join("/");
            let path = Path::new(&joined);

            if path.is_file() {
                path.to_string_lossy().to_string()
            } else {
                name.name.join("/")
            }
        }
    }
}

impl Parser {
    pub fn new(path: &str) -> Self {
        Self {
            parse_jobs: vec![],
            name: path.to_string(),
        }
    }

    pub fn parse(&mut self, path: &str) -> Module {
        let mut contents = String::new();
        File::open(path)
            .expect(&format!("Cannot find `{path}`."))
            .read_to_string(&mut contents);

        let mut lexer = Lexer::new(&contents);
        let tokens = lexer.lex();

        let (sender, receiver) = channel::<Name>();

        let name = self.name.clone();

        let sender_for_this_one = sender.clone();
        let handle = thread::spawn(move || {
            let mut input = Input::new(tokens, sender_for_this_one);
            module(&mut input, name)
        });

        let modules = thread::spawn(move || {
            let mut data: Vec<Result<Module, String>> = vec![];

            while let Ok(msg) = receiver.recv() {
                let path = seek_file(msg.clone());

                let mut contents = String::new();
                let mut file = match File::open(&path) {
                    Ok(f) => f,
                    Err(err) => {
                        data.push(Err(err.to_string()));
                        continue;
                    }
                };

                match file.read_to_string(&mut contents) {
                    Ok(_) => (),
                    Err(err) => {
                        data.push(Err(err.to_string()));
                        continue;
                    }
                };

                let mut lexer = Lexer::new(&contents);
                let tokens = lexer.lex();

                let mut input = Input::new(tokens, sender.clone());

                let module = module(&mut input, msg.name.join("/"));
                data.push(Ok(module));
            }

            data
        });

        // TODO: Unstuck
        // modules.join().unwrap();
        handle.join().unwrap()
    }
}

pub struct Input {
    stream: Vec<Token>,
    pos: usize,
    prev_pos: Range<usize>,
    prev_row: usize,
    sender: Sender<Name>,
}

impl Input {
    pub fn new(tokens: Vec<Token>, sender: Sender<Name>) -> Self {
        Self {
            stream: tokens,
            pos: 0,
            prev_pos: 0..0,
            prev_row: 0,
            sender,
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
    }
}

pub fn module(input: &mut Input, name: String) -> Module {
    let mut fn_decls = HashMap::<String, (Type, Location)>::new();
    let mut fn_defns = HashMap::<String, (FuncNode, Location)>::new();
    let mut externs = HashMap::<String, (Vec<(String, TypeValue)>, TypeValue)>::new();
    let mut imports = None;

    let mut doc_comments: Vec<DocComment> = vec![];

    match input.peek() {
        Some(t) if t.kind() == TokenKind::KwImport => {
            let (_imports, imports_errors, is_eof) = imports_parser(input, true);

            imports = Some(_imports);

            // TODO: Report the errors

            if is_eof {
                return Module {
                    name,
                    imports: None,
                    fn_decls,
                    fn_defns,
                    externs,
                };
            }
        }
        _ => (),
    }

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
            if let TypeValue::Func(_, _, _) = var_type.type_value {
                true
            } else if let TypeValue::ExFunc((_, _)) = var_type.type_value {
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

                if let TypeValue::ExFunc((params, return_type)) = var._type.type_value {
                    externs.insert(var.lhs.name[0].clone(), (params, *return_type));
                } else {
                    fn_decls.insert(var.lhs.name[0].clone(), (var._type, var.lhs.location));
                }
            }
            Stmt::DocComment(md) => {
                doc_comments.push(md);
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
        imports,
        fn_decls,
        fn_defns,
        externs,
    }
}

// #[test]
// fn test_module_parser() {
//     let input = "\
//     transform : fn(i32) i32 \
//     transform = fn(x) { \
//         5 + x \
//     } \
//     \
//     add : fn(i32 i32) i32 \
//     add = fn(a b) { \
//         a + b \
//     } \
//     ";
//     let mut lexer = Lexer::new(input);
//     let tokens = lexer.lex();
//     let mut input = Input::new(tokens);
//     let _module = module(&mut input, String::from("main.gh"));
// }
