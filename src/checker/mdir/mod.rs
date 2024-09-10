use std::collections::HashMap;

use crate::ast::{Type, TypeValue};

#[derive(Debug)]
pub struct MiddleIR {
    functions: HashMap<String, Function>,
}

impl MiddleIR {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn functions(&self) -> &HashMap<String, Function> {
        &self.functions
    }

    pub fn insert_function(&mut self, function: Function) {
        self.functions.insert(function.name.clone(), function);
    }
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, TypeValue)>,
    pub return_type: TypeValue,
    pub block: Vec<Statement>,
}

impl Function {
    pub fn default() -> Self {
        Self {
            name: String::new(),
            params: vec![],
            return_type: TypeValue::Void,
            block: vec![],
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Expr(Expression),
    Var(Var),
}

#[derive(Debug)]
pub struct Var {
    lhs: String,
    rhs: Expression,
    ty: TypeValue,
}

impl Var {
    pub fn new(lhs: String, rhs: Expression, ty: TypeValue) -> Self {
        Self { lhs, rhs, ty }
    }
}

#[derive(Debug)]
pub struct Expression {
    pub ty: TypeValue,
    pub inner: Expr,
}

impl Expression {
    pub fn default() -> Self {
        Self {
            ty: TypeValue::Void,
            inner: Expr::Void,
        }
    }

    pub fn new(ty: TypeValue, inner: Expr) -> Self {
        Self { ty, inner }
    }
}

#[derive(Debug)]
pub enum Expr {
    Void,
    Add(Box<Expression>, Box<Expression>),
    Min(Box<Expression>, Box<Expression>),
    Literal(Literal),
    Call(String, Vec<Expression>),
}

#[derive(Debug)]
pub enum Literal {
    Int(String),
    Identifier(String),
}
