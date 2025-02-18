use core::fmt;
use std::collections::{HashMap, VecDeque};

use crate::ast::{DocComment, TypeValue};

#[derive(Debug)]
pub struct MiddleIR {
    imported_functions: HashMap<String, (Vec<(String, TypeValue)>, TypeValue)>,
    functions: HashMap<String, Function>,
    externs: Vec<ExternFunction>,
}

impl MiddleIR {
    pub fn new() -> Self {
        Self {
            imported_functions: HashMap::new(),
            functions: HashMap::new(),
            externs: vec![],
        }
    }

    pub fn set_externs(&mut self, externs: Vec<ExternFunction>) {
        self.externs = externs;
    }

    pub fn set_imported_functions(
        &mut self,
        imported_functions: HashMap<String, (Vec<(String, TypeValue)>, TypeValue)>,
    ) {
        self.imported_functions = imported_functions;
    }

    pub fn functions(&self) -> &HashMap<String, Function> {
        &self.functions
    }

    pub fn externs(&self) -> &Vec<ExternFunction> {
        &self.externs
    }

    pub fn imported_functions(&self) -> &HashMap<String, (Vec<(String, TypeValue)>, TypeValue)> {
        &self.imported_functions
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
    pub vars: Vec<(String, TypeValue)>,
    pub block: Vec<Statement>,
    pub doc_comments: Vec<DocComment>,
}

impl Function {
    pub fn default() -> Self {
        Self {
            name: String::new(),
            params: vec![],
            return_type: TypeValue::Void,
            vars: vec![],
            block: vec![],
            doc_comments: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExternFunction {
    pub name: String,
    pub params: Vec<(String, TypeValue)>,
    pub return_type: Box<TypeValue>,
}

#[derive(Debug)]
pub enum Statement {
    Expr(VecDeque<Expression>),
    Var(Var),
}

#[derive(Debug)]
pub struct Var {
    pub lhs: String,
    pub rhs: VecDeque<Expression>,
    pub ty: TypeValue,
}

pub struct Decl {
    pub lhs: String,
    pub ty: TypeValue,
}

pub struct Assign {
    pub lhs: String,
    pub rhs: Expression,
}

impl Var {
    pub fn new(lhs: String, rhs: VecDeque<Expression>, ty: TypeValue) -> Self {
        Self { lhs, rhs, ty }
    }
}

impl Decl {
    pub fn new(lhs: String, ty: TypeValue) -> Self {
        Self { lhs, ty }
    }
}

#[derive(Debug)]
pub enum Expression {
    Add,
    Min,
    Mul,
    Div,
    Pow,
    Literal(Literal),
    LParen,
    RParen,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Add => write!(f, "add"),
            Expression::Min => write!(f, "min"),
            Expression::Mul => write!(f, "mul"),
            Expression::Div => write!(f, "div"),
            Expression::Pow => write!(f, "pow"),
            Expression::Literal(lit) => write!(f, "{}", lit),
            Expression::LParen => write!(f, "("),
            Expression::RParen => write!(f, ")"),
        }
    }
}

impl Expression {
    pub fn precedence(&self) -> u8 {
        match self {
            Expression::Add | Expression::Min => 1,
            Expression::Mul | Expression::Div => 2,
            Expression::Pow => 3,
            _ => 0,
        }
    }

    pub fn is_op(&self) -> bool {
        match self {
            Expression::Add
            | Expression::Min
            | Expression::Mul
            | Expression::Div
            | Expression::Pow => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum Literal {
    Int(TypeValue, String),
    Call(TypeValue, String, Vec<(VecDeque<Expression>, TypeValue)>),
    // type, value, is_function_parameter
    Identifier(TypeValue, String, bool),
    String(String),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Int(_, value) => write!(f, "{}\0", value),
            Literal::Identifier(_, value, _is_function_param) => write!(f, "%{}", value),
            Literal::Call(_, _, _) => write!(f, "call"),
            Literal::String(value) => write!(f, "\"%{}\"", value),
        }
    }
}

impl Literal {
    pub fn _type(&self) -> &TypeValue {
        match self {
            Literal::Int(t, _) => t,
            Literal::Identifier(t, _, _) => t,
            Literal::Call(t, _, _) => t,
            Literal::String(_) => &TypeValue::String,
        }
    }
}

pub fn shunting_yard_this_mf(stream: Vec<Expression>) -> VecDeque<Expression> {
    let mut output = VecDeque::new();
    let mut stack = Vec::<Expression>::new();
    use Expression as E;

    for e in stream {
        match e {
            E::Literal(_) => output.push_back(e),
            E::LParen => stack.push(e),
            E::Add | E::Min | E::Mul | E::Div | E::Pow if !stack.is_empty() => {
                while let Some(top) = stack.last() {
                    if let E::LParen = top {
                        break;
                    }

                    if let E::Add | E::Min | E::Mul | E::Div | E::Pow = top {
                        if e.precedence() <= top.precedence() {
                            output.push_back(stack.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                }

                stack.push(e);
            }
            E::Add | E::Min | E::Mul | E::Div | E::Pow => {
                stack.push(e);
            }
            _ => {
                println!("Not implemented yet");
            }
        }
    }

    while let Some(e) = stack.pop() {
        output.push_back(e);
    }

    output
}
