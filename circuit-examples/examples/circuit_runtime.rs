#![allow(unused_imports)]

mod circuit_runtime_lib;

use ark_bn254::Fr as F;
use std::env::args;

fn main() {
    let args: Vec<String> = args().collect();
    let val = F::from(args.get(1).unwrap().parse::<i32>().unwrap());
    circuit_runtime_lib::generate_witnesses(vec![val]);
}
