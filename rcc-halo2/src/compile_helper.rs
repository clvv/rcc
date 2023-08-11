use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about)]
pub struct CompilationArgs {
    /// Path to write the runtime witness generation library
    #[arg(short, long)]
    pub runtime: Option<PathBuf>,

    /// Path to write the plaf toml config file for the circuit
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

