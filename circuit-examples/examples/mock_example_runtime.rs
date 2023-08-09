#![allow(unused_imports)]
#![allow(unused_variables)]

mod mock_example_runtime_lib;
use mock_example_runtime_lib::generate_witnesses;

use ark_bn254::Fr as F;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut inputs = std::collections::HashMap::<String, F>::new();
    inputs.insert("val".into(), F::from(args.get(1).unwrap().parse::<i32>().unwrap()));
    generate_witnesses(inputs);
}
