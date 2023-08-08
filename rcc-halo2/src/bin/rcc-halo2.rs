// use std::path::PathBuf;
// use clap::{Parser, Subcommand};

// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// pub struct RccHalo2 {
//     // /// Turn debugging information on
//     // #[arg(short, long, action = clap::ArgAction::Count)]
//     // debug: u8,

//     #[command(subcommand)]
//     command: Commands,
// }

// #[derive(Subcommand, Debug)]
// enum Commands {
//     MockProve {
//         circuit: PathBuf
//     },
//     Prove {
//         circuit: PathBuf
//     },
//     Setup {
//         circuit: PathBuf
//     },
// }

// fn main() {
//     let rcc_halo2 = RccHalo2::parse();

//     match rcc_halo2.command {
//         Commands::MockProve { ref circuit } => {

//         },
//         Commands::Prove { ref circuit } => todo!,
//         Commands::Setup { ref circuit } => todo!,
//     }
// }
