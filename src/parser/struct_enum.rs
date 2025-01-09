use crate::{
    ast::{EnumDecl, TypeValue},
    lexer::token::TokenKind as TK,
    parser::error::ParseError,
};

use super::{error::ParseResult, Input, _type::_type};

pub fn parse_enum(input: &mut Input) -> ParseResult<EnumDecl> {
    let mut errors = Vec::<ParseError>::new();
    let mut product = EnumDecl(vec![]);

    let (start_loc, start_row) = match input.eat() {
        Some(t) if t.kind() == TK::KwEnum => (t.pos().start, t.row_col().0),
        None => {
            return (product, errors, true);
        }
        _ => todo!(),
    };
    
    let name = match input.peek() {
        Some(t) if t.kind() == TK::Identifier => {
            input.eat().unwrap().literal()
        }
        _ => todo!()
    };

    let mut parsed_one = false;

    loop {
        match input.peek() {
            Some(t) if t.kind() == TK::Pipe => {
                input.eat();
            }
            // TODO: Improve the error handling and recovery.
            Some(_t) if parsed_one => break,
            None => {
                return (product, errors, true);
            }
            t => {
                println!("Le token: {:?}", t);
                todo!()
            },
        }
        let (record, mut record_errors, is_eof) = parse_enum_record(input);
        errors.append(&mut record_errors);

        if is_eof {
            return (product, errors, is_eof);
        }

        product.0.push(record);
        parsed_one = true;
    }

    (product, errors, false)
}

pub fn parse_enum_record(input: &mut Input) -> ParseResult<(String, Vec<TypeValue>)> {
    let mut errors = Vec::<ParseError>::new();

    let name: String;
    let mut types = vec![];

    match input.peek() {
        Some(t) if t.kind() == TK::Identifier => {
            name = input.eat().unwrap().literal();
        }
        None => {
            // return (, errors, true);
            todo!()
        }
        _ => todo!(),
    }
    
    match input.peek() {
        None => {
            types.push(TypeValue::EnumVariant(name.clone()));
            return ((name, types), vec![], false)
        },
        Some(t) if
        t.kind() != TK::Pipe && !t.kind().is_type() => {
            types.push(TypeValue::EnumVariant(name.clone()));
            return ((name, types), vec![], false)
        }
        _ => ()
    }

    let mut is_multi_param = false;

    loop {
        match input.peek() {
            Some(t) if t.kind() == TK::Comma && is_multi_param => {
                input.eat();
            }
            // TODO: Improve the error handling and recovery.
            Some(t) if t.kind().is_type() => (),
            Some(t) if t.kind() == TK::Pipe => break,
            Some(t) => {
                println!("Got: {:?}, is_multi_param: {is_multi_param}, name: {name}", t.kind());
                todo!()
            },
            None => todo!(),
        }

        let (type_, mut type_errors, is_eof) = _type(input);
        errors.append(&mut type_errors);

        if is_eof {
            return ((name, types), errors, is_eof);
        }

        types.push(type_.type_value);

        is_multi_param = true;
    }

    ((name, types), errors, false)
}

pub fn parse_struct() {}
