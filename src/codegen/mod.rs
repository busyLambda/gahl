use std::{collections::VecDeque, fmt::format};

use crate::{
    ast::TypeValue,
    checker::mdir::{Expression, ExternFunction, Function, Literal, MiddleIR, Statement},
};

pub struct CodeGen {
    mdir: MiddleIR,
    llvm_ir: String,
}

impl CodeGen {
    pub fn new(mdir: MiddleIR) -> Self {
        Self {
            mdir,
            llvm_ir: String::new(),
        }
    }

    pub fn compile(&mut self) {
        self.mdir.externs().iter().for_each(|f| {
            self.llvm_ir += &extern_to_llvm_ir(f);
        });
        self.mdir.functions().iter().for_each(|(_, function)| {
            self.llvm_ir += &function_to_llvm_ir(function);
        });
    }

    pub fn llvm_ir(&self) -> &String {
        &self.llvm_ir
    }
}

fn extern_to_llvm_ir(function: &ExternFunction) -> String {
    let return_type = type_value_to_llvm_ir(&function.return_type);
    let params = function_params_to_llvm_ir(&function.params);
    let name = &function.name;

    format!("declare {return_type} @{name}({params})\n")
}

fn function_to_llvm_ir(function: &Function) -> String {
    let return_type = type_value_to_llvm_ir(&function.return_type);
    let name = &function.name;
    let params = function_params_to_llvm_ir(&function.params);
    let block = function_block_to_llvm_ir(name, &function.block, &function.return_type);

    format!("define {return_type} @{name}({params}) {{\n{block}}}\n")
}

fn function_params_to_llvm_ir(params: &Vec<(String, TypeValue)>) -> String {
    let mut result = String::new();

    let mut is_multi_param = false;

    for (name, ty) in params {
        if is_multi_param {
            result += ", "
        }

        let param_type = type_value_to_llvm_ir(ty);

        result += &format!("{param_type} %{name}");

        is_multi_param = true;
    }

    result
}

fn function_block_to_llvm_ir(
    context: &String,
    block: &Vec<Statement>,
    return_type: &TypeValue,
) -> String {
    let mut result = String::from("entry:\n");

    if block.len() == 0 {
        return "    ret void\n".to_string();
    }

    for stmt in block {
        match stmt {
            Statement::Expr(expr) => {
                let (expr_ir, name, _type) = &expr_to_llvm_ir(expr, context, false);

                let ty = type_value_to_llvm_ir(return_type);

                if return_type == &TypeValue::Void {
                    result += expr_ir;
                    break;
                }

                match name {
                    Some(name) => {
                        result += expr_ir;
                        result += &format!("    ret {ty} {name}\n");
                    }
                    None => {
                        result += &format!("    ret {ty} {expr_ir}\n");
                    }
                }
            }
            Statement::Var(var) => {
                let context = &format!("{}_var", var.lhs);
                let (expr_ir, name, _type) = &expr_to_llvm_ir(&var.rhs, context, true);
                let ty = type_value_to_llvm_ir(&var.ty);

                result += "    ; var\n";

                match name {
                    Some(name) => {
                        if &var.ty == &TypeValue::String {
                            result += &format!("    %{} = {expr_ir}\n", var.lhs);
                            result += &format!("    store {name}, ptr %{}\n", var.lhs);
                        } else {
                            result += expr_ir;
                            result += &format!("    %{} = alloca {ty}\n", var.lhs);
                            result += &format!("    store {ty} {name}, {ty}* %{}\n", var.lhs);
                        }
                    }
                    None => {
                        result += &format!("    %{} = alloca {ty}\n", var.lhs);
                        result += &format!("    store {ty} {expr_ir}, {ty}* %{}\n", var.lhs);
                    }
                }
            }
        }
    }
    if return_type == &TypeValue::Void {
        result += &format!("    ; Automatic void return\n");
        result += &format!("    ret void\n");
    };

    result
}

// fn -> (intermediate_ir, ir, is_final)
fn literal_to_llvm_ir(
    literal: &Literal,
    context: &String,
    i: usize,
    _type: &TypeValue,
    is_var: bool,
) -> (String, String, bool) {
    let mut result = String::new();
    let mut ir = String::new();
    let mut is_final = false;

    match literal {
        Literal::Int(_ty, int) => {
            let _ty = type_value_to_llvm_ir(_type);
            ir += &format!("{int}");
            is_final = true;
        }
        Literal::Identifier(ty, value, false) => {
            let param_name = format!("%{value}_load_{context}_{i}");
            let ty = type_value_to_llvm_ir(ty);
            result += &format!("    {param_name} = load {ty}, {ty}* %{value}\n");
            ir = param_name;
            is_final = true;
        }
        Literal::Identifier(_ty, value, true) => {
            ir = format!("%{}", value);
            is_final = true;
        }
        Literal::String(string) => {
            let null_terminated = format!(
                "{}\0{}",
                &string[..string.len() - 1],
                &string[string.len() - 1..]
            );

            println!("String: {}", null_terminated);

            let length = null_terminated.len() - 2;

            if is_var {
                result += &format!("alloca [{length} x i8]");
                ir = format!("[{length} x i8] c{null_terminated}",);
                is_final = true;
            } else {
                let string_name = format!("%{context}_string_{i}");
                result += &format!("    {string_name} = alloca [{length} x i8]\n");
                result += &format!(
                    "    store [{length} x i8] c{null_terminated}, [{length} x i8]* {string_name}\n"
                );
                result += &format!("    {string_name}_ptr = getelementptr inbounds [{length} x i8], [{length} x i8]* {string_name}, i32 0, i32 0\n");
                ir = string_name;
                is_final = true;
            }
        }
        Literal::Call(ret_ty, name, args) => {
            let ret_ty_ir = type_value_to_llvm_ir(ret_ty);

            let args_ir = args
                .iter()
                .map(|arg| {
                    let (arg_ir, arg_name, _type) = expr_to_llvm_ir(arg, context, false);
                    result += &arg_ir;
                    format!("{_type} {}", arg_name.unwrap())
                })
                .collect::<Vec<String>>()
                .join(", ");

            let call_name = format!("%{context}_call_{i}");
            result += &format!("    {call_name} = call {ret_ty_ir} @{name}({args_ir})\n");
            ir = call_name;
            is_final = true;
        }
    }

    (result, ir, is_final)
}

fn expr_to_llvm_ir(
    expr: &VecDeque<Expression>,
    context: &String,
    is_var: bool,
) -> (String, Option<String>, String) {
    let mut result = String::new();
    let mut prev_name: Option<String> = None;

    let mut expr_stack: Vec<&Expression> = vec![];

    let mut final_type = &TypeValue::Void;

    if expr.len() == 1 {
        if let Expression::Literal(rhs) = &expr[0] {
            let i = 0;
            let final_name = format!("%{}_expr_{}", context, i);

            let _type = type_value_to_llvm_ir(rhs._type()).to_string();

            let (in_ir, expr_ir, is_final) =
                literal_to_llvm_ir(rhs, context, 0, rhs._type(), is_var);

            result += &in_ir;
            if is_final {
                return (result, Some(expr_ir), _type);
            }
            result += &format!("    {} = {}\n", final_name, expr_ir,);
            prev_name = Some(final_name);

            return (result, prev_name, _type);
        }
    }

    let mut i = 0;
    for e in expr.iter() {
        if e.is_op() {
            if expr_stack.len() == 2 {
                let rhs = expr_stack.pop().unwrap();
                let lhs = expr_stack.pop().unwrap();

                use Expression as E;

                if let (E::Literal(lhs), E::Literal(rhs)) = (lhs, rhs) {
                    let final_name = format!("%{}_expr_{}", context, i);
                    i += 1;
                    let _type = type_value_to_llvm_ir(lhs._type());
                    final_type = lhs._type();
                    let lhs_type = lhs._type();
                    let rhs_type = rhs._type();

                    let (in_ir, rhs_ir, _is_final) =
                        literal_to_llvm_ir(rhs, context, i, rhs_type, is_var);
                    result += &in_ir;

                    let (in_ir, lhs_ir, _is_final) =
                        literal_to_llvm_ir(lhs, context, i, lhs_type, is_var);
                    result += &in_ir;

                    result += &format!(
                        "    {} = {e} {} {}, {}\n",
                        final_name, _type, lhs_ir, rhs_ir
                    );
                    prev_name = Some(final_name);
                }
            } else {
                let rhs = expr_stack.pop().unwrap();

                use Expression as E;

                if let E::Literal(rhs) = rhs {
                    let final_name = format!("%{}_expr_{}", context, i);
                    i += 1;
                    let rhs_type = rhs._type();
                    final_type = rhs._type();
                    let _type = type_value_to_llvm_ir(rhs_type);

                    let (in_ir, rhs_ir, _is_final) =
                        literal_to_llvm_ir(rhs, context, i, rhs_type, is_var);
                    result += &in_ir;

                    result += &format!(
                        "    {} = {} i32 {}, {}\n",
                        final_name,
                        e.to_string(),
                        prev_name.unwrap(),
                        rhs_ir,
                    );
                    prev_name = Some(final_name);
                }
            }
        } else {
            expr_stack.push(e);
        }
    }

    let _type = type_value_to_llvm_ir(final_type).to_string();

    (result, prev_name, _type)
}

fn type_value_to_llvm_ir(type_value: &TypeValue) -> &str {
    match type_value {
        TypeValue::Void => "void",
        TypeValue::I32 => "i32",
        TypeValue::String => "i8*",
        tyv => {
            println!("Unhandled TypeValue: `{:?}`", tyv);
            todo!()
        }
    }
}
