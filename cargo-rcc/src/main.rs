use cargo_rcc::utils::{build_circuit, clean_manifest_for, clean_manifest_forall};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
    Rcc(Rcc),
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
        circuit: PathBuf,
        #[clap(allow_hyphen_values = true, trailing_var_arg = true)]
        rest: Vec<String>,
    },
    CleanManifest {
        maybe_circuit: Option<PathBuf>,
    },
}

fn main() {
    let Cargo::Rcc(rcc) = Cargo::parse();

    match rcc.command {
        Commands::Compile { ref circuit, rest } => {
            build_circuit(circuit, rest.iter().map(AsRef::as_ref).collect())
                .expect("Cannot build circuit.");
        }
        Commands::CleanManifest { ref maybe_circuit } => {
            if let Some(circuit) = maybe_circuit {
                clean_manifest_for(&circuit).expect("Cannot clean manifest file.")
            } else {
                clean_manifest_forall().expect("Cannot clean manifest file.")
            }
        }
    }
}
