use std::{
    collections::HashMap,
    env::args,
    fs::{self, DirBuilder, File},
    io::Write,
    process::{exit, Command},
    sync::Arc,
};

use ast::Module;
use checker::analyzer::Analyzer;
use clap::Parser;
use cli::{Args, SubCommand};
use parser::Parser as GahlParser;

pub mod ast;
pub mod checker;
mod cli;
pub mod codegen;
pub mod config;
pub mod docgen;
pub mod lexer;
pub mod parser;

fn main() {
    let args = Args::parse();

    let config = match config::parse_config() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error parsing `config.toml` file: {}", err);
            exit(1);
        }
    };

    match args.subcmd {
        SubCommand::New { project_name } => {
            match DirBuilder::new().create(&project_name) {
                Ok(_) => (),
                Err(err) => {
                    println!("Error: {}", err.to_string());
                    exit(1);
                }
            };

            let mut main_file = match File::create(format!("{project_name}/main.gh")) {
                Ok(f) => f,
                Err(err) => {
                    println!("Error: {}", err.to_string());
                    exit(1);
                }
            };

            let main_contents = "import {}\n\nmain : fn() void\nmain = fn() {}\n";

            match main_file.write_all(main_contents.as_bytes()) {
                Ok(_) => (),
                Err(err) => {
                    println!("Error: {}", err.to_string());
                    exit(1);
                }
            };

            let mut config_file = match File::create(format!("{project_name}/config.toml")) {
                Ok(f) => f,
                Err(err) => {
                    println!("Error: {}", err.to_string());
                    exit(1);
                }
            };

            let config_contents = "[project]\nname=\"\"\nauthor=\"\"\nexec_entry=\"\"\n";

            match config_file.write_all(config_contents.as_bytes()) {
                Ok(_) => (),
                Err(err) => {
                    println!("Error: {}", err.to_string());
                    exit(1);
                }
            };

            println!("Project \x1b[1m\x1b[33m`{project_name}`\x1b[0m created successfully!");
        }
        SubCommand::Build | SubCommand::Run => {
            let entry_file = &config.project.exec_entry;

            let mut parser = GahlParser::new(&entry_file);

            let modules = parser.parse(&entry_file);
            let modules = Arc::new(
                modules
                    .into_iter()
                    .map(|(n, m)| (n.clone(), Arc::new(m)))
                    .collect::<HashMap<String, Arc<Module>>>(),
            );

            let mut analyzer = Analyzer::new();
            match analyzer.analyze(modules.clone()) {
                Ok(_) => {
                    println!("Analyzer finished!");
                }
                Err(_) => {
                    eprintln!("Analyzer finished with errors!");
                    exit(1);
                }
            };
        }
        _ => todo!(),
    };

    // let mut checker = Checker::new(&module);
    // let mdir = checker.types();

    // if checker.errors().len() != 0 {
    //     checker.print_interrupts();
    //     exit(1);
    // }

    // let subcommand = args().nth(1).unwrap();
    // if subcommand == "build" || subcommand == "b" {
    //     let mut codegen = CodeGen::new(mdir);
    //     println!("[1/3] Emitting LLVM IR...");
    //     codegen.compile();

    //     let mut file = File::create("out.ll").unwrap();

    //     file.write_all(codegen.llvm_ir().as_bytes()).unwrap();

    //     println!("[2/3] Linking bitcode to object file...");
    //     Command::new("clang")
    //         .args(["-c", "out.ll", "-o", "out.o"])
    //         .status()
    //         .unwrap();
    //     println!("[3/3] Linking object file to executable...");

    //     let mut libs: Vec<String> = vec![];
    //     if let Some(clibs) = config.clibs {
    //         for clib in clibs.clibs {
    //             libs.push(clib.path);
    //         }
    //     }

    //     let mut args: Vec<String> = vec!["-o", "out", "out.o"]
    //         .iter()
    //         .map(|s| s.to_string())
    //         .collect();

    //     args.append(&mut libs);

    //     let mut command = Command::new("clang");
    //     command.args(args);
    //     command.status().unwrap();
    // } else if subcommand == "doc" || subcommand == "d" {
    //     let docs = gen_docs(mdir);

    //     println!("Docs:\n{}", docs);
    // } else {
    //     println!("Invalid subcommand: `{subcommand}`");
    //     exit(1);
    // }
}
