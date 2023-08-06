use std::path::PathBuf;
use clap::{Parser, Subcommand};
use cargo_rcc::utils::{build_circuit, clean_manifest_for, clean_manifest_forall};

#[derive(Parser, Debug)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
    Rcc(Rcc)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Rcc {
    // /// Turn debugging information on
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Compile {
        circuit: PathBuf
    },
    CleanManifest {
        maybe_circuit: Option<PathBuf>
    }
}

fn main() {
    let Cargo::Rcc(rcc) = Cargo::parse();

    match rcc.command {
        Commands::Compile { ref circuit } =>
            build_circuit(circuit).expect("Cannot build circuit."),
        Commands::CleanManifest { ref maybe_circuit } =>
            if let Some(circuit) = maybe_circuit {
                clean_manifest_for(&circuit).expect("Cannot clean manifest file.")
            } else {
                clean_manifest_forall().expect("Cannot clean manifest file.")
            }
    }
}
