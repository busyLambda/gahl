use std::ops::Range;

use crate::parser::error::ParseError;

pub enum Stmt {
    Func(FuncNode),
}

#[derive(Debug)]
pub struct Location {
    span: Range<usize>,
    rows: (usize, usize),
}

impl Location {
    pub fn new(span: Range<usize>, rows: (usize, usize)) -> Self {
        Self { span, rows }
    }

    pub fn default() -> Self {
        Self {
            span: 0..0,
            rows: (0, 0),
        }
    }
}

#[derive(Debug)]
pub struct Name {
    name: Vec<String>,

    pub location: Location,
}

impl Name {
    pub fn new(name: Vec<String>, location: Location) -> Self {
        Self { name, location }
    }
}

#[derive(Debug)]
pub enum TypeValue {
    Void,

    I8,
    I16,
    I32,
    I64,
    I128,

    U8,
    U16,
    U32,
    U64,
    U128,

    F32,
    F64,

    Array(Box<TypeValue>),
    Generic(Box<TypeValue>),
    Func(Vec<TypeValue>, Box<TypeValue>),
}

#[derive(Debug)]
pub struct Type {
    pub type_value: TypeValue,

    pub location: Location,
}

impl Type {
    pub fn default() -> Self {
        Self {
            type_value: TypeValue::Void,
            location: Location::default(),
        }
    }
}

pub struct FuncNode {
    name: Name,
    args: Vec<(Name, Type)>,
    return_type: Type,

    pub location: Location,
    pub errors: ParseError,
}

pub enum VarLhs {
    Tuple(Vec<String>),
    Name(String),
}

impl VarLhs {
    pub fn default() -> Self {
        Self::Name(String::new())
    }
}

pub enum Expr {
    Void,

    Int(i128, Location),
    Uint(u128, Location),
    Float(f64, Location),

    Identifier(Name, Location),
    FuncCall(Name, Vec<Expr>, Location),
    Func(FuncNode),

    Add(Box<Expr>, Box<Expr>, Location),
    Min(Box<Expr>, Box<Expr>, Location),

    Mul(Box<Expr>, Box<Expr>, Location),
    Div(Box<Expr>, Box<Expr>, Location),

    Paren(Box<Expr>, Location),
}

pub struct Var {
    pub lhs: VarLhs,
    pub _type: Type,
    pub rhs: Expr,
    pub is_decl: bool,

    pub location: Location,
    pub errors: Vec<ParseError>,
}

impl Var {
    pub fn default() -> Var {
        Var {
            lhs: VarLhs::default(),
            _type: Type::default(),
            rhs: Expr::Void,
            is_decl: false,
            location: Location::default(),
            errors: vec![],
        }
    }
}
