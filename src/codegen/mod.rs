use crate::{
    ast::TypeValue,
    checker::mdir::{Expr, Expression, Function, Literal, MiddleIR, Statement},
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
    let block = function_block_to_llvm_ir(&function.block);

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

fn function_block_to_llvm_ir(block: &Vec<Statement>) -> String {
    let mut result = String::from("entry:\n");

    for statement in block {
        match statement {
            Statement::Var(var) => {
                todo!()
            }
            Statement::Expr(expr) => result += &expr_to_llvm_ir(expr, 0),
        }
    }

    result
}

fn expr_to_llvm_ir(expr: &Expression, depth: u32) -> String {
    match &expr.inner {
        Expr::Literal(lit) => match lit {
            Literal::Int(int) => int.to_owned(),
            Literal::Identifier(ident) => format!("%{ident}"),
        },
        Expr::Add(lhs, rhs) => {
            let ty = type_value_to_llvm_ir(&lhs.ty);

            let lhs_ir = expr_to_llvm_ir(&lhs, depth + 1);
            let rhs_ir = expr_to_llvm_ir(&rhs, depth + 2);

            format!("    %add_expr_{depth} = add {ty} {lhs_ir}, {rhs_ir}\n")
        }
        expr => {
            println!("Unhandled expression: {:?}", expr);
            todo!()
        }
    }
}

fn type_value_to_llvm_ir(type_value: &TypeValue) -> &str {
    match type_value {
        TypeValue::I32 => "i32",
        tyv => {
            println!("Unhandled TypeValue: `{:?}`", tyv);
            todo!()
        }
    }
}
