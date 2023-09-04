#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod sha256_example_runtime_lib;
use sha256_example_runtime_lib::generate_witnesses;

use ark_bn254::Fr as F;

use rcc_halo2::plaf::deserialize;
use rcc_halo2::prover::mock_prove;
use rcc_halo2::runtime::{rcc_output_to_plaf_witness_and_instance, BigInt, BigInteger, PrimeField};
use rcc_mockbuilder::runtime::BigUint;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut inputs = std::collections::HashMap::<String, F>::new();
    inputs.insert(
        "val".into(),
        F::from(args.get(1).unwrap().parse::<i32>().unwrap()),
    );
    let (witness, public) = generate_witnesses(inputs);

    print!("hash: ");
    (0..8).for_each(|i| {
        let bu: BigUint = public.get(format!("{i}").as_str()).unwrap().into_bigint().into();
        print!("{}", bu.to_str_radix(16));
    });
    println!("");
}
