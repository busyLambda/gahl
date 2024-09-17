use std::collections::VecDeque;

use crate::{
    ast::TypeValue,
    checker::mdir::{Expression, Function, Literal, MiddleIR, Statement},
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
        self.mdir.functions().iter().for_each(|(_, function)| {
            self.llvm_ir += &function_to_llvm_ir(function);
        })
    }

    pub fn llvm_ir(&self) -> &String {
        &self.llvm_ir
    }
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
                let (expr_ir, name) = &expr_to_llvm_ir(expr, context);

                let ty = "i32";

                if return_type == &TypeValue::Void {
                    result += expr_ir;
                    result += &format!("    ret void\n");
                    return result;
                };

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
                let (expr_ir, name) = &expr_to_llvm_ir(&var.rhs, context);
                let ty = type_value_to_llvm_ir(&var.ty);

                result += "    ; var\n";

                match name {
                    Some(name) => {
                        result += expr_ir;
                        result += &format!("    %{} = alloca {ty}\n", var.lhs);
                        result += &format!("    store {ty} {name}, {ty}* %{}\n", var.lhs);
                    }
                    None => {
                        result += &format!("    %{} = alloca {ty}\n", var.lhs);
                        result += &format!("    store {ty} {expr_ir}, {ty}* %{}\n", var.lhs);
                    }
                }
            }
        }
    }

    result
}

fn expr_to_llvm_ir(expr: &VecDeque<Expression>, context: &String) -> (String, Option<String>) {
    let mut result = String::new();
    let mut prev_name: Option<String> = None;

    let mut expr_stack: Vec<&Expression> = vec![];

    if expr.len() == 1 {
        if let Expression::Literal(lit) = &expr[0] {
            return (lit.to_string(), None);
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

                    let rhs_ir: String;
                    match rhs {
                        Literal::Identifier(_, value, false) => {
                            let param_name = format!("%{value}_load");
                            result += &format!("    {param_name} = ");
                            rhs_ir = param_name;
                        }
                        _ => {
                            rhs_ir = rhs.to_string();
                        }
                    }

                    let lhs_ir: String;
                    match lhs {
                        Literal::Identifier(ty, value, false) => {
                            result += "    ; Load\n";
                            let _type = type_value_to_llvm_ir(ty);
                            let param_name = format!("%{value}_load");
                            result +=
                                &format!("    {param_name} = load {_type}, {_type}* %{value}\n");
                            lhs_ir = param_name;
                        }
                        _ => {
                            lhs_ir = lhs.to_string();
                        }
                    }

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

                    result += &format!(
                        "    {} = add i32 {}, {}\n",
                        final_name,
                        prev_name.unwrap(),
                        rhs.to_string()
                    );
                    prev_name = Some(final_name);
                }
            }
        } else {
            expr_stack.push(e);
        }
    }

    (result, prev_name)
}

fn type_value_to_llvm_ir(type_value: &TypeValue) -> &str {
    match type_value {
        TypeValue::Void => "void",
        TypeValue::I32 => "i32",
        tyv => {
            println!("Unhandled TypeValue: `{:?}`", tyv);
            todo!()
        }
    }
}
