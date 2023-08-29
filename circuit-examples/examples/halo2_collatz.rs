// Given `start_number` and `output_number`, this circuit checks that applying the
// Collatz map `collatz_iterations` many times results in `output_number`.
//
// The following should succeed with 6 collatz iterations:
// `cargo run --release --example halo2_collatz_runtime 27 94`
//
use ark_bn254::Fr;
use rcc::traits::BoolWire;
use rcc_halo2::builder::{H2Wire as W, *};

#[main_component]
fn my_circuit() {
    let collatz_iterations = 6;

    let start_number = input_wire("start_number");
    let output_number = input_wire("output_number");
    output_number.declare_public("output_number");

    let mut current_number = start_number;

    let builder = start_number.builder();
    let two = builder.new_constant_wire(Fr::from(2));
    let two_inverse = builder.inv_or_panic(two);

    for iteration in 0..collatz_iterations {
        current_number.declare_public(format!("{iteration}-th number").as_str());

        // Check parity using lowest order bit of bit-decomposition
        let mut bits = current_number.to_bits_be_strict();
        let parity_bit = bits.pop().unwrap();
        parity_bit.declare_public(format!("{iteration}-th parity").as_str());

        // Collatz constraint
        //
        let next_number = parity_bit.to_alg() * ((current_number * 3) + 1)
            - (parity_bit.to_alg() - 1) * (current_number * two_inverse);

        current_number = next_number;
    }

    current_number.declare_public(format!("{collatz_iterations}-th number").as_str());

    // Constrains the last collatz sequence number to equal the second public input.
    //
    current_number == output_number;
}
