#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]

mod circuit2_runtime_lib;

use ark_bn254::Fr as F;
use std::env::args;

fn main() {
    let args: Vec<String> = args().collect();
    let val = F::from(args.get(1).unwrap().parse::<i32>().unwrap());
    let wires = circuit2_runtime_lib::compute(vec![val]);
    println!("Public Instance: {:?}", wires[1]);
}
