use core::task;
use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::Read,
    mem::size_of,
    ops::{Range, Sub},
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        mpsc::{channel, Receiver, Sender, TryRecvError},
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

    fn handle_task(
        module_map_c: Arc<Mutex<HashMap<String, Module>>>,
        task_sender_c: Sender<(Name, Sender<()>)>,
        tc: Arc<AtomicUsize>,
        name: Name,
        sender: Sender<()>,
        block_counter: Arc<AtomicUsize>,
    ) {
        tc.fetch_add(1, SeqCst);

        let path = seek_file(name);

        let mut file = File::open(path.clone()).unwrap();
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let mut lexer = Lexer::new(&contents);
        let tokens = lexer.lex();

        let mut input = Input::new(tokens, task_sender_c.clone(), sender, block_counter);

        let module = module(&mut input, path.clone());

        module_map_c.clone().lock().unwrap().insert(path, module);
    }

    pub fn parse(&mut self, path: &str) -> Module {
        let (task_sender, task_reciever) = channel::<(Name, Sender<()>)>();

        let task_counter = Arc::new(AtomicUsize::new(0));
        // Increment by 1 so that we account for the main module.
        let block_counter = Arc::new(AtomicUsize::new(1));

        // Clone for ownership's sake.
        let task_sender_c = task_sender.clone();
        let modules = thread::spawn(move || {
            let module_map = Arc::new(Mutex::new(HashMap::<String, Module>::new()));

            loop {
                match task_reciever.try_recv() {
                    Ok((name, sender)) => {
                        let module_map_c = module_map.clone();
                        let task_sender_c = task_sender_c.clone();

                        let tc = task_counter.clone();
                        let bc = block_counter.clone();
                        thread::spawn(move || {
                            Self::handle_task(module_map_c, task_sender_c, tc, name, sender, bc);
                        });
                    }
                    Err(TryRecvError::Empty) => {
                        if (task_counter.load(SeqCst) == 0) && (block_counter.load(SeqCst) == 0) {
                            break;
                        }
                    }
                    _ => todo!(),
                }
            }

            Arc::try_unwrap(module_map).unwrap().into_inner().unwrap()
        });

        modules.join().unwrap();

        todo!()
    }
}

pub struct Input {
    stream: Vec<Token>,
    pos: usize,
    prev_pos: Range<usize>,
    prev_row: usize,

    // Multithreading / Multimodule stuff
    sender: Sender<(Name, Sender<()>)>,
    initiator_sender: Sender<()>,
    block_counter: Arc<AtomicUsize>,
}

impl Input {
    pub fn new(
        tokens: Vec<Token>,
        sender: Sender<(Name, Sender<()>)>,
        initiator_sender: Sender<()>,
        block_counter: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            stream: tokens,
            pos: 0,
            prev_pos: 0..0,
            prev_row: 0,
            sender,
            initiator_sender,
            block_counter,
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
