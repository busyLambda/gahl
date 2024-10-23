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
use codegen::compile;
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
            let modules = match analyzer.analyze(modules.clone()) {
                Ok(m) => {
                    println!("Analyzer finished!");
                    m
                }
                Err(_) => {
                    eprintln!("Analyzer finished with errors!");
                    exit(1);
                }
            };

            let mut libs: Vec<String> = vec![];
            if let Some(clibs) = config.clibs {
                for clib in clibs.clibs {
                    libs.push(clib.path);
                }
            }

            compile(modules, libs, &config.project.name);
            
            if let SubCommand::Run = args.subcmd {
                let path = format!("./build/{}", config.project.name);
                println!("\nRunning: {path}\n");
                Command::new(path).status().unwrap();
            }
        }
        _ => todo!(),
    };

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
