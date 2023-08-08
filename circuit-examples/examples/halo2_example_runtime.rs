#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#![allow(unused_variables)]

mod halo2_example_runtime_lib;
use halo2_example_runtime_lib::generate_witnesses;

use ark_bn254::Fr as F;

use rcc_halo2::plaf::deserialize;
use rcc_halo2::runtime::rcc_output_to_plaf_witness_and_instance;
use rcc_halo2::prover::mock_prove;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let val = F::from(args.get(1).unwrap().parse::<i32>().unwrap());
    let wires = generate_witnesses(vec![val]);
    let (witness, instance) = rcc_output_to_plaf_witness_and_instance(wires);
    let plaf_string = std::fs::read_to_string("examples/halo2_example_config.toml").expect("Cannot read plaf file");
    let plaf = deserialize(plaf_string);
    mock_prove(plaf, witness, instance);
}
