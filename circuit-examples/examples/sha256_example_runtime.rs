#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod sha256_example_runtime_lib;
use sha256_example_runtime_lib::generate_witnesses;

use ark_bn254::Fr as F;
use rcc_mockbuilder::runtime::{BigUint, PrimeField};

/// Generated via ChatGPT
fn string_to_bool_vector(input: &str) -> Vec<bool> {
    // Check if the input string has exactly 4 bytes
    if input.len() != 4 {
        panic!("Input string must be exactly 4 bytes long");
    }

    let mut bool_vector = Vec::with_capacity(32); // 4 bytes * 8 bits per byte = 32 bits

    for byte in input.bytes() {
        // Convert the byte into a u8
        let byte_value = byte as u8;

        // Iterate through each bit in the byte (from left to right)
        for i in (0..8).rev() {
            // Check if the i-th bit is set (1) or not (0)
            let bit_is_set = (byte_value >> i) & 1 == 1;

            // Push the result into the bool vector
            bool_vector.push(bit_is_set);
        }
    }

    bool_vector
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut inputs = std::collections::HashMap::<String, F>::new();

    for (i, b) in string_to_bool_vector(args[1].as_str()).iter().enumerate() {
        inputs.insert(format!("{i}-th bit"), F::from(*b));
    }

    let (witness, public) = generate_witnesses(inputs);

    print!("hash: ");
    (0..8).for_each(|i| {
        let bu: BigUint = public.get(format!("{i}").as_str()).unwrap().into_bigint().into();
        print!("{}", bu.to_str_radix(16));
    });
    println!("");
}
