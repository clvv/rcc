use clap::Parser;
use std::path::PathBuf;

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

pub fn rs_path_to_config(path: &str) -> PathBuf {
    let mut path = std::path::PathBuf::from(path);
    let name = path.file_stem().unwrap().to_str().unwrap();
    path.set_file_name(format!("{name}_config.toml"));
    path
}

pub fn rs_path_to_runtime_lib(path: &str) -> PathBuf {
    let mut path = std::path::PathBuf::from(path);
    let name = path.file_stem().unwrap().to_str().unwrap();
    path.set_file_name(format!("{name}_runtime_lib.rs"));
    path
}
