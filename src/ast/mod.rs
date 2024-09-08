use std::{collections::HashMap, ops::Range};

use crate::parser::error::ParseError;

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr, Vec<ParseError>),
    Var(Var),
}

#[derive(Debug, Clone)]
pub struct Location {
    pub span: Range<usize>,
    pub rows: (usize, usize),
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
    pub name: Vec<String>,

    pub location: Location,
}

pub struct PhantomName {
    pub name: Vec<String>,

    pub location: Option<Location>,
}

impl Name {
    pub fn new(name: Vec<String>, location: Location) -> Self {
        Self { name, location }
    }
}

impl PhantomName {
    pub fn new_single(name: String, location: Option<Location>) -> Self {
        Self {
            name: vec![name],
            location,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TypeValue {
    Void,

    Bool,

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

#[derive(Debug)]
pub struct FuncNode {
    pub name: Name,
    pub args: Vec<String>,
    pub return_type: Type,
    pub block: Vec<Stmt>,

    pub location: Location,
    pub errors: Vec<ParseError>,
}

impl FuncNode {
    pub fn default() -> Self {
        Self {
            name: Name::new(vec![], Location::default()),
            args: vec![],
            return_type: Type::default(),
            block: vec![],

            location: Location::default(),
            errors: vec![],
        }
    }
}

#[derive(Debug)]
pub struct VarLhs {
    pub name: Vec<String>,

    pub location: Location,
}

impl VarLhs {
    pub fn default() -> Self {
        Self {
            name: vec![],
            location: Location::default(),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Void,

    Int(i128, Location),
    Uint(u128, Location),
    Float(f64, Location),

    Neg(Box<Expr>, Location),

    Identifier(Name, Location),
    FuncCall(Name, Vec<Expr>, Location),
    Func(FuncNode),

    Add(Box<Expr>, Box<Expr>, Location),
    Min(Box<Expr>, Box<Expr>, Location),

    Mul(Box<Expr>, Box<Expr>, Location),
    Div(Box<Expr>, Box<Expr>, Location),

    Power(Box<Expr>, Box<Expr>, Location),

    Paren(Box<Expr>, Location),
}

impl Expr {
    pub fn is_void(&self) -> bool {
        match self {
            Expr::Void => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub fn_decls: HashMap<String, (Type, Location)>,
    pub fn_defns: HashMap<String, (FuncNode, Location)>,
}
