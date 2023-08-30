// Given `start_number` and `output_number`, this circuit checks that applying the
// Collatz map `collatz_iterations` many times results in `output_number`.
//
// The following, with 6 collatz iterations, should produce a wire `6-th number` with value `94`:
// `cargo run --release --example halo2_collatz_runtime 27`
//
use ark_bn254::Fr;
use rcc::traits::BoolWire;
use rcc_halo2::builder::{H2Wire as W, *};
use rcc_halo2::prover::Field;

#[main_component]
fn my_circuit() {
    let collatz_iterations = 6;

    let start_number = input_wire("start_number");
    start_number.declare_public("start_number");

    let mut current_number = start_number;

    let two_inverse = Fr::from(2).inverse().unwrap();

    for _ in 0..collatz_iterations {
        // Check parity using lowest order bit of bit-decomposition
        let mut bits = current_number.to_bits_be_strict();
        let parity_bit = bits.pop().unwrap();

        // Collatz constraint
        //
        let next_number =
            parity_bit.then_or_else((current_number * 3) + 1, current_number * two_inverse);

        current_number = next_number;
    }

    current_number.declare_public(format!("{collatz_iterations}-th number").as_str());
}
