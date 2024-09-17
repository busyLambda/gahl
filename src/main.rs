use std::{
    env::args, fs::{self, File}, io::Write, os::unix::process::CommandExt, process::{self, exit, Command}
};

use checker::Checker;
use codegen::CodeGen;
use lexer::Lexer;
use parser::{module, Input};

pub mod ast;
pub mod checker;
pub mod codegen;
pub mod lexer;
pub mod parser;

fn main() {
    let main_file = args().nth(1).expect("No input file provided");

    let input = fs::read_to_string(main_file).expect("Cannot find `examples/main.gh`");

    println!("Lexing...");
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.lex();
    let mut parser = Input::new(tokens);

    println!("Parsing...");
    let module = module(&mut parser, "examples/main.gh".to_string());

    let mut checker = Checker::new(&module);
    println!("Type checking...");
    let mdir = checker.types();

    if checker.errors().len() != 0 {
        checker.print_interrupts();
        exit(1);
    }

    let mut codegen = CodeGen::new(mdir);
    println!("Emitting LLVM IR...");
    codegen.compile();

    // println!("\x1b[33mLLVM IR:\n\x1b[34m{}\x1b[0m", codegen.llvm_ir());

    let mut file = File::create("out.ll").unwrap();

    file.write_all(codegen.llvm_ir().as_bytes()).unwrap();

    println!("Bulding...");
    println!("[1/3] Assembling LLVM IR to bitcode...");
    Command::new("llvm-as").args(["out.ll", "-o", "out.bs"]).status().unwrap();
    println!("[2/3] Linking bitcode to object file...");
    Command::new("clang").args(["-c", "out.ll", "-o", "out.o"]).status().unwrap();
    println!("[3/3] Linking object file to executable...");
    Command::new("clang").args(["out.o", "-o", "out"]).status().unwrap();
}
