#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod halo2_example_runtime_lib;
use halo2_example_runtime_lib::generate_witnesses;

use ark_bn254::Fr as F;

use rcc_halo2::plaf::deserialize;
use rcc_halo2::prover::mock_prove;
use rcc_halo2::runtime::rcc_output_to_plaf_witness_and_instance;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut inputs = std::collections::HashMap::<String, F>::new();
    inputs.insert(
        "val".into(),
        F::from(args.get(1).unwrap().parse::<i32>().unwrap()),
    );
    let (wires, public) = generate_witnesses(inputs);
    println!("{:?}", public);
    let (witness, instance) = rcc_output_to_plaf_witness_and_instance(wires);
    let plaf_string =
        std::fs::read_to_string("circuit-examples/examples/halo2_example_config.toml")
            .expect("Cannot read plaf file");
    let plaf = deserialize(plaf_string);
    mock_prove(plaf, witness, instance);
}
