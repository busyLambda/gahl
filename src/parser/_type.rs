use crate::{
    ast::{Location, Type, TypeValue},
    lexer::{
        token::{Span, TokenKind as TK},
        Lexer,
    },
    parser::error::ParseError,
};

use super::{error::ParseResult, Input};

pub fn _type(input: &mut Input) -> ParseResult<Type> {
    let mut product = Type::default();
    let mut errors: Vec<ParseError> = vec![];

    let (first_kind, start_pos, start_row) = match input.peek() {
        Some(t) => (t.kind(), t.pos(), t.row_col().0),
        None => todo!(),
    };

    let tv = match first_kind {
        TK::Tvoid => {
            input.eat();
            TypeValue::Void
        }
        TK::Tbool => {
            input.eat();
            TypeValue::Bool
        }
        TK::Ti8 => {
            input.eat();
            TypeValue::I8
        }
        TK::Ti16 => {
            input.eat();
            TypeValue::I16
        }
        TK::Ti32 => {
            input.eat();
            TypeValue::I32
        }
        TK::Ti64 => {
            input.eat();
            TypeValue::I64
        }
        TK::Ti128 => {
            input.eat();
            TypeValue::I128
        }
        TK::Tu8 => {
            input.eat();
            TypeValue::U8
        }
        TK::Tu16 => {
            input.eat();
            TypeValue::U16
        }
        TK::Tu32 => {
            input.eat();
            TypeValue::U32
        }
        TK::Tu64 => {
            input.eat();
            TypeValue::U64
        }
        TK::Tu128 => {
            input.eat();
            TypeValue::U128
        }
        TK::Tf32 => {
            input.eat();
            TypeValue::F32
        }
        TK::Tf64 => {
            input.eat();
            TypeValue::F64
        }
        TK::OpenBracket => {
            let (array_type, mut array_errors, is_eof) = array_type(input);
            if is_eof {
                todo!()
            }
            errors.append(&mut array_errors);
            array_type
        }
        TK::KwFn => {
            let (function_type, mut function_errors, is_eof) = function_type(input);
            if is_eof {
                todo!()
            }
            errors.append(&mut function_errors);
            function_type
        }
        _ => todo!(),
    };

    product.type_value = tv;

    let span = start_pos.start..input.prev_pos.end;
    let rows = (start_row, input.prev_row);
    let location = Location::new(span, rows);

    product.location = location;

    // TODO: Remove errors from Type.
    (product, errors, false)
}

pub fn function_type(input: &mut Input) -> ParseResult<TypeValue> {
    let mut errors: Vec<ParseError> = vec![];
    let mut params: Vec<TypeValue> = vec![];
    let mut return_ty: TypeValue = TypeValue::Void;

    input.eat();

    let (first_kind, _start_pos, _start_row) = match input.peek() {
        Some(t) => (t.kind(), t.pos(), t.row_col().0),
        None => return (TypeValue::Void, errors, true),
    };

    match first_kind {
        TK::OpenParen => {
            input.eat();

            loop {
                match input.peek() {
                    Some(t) if t.kind() == TK::ClosedParen => {
                        input.eat();
                        break;
                    }
                    Some(_) => (),
                    None => return (TypeValue::Void, errors, true),
                }

                let (param_type, mut param_errors, is_eof) = _type(input);

                errors.append(&mut param_errors);
                params.push(param_type.type_value);

                if is_eof {
                    return (TypeValue::Void, errors, is_eof);
                }
            }
        }
        _ => todo!(),
    };

    let (return_ty_, mut return_ty_errors, is_eof) = _type(input);
    return_ty = return_ty_.type_value;
    errors.append(&mut return_ty_errors);

    (TypeValue::Func(params, Box::new(return_ty)), errors, is_eof)
}

#[test]
fn test_function_type_parser() {
    let input = "fn (i32) void";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);

    let (_type, errors, is_eof) = _type(&mut input);
    assert_eq!(errors.len(), 0);
    assert_eq!(is_eof, false);

    println!("{:?}", _type);
}

pub fn array_type(input: &mut Input) -> ParseResult<TypeValue> {
    let mut errors: Vec<ParseError> = vec![];
    let mut product = TypeValue::Array(Box::new(TypeValue::Void));

    let (first_kind, _start_pos, _start_row) = match input.peek() {
        Some(t) => (t.kind(), t.pos(), t.row_col().0),
        None => return (product, errors, true),
    };
    // TODO: Redundant.
    if first_kind != TK::OpenBracket {
        panic!()
    }
    input.eat();

    let (inner_type, mut type_errors, is_eof) = _type(input);

    errors.append(&mut type_errors);
    product = TypeValue::Array(Box::new(inner_type.type_value));

    if is_eof {
        return (product, errors, true);
    }

    match input.peek() {
        Some(t) if t.kind() == TK::ClosedBracket => {
            input.eat();
        }
        Some(t) => todo!(),
        None => return (product, errors, false),
    }

    (product, errors, false)
}

#[test]
fn test_type_parser() {
    let input = "[i32]";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex();
    let mut input = Input::new(tokens);

    let (_type, errors, is_eof) = _type(&mut input);
    assert_eq!(errors.len(), 0);
    assert_eq!(is_eof, false);
}
