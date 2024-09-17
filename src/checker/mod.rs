use std::{collections::HashMap, fs};

pub mod mdir;

use mdir::{
    shunting_yard_this_mf, Expression, Function, Literal, MiddleIR, Statement, Var as MdIrVar,
};

use crate::{
    ast::{Expr, FuncNode, Location, Module, Name, Stmt, Type, TypeValue, Var},
    parser::error::ParseError,
};

#[derive(Debug)]
pub struct CheckError {
    location: Location,
    message: String,
}

impl CheckError {
    fn new(location: Location, message: String) -> Self {
        Self { location, message }
    }

    fn from_parse_error(error: &ParseError) -> Self {
        Self {
            location: error.location.clone(),
            message: error.message.clone(),
        }
    }
}

pub struct Checker<'a> {
    errors: Vec<CheckError>,
    warnings: Vec<CheckError>,
    module: &'a Module,
    symbol_stack: Vec<HashMap<&'a String, (TypeValue, bool)>>,
}

impl<'a> Checker<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self {
            errors: vec![],
            warnings: vec![],
            symbol_stack: vec![],
            module,
        }
    }

    pub fn errors(&self) -> &Vec<CheckError> {
        &self.errors
    }

    pub fn push_stack(&mut self) {
        let symbols: HashMap<&String, (TypeValue, bool)> = HashMap::new();
        self.symbol_stack.push(symbols);
    }

    pub fn pop_stack(&mut self) {
        self.symbol_stack.pop();
    }

    pub fn get_symbol(&self, key: &String) -> Option<&(TypeValue, bool)> {
        self.symbol_stack.last().unwrap().get(key)
    }

    fn insert_symbol(&mut self, key: &'a String, value: (TypeValue, bool)) {
        self.symbol_stack.last_mut().unwrap().insert(key, value);
    }

    pub fn print_interrupts(&self) {
        let file = fs::read_to_string(&self.module.name).unwrap();

        let mut summation = 0usize;
        for (i, l) in file.lines().enumerate() {
            let l_len = l.len();

            for error in &self.errors {
                if error.location.rows.0 == i {
                    let start = error.location.span.start;
                    let end = error.location.span.end;

                    let offset = start - summation;
                    let repeat = end - start;
                    let message_offset = (i + 1).to_string().len();

                    // TODO: Unfuck this
                    let errstr = format!(
                        "\x1b[31mError in {}:{}:{}\x1b[0m\n\x1b[34m{} |\x1b[0m {}\n\x1b[34m-{}| {}\x1b[31m{}\n\x1b[34m{}\x1b[0m\n",
                        self.module.name,
                        i+1,
                        offset,
                        i + 1,
                        l,
                        " ".repeat(message_offset),
                        " ".repeat(offset),
                        "~".repeat(repeat),
                        error.message,
                    );

                    println!("{}", errstr);
                }
            }

            summation += l_len + 1;
        }
    }

    pub fn types(&mut self) -> MiddleIR {
        let mut middle_ir = MiddleIR::new();

        for (name, (func_node, location)) in &self.module.fn_defns {
            match self.module.fn_decls.get(name) {
                Some((_type, _)) => {
                    let function = self.fn_ty(name, func_node, _type);
                    middle_ir.insert_function(function);
                }
                None => {
                    let error = CheckError {
                        location: location.clone(),
                        message: format!(
                            "No function declaration found for definition: `{}`",
                            name
                        ),
                    };

                    self.errors.push(error);
                }
            }
        }

        middle_ir
    }

    pub fn fn_ty(&mut self, name: &String, func_node: &'a FuncNode, _type: &'a Type) -> Function {
        let mut function = Function::default();
        function.name = name.clone();

        func_node.errors.iter().for_each(|error| {
            let check_error = CheckError::from_parse_error(error);
            self.errors.push(check_error);
        });

        if func_node.errors.len() > 0 {
            return function;
        }

        self.push_stack();

        let stack = self.symbol_stack.last_mut().unwrap();

        if let TypeValue::Func(params, ret_type) = &_type.type_value {
            function.return_type = *ret_type.clone();

            for i in 0..func_node.args.len() {
                let arg = &func_node.args[i];
                let _type = params[i].clone();

                function.params.push((arg.clone(), _type.clone()));

                stack.insert(arg, (_type, true));
            }
        } else {
            todo!()
        }

        func_node.block.iter().for_each(|stmt| {
            let stmt = self.stmt_ty(stmt);
            function.block.push(stmt);
        });

        self.pop_stack();

        function
    }

    pub fn stmt_ty(&mut self, stmt: &'a Stmt) -> Statement {
        match stmt {
            Stmt::Expr(expr, _location) => {
                let (expr, _) = self.expr_ty(expr);
                let out = shunting_yard_this_mf(expr);

                Statement::Expr(out)
            }
            Stmt::Var(var) => Statement::Var(self.var_ty(var)),
        }
    }

    pub fn expr_ty(&mut self, expr: &Expr) -> (Vec<Expression>, TypeValue) {
        let mut output: Vec<Expression> = vec![];

        match expr {
            Expr::Add(lhs, rhs, location) => {
                let (mut lhs_expr, lhs_type) = self.expr_ty(lhs);
                let (mut rhs_expr, rhs_type) = self.expr_ty(rhs);

                output.append(&mut lhs_expr);
                output.push(Expression::Add);
                output.append(&mut rhs_expr);

                // TODO: Also do double check, while this may solve certain scenarios it doesn't solve all.
                if lhs_type == TypeValue::Undefined {
                    return (vec![], rhs_type);
                }

                if rhs_type == TypeValue::Undefined {
                    return (vec![], lhs_type);
                }

                if lhs_type != rhs_type {
                    let error = CheckError {
                        location: location.clone(),
                        message: format!(
                            "Cannot `{:?} + {:?}` as these types do not match.",
                            lhs_type, rhs_type
                        ),
                    };

                    self.errors.push(error);

                    (vec![], lhs_type)
                } else {
                    (output, lhs_type)
                }
            }
            Expr::Min(lhs, rhs, location) => {
                let (mut lhs_expr, lhs_type) = self.expr_ty(lhs);
                let (mut rhs_expr, rhs_type) = self.expr_ty(rhs);

                output.append(&mut lhs_expr);
                output.push(Expression::Min);
                output.append(&mut rhs_expr);

                if lhs_type != rhs_type {
                    let error = CheckError {
                        location: location.clone(),
                        message: format!(
                            "Cannot `{:?} - {:?}` as these types do not match.",
                            lhs_type, rhs_type
                        ),
                    };

                    self.errors.push(error);

                    (vec![], TypeValue::Void)
                } else {
                    (output, lhs_type)
                }
            }
            Expr::Mul(lhs, rhs, location) => {
                let (mut lhs_expr, lhs_type) = self.expr_ty(lhs);
                let (mut rhs_expr, rhs_type) = self.expr_ty(rhs);

                output.append(&mut lhs_expr);
                output.push(Expression::Mul);
                output.append(&mut rhs_expr);

                if lhs_type != rhs_type {
                    let error = CheckError {
                        location: location.clone(),
                        message: format!(
                            "Cannot `{:?} * {:?}` as these types do not match.",
                            lhs_type, rhs_type
                        ),
                    };

                    self.errors.push(error);

                    (vec![], TypeValue::Void)
                } else {
                    (output, lhs_type)
                }
            }
            Expr::Div(lhs, rhs, location) => {
                let (mut lhs_expr, lhs_type) = self.expr_ty(lhs);
                let (mut rhs_expr, rhs_type) = self.expr_ty(rhs);

                output.append(&mut lhs_expr);
                output.push(Expression::Div);
                output.append(&mut rhs_expr);

                if lhs_type != rhs_type {
                    let error = CheckError {
                        location: location.clone(),
                        message: format!(
                            "Cannot `{:?} / {:?}` as these types do not match.",
                            lhs_type, rhs_type
                        ),
                    };

                    self.errors.push(error);

                    (vec![], TypeValue::Void)
                } else {
                    (output, lhs_type)
                }
            }
            Expr::Int(int, _) => {
                let expr = Expression::Literal(Literal::Int(TypeValue::I32, int.to_string()));
                (vec![expr], TypeValue::I32)
            }
            Expr::Identifier(ident, location) => {
                // TODO: Temporary asf.
                let name = &ident.name[0];

                match self.get_symbol(name) {
                    Some((ty, is_function_param)) => {
                        let literal =
                            Literal::Identifier(ty.clone(), name.clone(), *is_function_param);
                        let expr = Expression::Literal(literal);

                        (vec![expr], ty.clone())
                    }
                    None => {
                        let error = CheckError {
                            location: location.clone(),
                            message: format!("Identifier `{}` is undefined at this point.", name),
                        };

                        self.errors.push(error);

                        (vec![], TypeValue::Undefined)
                    }
                }
            }
            Expr::FuncCall(name, args, location) => self.func_call_expr_ty(name, args, location),
            e => {
                println!("Unhandled expression: {:?}", e);
                todo!()
            }
        }
    }

    fn func_call_expr_ty(
        &mut self,
        name: &Name,
        args: &Vec<Expr>,
        _location: &Location,
    ) -> (Vec<Expression>, TypeValue) {
        // TODO: Don't do this weird "Name" shit...
        let tmp_name = name.name[0].clone();

        let (func_decl, _) = self.module.fn_decls.get(&tmp_name).unwrap();
        let (func_defn, _) = self.module.fn_defns.get(&tmp_name).unwrap();

        let mut mdir_params: Vec<Expression> = vec![];

        if let TypeValue::Func(params, ret_ty) = &func_decl.type_value {
            for (i, arg) in args.iter().enumerate() {
                let param = &params[i];
                let (mut arg_expr, arg_type) = self.expr_ty(arg);

                mdir_params.append(&mut arg_expr);

                if param != &arg_type {
                    let param_name = &func_defn.args[i];

                    let error = CheckError::new(
                        arg.get_location(),
                        format!(
                            "Argument `{}` in call to `{}` is incorrect, expected `{:?}` but found `{:?}`.",
                            param_name, tmp_name, param, arg_type
                        ),
                    );

                    self.errors.push(error);
                }
            }

            let call = Expression::Call(*ret_ty.clone(), tmp_name.clone(), mdir_params);
            (vec![call], *ret_ty.clone())
        } else {
            (vec![], TypeValue::Void)
        }
    }

    pub fn var_ty(&mut self, var: &'a Var) -> MdIrVar {
        match () {
            _ if var.is_decl && var.rhs.is_void() => {
                self.insert_symbol(
                    var.lhs.name.last().unwrap(),
                    (var._type.type_value.clone(), false),
                );

                let _name = var.lhs.name.last().unwrap().clone();
                let _ty = var._type.type_value.clone();

                // Decl::new(name, ty);
                todo!()
            }
            _ if var.is_decl => {
                // TODO: Do not unwrap here.
                let (rhs_expr, rhs_type) = self.expr_ty(&var.rhs);
                let out = shunting_yard_this_mf(rhs_expr);

                self.insert_symbol(var.lhs.name.last().unwrap(), (rhs_type.clone(), false));

                let name = var.lhs.name.last().unwrap().clone();
                MdIrVar::new(name, out, rhs_type)
            }
            _ => {
                let key = var.lhs.name.last().unwrap();
                // TODO: Actually make this function do stuff :3

                match self.get_symbol(key) {
                    None => {
                        let error = CheckError {
                            location: var.lhs.location.clone(),
                            message: format!(
                                "Attempted to assign to `{}`, but it was never decleared.",
                                key
                            ),
                        };

                        self.errors.push(error);
                    }
                    _ => (),
                };

                todo!()
            }
        }
    }
}
