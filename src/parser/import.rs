use std::{
    collections::HashMap, sync::{
        atomic::Ordering::SeqCst,
        mpsc::{channel, TryRecvError},
    }, thread
};

use crate::{
    ast::{Import, Imports, Location},
    lexer::token::{Span, TokenKind as TK},
};

use super::{
    error::{ParseError, ParseResult},
    name::name,
    seek_file, Input,
};

// Parses the import block atop a file.
pub fn imports(input: &mut Input, is_first_import: bool) -> ParseResult<HashMap<String, Option<String>>> {
    let mut imports = Imports::default();
    let mut product = HashMap::<String, Option<String>>::new();
    let mut errors: Vec<ParseError> = vec![];

    let (first_kind, first_pos, first_row) = match input.peek() {
        Some(t) => (t.kind(), t.pos(), t.row_col().0),
        None => return (product, errors, true),
    };

    if is_first_import {
        match first_kind {
            TK::KwImport => {
                input.eat();
            }
            tk => {
                println!("Unhandled token: {:?}", tk);
                todo!()
            }
        }
    }

    match input.peek() {
        Some(t) if t.kind() == TK::OpenCurly => {
            input.eat();
        }
        Some(t) => {
            let rows = (t.row_col().0, t.row_col().0);
            let location = Location::new(t.pos(), rows);
            let error = ParseError::new(
                format!("Expected `OpenCurly` but found `{:?}`.", t.kind()),
                location,
            );

            errors.push(error);
        }
        None => return (product, errors, true),
    }

    loop {
        match input.peek() {
            Some(t) if t.kind() == TK::ClosedCurly => {
                input.eat();
                break;
            }
            Some(t) if t.kind() == TK::Identifier => {}
            Some(t) => {
                let rows = (t.row_col().0, t.row_col().0);
                let location = Location::new(t.pos(), rows);
                let error = ParseError::new(
                    format!(
                        "Expected `ClosedCurly` or `Identifier` but found `{:?}`.",
                        t.kind()
                    ),
                    location,
                );

                errors.push(error);
            }
            None => return (product, errors, true),
        }

        match input.peek() {
            Some(t) if t.kind() == TK::Identifier => {
                let (name, mut name_errors, is_eof) = name(input);

                errors.append(&mut name_errors);

                let (path, import) = seek_file(name.clone());

                product.insert(path, import);

                if is_eof {
                    return (product, errors, is_eof);
                }

                match input.peek() {
                    Some(t) if t.kind() == TK::OpenCurly => {
                        todo!()
                    }
                    _ => {
                        imports.imports.push(Import::ImportSingle(name));
                    }
                }
            }
            Some(t) => {
                println!("Unhandled token: {:?}", t.kind());
                todo!()
            }
            None => return (product, errors, true),
        }
    }

    if imports.imports.is_empty() {
        // Done with nothing else to do!
        input.initiator_sender.send(()).unwrap();
    } else {
        imports.imports.iter().for_each(|import| {
            input.block_counter.clone().fetch_add(1, SeqCst);

            let (callback_sender, callback_reciever) = channel::<()>();

            let block_coutner_clone = input.block_counter.clone();
            let initiator_sender_clone = input.initiator_sender.clone();
            thread::spawn(move || loop {
                match callback_reciever.try_recv() {
                    Ok(_) => {
                        // You do your stuff lil buddy :3
                        // println!("Got callback! ☑️");
                        block_coutner_clone.fetch_sub(1, SeqCst);
                        initiator_sender_clone.send(()).unwrap();
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        continue;
                    }
                    _ => todo!(),
                }
            });

            match import {
                Import::ImportSingle(name) => {
                    input.sender.send((name.clone(), callback_sender)).unwrap();
                }
                _ => todo!(),
            }
        });
    }

    (product, errors, false)
}
