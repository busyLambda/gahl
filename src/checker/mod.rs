use std::{collections::HashMap, fmt::format, fs};

use crate::ast::{Expr, FuncNode, Location, Module, Name, Stmt, Type, TypeValue, Var, VarLhs};

#[derive(Debug)]
pub struct CheckError {
    location: Location,
    message: String,
}

impl CheckError {
    pub fn new(location: Location, message: String) -> Self {
        Self { location, message }
    }
}

pub struct Checker<'a> {
    errors: Vec<CheckError>,
    warnings: Vec<CheckError>,
    module: &'a Module,
    symbol_stack: Vec<HashMap<&'a String, TypeValue>>,
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
        let symbols: HashMap<&String, TypeValue> = HashMap::new();
        self.symbol_stack.push(symbols);
    }

    pub fn pop_stack(&mut self) {
        self.symbol_stack.pop();
    }

    pub fn get_symbol(&self, key: &String) -> Option<&TypeValue> {
        self.symbol_stack.last().unwrap().get(key)
    }

    fn insert_symbol(&mut self, key: &'a String, value: TypeValue) {
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

                    // TODO: Unfuck this
                    let errstr = format!(
                        "\x1b[31mError in {}\x1b[0m\n\x1b[34m{} |\x1b[0m {}\n\x1b[34m- | {}\x1b[31m{}\n\x1b[34m{}\x1b[0m\n",
                        self.module.name,
                        i + 1,
                        l,
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

    pub fn types(&mut self) {
        for (name, (func_node, location)) in &self.module.fn_defns {
            match self.module.fn_decls.get(name) {
                Some((_type, _)) => {
                    self.fn_ty(name, func_node, _type);
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
    }

    pub fn fn_ty(&mut self, name: &String, func_node: &'a FuncNode, _type: &'a Type) {
        self.push_stack();

        let stack = self.symbol_stack.last_mut().unwrap();

        if let TypeValue::Func(params, ret_type) = &_type.type_value {
            for i in 0..func_node.args.len() {
                let arg = &func_node.args[i];
                let _type = params[i].clone();

                stack.insert(arg, _type);
            }
        } else {
            todo!()
        }

        func_node.block.iter().for_each(|stmt| {
            self.stmt_ty(stmt);
        });

        self.pop_stack();
    }

    pub fn stmt_ty(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Expr(expr, location) => {
                self.expr_ty(expr);
            }
            Stmt::Var(var) => {
                self.var_ty(var);
            }
        }
    }

    pub fn expr_ty(&mut self, expr: &Expr) -> Option<TypeValue> {
        match expr {
            Expr::Add(lhs, rhs, location) => {
                let lhs_ty = self.expr_ty(lhs)?;
                let rhs_ty = self.expr_ty(rhs)?;

                if lhs_ty != rhs_ty {
                    let error = CheckError {
                        location: location.clone(),
                        message: format!(
                            "Cannot `{:?} + {:?}` as these types do not match.",
                            lhs_ty, rhs_ty
                        ),
                    };

                    self.errors.push(error);
                    None
                } else {
                    Some(lhs_ty)
                }
            }
            Expr::Int(_, _) => Some(TypeValue::I32),
            Expr::Identifier(ident, location) => {
                // TODO: Temporary asf.
                let name = &ident.name[0];

                match self.get_symbol(name) {
                    Some(ty) => Some(ty.clone()),
                    None => {
                        let error = CheckError {
                            location: location.clone(),
                            message: format!("Identifier `{}` is undefined at this point.", name),
                        };

                        self.errors.push(error);

                        None
                    }
                }
            }
            Expr::FuncCall(name, args, location) => self.func_call_expr_ty(name, args, location),
            e => {
                println!("Unhandled expression: {:?}", expr);
                todo!()
            }
        }
    }

    fn func_call_expr_ty(
        &mut self,
        name: &Name,
        args: &Vec<Expr>,
        _location: &Location,
    ) -> Option<TypeValue> {
        // TODO: Don't do this weird "Name" shit...
        let tmp_name = &name.name[0];

        let (func_decl, _) = self.module.fn_decls.get(tmp_name)?;
        let (func_defn, _) = self.module.fn_defns.get(tmp_name)?;

        if let TypeValue::Func(params, ret_ty) = &func_decl.type_value {
            for (i, arg) in args.iter().enumerate() {
                let param = &params[i];
                let arg_ty = self.expr_ty(arg)?;

                if param != &arg_ty {
                    let param_name = &func_defn.args[i];

                    let error = CheckError::new(
                        name.location.clone(),
                        format!(
                            "Argument `{}` in call to `{}` is incorrect, expected `{:?}` but found `{:?}`.",
                            param_name, tmp_name, param, arg_ty
                        ),
                    );

                    self.errors.push(error);
                }
            }

            Some(*ret_ty.clone())
        } else {
            None
        }
    }

    pub fn var_ty(&mut self, var: &'a Var) {
        match () {
            _ if var.is_decl && var.rhs.is_void() => {
                self.insert_symbol(var.lhs.name.last().unwrap(), var._type.type_value.clone());
            }
            _ if var.is_decl => {
                // TODO: Do not unwrap here.
                let rhs_type = match self.expr_ty(&var.rhs) {
                    Some(ty) => ty,
                    None => TypeValue::Void,
                };
                self.insert_symbol(var.lhs.name.last().unwrap(), rhs_type)
            }
            _ => {
                let key = var.lhs.name.last().unwrap();
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
                }
            }
        };
    }
}
