use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::Write,
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        Arc, Mutex,
    },
    thread,
};

use crate::{
    ast::TypeValue,
    checker::mdir::{Expression, ExternFunction, Function, Literal, MiddleIR, Statement},
};

struct VarCounter {
    count: u32,
    var_mapping: HashMap<String, u32>,
}

impl VarCounter {
    fn use_c(&mut self) -> u32 {
        let result = self.count;
        self.count += 1;
        result
    }

    fn new(count: u32) -> Self {
        VarCounter { count, var_mapping: HashMap::new() }
    }
    
    fn insert(&mut self, name: String, id: u32) {
        self.var_mapping.insert(name, id);
    }
    
    fn get(&self, name: &String) -> u32 {
        self.var_mapping.get(name).unwrap().clone()
    }
}

pub fn compile(modules: HashMap<String, MiddleIR>, mut libs: Vec<String>, project_name: &String) {
    let results = thread::spawn(move || {
        let results = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
        let task_counter = Arc::new(AtomicUsize::new(modules.len()));

        modules.into_iter().for_each(|(name, module)| {
            let task_counter_c = task_counter.clone();
            let results_c = results.clone();
            thread::spawn(move || {
                let mut codegen = CodeGen::new(module, name);
                codegen.compile();

                results_c
                    .lock()
                    .unwrap()
                    .push((codegen.name, codegen.llvm_ir));

                task_counter_c.fetch_sub(1, SeqCst)
            });
        });

        loop {
            if task_counter.load(SeqCst) == 0 {
                break;
            }
        }

        results
    });

    let results = Arc::try_unwrap(results.join().unwrap())
        .unwrap()
        .into_inner()
        .unwrap();

    let mut frags = vec![];

    results.iter().for_each(|(name, llvm_ir)| {
        let object_file_path = format!("build/{}.o", name.replace('/', ""));

        frags.push(object_file_path.clone());

        let output_path = format!("build/{}.ll", name.replace('/', ""));
        let mut file = File::create(&output_path).unwrap();
        file.write_all(llvm_ir.as_bytes()).unwrap();

        let args = ["-c", output_path.as_str(), "-o", object_file_path.as_str()];

        Command::new("clang").args(args).status().unwrap();
    });

    frags.append(&mut libs);

    let out_path = format!("build/{project_name}");
    let mut args: Vec<String> = vec!["-o", &out_path]
        .iter()
        .map(|s| s.to_string())
        .collect();

    args.append(&mut frags);

    Command::new("clang").args(args).status().unwrap();
}

pub struct CodeGen {
    name: String,
    mdir: MiddleIR,
    llvm_ir: String,
}

impl CodeGen {
    pub fn new(mdir: MiddleIR, name: String) -> Self {
        Self {
            name,
            mdir,
            llvm_ir: String::new(),
        }
    }

    pub fn compile(&mut self) {
        self.llvm_ir += "target triple = \"x86_64-pc-linux-gnu\"\n";
        self.llvm_ir += "declare ptr @GC_malloc(i64)\n";
        self.llvm_ir += "declare void @GC_init()\n";

        self.mdir.externs().iter().for_each(|f| {
            self.llvm_ir += &extern_to_llvm_ir(f);
        });
        self.mdir
            .imported_functions()
            .iter()
            .for_each(|(n, f)| self.llvm_ir += &imported_function_to_llvm_ir(n, f));
        self.mdir.functions().iter().for_each(|(_, function)| {
            self.llvm_ir += &function_to_llvm_ir(function);
        });
    }

    pub fn llvm_ir(&self) -> &String {
        &self.llvm_ir
    }
}

fn imported_function_to_llvm_ir(
    name: &String,
    imported_function: &(Vec<(String, TypeValue)>, TypeValue),
) -> String {
    let (params, return_type) = imported_function;
    let return_type = type_value_to_llvm_ir(return_type);
    let params = function_params_to_llvm_ir(params);
    format!("declare {return_type} @{name}({params})\n")
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

    let mut var_counter = VarCounter::new(0);

    if block.len() == 0 {
        return "    ret void\n".to_string();
    }

    for (i, stmt) in block.iter().enumerate() {
        match stmt {
            Statement::Expr(expr) => {
                let (expr_ir, name, _type) =
                    &expr_to_llvm_ir(expr, context, i, false, &mut var_counter);

                let ty = type_value_to_llvm_ir(return_type);

                if return_type == &TypeValue::Void && i == block.len() - 1 {
                    result += expr_ir;
                    break;
                }

                if i == block.len() - 1 {
                    match name {
                        Some(name) => {
                            result += expr_ir;
                            result += &format!("    ret {ty} {name}\n");
                        }
                        None => {
                            result += &format!("    ret {ty} {expr_ir}\n");
                        }
                    }
                } else {
                    result += expr_ir;
                }
            }
            Statement::Var(var) => {
                let context = &format!("{}_var", var.lhs);
                let (expr_ir, name, _type) =
                    &expr_to_llvm_ir(&var.rhs, context, i, true, &mut var_counter);
                let ty = type_value_to_llvm_ir(&var.ty);

                result += "    ; var\n";

                let var_id = var_counter.use_c();
                var_counter.insert(var.lhs.clone(), var_id);
                result += &format!("    %{var_id} = alloca ptr\n",);

                // %{context}_alloca_ptr
                let var_alloca_ptr_id = var_counter.use_c();
                result += &format!("    %{var_alloca_ptr_id} = call ptr @GC_malloc(i64 8)\n");
                result += &format!("    store {ty} 0, {ty}* %{var_alloca_ptr_id}\n");

                // %{context}_alloca_load
                let var_alloca_load_id = var_counter.use_c();
                result += &format!("    %{var_alloca_load_id} = load ptr, ptr %{var_id}\n");

                match name {
                    Some(name) => {
                        if &var.ty == &TypeValue::String {
                            result += &format!("    %{var_id} = {expr_ir}\n");
                            result += &format!("    store {name}, ptr %{var_id}\n");
                        } else {
                            // println!("store {ty} {name}, {ty} %{context}_alloca_load");
                            result += expr_ir;
                            result += &format!("    store {ty} {name}, ptr %{var_alloca_load_id}\n")
                            // result += &format!("    %{} = alloca {ty}\n", var.lhs);
                            // result += &format!("    store {ty} {name}, {ty}* %{}\n", var.lhs);
                        }
                    }
                    None => {
                        result += &format!("    %{var_id} = alloca {ty}\n");
                        result += &format!("    store {ty} {expr_ir}, {ty}* %{var_id}\n");
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
    var_counter: &mut VarCounter,
) -> (String, String, bool) {
    let mut result = String::new();
    let mut ir = String::new();
    let is_final;

    match literal {
        Literal::Int(_ty, int) => {
            let _ty = type_value_to_llvm_ir(_type);
            ir += &format!("{int}");
            is_final = true;
        }
        Literal::Identifier(ty, value, false) => match ty {
            TypeValue::String => {
                let ty = type_value_to_llvm_ir(ty);
                let param_name = format!("%{}", var_counter.use_c());
                result += &format!("    {param_name} = getelementptr inbounds [14 x i8], [14 x i8]* %{value}, i32 0, i32 0\n");
                ir = param_name;
                is_final = true;
            }
            _ => {
                let ty = type_value_to_llvm_ir(ty);
                let value_clone_id = var_counter.use_c();
                let value_clone = format!("%{value_clone_id}");
                result += &format!("    {value_clone} = alloca {ty}\n");
                let value_cpy_clone_id = var_counter.use_c();
                
                let var_id = var_counter.get(&value);
                result += &format!("    %{value_cpy_clone_id} = load ptr, ptr %{var_id}\n");
                result += &format!("    call void @llvm.memcpy.p0.p0.i64(ptr align 4 {value_clone}, ptr align 4 %{value_cpy_clone_id}, i64 4, i1 false)\n");

                let value_clone_load_to_value_id = var_counter.use_c();
                result += &format!(
                    "    %{value_clone_load_to_value_id} = load i32, ptr {value_clone}, align 4\n"
                );
                ir = format!("%{value_clone_load_to_value_id}");
                is_final = true;
            }
        },
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

            let length = null_terminated.len() - 2;

            if is_var {
                result += &format!("alloca [{length} x i8]");
                ir = format!("[{length} x i8] c{null_terminated}",);
                is_final = true;
            } else {
                let string_name = format!("%{}", var_counter.use_c());
                result += &format!("    {string_name} = alloca [{length} x i8]\n");
                result += &format!(
                    "    store [{length} x i8] c{null_terminated}, [{length} x i8]* {string_name}\n"
                );
                let string_name_ptr_id = var_counter.use_c();
                result += &format!("    %{string_name_ptr_id} = getelementptr inbounds [{length} x i8], [{length} x i8]* {string_name}, i32 0, i32 0\n");
                ir = format!("%{string_name_ptr_id}");
                is_final = true;
            }
        }
        Literal::Call(ret_ty, func_name, args) => {
            let ret_ty_ir = type_value_to_llvm_ir(ret_ty);

            let args_ir = args
                .iter()
                .map(|(arg, arg_type_value)| {
                    let (arg_ir, arg_name, arg_type) = expr_to_llvm_ir(arg, context, i, false, var_counter);
                    result += &arg_ir;
                    if let Expression::Literal(Literal::Identifier(_type, name, _)) = &arg[0] {
                        let arg_name = arg_name.unwrap();

                        let ty = type_value_to_llvm_ir(_type);

                        let arg_clone_id = var_counter.use_c();
                        result += &format!("    %{arg_clone_id} = alloca {ty}\n");
                        
                        let name_cpy_clone_id = var_counter.use_c();
                        
                        let name_id = var_counter.get(name);
                        result += &format!("    %{name_cpy_clone_id} = load ptr, ptr %{name_id}\n");
                        result += &format!("    call void @llvm.memcpy.p0.p0.i64(ptr align 4 %{arg_clone_id}, ptr align 4 %{name_cpy_clone_id}, i64 4, i1 false)\n");

                        if let TypeValue::Ptr(_inner_type) = arg_type_value {
                            format!("ptr %{arg_clone_id}")
                        } else {
                            let arg_name_load_to_value_id = var_counter.use_c();
                            result += &format!("    %{arg_name_load_to_value_id} = load i32, ptr %{arg_clone_id}, align 4\n");
                            format!("{ty} %{arg_name_load_to_value_id}")
                        }
                    } else {
                        // TODO: Find out why this is here, I forgor...
                        format!("{arg_type} {}", arg_name.unwrap())
                    }
                })
                .collect::<Vec<String>>()
                .join(", ");

            let call_id = var_counter.use_c();
            result += &format!("    %{call_id} = call {ret_ty_ir} @{func_name}({args_ir})\n");
            ir = format!("%{call_id}");
            is_final = true;
        }
    }

    (result, ir, is_final)
}

fn expr_to_llvm_ir(
    expr: &VecDeque<Expression>,
    context: &String,
    i: usize,
    is_var: bool,
    var_counter: &mut VarCounter,
) -> (String, Option<String>, String) {
    let mut result = String::new();
    let mut prev_name: Option<String> = None;

    let mut expr_stack: Vec<&Expression> = vec![];

    let mut final_type = &TypeValue::Void;

    if expr.len() == 1 {
        if let Expression::Literal(rhs) = &expr[0] {
            let final_name = format!("%{}", var_counter.use_c());

            let _type = type_value_to_llvm_ir(rhs._type()).to_string();

            let (in_ir, expr_ir, is_final) =
                literal_to_llvm_ir(rhs, context, i + 1, rhs._type(), is_var, var_counter);

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
                    let final_name = format!("%{}", var_counter.use_c());
                    i += 1;
                    let _type = type_value_to_llvm_ir(lhs._type());
                    final_type = lhs._type();
                    let lhs_type = lhs._type();
                    let rhs_type = rhs._type();

                    let (in_ir, rhs_ir, _is_final) =
                        literal_to_llvm_ir(rhs, context, i, rhs_type, is_var, var_counter);
                    result += &in_ir;

                    let (in_ir, lhs_ir, _is_final) =
                        literal_to_llvm_ir(lhs, context, i, lhs_type, is_var, var_counter);
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
                        literal_to_llvm_ir(rhs, context, i, rhs_type, is_var, var_counter);
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

fn type_value_to_llvm_ir(type_value: &TypeValue) -> String {
    match type_value {
        TypeValue::Void => "void".to_string(),
        TypeValue::I32 => "i32".to_string(),
        TypeValue::String => "i8*".to_string(),
        TypeValue::Ptr(_inner_ty) => {
            // let inner_ty = type_value_to_llvm_ir(inner_ty);
            // format!("{}*", inner_ty)
            format!("ptr")
        }
        tyv => {
            println!("Unhandled TypeValue: `{:?}`", tyv);
            todo!()
        }
    }
}

fn type_and_expr_to_size(expr: &Expression, type_value: &TypeValue) -> u32 {
    match type_value {
        TypeValue::I8 => 1,
        TypeValue::I16 => 2,
        TypeValue::I32 => 4,
        TypeValue::I64 => 8,
        TypeValue::String => {
            if let Expression::Literal(literal) = expr {
                if let Literal::String(string) = literal {
                    return string.len() as u32;
                }
            }
            0
        }
        _ => todo!(),
    }
}
