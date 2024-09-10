use std::{fs, process::exit};

use checker::Checker;
use codegen::CodeGen;
use lexer::Lexer;
use parser::{module, Input};

pub mod ast;
pub mod checker;
pub mod lexer;
pub mod parser;
pub mod codegen;

fn main() {
    let input = fs::read_to_string("examples/main.gh").expect("Cannot find `examples/main.gh`");

    let mut lexer = Lexer::new(&input);
    let tokens = lexer.lex();
    let mut parser = Input::new(tokens);

    let module = module(&mut parser, "examples/main.gh".to_string());

    let mut checker = Checker::new(&module);
    let mdir = checker.types();

    if checker.errors().len() != 0 {
        checker.print_interrupts();
        exit(1);
    }
    
    let mut codegen = CodeGen::new(mdir);
    codegen.compile();
    
    println!("\x1b[33mLLVM IR:\n\x1b[34m{}\x1b[0m", codegen.llvm_ir())
}
