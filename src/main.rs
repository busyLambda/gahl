use std::{
    env::args,
    fs::{self, File},
    io::Write,
    process::{exit, Command},
};

use checker::Checker;
use codegen::CodeGen;
use docgen::gen_docs;
use lexer::Lexer;
use parser::{module, Input, Parser};

pub mod ast;
pub mod checker;
pub mod codegen;
pub mod config;
pub mod docgen;
pub mod lexer;
pub mod parser;

fn main() {
    let main_file = args()
        .nth(2)
        .expect("No input file provided, expected `<subcommand> <filepath>`.");

    if args().nth(1).unwrap() == "new" {
        fs::create_dir(&main_file).expect("Error creating project directory.");

        let contents = format!("[project]name = \"{}\"\n", &main_file);

        let mut config_file = fs::File::open(format!("{}/Gahlconf.toml", main_file)).unwrap();
        config_file.write_all(contents.as_bytes()).unwrap();

        exit(0);
    }

    let config = match config::parse_config() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error parsing `config.toml` file: {}", err);
            exit(1);
        }
    };

    let mut parser = Parser::new(&main_file);

    let module = parser.parse(&main_file);

    let mut checker = Checker::new(&module);
    let mdir = checker.types();

    if checker.errors().len() != 0 {
        checker.print_interrupts();
        exit(1);
    }

    let subcommand = args().nth(1).unwrap();
    if subcommand == "build" || subcommand == "b" {
        let mut codegen = CodeGen::new(mdir);
        println!("[1/3] Emitting LLVM IR...");
        codegen.compile();

        let mut file = File::create("out.ll").unwrap();

        file.write_all(codegen.llvm_ir().as_bytes()).unwrap();

        println!("[2/3] Linking bitcode to object file...");
        Command::new("clang")
            .args(["-c", "out.ll", "-o", "out.o"])
            .status()
            .unwrap();
        println!("[3/3] Linking object file to executable...");

        let mut libs: Vec<String> = vec![];
        if let Some(clibs) = config.clibs {
            for clib in clibs.clibs {
                libs.push(clib.path);
            }
        }

        let mut args: Vec<String> = vec!["-o", "out", "out.o"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        args.append(&mut libs);

        let mut command = Command::new("clang");
        command.args(args);
        command.status().unwrap();
    } else if subcommand == "doc" || subcommand == "d" {
        let docs = gen_docs(mdir);

        println!("Docs:\n{}", docs);
    } else {
        println!("Invalid subcommand: `{subcommand}`");
        exit(1);
    }
}
