#![allow(unused_imports)]
#![allow(unused_variables)]

mod mock_example_runtime_lib;
use mock_example_runtime_lib::generate_witnesses;

use ark_bn254::Fr as F;
use std::env::args;

fn main() {
    let args: Vec<String> = args().collect();
    let val = F::from(args.get(1).unwrap().parse::<i32>().unwrap());
    generate_witnesses(vec![val]);
}
