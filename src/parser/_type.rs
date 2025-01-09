use crate::{
    ast::{Location, Type, TypeValue},
    lexer::token::TokenKind as TK,
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
        TK::Mul => {
            input.eat();
            let (inner_type, mut type_errors, is_eof) = _type(input);

            errors.append(&mut type_errors);

            if is_eof {
                todo!()
            }

            TypeValue::Ptr(Box::new(inner_type.type_value))
        }
        TK::Tvoid => {
            input.eat();
            TypeValue::Void
        }
        TK::Tbool => {
            input.eat();
            TypeValue::Bool
        }
        TK::Tstring => {
            input.eat();
            TypeValue::String
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
        TK::KwFn | TK::KwExtern => {
            let (function_type, mut function_errors, is_eof) = function_type(input);
            if is_eof {
                todo!()
            }
            errors.append(&mut function_errors);
            function_type
        }
        _ => {
            println!("{:?}", first_kind);
            todo!()
        }
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
    let mut named_params: Vec<String> = vec![];
    let mut params: Vec<TypeValue> = vec![];
    let return_ty: TypeValue;
    let mut is_extern = false;

    match input.peek() {
        Some(t) if t.kind() == TK::KwExtern => {
            input.eat();
            is_extern = true;
        }
        Some(_) => (),
        None => return (TypeValue::Void, errors, true),
    }

    input.eat();

    let (first_kind, _start_pos, _start_row) = match input.peek() {
        Some(t) => (t.kind(), t.pos(), t.row_col().0),
        None => return (TypeValue::Void, errors, true),
    };

    match first_kind {
        TK::OpenParen => {
            input.eat();

            let mut is_multi_param = false;

            loop {
                match input.peek() {
                    Some(t) if t.kind() == TK::ClosedParen => {
                        input.eat();
                        break;
                    }
                    Some(_) => (),
                    None => return (TypeValue::Void, errors, true),
                }

                if is_multi_param && is_extern {
                    // TODO: Handle errors...
                    match input.peek() {
                        Some(t) if t.kind() == TK::Comma => {
                            input.eat();
                        }
                        Some(t) => {
                            println!("t: {:?}", t);
                            todo!()
                        }
                        None => todo!(),
                    }
                }

                if is_extern {
                    match input.peek() {
                        Some(t) if t.kind() == TK::Identifier => {
                            named_params.push(input.eat().unwrap().literal());
                        }
                        Some(_t) => todo!(),
                        None => todo!(),
                    }

                    match input.peek() {
                        Some(t) if t.kind() == TK::Column => {
                            input.eat();
                        }
                        Some(_t) => todo!(),
                        None => todo!(),
                    }
                }

                let (param_type, mut param_errors, is_eof) = _type(input);

                errors.append(&mut param_errors);
                params.push(param_type.type_value);

                if is_eof {
                    return (TypeValue::Void, errors, is_eof);
                }

                is_multi_param = true;
            }
        }
        _ => todo!(),
    };

    let (return_ty_, mut return_ty_errors, is_eof) = _type(input);
    return_ty = return_ty_.type_value;
    errors.append(&mut return_ty_errors);

    let type_value = if is_extern {
        let mut final_params: Vec<(String, TypeValue)> = vec![];

        for i in 0..named_params.len() {
            let name = named_params[i].clone();
            let _type = params[i].clone();

            final_params.push((name, _type));
        }

        let extern_function = (final_params, Box::new(return_ty));

        TypeValue::ExFunc(extern_function)
    } else {
        TypeValue::Func(params, Box::new(return_ty), is_extern)
    };

    (type_value, errors, is_eof)
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
        Some(_t) => todo!(),
        None => return (product, errors, false),
    }

    (product, errors, false)
}
