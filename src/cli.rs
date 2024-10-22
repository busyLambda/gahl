use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SubCommand {
    #[clap(about = "Builds the project.")]
    Build,
    #[clap(about = "Runs the project.")]
    Run,
    #[clap(about = "Creates a new project.")]
    New {
        project_name: String,
    },
}