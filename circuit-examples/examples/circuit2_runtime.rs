#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]

mod circuit2_runtime_lib;

use ark_bn254::Fr as F;

use rcc_halo2::runtime::rcc_output_to_plaf_witness;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let val = F::from(args.get(1).unwrap().parse::<i32>().unwrap());
    let wires = circuit2_runtime_lib::compute(vec![val]);
    println!("Public Instance: {:?}", wires[1]);
    let witness = rcc_output_to_plaf_witness(wires);
    println!("{:?}", witness);
}
