use std::fs;

use checker::Checker;
use lexer::Lexer;
use parser::{module, Input};

pub mod ast;
pub mod checker;
pub mod lexer;
pub mod parser;

fn main() {
    let input = fs::read_to_string("examples/main.gh").expect("Cannot find `examples/main.gh`");

    let mut lexer = Lexer::new(&input);
    let tokens = lexer.lex();
    let mut parser = Input::new(tokens);

    let module = module(&mut parser, "examples/main.gh".to_string());

    {
        let mut checker = Checker::new(&module);
        checker.types();
        
        checker.print_interrupts();
    }
}
